#![no_std]
#![forbid(unsafe_code)]
//! Bounded display/input transport contract. `wire` alone has no SDK/runtime
//! dependency and can be shared with the kernel's fixed hardware adapters.
#[cfg(feature = "client")]
mod client;
pub mod wire;
#[cfg(feature = "client")]
pub use client::*;
