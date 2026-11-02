#![no_std]
//! Trusted userspace window policy. No hardware, storage or address-space access.
#![forbid(unsafe_code)]
#[cfg(test)]
extern crate std;
pub mod boot;
pub mod idle;
pub mod input;
pub mod recovery;
pub mod scene;
pub mod surface;
