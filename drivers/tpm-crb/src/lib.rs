#![no_std]
#![forbid(unsafe_code)]
//! Bounded QEMU TPM2 CRB transport core. No MMIO adapter, crypto or trust-anchor
//! policy lives here. Host Io models and crosscompilation are not native proof.
pub mod codec;
pub mod transport;
