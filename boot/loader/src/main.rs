#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

// Each profile expands exactly one loader implementation at crate root.
// This prevents the legacy SMP BootInfo extension from changing the stable ABI.
#[cfg(not(feature = "smp-handoff"))]
include!("main_default.rs");
#[cfg(feature = "smp-handoff")]
include!("main_legacy.rs");
