#![no_std]
#![forbid(unsafe_code)]

//! The stable M1 wire ABI remains the default; the isolated legacy SMP wire
//! layout is selected only by the explicit opt-in loader/kernel feature.

pub mod acpi;
#[cfg(not(feature = "smp-legacy-wire"))]
pub mod graphics;
#[cfg(feature = "smp-legacy-wire")]
#[path = "graphics_legacy.rs"]
pub mod graphics;
#[cfg(feature = "smp-legacy-wire")]
mod legacy;
#[cfg(feature = "smp-legacy-wire")]
pub mod smp;
#[cfg(not(feature = "smp-legacy-wire"))]
mod stable;
#[cfg(feature = "smp-legacy-wire")]
pub use legacy::*;
#[cfg(not(feature = "smp-legacy-wire"))]
pub use stable::*;
