#![no_std]

//! Bounded USB descriptor framing and interface-class matching.
//! Device I/O and class-driver policy belong to separate layers.

use core::iter::FusedIterator;

/// A descriptor borrowed from a validated byte range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Descriptor<'a> {
    raw: &'a [u8],
}

impl<'a> Descriptor<'a> {
    /// Returns the descriptor type byte.
    pub fn descriptor_type(self) -> u8 {
        self.raw[1]
    }

    /// Returns the complete descriptor, including its two-byte header.
    pub fn raw_bytes(self) -> &'a [u8] {
        self.raw
    }

    /// Returns descriptor bytes following `bLength` and `bDescriptorType`.
    pub fn payload(self) -> &'a [u8] {
        &self.raw[2..]
    }
}

/// USB descriptor type for an interface descriptor.
pub const INTERFACE_DESCRIPTOR_TYPE: u8 = 4;
/// Required byte length of a standard USB interface descriptor.
pub const INTERFACE_DESCRIPTOR_LENGTH: usize = 9;

/// A decoded standard USB interface descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterfaceDescriptor {
    /// Interface number within the active configuration.
    pub number: u8,
    /// Alternate setting described by this descriptor.
    pub alternate_setting: u8,
    /// Number of endpoint descriptors following this interface descriptor.
    pub endpoint_count: u8,
    /// USB interface class code.
    pub class_code: u8,
    /// USB interface subclass code.
    pub subclass_code: u8,
    /// USB interface protocol code.
    pub protocol_code: u8,
    /// String descriptor index, or zero when no string is provided.
    pub string_index: u8,
}

/// Why a framed descriptor could not be decoded as a standard interface
/// descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterfaceDescriptorError {
    /// The descriptor is not an interface descriptor.
    WrongDescriptorType { actual: u8 },
    /// The descriptor does not have the standard interface-descriptor length.
    InvalidLength { actual: usize },
}

impl InterfaceDescriptor {
    /// Decodes the standard fields from an already-framed interface descriptor.
    pub fn parse(descriptor: Descriptor<'_>) -> Result<Self, InterfaceDescriptorError> {
        let actual_type = descriptor.descriptor_type();
        if actual_type != INTERFACE_DESCRIPTOR_TYPE {
            return Err(InterfaceDescriptorError::WrongDescriptorType {
                actual: actual_type,
            });
        }

        let bytes = descriptor.raw_bytes();
        if bytes.len() != INTERFACE_DESCRIPTOR_LENGTH {
            return Err(InterfaceDescriptorError::InvalidLength {
                actual: bytes.len(),
            });
        }

        Ok(Self {
            number: bytes[2],
            alternate_setting: bytes[3],
            endpoint_count: bytes[4],
            class_code: bytes[5],
            subclass_code: bytes[6],
            protocol_code: bytes[7],
            string_index: bytes[8],
        })
    }

    /// Returns whether this interface satisfies a class/subclass/protocol selector.
    pub const fn matches(self, selector: InterfaceClassMatch) -> bool {
        self.class_code == selector.class_code
            && match selector.subclass_code {
                Some(code) => self.subclass_code == code,
                None => true,
            }
            && match selector.protocol_code {
                Some(code) => self.protocol_code == code,
                None => true,
            }
    }
}

/// USB interface class selector; absent subclass/protocol fields are wildcards.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterfaceClassMatch {
    /// Required interface class.
    pub class_code: u8,
    /// Required subclass, or `None` for any subclass.
    pub subclass_code: Option<u8>,
    /// Required protocol, or `None` for any protocol.
    pub protocol_code: Option<u8>,
}

/// A malformed descriptor boundary in the supplied transfer bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// Fewer than the two required header bytes remain.
    TruncatedHeader { offset: usize, remaining: usize },
    /// `bLength` is too small to contain the header.
    LengthTooShort { offset: usize, length: u8 },
    /// `bLength` extends beyond the supplied transfer bytes.
    TruncatedDescriptor {
        offset: usize,
        length: u8,
        remaining: usize,
    },
    /// Advancing the byte offset would overflow `usize`.
    OffsetOverflow { offset: usize, length: u8 },
}

/// Iterates over descriptors without allocating or resynchronizing on errors.
#[derive(Debug)]
pub struct DescriptorIter<'a> {
    bytes: &'a [u8],
    offset: usize,
    failed: bool,
}

impl<'a> DescriptorIter<'a> {
    /// Creates an iterator over one already-bounded transfer result.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            failed: false,
        }
    }

    fn fail(&mut self, error: ParseError) -> Option<Result<Descriptor<'a>, ParseError>> {
        self.failed = true;
        Some(Err(error))
    }
}

impl<'a> Iterator for DescriptorIter<'a> {
    type Item = Result<Descriptor<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed || self.offset == self.bytes.len() {
            return None;
        }

        let remaining = self.bytes.len() - self.offset;
        if remaining < 2 {
            return self.fail(ParseError::TruncatedHeader {
                offset: self.offset,
                remaining,
            });
        }

        let length = self.bytes[self.offset];
        if length < 2 {
            return self.fail(ParseError::LengthTooShort {
                offset: self.offset,
                length,
            });
        }

        let Some(end) = self.offset.checked_add(usize::from(length)) else {
            return self.fail(ParseError::OffsetOverflow {
                offset: self.offset,
                length,
            });
        };
        if end > self.bytes.len() {
            return self.fail(ParseError::TruncatedDescriptor {
                offset: self.offset,
                length,
                remaining,
            });
        }

        let descriptor = Descriptor {
            raw: &self.bytes[self.offset..end],
        };
        self.offset = end;
        Some(Ok(descriptor))
    }
}

impl FusedIterator for DescriptorIter<'_> {}

#[cfg(test)]
mod tests {
    use super::{
        DescriptorIter, InterfaceClassMatch, InterfaceDescriptor, InterfaceDescriptorError,
        ParseError,
    };

    #[test]
    fn empty_transfer_has_no_descriptors() {
        assert_eq!(DescriptorIter::new(&[]).next(), None);
    }

    #[test]
    fn parses_borrowed_back_to_back_and_unknown_descriptors() {
        let bytes = [2, 0x99, 4, 0xfe, 0xaa, 0xbb];
        let mut descriptors = DescriptorIter::new(&bytes);

        let first = descriptors.next().unwrap().unwrap();
        assert_eq!(first.descriptor_type(), 0x99);
        assert_eq!(first.raw_bytes(), &bytes[..2]);
        assert_eq!(first.payload(), &[]);

        let second = descriptors.next().unwrap().unwrap();
        assert_eq!(second.descriptor_type(), 0xfe);
        assert_eq!(second.raw_bytes(), &bytes[2..]);
        assert_eq!(second.payload(), &[0xaa, 0xbb]);
        assert_eq!(descriptors.next(), None);
    }

    #[test]
    fn rejects_incomplete_header_and_then_stops() {
        let mut descriptors = DescriptorIter::new(&[2, 0x80, 1]);
        assert!(descriptors.next().unwrap().is_ok());
        assert_eq!(
            descriptors.next(),
            Some(Err(ParseError::TruncatedHeader {
                offset: 2,
                remaining: 1
            }))
        );
        assert_eq!(descriptors.next(), None);
    }

    #[test]
    fn rejects_lengths_smaller_than_header() {
        for length in [0, 1] {
            assert_eq!(
                DescriptorIter::new(&[length, 0x80]).next(),
                Some(Err(ParseError::LengthTooShort { offset: 0, length }))
            );
        }
    }

    #[test]
    fn rejects_descriptor_truncated_by_transfer_boundary() {
        assert_eq!(
            DescriptorIter::new(&[4, 0x81, 0xaa]).next(),
            Some(Err(ParseError::TruncatedDescriptor {
                offset: 0,
                length: 4,
                remaining: 3
            }))
        );
    }

    #[test]
    fn accepts_minimum_and_maximum_encodable_lengths() {
        let minimum = [2, 1];
        assert_eq!(
            DescriptorIter::new(&minimum)
                .next()
                .unwrap()
                .unwrap()
                .raw_bytes(),
            &minimum
        );

        let mut maximum = [0; u8::MAX as usize];
        maximum[0] = u8::MAX;
        maximum[1] = 2;
        let descriptor = DescriptorIter::new(&maximum).next().unwrap().unwrap();
        assert_eq!(descriptor.raw_bytes().len(), usize::from(u8::MAX));
        assert_eq!(descriptor.descriptor_type(), 2);
    }

    #[test]
    fn never_resynchronizes_after_malformed_descriptor() {
        let bytes = [2, 1, 0, 2, 2, 0xaa];
        let mut descriptors = DescriptorIter::new(&bytes);
        assert!(descriptors.next().unwrap().is_ok());
        assert!(descriptors.next().unwrap().is_err());
        assert_eq!(descriptors.next(), None);
    }

    #[test]
    fn decodes_standard_interface_fields_and_matches_class_tuple() {
        let bytes = [9, 4, 2, 1, 3, 3, 1, 1, 7];
        let descriptor = DescriptorIter::new(&bytes).next().unwrap().unwrap();
        let interface = InterfaceDescriptor::parse(descriptor).unwrap();
        assert_eq!(
            interface,
            InterfaceDescriptor {
                number: 2,
                alternate_setting: 1,
                endpoint_count: 3,
                class_code: 3,
                subclass_code: 1,
                protocol_code: 1,
                string_index: 7,
            }
        );
        assert!(interface.matches(InterfaceClassMatch {
            class_code: 3,
            subclass_code: Some(1),
            protocol_code: Some(1),
        }));
        assert!(!interface.matches(InterfaceClassMatch {
            class_code: 3,
            subclass_code: Some(1),
            protocol_code: Some(2),
        }));
    }

    #[test]
    fn class_matching_can_ignore_subclass_and_protocol() {
        let bytes = [9, 4, 0, 0, 1, 8, 6, 80, 0];
        let descriptor = DescriptorIter::new(&bytes).next().unwrap().unwrap();
        let interface = InterfaceDescriptor::parse(descriptor).unwrap();
        assert!(interface.matches(InterfaceClassMatch {
            class_code: 8,
            subclass_code: None,
            protocol_code: None,
        }));
        assert!(interface.matches(InterfaceClassMatch {
            class_code: 8,
            subclass_code: Some(6),
            protocol_code: None,
        }));
        assert!(!interface.matches(InterfaceClassMatch {
            class_code: 3,
            subclass_code: None,
            protocol_code: None,
        }));
    }

    #[test]
    fn rejects_wrong_type_and_nonstandard_interface_length() {
        let endpoint = [7, 5, 0x81, 3, 8, 0, 10];
        let descriptor = DescriptorIter::new(&endpoint).next().unwrap().unwrap();
        assert_eq!(
            InterfaceDescriptor::parse(descriptor),
            Err(InterfaceDescriptorError::WrongDescriptorType { actual: 5 })
        );

        let truncated = [8, 4, 0, 0, 0, 3, 0, 0];
        let descriptor = DescriptorIter::new(&truncated).next().unwrap().unwrap();
        assert_eq!(
            InterfaceDescriptor::parse(descriptor),
            Err(InterfaceDescriptorError::InvalidLength { actual: 8 })
        );

        let extended = [10, 4, 0, 0, 0, 3, 0, 0, 0, 0];
        let descriptor = DescriptorIter::new(&extended).next().unwrap().unwrap();
        assert_eq!(
            InterfaceDescriptor::parse(descriptor),
            Err(InterfaceDescriptorError::InvalidLength { actual: 10 })
        );
    }
}
