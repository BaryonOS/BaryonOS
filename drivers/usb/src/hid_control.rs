#![forbid(unsafe_code)]

//! HID 1.11 boot-interface control setup packets (7.2.4 and 7.2.6).
//! Encoding is not device or interface authority. The caller must bind the
//! interface to its own device lifetime and check the actual control completion
//! before treating boot protocol or idle policy as configured.

/// One host-to-device class/interface request with no data stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request {
    setup: [u8; 8],
}

impl Request {
    /// Explicitly select boot protocol, including after a device reset.
    /// A newly initialized device must not be assumed to be in boot protocol.
    pub const fn set_boot_protocol(interface: u8) -> Self {
        Self::encode(0x0b, 0, interface)
    }

    /// Report ID zero applies to all input reports. Duration is in 4 ms units;
    /// zero requests reports only when data changes, not periodic repeats.
    /// Host-side keyboard repeat remains the input consumer's responsibility.
    pub const fn set_idle(interface: u8, duration: u8) -> Self {
        Self::encode(0x0a, (duration as u16) << 8, interface)
    }

    const fn encode(request: u8, value: u16, interface: u8) -> Self {
        let value = value.to_le_bytes();
        Self {
            setup: [0x21, request, value[0], value[1], interface, 0, 0, 0],
        }
    }

    /// USB setup-stage bytes, including interface wIndex, zero high index byte,
    /// and zero wLength. No generic arbitrary control-request constructor.
    pub const fn setup(self) -> [u8; 8] {
        self.setup
    }
}
