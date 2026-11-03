#![forbid(unsafe_code)]

//! Bounded USB HID boot-interface selection from one configuration snapshot.
//! Metadata only: no device identity, SET_PROTOCOL, transfer, or input authority.
//! This is a low/full-speed profile, not a high/super-speed endpoint parser.

use crate::{DescriptorIter, ParseError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Protocol {
    Keyboard,
    Mouse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Speed {
    Low,
    Full,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Configuration,
    Framing(ParseError),
    Interface,
    NotBoot,
    MissingInterface,
    DuplicateInterface,
    Endpoint,
    EndpointCount,
    DuplicateEndpoint,
    AmbiguousInput,
    MissingInput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootInterface {
    pub configuration_value: u8,
    pub interface_number: u8,
    pub protocol: Protocol,
    pub input_endpoint: u8,
    pub max_packet_size: u16,
    pub interval_ms: u8,
}

struct Selected {
    binding: BootInterface,
    expected: u8,
    seen: u8,
    addresses: u32,
    input: bool,
}

impl Selected {
    fn endpoint(&mut self, bytes: &[u8], speed: Speed) -> Result<(), Error> {
        if bytes.len() != 7 {
            return Err(Error::Endpoint);
        }
        if self.seen >= self.expected {
            return Err(Error::EndpointCount);
        }
        self.seen += 1;
        let address = bytes[2];
        let number = address & 15;
        let packet = u16::from_le_bytes([bytes[4], bytes[5]]);
        let limit = match speed {
            Speed::Low => 8,
            Speed::Full => 64,
        };
        if number == 0
            || address & 0x70 != 0
            || bytes[3] != 3
            || packet == 0
            || packet > limit
            || bytes[6] == 0
        {
            return Err(Error::Endpoint);
        }
        let input = address & 0x80 != 0;
        let bit = 1u32 << (u32::from(number) + if input { 16 } else { 0 });
        if self.addresses & bit != 0 {
            return Err(Error::DuplicateEndpoint);
        }
        self.addresses |= bit;
        if input {
            let minimum = match self.binding.protocol {
                Protocol::Keyboard => 8,
                Protocol::Mouse => 3,
            };
            if packet < minimum {
                return Err(Error::Endpoint);
            }
            if self.input {
                return Err(Error::AmbiguousInput);
            }
            self.input = true;
            self.binding.input_endpoint = address;
            self.binding.max_packet_size = packet;
            self.binding.interval_ms = bytes[6];
        }
        Ok(())
    }

    fn finish(&self) -> Result<BootInterface, Error> {
        if self.seen != self.expected {
            return Err(Error::EndpointCount);
        }
        if !self.input {
            return Err(Error::MissingInput);
        }
        Ok(self.binding)
    }
}

/// Selects only alternate setting zero of the requested interface. The entire
/// transfer must equal wTotalLength; malformed tails cannot hide behind an
/// earlier match. HID/report-descriptor semantics and device identity still
/// need independent validation by the enumeration owner.
pub fn select(bytes: &[u8], interface_number: u8, speed: Speed) -> Result<BootInterface, Error> {
    if bytes.len() < 9
        || bytes[0] != 9
        || bytes[1] != 2
        || usize::from(u16::from_le_bytes([bytes[2], bytes[3]])) != bytes.len()
        || bytes[4] == 0
        || bytes[5] == 0
    {
        return Err(Error::Configuration);
    }
    let mut selected: Option<Selected> = None;
    let mut result = None;
    let mut active = false;
    for descriptor in DescriptorIter::new(&bytes[9..]) {
        let descriptor = descriptor.map_err(Error::Framing)?;
        let raw = descriptor.raw_bytes();
        match descriptor.descriptor_type() {
            2 => return Err(Error::Configuration),
            4 => {
                if active {
                    result = Some(selected.as_ref().ok_or(Error::Interface)?.finish()?);
                    active = false;
                }
                if raw.len() != 9 {
                    return Err(Error::Interface);
                }
                if raw[2] != interface_number || raw[3] != 0 {
                    continue;
                }
                if selected.is_some() {
                    return Err(Error::DuplicateInterface);
                }
                if raw[5] != 3 || raw[6] != 1 {
                    return Err(Error::NotBoot);
                }
                let protocol = match raw[7] {
                    1 => Protocol::Keyboard,
                    2 => Protocol::Mouse,
                    _ => return Err(Error::NotBoot),
                };
                selected = Some(Selected {
                    binding: BootInterface {
                        configuration_value: bytes[5],
                        interface_number,
                        protocol,
                        input_endpoint: 0,
                        max_packet_size: 0,
                        interval_ms: 0,
                    },
                    expected: raw[4],
                    seen: 0,
                    addresses: 0,
                    input: false,
                });
                active = true;
            }
            5 if active => selected
                .as_mut()
                .ok_or(Error::Interface)?
                .endpoint(raw, speed)?,
            _ => {}
        }
    }
    if active {
        result = Some(selected.as_ref().ok_or(Error::Interface)?.finish()?);
    }
    result.ok_or(Error::MissingInterface)
}
