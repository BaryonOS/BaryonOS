#![forbid(unsafe_code)]
//! ABI 1.6 boot-monotonic PIT clock. No walltime or certificate-date semantics.
/// Nominal ticks per second in the supported single-CPU QEMU profile.
pub const MONOTONIC_HZ: u64 = 100;
