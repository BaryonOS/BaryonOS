#![no_std]
//! Trusted userspace window policy. No hardware, storage or address-space access.
#![forbid(unsafe_code)]
#[cfg(test)]
extern crate std;
pub mod boot;
pub mod input;
pub mod scene;
