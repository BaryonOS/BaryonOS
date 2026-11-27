#![no_std]
#![forbid(unsafe_code)]
//! Native TLS profile, platform adapters and optional B1 cryptographic provider.
//!
//! The adapters do not grant endpoints, identity, network or signing rights.
//! No currently implemented clock passes the certificate-time boundary.
//! `profile` restricts a caller-supplied provider; it selects none itself.

extern crate alloc;

#[cfg(feature = "b1-provider")]
pub mod b1;
pub mod profile;
pub mod random;
pub mod time;
