#![no_std]

//! Bounded USB descriptor framing. Device I/O and descriptor policy belong to
//! separate layers.

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
    use super::{DescriptorIter, ParseError};

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
}
