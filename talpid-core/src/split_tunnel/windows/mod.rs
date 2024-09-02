mod driver;
mod event;
mod path_monitor;
mod service;
mod volume_monitor;
mod windows;

use crate::{tunnel::TunnelMetadata, tunnel_state_machine::TunnelCommand};
use driver::DeviceHandle;
use futures::channel::{mpsc, oneshot};
use path_monitor::PathMonitorHandle;
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc as sync_mpsc, Arc, Mutex, MutexGuard, RwLock, Weak,
    },
    time::Duration,
};
use talpid_routing::{get_best_default_route, CallbackHandle, EventType, RouteManagerHandle};
use talpid_types::{split_tunnel::ExcludedProcess, tunnel::ErrorStateCause, ErrorExt};
use talpid_windows::{
    net::{get_ip_address_for_interface, AddressFamily},
    sync::Event,
};

const RESERVED_IP_V4: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 123);

/// Errors that may occur in [`SplitTunnel`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to install or start driver service
    #[error("Failed to start driver service")]
    ServiceError(#[source] service::Error),

    /// Failed to initialize the driver
    #[error("Failed to initialize driver")]
    InitializationError(#[source] driver::DeviceHandleError),

    /// Failed to reset the driver
    #[error("Failed to reset driver")]
    ResetError(#[source] io::Error),

    /// Failed to set paths to excluded applications
    #[error("Failed to set list of excluded applications")]
    SetConfiguration(#[source] io::Error),

    /// Failed to register interface IP addresses
    #[error("Failed to register IP addresses for exclusions")]
    RegisterIps(#[source] io::Error),

    /// Failed to set up the driver event loop
    #[error("Failed to set up the driver event loop")]
    EventThreadError(#[source] io::Error),

    /// Failed to obtain default route
    #[error("Failed to obtain the default route")]
    ObtainDefaultRoute(#[source] talpid_routing::Error),

    /// Failed to obtain an IP address given a network interface LUID
    #[error("Failed to obtain IP address for interface LUID")]
    LuidToIp(#[source] talpid_windows::net::Error),

    /// Failed to set up callback for monitoring default route changes
    #[error("Failed to register default route change callback")]
    RegisterRouteChangeCallback,

    /// The request handling thread is stuck
    #[error("The ST request thread is stuck")]
    RequestThreadStuck,

    /// The request handling thread is down
    #[error("The split tunnel monitor is down")]
    SplitTunnelDown,

    /// Failed to start the NTFS reparse point monitor
    #[error("Failed to start path monitor")]
    StartPathMonitor(#[source] io::Error),

    /// A previous path update has not yet completed
    #[error("A previous update is not yet complete")]
    AlreadySettingPaths,
}

/// Manages applications whose traffic to exclude from the tunnel.
pub struct SplitTunnel {
    runtime: tokio::runtime::Handle,
    request_tx: sync_mpsc::Sender<Request>,
    event_thread: Option<std::thread::JoinHandle<()>>,
    quit_event: Arc<Event>,
    excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    _route_change_callback: Option<CallbackHandle>,
    async_path_update_in_progress: Arc<AtomicBool>,
    route_manager: RouteManagerHandle,
}

/// A request to the split tunnel monitor
struct Request {
    /// Request details
    details: RequestDetails,
    /// Whether to block if the request fails
    must_succeed: bool,
    /// Response channel
    response_tx: Option<sync_mpsc::Sender<Result<(), Error>>>,
}

enum RequestDetails {
    /// Update paths to exclude
    SetPaths(Vec<OsString>),
    /// Update default and VPN tunnel addresses
    RegisterIps(InterfaceAddresses),
    /// Stop the split tunnel monitor
    Stop,
}

impl Request {
    fn new(details: RequestDetails) -> Self {
        Request {
            details,
            must_succeed: false,
            response_tx: None,
        }
    }

    fn response_tx(mut self, response_tx: sync_mpsc::Sender<Result<(), Error>>) -> Self {
        self.response_tx = Some(response_tx);
        self
    }

    fn must_succeed(mut self) -> Self {
        self.must_succeed = true;
        self
    }

    fn request_name(&self) -> &'static str {
        match self.details {
            RequestDetails::SetPaths(_) => "SetPaths",
            RequestDetails::RegisterIps(_) => "RegisterIps",
            RequestDetails::Stop => "Stop",
        }
    }
}

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default, PartialEq, Clone)]
struct InterfaceAddresses {
    tunnel_ipv4: Option<Ipv4Addr>,
    tunnel_ipv6: Option<Ipv6Addr>,
    internet_ipv4: Option<Ipv4Addr>,
    internet_ipv6: Option<Ipv6Addr>,
}

/// Cloneable handle for interacting with the split tunnel module.
#[derive(Debug, Clone)]
pub struct SplitTunnelHandle {
    excluded_processes: Weak<RwLock<HashMap<usize, ExcludedProcess>>>,
}

impl SplitTunnelHandle {
    /// Return processes that are currently being excluded, including
    /// their pids, paths, and reason for being excluded.
    pub fn get_processes(&self) -> Result<Vec<ExcludedProcess>, Error> {
        let processes = self
            .excluded_processes
            .upgrade()
            .ok_or(Error::SplitTunnelDown)?;
        let processes = processes.read().unwrap();
        Ok(processes.values().cloned().collect())
    }
}

impl SplitTunnel {
    /// Initialize the split tunnel device.
    pub fn new(
        runtime: tokio::runtime::Handle,
        resource_dir: PathBuf,
        daemon_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
        volume_update_rx: mpsc::UnboundedReceiver<()>,
        route_manager: RouteManagerHandle,
    ) -> Result<Self, Error> {
        let excluded_processes = Arc::new(RwLock::new(HashMap::new()));

        let (request_tx, handle) = Self::spawn_request_thread(
            resource_dir,
            daemon_tx,
            volume_update_rx,
            excluded_processes.clone(),
        )?;

        let (event_thread, quit_event) = event::spawn_listener(handle, excluded_processes.clone())
            .map_err(Error::EventThreadError)?;

        Ok(SplitTunnel {
            runtime,
            request_tx,
            event_thread: Some(event_thread),
            quit_event,
            _route_change_callback: None,
            async_path_update_in_progress: Arc::new(AtomicBool::new(false)),
            excluded_processes,
            route_manager,
        })
    }

    fn spawn_request_thread(
        resource_dir: PathBuf,
        daemon_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
        volume_update_rx: mpsc::UnboundedReceiver<()>,
        excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    ) -> Result<(sync_mpsc::Sender<Request>, Arc<driver::DeviceHandle>), Error> {
        let (tx, rx): (sync_mpsc::Sender<Request>, _) = sync_mpsc::channel();
        let (init_tx, init_rx) = sync_mpsc::channel();

        let monitored_paths = Arc::new(Mutex::new(vec![]));
        let monitored_paths_copy = monitored_paths.clone();

        let (monitor_tx, monitor_rx) = sync_mpsc::channel();

        let path_monitor = path_monitor::PathMonitor::spawn(monitor_tx.clone())
            .map_err(Error::StartPathMonitor)?;
        let volume_monitor = volume_monitor::VolumeMonitor::spawn(
            path_monitor.clone(),
            monitor_tx,
            monitored_paths.clone(),
            volume_update_rx,
        );

        std::thread::spawn(move || {
            // Ensure that the device driver service is running and that we have a handle to it
            let handle = match Self::setup_and_create_device(&resource_dir) {
                Ok(handle) => {
                    let _ = init_tx.send(Ok(handle.clone()));
                    handle
                }
                Err(error) => {
                    let _ = init_tx.send(Err(error));
                    return;
                }
            };

            Self::request_loop(
                handle.clone(),
                rx,
                daemon_tx,
                monitored_paths,
                path_monitor.clone(),
                excluded_processes,
            );

            // Shut components down in a sane order
            drop(volume_monitor);
            if let Err(error) = path_monitor.shutdown() {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to shut down path monitor")
                );
            }

            // The device handle must be dropped before stopping the service
            debug_assert_eq!(Arc::strong_count(&handle), 1);
            drop(handle);

            log::debug!("Stopping ST service");
            if let Err(error) = service::stop_driver_service() {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to stop ST service")
                );
            }
        });

        let handle = init_rx
            .recv_timeout(REQUEST_TIMEOUT)
            .map_err(|_| Error::RequestThreadStuck)??;

        let handle_copy = handle.clone();

        std::thread::spawn(move || {
            while let Ok(()) = monitor_rx.recv() {
                let paths = monitored_paths_copy.lock().unwrap();
                let result = if paths.len() > 0 {
                    log::debug!("Re-resolving excluded paths");
                    handle_copy.set_config(&paths)
                } else {
                    continue;
                };
                if let Err(error) = result {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to update excluded paths")
                    );
                }
            }
        });

        Ok((tx, handle))
    }

    /// Install the driver and create a device for it
    fn setup_and_create_device(resource_dir: &Path) -> Result<Arc<DeviceHandle>, Error> {
        service::install_driver_if_required(resource_dir).map_err(Error::ServiceError)?;
        driver::DeviceHandle::new()
            .map(Arc::new)
            .map_err(Error::InitializationError)
    }

    /// Service requests to the device driver
    fn request_loop(
        handle: Arc<driver::DeviceHandle>,
        cmd_rx: sync_mpsc::Receiver<Request>,
        daemon_tx: Weak<mpsc::UnboundedSender<TunnelCommand>>,
        monitored_paths: Arc<Mutex<Vec<OsString>>>,
        path_monitor: PathMonitorHandle,
        excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    ) {
        let mut previous_addresses = InterfaceAddresses::default();

        while let Ok(request) = cmd_rx.recv() {
            let request_name = request.request_name();

            let (should_stop, response) = Self::handle_request(
                request.details,
                &handle,
                &path_monitor,
                &monitored_paths,
                &excluded_processes,
                &mut previous_addresses,
            );

            Self::handle_request_result(
                &daemon_tx,
                response,
                request.must_succeed,
                request_name,
                request.response_tx,
            );

            // Stop request loop
            if should_stop {
                break;
            }
        }
    }

    /// Handle a request to the split tunnel device
    fn handle_request(
        request: RequestDetails,
        handle: &DeviceHandle,
        path_monitor: &path_monitor::PathMonitorHandle,
        monitored_paths: &Arc<Mutex<Vec<OsString>>>,
        excluded_processes: &Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
        previous_addresses: &mut InterfaceAddresses,
    ) -> (bool, Result<(), Error>) {
        let (should_stop, result) = match request {
            RequestDetails::SetPaths(paths) => {
                let mut monitored_paths_guard = monitored_paths.lock().unwrap();

                let result = if !paths.is_empty() {
                    handle.set_config(&paths).map_err(Error::SetConfiguration)
                } else {
                    handle.clear_config().map_err(Error::SetConfiguration)
                };

                if result.is_ok() {
                    if let Err(error) = path_monitor.set_paths(&paths) {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Failed to update path monitor")
                        );
                    }
                    *monitored_paths_guard = paths.to_vec();
                }

                (false, result)
            }
            RequestDetails::RegisterIps(mut ips) => {
                if ips.internet_ipv4.is_none() && ips.internet_ipv6.is_none() {
                    ips.tunnel_ipv4 = None;
                    ips.tunnel_ipv6 = None;
                }
                if previous_addresses == &ips {
                    (false, Ok(()))
                } else {
                    let result = handle
                        .register_ips(
                            ips.tunnel_ipv4,
                            ips.tunnel_ipv6,
                            ips.internet_ipv4,
                            ips.internet_ipv6,
                        )
                        .map_err(Error::RegisterIps);
                    if result.is_ok() {
                        *previous_addresses = ips;
                    }
                    (false, result)
                }
            }
            RequestDetails::Stop => {
                if let Err(error) = handle.reset().map_err(Error::ResetError) {
                    // Shut down failed, so continue to live
                    return (false, Err(error));
                }

                // Clean up
                monitored_paths.lock().unwrap().clear();
                excluded_processes.write().unwrap().clear();

                // Stop listening to commands
                (true, Ok(()))
            }
        };

        (should_stop, result)
    }

    /// Handle the result of a request
    fn handle_request_result(
        daemon_tx: &Weak<mpsc::UnboundedSender<TunnelCommand>>,
        result: Result<(), Error>,
        must_succeed: bool,
        request_name: &str,
        response_tx: Option<sync_mpsc::Sender<Result<(), Error>>>,
    ) {
        let log_request_failure = |response: &Result<(), Error>| {
            if let Err(error) = response {
                log::error!(
                    "Request/ioctl failed: {}\n{}",
                    request_name,
                    error.display_chain()
                );
            }
        };

        let request_failed = result.is_err();

        if let Some(response_tx) = response_tx {
            if let Err(error) = response_tx.send(result) {
                log::error!(
                    "A response could not be sent for completed request/ioctl: {}",
                    request_name
                );
                log_request_failure(&error.0);
            }
        } else {
            log_request_failure(&result);
        }

        // Move to error state if the request failed but must succeed
        if request_failed && must_succeed {
            if let Some(daemon_tx) = daemon_tx.upgrade() {
                log::debug!(
                    "Entering error state due to failed request/ioctl: {}",
                    request_name
                );
                let _ = daemon_tx
                    .unbounded_send(TunnelCommand::Block(ErrorStateCause::SplitTunnelError));
            } else {
                log::error!("Cannot handle failed request since tunnel state machine is down");
            }
        }
    }

    fn send_request(&self, request: RequestDetails) -> Result<(), Error> {
        Self::send_request_inner(&self.request_tx, request)
    }

    fn send_request_inner(
        request_tx: &sync_mpsc::Sender<Request>,
        request: RequestDetails,
    ) -> Result<(), Error> {
        let (response_tx, response_rx) = sync_mpsc::channel();

        request_tx
            .send(Request::new(request).response_tx(response_tx))
            .map_err(|_| Error::SplitTunnelDown)?;

        response_rx
            .recv_timeout(REQUEST_TIMEOUT)
            .map_err(|_| Error::RequestThreadStuck)?
    }

    /// Set a list of applications to exclude from the tunnel.
    pub fn set_paths_sync<T: AsRef<OsStr>>(&self, paths: &[T]) -> Result<(), Error> {
        self.send_request(RequestDetails::SetPaths(
            paths
                .iter()
                .map(|path| path.as_ref().to_os_string())
                .collect(),
        ))
    }

    /// Set a list of applications to exclude from the tunnel.
    pub fn set_paths<T: AsRef<OsStr>>(
        &self,
        paths: &[T],
        result_tx: oneshot::Sender<Result<(), Error>>,
    ) {
        let busy = self
            .async_path_update_in_progress
            .swap(true, Ordering::SeqCst);
        if busy {
            let _ = result_tx.send(Err(Error::AlreadySettingPaths));
            return;
        }
        let (response_tx, response_rx) = sync_mpsc::channel();
        let request = RequestDetails::SetPaths(
            paths
                .iter()
                .map(|path| path.as_ref().to_os_string())
                .collect(),
        );
        let request_tx = self.request_tx.clone();

        let wait_task = move || {
            request_tx
                .send(Request::new(request).response_tx(response_tx))
                .map_err(|_| Error::SplitTunnelDown)?;
            response_rx.recv().map_err(|_| Error::SplitTunnelDown)?
        };
        let in_progress = self.async_path_update_in_progress.clone();
        self.runtime.spawn_blocking(move || {
            let _ = result_tx.send(wait_task());
            in_progress.store(false, Ordering::SeqCst);
        });
    }

    /// Instructs the driver to redirect connections for sockets bound to 0.0.0.0, ::, or the
    /// tunnel addresses (if any) to the default route.
    pub fn set_tunnel_addresses(&mut self, metadata: Option<&TunnelMetadata>) -> Result<(), Error> {
        let mut tunnel_ipv4 = None;
        let mut tunnel_ipv6 = None;

        if let Some(metadata) = metadata {
            for ip in &metadata.ips {
                match ip {
                    IpAddr::V4(address) => tunnel_ipv4 = Some(*address),
                    IpAddr::V6(address) => tunnel_ipv6 = Some(*address),
                }
            }
        }

        let tunnel_ipv4 = Some(tunnel_ipv4.unwrap_or(RESERVED_IP_V4));
        let context_mutex = Arc::new(Mutex::new(
            SplitTunnelDefaultRouteChangeHandlerContext::new(
                self.request_tx.clone(),
                tunnel_ipv4,
                tunnel_ipv6,
            ),
        ));

        self._route_change_callback = None;
        let moved_context_mutex = context_mutex.clone();
        let context = context_mutex.lock().unwrap();
        let callback = self
            .runtime
            .block_on(
                self.route_manager
                    .add_default_route_change_callback(Box::new(move |event, addr_family| {
                        split_tunnel_default_route_change_handler(
                            event,
                            addr_family,
                            &moved_context_mutex,
                        )
                    })),
            )
            .map(Some)
            // NOTE: This cannot fail if a callback is created. If that assumption is wrong, this
            // could deadlock if the dropped callback is invoked (see `init_context`).
            .map_err(|_| Error::RegisterRouteChangeCallback)?;

        Self::init_context(context, &self.request_tx)?;
        self._route_change_callback = callback;

        Ok(())
    }

    fn init_context(
        mut context: MutexGuard<'_, SplitTunnelDefaultRouteChangeHandlerContext>,
        request_tx: &sync_mpsc::Sender<Request>,
    ) -> Result<(), Error> {
        // NOTE: This should remain a separate function. Dropping the context after `callback`
        // causes a deadlock if `split_tunnel_default_route_change_handler` is called at the same
        // time (i.e. if a route change has occurred), since it waits on the context and
        // `CallbackHandle::drop` also waits for `split_tunnel_default_route_change_handler`
        // to complete.

        context.initialize_internet_addresses()?;
        SplitTunnel::send_request_inner(
            request_tx,
            RequestDetails::RegisterIps(context.addresses.clone()),
        )
    }

    /// Instructs the driver to stop redirecting connections.
    pub fn clear_tunnel_addresses(&mut self) -> Result<(), Error> {
        self._route_change_callback = None;
        self.send_request(RequestDetails::RegisterIps(InterfaceAddresses::default()))
    }

    /// Returns a handle used for interacting with the split tunnel module.
    pub fn handle(&self) -> SplitTunnelHandle {
        SplitTunnelHandle {
            excluded_processes: Arc::downgrade(&self.excluded_processes),
        }
    }
}

impl Drop for SplitTunnel {
    fn drop(&mut self) {
        if let Some(_event_thread) = self.event_thread.take() {
            if let Err(error) = self.quit_event.set() {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to close ST event thread")
                );
            }
            // Not joining `event_thread`: It may be unresponsive.
        }

        if let Err(error) = self.send_request(RequestDetails::Stop) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to stop ST driver service")
            );
        }
    }
}

struct SplitTunnelDefaultRouteChangeHandlerContext {
    request_tx: sync_mpsc::Sender<Request>,
    pub addresses: InterfaceAddresses,
}

impl SplitTunnelDefaultRouteChangeHandlerContext {
    pub fn new(
        request_tx: sync_mpsc::Sender<Request>,
        tunnel_ipv4: Option<Ipv4Addr>,
        tunnel_ipv6: Option<Ipv6Addr>,
    ) -> Self {
        SplitTunnelDefaultRouteChangeHandlerContext {
            request_tx,
            addresses: InterfaceAddresses {
                tunnel_ipv4,
                tunnel_ipv6,
                internet_ipv4: None,
                internet_ipv6: None,
            },
        }
    }

    pub fn initialize_internet_addresses(&mut self) -> Result<(), Error> {
        // Identify IP address that gives us Internet access
        let internet_ipv4 = get_best_default_route(AddressFamily::Ipv4)
            .map_err(Error::ObtainDefaultRoute)?
            .map(|route| {
                get_ip_address_for_interface(AddressFamily::Ipv4, route.iface).map(|ip| match ip {
                    Some(IpAddr::V4(addr)) => Some(addr),
                    Some(_) => unreachable!("wrong address family (expected IPv4)"),
                    None => {
                        log::warn!("No IPv4 address was found for the default route interface");
                        None
                    }
                })
            })
            .transpose()
            .map_err(Error::LuidToIp)?
            .flatten();
        let internet_ipv6 = get_best_default_route(AddressFamily::Ipv6)
            .map_err(Error::ObtainDefaultRoute)?
            .map(|route| {
                get_ip_address_for_interface(AddressFamily::Ipv6, route.iface).map(|ip| match ip {
                    Some(IpAddr::V6(addr)) => Some(addr),
                    Some(_) => unreachable!("wrong address family (expected IPv6)"),
                    None => {
                        log::warn!("No IPv6 address was found for the default route interface");
                        None
                    }
                })
            })
            .transpose()
            .map_err(Error::LuidToIp)?
            .flatten();

        self.addresses.internet_ipv4 = internet_ipv4;
        self.addresses.internet_ipv6 = internet_ipv6;

        Ok(())
    }
}

fn split_tunnel_default_route_change_handler(
    event_type: EventType<'_>,
    address_family: AddressFamily,
    ctx_mutex: &Arc<Mutex<SplitTunnelDefaultRouteChangeHandlerContext>>,
) {
    use talpid_routing::EventType::*;

    // Update the "internet interface" IP when best default route changes
    let mut ctx = ctx_mutex.lock().expect("ST route handler mutex poisoned");

    let prev_addrs = ctx.addresses.clone();

    match event_type {
        Updated(default_route) | UpdatedDetails(default_route) => {
            match get_ip_address_for_interface(address_family, default_route.iface) {
                Ok(Some(ip)) => match ip {
                    IpAddr::V4(addr) => ctx.addresses.internet_ipv4 = Some(addr),
                    IpAddr::V6(addr) => ctx.addresses.internet_ipv6 = Some(addr),
                },
                Ok(None) => {
                    log::warn!("Failed to obtain default route interface address");
                    match address_family {
                        AddressFamily::Ipv4 => {
                            ctx.addresses.internet_ipv4 = None;
                        }
                        AddressFamily::Ipv6 => {
                            ctx.addresses.internet_ipv6 = None;
                        }
                    }
                }
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Failed to obtain default route interface address"
                        )
                    );
                }
            };
        }
        // no default route
        Removed => match address_family {
            AddressFamily::Ipv4 => {
                ctx.addresses.internet_ipv4 = None;
            }
            AddressFamily::Ipv6 => {
                ctx.addresses.internet_ipv6 = None;
            }
        },
    }

    if prev_addrs == ctx.addresses {
        return;
    }

    if ctx
        .request_tx
        .send(Request::new(RequestDetails::RegisterIps(ctx.addresses.clone())).must_succeed())
        .is_err()
    {
        log::error!("Split tunnel request thread is down");
    }
}
