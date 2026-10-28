//! Allocator-free M6c names, catalog protocol and persistent state transitions.
//! Native IPC and I/O live in the separately compiled ring-3 adapter.
#![no_std]
#![forbid(unsafe_code)]

pub mod journal;
pub mod wire;
