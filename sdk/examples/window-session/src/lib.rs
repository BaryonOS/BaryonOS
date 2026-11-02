#![no_std]
//! Trusted userspace window policy. No hardware, storage or address-space access.
#![forbid(unsafe_code)]
#[cfg(test)]
extern crate std;
pub mod boot;
pub mod clipboard;
pub mod idle;
pub mod input;
pub mod recovery;
pub mod scene;
pub mod surface;
// One interaction contract shared with the existing native preview. This is
// not an authentication provider and confers no kernel authority.
#[path = "../../../../ui/native/src/session.rs"]
pub mod session;
pub mod shell;
