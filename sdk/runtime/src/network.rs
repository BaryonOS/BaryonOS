#![forbid(unsafe_code)]
//! Raw NIC transport representation for the isolated network service (ABI 1.5).
//! It conveys no app-level destination grant; protocol policy stays in ring 3.
use crate::Handle;
pub const NET_FRAME_MAX: usize = 1514;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkSource(Handle);
impl NetworkSource {
    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }
    pub const fn handle(self) -> Handle {
        self.0
    }
}
