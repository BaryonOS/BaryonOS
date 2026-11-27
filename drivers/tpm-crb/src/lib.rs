#![no_std]
#![forbid(unsafe_code)]
//! Bounded QEMU TPM2 CRB transport core. No MMIO adapter, crypto or trust-anchor
//! policy lives here. Host Io models and crosscompilation are not native proof.
//! `event_log` parses the firmware TCG2 event log for the REL-05 loader.
pub mod codec;
pub mod event_log;
pub mod transport;
