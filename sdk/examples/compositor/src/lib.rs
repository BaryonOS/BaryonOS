#![no_std]
#![forbid(unsafe_code)]
//! Single assigned surface, real display client and authenticated focus RPC.
//! Production owns no personal storage, raw input, network or renderer controller.
pub mod boot;
pub mod chrome;
pub mod control;
#[path = "../../../../ui/native/src/design.rs"]
pub mod design;
pub mod surface;
