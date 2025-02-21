use mullvad_api::{
    rest::{self, MullvadRestHandle},
    AccountsProxy, ApiProxy,
};
use mullvad_types::account::AccountNumber;
use talpid_future::retry::retry_future;

use super::{
    cancellation::{RequestCancelHandle, SwiftCancelHandle},
    completion::{CompletionCookie, SwiftCompletionHandler},
    response::SwiftMullvadApiResponse,
    retry_strategy::{RetryStrategy, SwiftRetryStrategy},
    SwiftApiContext,
};

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is
/// safe because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this
/// type is intended to be used.
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_api_get_addresses(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let completion = completion_handler.clone();
    let task = tokio_handle.clone().spawn(async move {
        match mullvad_api_get_addresses_inner(api_context.rest_handle(), retry_strategy).await {
            Ok(response) => completion.finish(response),
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(SwiftMullvadApiResponse::rest_error(err));
            }
        }
    });

    RequestCancelHandle::new(task, completion_handler.clone()).into_swift()
}

async fn mullvad_api_get_addresses_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let api = ApiProxy::new(rest_client);

    let future_factory = || api.get_api_addrs_response();

    let should_retry = |result: &Result<_, rest::Error>| match result {
        Err(err) => err.is_network_error(),
        Ok(_) => false,
    };

    let response = retry_future(future_factory, should_retry, retry_strategy.delays()).await?;

    SwiftMullvadApiResponse::with_body(response).await
}

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is
/// safe because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this
/// type is intended to be used.
///
/// `account` must be a pointer to a null terminated string to the account number
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_api_init_storekit_payment(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account: *const u8,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let completion = completion_handler.clone();

    let account = unsafe { std::ffi::CStr::from_ptr(account.cast()) };
    let Ok(account) = account.to_str() else {
        completion_handler.finish(SwiftMullvadApiResponse::invalid_input(
            c"Invalid account string",
        ));
        return SwiftCancelHandle::empty();
    };
    let account = AccountNumber::from(account);

    let task = tokio_handle.clone().spawn(async move {
        match mullvad_api_init_storekit_payment_inner(
            api_context.rest_handle(),
            retry_strategy,
            account,
        )
        .await
        {
            Ok(response) => completion.finish(response),
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(SwiftMullvadApiResponse::rest_error(err));
            }
        }
    });

    RequestCancelHandle::new(task, completion_handler.clone()).into_swift()
}

async fn mullvad_api_init_storekit_payment_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account: AccountNumber,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let account_proxy = AccountsProxy::new(rest_client);

    let future_factory = || account_proxy.init_storekit_payment(account.clone());

    let should_retry = |result: &Result<_, rest::Error>| match result {
        Err(err) => err.is_network_error(),
        Ok(_) => false,
    };

    let response = retry_future(future_factory, should_retry, retry_strategy.delays()).await?;

    SwiftMullvadApiResponse::with_body(response).await
}

/// # Safety
///
/// `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
/// by calling `mullvad_api_init_new`.
///
/// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is
/// safe because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this
/// type is intended to be used.
///
/// `account` must be a pointer to a null terminated string to the account number
///
/// `transaction` must be a pointer to a null terminated string to the jws representation of the transaction
///
/// This function is not safe to call multiple times with the same `CompletionCookie`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_api_check_storekit_payment(
    api_context: SwiftApiContext,
    completion_cookie: *mut libc::c_void,
    retry_strategy: SwiftRetryStrategy,
    account: *const u8,
    transaction: *const u8,
) -> SwiftCancelHandle {
    let completion_handler = SwiftCompletionHandler::new(CompletionCookie(completion_cookie));

    let Ok(tokio_handle) = crate::mullvad_ios_runtime() else {
        completion_handler.finish(SwiftMullvadApiResponse::no_tokio_runtime());
        return SwiftCancelHandle::empty();
    };

    let api_context = api_context.into_rust_context();
    let retry_strategy = unsafe { retry_strategy.into_rust() };

    let completion = completion_handler.clone();

    let account = unsafe { std::ffi::CStr::from_ptr(account.cast()) };
    let Ok(account) = account.to_str() else {
        completion_handler.finish(SwiftMullvadApiResponse::invalid_input(
            c"Invalid account string",
        ));
        return SwiftCancelHandle::empty();
    };
    let account = AccountNumber::from(account);

    let transaction = unsafe { std::ffi::CStr::from_ptr(transaction.cast()) };
    let Ok(transaction) = transaction.to_str() else {
        completion_handler.finish(SwiftMullvadApiResponse::invalid_input(
            c"Invalid transaction string",
        ));
        return SwiftCancelHandle::empty();
    };
    let transaction = String::from(transaction);

    let task = tokio_handle.clone().spawn(async move {
        match mullvad_api_check_storekit_payment_inner(
            api_context.rest_handle(),
            retry_strategy,
            account,
            transaction,
        )
        .await
        {
            Ok(response) => completion.finish(response),
            Err(err) => {
                log::error!("{err:?}");
                completion.finish(SwiftMullvadApiResponse::rest_error(err));
            }
        }
    });

    RequestCancelHandle::new(task, completion_handler.clone()).into_swift()
}

async fn mullvad_api_check_storekit_payment_inner(
    rest_client: MullvadRestHandle,
    retry_strategy: RetryStrategy,
    account: AccountNumber,
    transaction: String,
) -> Result<SwiftMullvadApiResponse, rest::Error> {
    let account_proxy = AccountsProxy::new(rest_client);

    let future_factory =
        || account_proxy.check_storekit_payment(account.clone(), transaction.clone());

    let should_retry = |result: &Result<_, rest::Error>| match result {
        Err(err) => err.is_network_error(),
        Ok(_) => false,
    };

    let response = retry_future(future_factory, should_retry, retry_strategy.delays()).await?;

    SwiftMullvadApiResponse::with_body(response).await
}
