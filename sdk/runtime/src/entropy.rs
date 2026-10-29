#![forbid(unsafe_code)]
//! Representation for the B01 entropy broker ABI (syscalls 0x60/0x61, ABI
//! 1.4). The value is a handle number; only the kernel checks that it is a
//! live `Entropy` capability of the calling process.
use crate::Handle;

/// Largest number of raw bytes one request may ask for.
pub const ENTROPY_MAX: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntropySource(Handle);

impl EntropySource {
    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }
    pub const fn handle(self) -> Handle {
        self.0
    }
}
