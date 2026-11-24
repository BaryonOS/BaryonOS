//! The volume adapter's send/receive boundary, shared with its host regressions.

use nieuw_os_sdk_runtime::Error;
use volume_core::{wire, Status};

/// Exchange one volume request using the adapter's native send and receive.
/// A send error must mean the request was not queued. After a successful send,
/// every receive or envelope error leaves the operation's outcome unknown;
/// a matching response, including a service refusal, is returned unchanged.
pub fn exchange(
    send: impl FnOnce() -> Result<(), Error>,
    receive: impl FnOnce() -> Result<[u8; wire::SIZE], Error>,
) -> Result<[u8; wire::SIZE], Status> {
    send().map_err(|error| match error {
        Error::Kernel(code) => Status::from_code(code),
        _ => Status::Protocol,
    })?;
    receive().map_err(|_| Status::Unknown)
}
