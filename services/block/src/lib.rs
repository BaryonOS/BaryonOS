#![no_std]
#![forbid(unsafe_code)]

pub mod wire;
pub use wire as legacy_wire;
pub mod journal;
pub mod m6_wire;
