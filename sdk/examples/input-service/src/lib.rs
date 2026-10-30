#![no_std]
#![forbid(unsafe_code)]
//! Serialized ring3 daemon adapters, not kernel provenance or host input.
//! Production IO comes from native SDK calls. Host transports test only the
//! bounded adapter state machine and real capability-engine envelope contract.
pub mod boot;
pub mod policy;
pub mod protocol;
pub mod source;

use nieuw_os_sdk_runtime::{Error, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint};

pub trait Ipc {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), Error>;
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, Error>;
    fn close(&mut self, handle: Handle) -> Result<(), Error>;
}
pub fn discard_caps(io: &mut impl Ipc, message: &Received) -> Result<(), Error> {
    let mut first = None;
    for index in 0..message.cap_count() {
        if let Some(cap) = message.cap(index) {
            if let Err(error) = io.close(cap.handle) {
                first.get_or_insert(error);
            }
        }
    }
    first.map_or(Ok(()), Err)
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub struct NativeIpc;
#[cfg(all(target_arch = "x86_64", target_os = "none"))]
impl Ipc for NativeIpc {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), Error> {
        nieuw_os_sdk_runtime::native::send(endpoint, message)
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, Error> {
        nieuw_os_sdk_runtime::native::receive(endpoint)
    }
    fn close(&mut self, handle: Handle) -> Result<(), Error> {
        nieuw_os_sdk_runtime::native::close(handle)
    }
}
