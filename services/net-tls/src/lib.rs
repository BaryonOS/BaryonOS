#![no_std]
#![forbid(unsafe_code)]
//! Native TLS port prerequisites, not a crypto provider or a transport service.
//!
//! The adapters do not grant endpoints, identity, network or signing rights.
//! No currently implemented clock passes the certificate-time boundary.

pub mod random;
pub mod time;
