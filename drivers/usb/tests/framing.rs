//! Pure transfer-byte fixtures; no controller, enumeration or hardware I/O.
use nieuw_os_usb_core::{DescriptorIter, ParseError};

#[test]
fn every_encodable_length_and_transfer_cut_has_an_exact_boundary_result() {
    for prefix_count in [0usize, 1, 7] {
        let offset = prefix_count * 2;
        for length in 0..=u8::MAX {
            let size = usize::from(length).max(2);
            let mut bytes = vec![0x5a; offset + size];
            for prefix in bytes[..offset].chunks_exact_mut(2) {
                prefix.copy_from_slice(&[2, 0xee]);
            }
            bytes[offset] = length;
            bytes[offset + 1] = 0xff;
            for remaining in 0..=size {
                let transfer = &bytes[..offset + remaining];
                let mut iter = DescriptorIter::new(transfer);
                for index in 0..prefix_count {
                    let descriptor = iter.next().unwrap().unwrap();
                    assert_eq!(descriptor.raw_bytes(), &transfer[index * 2..index * 2 + 2]);
                }
                let expected_error = if remaining == 0 {
                    None
                } else if remaining == 1 {
                    Some(ParseError::TruncatedHeader { offset, remaining })
                } else if length < 2 {
                    Some(ParseError::LengthTooShort { offset, length })
                } else if remaining < usize::from(length) {
                    Some(ParseError::TruncatedDescriptor {
                        offset,
                        length,
                        remaining,
                    })
                } else {
                    let descriptor = iter.next().unwrap().unwrap();
                    assert_eq!(descriptor.raw_bytes(), &transfer[offset..]);
                    assert_eq!(descriptor.raw_bytes().as_ptr(), transfer[offset..].as_ptr());
                    assert_eq!(descriptor.descriptor_type(), 0xff);
                    assert_eq!(descriptor.payload(), &transfer[offset + 2..]);
                    None
                };
                if let Some(error) = expected_error {
                    assert_eq!(
                        iter.next(),
                        Some(Err(error)),
                        "length={length} remaining={remaining} offset={offset}"
                    );
                }
                for _ in 0..3 {
                    assert_eq!(iter.next(), None);
                }
            }
        }
    }
}

#[test]
fn malformed_framing_never_exposes_a_valid_looking_suffix() {
    for prefix_count in [0usize, 1, 7] {
        let offset = prefix_count * 2;
        for length in 0..=u8::MAX {
            if (2..=4).contains(&length) {
                continue;
            }
            let mut transfer = [2, 0xee].repeat(prefix_count);
            transfer.extend_from_slice(&[length, 0xff, 2, 0x99]);
            let mut iter = DescriptorIter::new(&transfer);
            for _ in 0..prefix_count {
                assert!(iter.next().unwrap().is_ok());
            }
            let error = if length < 2 {
                ParseError::LengthTooShort { offset, length }
            } else {
                ParseError::TruncatedDescriptor {
                    offset,
                    length,
                    remaining: 4,
                }
            };
            assert_eq!(iter.next(), Some(Err(error)));
            assert_eq!(iter.count(), 0);
        }
    }
}
