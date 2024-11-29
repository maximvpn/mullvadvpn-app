use std::{
    future::Future,
    os::fd::{AsRawFd, RawFd},
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    config::{Config, MULLVAD_INTERFACE_NAME},
    stats::StatsMap,
    wireguard_go::get_tunnel_for_userspace,
    Tunnel, TunnelError,
};
use boringtun::device::{DeviceConfig, DeviceHandle};
use ipnetwork::IpNetwork;
use nix::unistd::write;
use talpid_tunnel::tun_provider::Tun;
use talpid_tunnel::tun_provider::TunProvider;

const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;

pub struct BoringTun {
    device_handle: DeviceHandle,

    /// holding on to the tunnel device and the log file ensures that the associated file handles
    /// live long enough and get closed when the tunnel is stopped
    tunnel_device: Tun,
}

impl BoringTun {
    pub fn start_tunnel(
        config: &Config,
        _log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        routes: impl Iterator<Item = IpNetwork>,
        #[cfg(daita)] _resource_dir: &Path,
    ) -> Result<Self, TunnelError> {
        log::info!("BoringTun::start_tunnel");
        log::info!("calling get_tunnel_for_userspace");

        // TODO: investigate timing bug when creating tun device?
        // Device or resource busy
        let (tun, _tunnel_fd) = get_tunnel_for_userspace(tun_provider, config, routes)?;

        log::info!("creating pipe");
        let (config_pipe_rx, config_pipe_tx) = nix::unistd::pipe().expect("failed to create pipe");
        let wg_config_str = config.to_userspace_format();
        let boringtun_config = DeviceConfig {
            n_threads: 8,
            use_connected_socket: true, // TODO: what is this?
            use_multi_queue: true,      // TODO: what is this?
            uapi_fd: config_pipe_rx,
        };

        log::info!("passing tunnel dev to boringtun");
        let device_handle: DeviceHandle =
            DeviceHandle::new(&_tunnel_fd.to_string(), boringtun_config)
                .map_err(TunnelError::BoringTunDevice)?;

        // TODO: remove null byte in a better way
        // TODO: make sure all the bytes are written
        // TODO: can we use a rust type instead of a raw fd?

        log::info!("writing wireguard-config to boringtun");
        let mut wg_config_bytes = wg_config_str.to_str().unwrap().as_bytes();
        while !wg_config_bytes.is_empty() {
            let n = write(config_pipe_tx, wg_config_bytes).expect("write failed");

            if n == 0 {
                panic!("didn't write??");
            }

            wg_config_bytes = &wg_config_bytes[n..];
        }
        log::info!("done! boringtun time!?");

        Ok(Self {
            device_handle,
            tunnel_device: tun,
        })
    }
}

impl Tunnel for BoringTun {
    fn get_interface_name(&self) -> String {
        self.tunnel_device.interface_name().to_string()
    }

    fn stop(mut self: Box<Self>) -> Result<(), TunnelError> {
        log::info!("BoringTun::stop");
        self.device_handle.clean();
        //self.device_handle.wait(); // TODO: do we need this<?

        Ok(())
    }

    fn get_tunnel_stats(&self) -> Result<StatsMap, TunnelError> {
        todo!("get_tunnel_stats")
    }

    fn set_config<'a>(
        &'a mut self,
        _config: Config,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), TunnelError>> + Send + 'a>> {
        todo!("set_config")
    }

    fn start_daita(&mut self) -> Result<(), TunnelError> {
        log::info!("Haha no");
        Ok(())
    }
}
