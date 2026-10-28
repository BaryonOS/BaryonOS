//! Bounded M6b journal and wire policy. The actual ring-3 adapter is compiled
//! separately into the native user ELF; this host-testable core performs no I/O.
#![no_std]
#![forbid(unsafe_code)]

pub mod journal;
pub mod wire;
