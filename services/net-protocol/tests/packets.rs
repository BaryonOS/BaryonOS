//! Fixed synthetic packets and hostile parser inputs; no network or VM.
use nieuw_os_net_protocol::{parse, Error, Layer, UdpChecksum, Unsupported, MAX_IPV4_BYTES};

// Values derived independently in docs/verification/parallel-b11/vectors.py.
// Tests never regenerate these expected packet bytes with the library checksum.
const ODD: &str = "0200000000020200000000010800450000231234400040113c5fc0000201c63364023039d431000f70ff6e696575774f53";
const EVEN: &str = "0200000000020200000000010800450000241234400040113c5ec0000201c63364023039d4310010425b3031323334353637";
const EMPTY: &str =
    "02000000000202000000000108004500001c1234400040113c66c0000201c63364023039d43100080f3c";
const ABSENT: &str = "0200000000020200000000010800450000231234400040113c5fc0000201c63364023039d431000f00006e696575774f53";
const ZERO_AS_FFFF: &str =
    "02000000000202000000000108004500001e1234400040113c64c0000201c63364023039d431000affff0f38";

fn bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}

// Host-only reference used to keep deliberately changed headers well formed.
// Sum all words into u64 before folding, unlike the library's streaming fold.
fn reference_checksum(data: &[u8]) -> u16 {
    let mut sum: u64 = data
        .chunks(2)
        .map(|pair| (u64::from(pair[0]) << 8) | u64::from(*pair.get(1).unwrap_or(&0)))
        .sum();
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

fn fix_ip(frame: &mut [u8]) {
    frame[24..26].fill(0);
    let checksum = reference_checksum(&frame[14..34]);
    frame[24..26].copy_from_slice(&checksum.to_be_bytes());
}

fn fixture(payload: &[u8]) -> Vec<u8> {
    let mut frame = bytes(EMPTY);
    frame.extend_from_slice(payload);
    let total: u16 = (28 + payload.len()).try_into().unwrap();
    let udp_length = total - 20;
    frame[16..18].copy_from_slice(&total.to_be_bytes());
    frame[38..40].copy_from_slice(&udp_length.to_be_bytes());
    frame[40..42].fill(0);
    let mut pseudo = frame[26..34].to_vec();
    pseudo.extend_from_slice(&[0, 17]);
    pseudo.extend_from_slice(&udp_length.to_be_bytes());
    pseudo.extend_from_slice(&frame[34..]);
    let checksum = reference_checksum(&pseudo);
    frame[40..42].copy_from_slice(&if checksum == 0 { 0xffff } else { checksum }.to_be_bytes());
    fix_ip(&mut frame);
    frame
}

#[test]
fn fixed_odd_even_empty_and_negative_zero_vectors() {
    for (hex, payload, checksum) in [
        (ODD, b"nieuwOS".as_slice(), 0x70ff),
        (EVEN, b"01234567".as_slice(), 0x425b),
        (EMPTY, b"".as_slice(), 0x0f3c),
        (ZERO_AS_FFFF, [0x0f, 0x38].as_slice(), 0xffff),
    ] {
        let raw = bytes(hex);
        let packet = parse(&raw).unwrap();
        assert_eq!(packet.payload(), payload);
        assert_eq!(packet.checksum(), UdpChecksum::Valid(checksum));
    }
}

#[test]
fn successful_parse_preserves_every_input_byte_and_borrows_payload() {
    let raw = bytes(ODD);
    let before = raw.clone();
    let packet = parse(&raw).unwrap();
    assert_eq!(raw, before);
    assert_eq!(packet.frame(), &raw);
    assert_eq!(packet.ethernet_header(), &raw[..14]);
    assert_eq!(packet.ipv4_packet(), &raw[14..]);
    assert_eq!(packet.udp_packet(), &raw[34..]);
    assert_eq!(packet.payload().as_ptr(), raw[42..].as_ptr());
    assert_eq!(packet.source_mac(), [2, 0, 0, 0, 0, 1]);
    assert_eq!(packet.destination_mac(), [2, 0, 0, 0, 0, 2]);
    assert_eq!(packet.source_address(), [192, 0, 2, 1]);
    assert_eq!(packet.destination_address(), [198, 51, 100, 2]);
    assert_eq!(
        (packet.source_port(), packet.destination_port()),
        (12345, 54321)
    );
    assert_eq!(packet.ttl(), 64);
    assert!(packet.dont_fragment());
    assert!(packet.ethernet_padding().is_empty());
}

#[test]
fn every_truncated_offset_of_each_fixed_packet_is_rejected() {
    for hex in [ODD, EVEN, EMPTY, ABSENT, ZERO_AS_FFFF] {
        let raw = bytes(hex);
        for cut in 0..raw.len() {
            assert!(
                matches!(parse(&raw[..cut]), Err(Error::Truncated { .. })),
                "accepted or misclassified cut {cut} of {}",
                raw.len()
            );
        }
    }
}

#[test]
fn ipv4_optional_zero_udp_checksum_is_visible_not_verified() {
    let mut raw = bytes(ABSENT);
    assert_eq!(parse(&raw).unwrap().checksum(), UdpChecksum::Absent);
    raw[42] ^= 0x80;
    assert_eq!(parse(&raw).unwrap().checksum(), UdpChecksum::Absent);
    raw[22] ^= 1;
    assert_eq!(parse(&raw), Err(Error::InvalidIpv4Checksum));
}

#[test]
fn every_single_bit_error_in_the_ipv4_header_is_rejected() {
    let raw = bytes(ODD);
    for offset in 14..34 {
        for bit in 0..8 {
            let mut changed = raw.clone();
            changed[offset] ^= 1 << bit;
            assert!(parse(&changed).is_err(), "accepted {offset}:{bit}");
        }
    }
}

#[test]
fn every_single_bit_error_in_udp_header_and_odd_payload_is_rejected() {
    let raw = bytes(ODD);
    for offset in 34..raw.len() {
        for bit in 0..8 {
            let mut changed = raw.clone();
            changed[offset] ^= 1 << bit;
            assert!(parse(&changed).is_err(), "accepted {offset}:{bit}");
        }
    }
}

#[test]
fn pseudoheader_binds_both_ipv4_addresses_even_after_ip_checksum_repair() {
    let raw = bytes(ODD);
    for offset in 26..34 {
        for bit in 0..8 {
            let mut changed = raw.clone();
            changed[offset] ^= 1 << bit;
            fix_ip(&mut changed);
            assert_eq!(parse(&changed), Err(Error::InvalidUdpChecksum));
        }
    }
}

#[test]
fn unsupported_ethernet_types_and_vlan_tags_are_explicit() {
    let original = bytes(ODD);
    for tag in [0x8100u16, 0x88a8] {
        let mut raw = original.clone();
        raw[12..14].copy_from_slice(&tag.to_be_bytes());
        assert_eq!(parse(&raw), Err(Error::Unsupported(Unsupported::Vlan(tag))));
    }
    for other in [0u16, 46, 1500, 0x0806, 0x86dd, 0xffff] {
        let mut raw = original.clone();
        raw[12..14].copy_from_slice(&other.to_be_bytes());
        assert_eq!(
            parse(&raw),
            Err(Error::Unsupported(Unsupported::EtherType(other)))
        );
    }
}

#[test]
fn ipv4_versions_and_all_ihl_values_have_explicit_errors() {
    let original = bytes(ODD);
    for version in 0..=15 {
        if version == 4 {
            continue;
        }
        let mut raw = original.clone();
        raw[14] = (version << 4) | 5;
        assert_eq!(
            parse(&raw),
            Err(Error::Unsupported(Unsupported::IpVersion(version)))
        );
    }
    for ihl in 0..=15 {
        if ihl == 5 {
            continue;
        }
        let mut raw = original.clone();
        raw[14] = 0x40 | ihl;
        let expected = if ihl < 5 {
            Error::InvalidIpv4HeaderLength(ihl)
        } else {
            Error::Unsupported(Unsupported::Ipv4Options(ihl))
        };
        assert_eq!(parse(&raw), Err(expected));
    }
}

#[test]
fn fragment_offsets_more_fragments_and_reserved_flag_are_rejected_but_df_is_allowed() {
    for flags in [1u16, 0x1fff, 0x2000, 0x2001, 0x6000] {
        let mut raw = bytes(ODD);
        raw[20..22].copy_from_slice(&flags.to_be_bytes());
        fix_ip(&mut raw);
        assert_eq!(
            parse(&raw),
            Err(Error::Unsupported(Unsupported::Ipv4Fragmentation(flags)))
        );
    }
    for flags in [0x8000u16, 0xc000, 0xffff] {
        let mut raw = bytes(ODD);
        raw[20..22].copy_from_slice(&flags.to_be_bytes());
        fix_ip(&mut raw);
        assert_eq!(parse(&raw), Err(Error::InvalidIpv4Flags(flags)));
    }
    for flags in [0u16, 0x4000] {
        let mut raw = bytes(ODD);
        raw[20..22].copy_from_slice(&flags.to_be_bytes());
        fix_ip(&mut raw);
        assert_eq!(parse(&raw).unwrap().dont_fragment(), flags != 0);
    }
}

#[test]
fn only_udp_protocol_is_supported() {
    for protocol in [0, 1, 6, 255] {
        let mut raw = bytes(ODD);
        raw[23] = protocol;
        fix_ip(&mut raw);
        assert_eq!(
            parse(&raw),
            Err(Error::Unsupported(Unsupported::IpProtocol(protocol)))
        );
    }
}

#[test]
fn ipv4_total_length_cannot_underflow_trust_padding_or_exceed_available_bytes() {
    for length in [0u16, 1, 19] {
        let mut raw = bytes(ODD);
        raw[16..18].copy_from_slice(&length.to_be_bytes());
        fix_ip(&mut raw);
        assert_eq!(parse(&raw), Err(Error::InvalidIpv4TotalLength(length)));
    }
    for length in 20u16..28 {
        let mut raw = bytes(ODD);
        raw[16..18].copy_from_slice(&length.to_be_bytes());
        fix_ip(&mut raw);
        assert_eq!(
            parse(&raw),
            Err(Error::Truncated {
                layer: Layer::Udp,
                needed: 8,
                available: usize::from(length - 20)
            })
        );
    }
    for length in [36u16, 256, u16::MAX] {
        let mut raw = bytes(ODD);
        raw[16..18].copy_from_slice(&length.to_be_bytes());
        fix_ip(&mut raw);
        assert_eq!(
            parse(&raw),
            Err(Error::Truncated {
                layer: Layer::Ipv4,
                needed: usize::from(length),
                available: 35
            })
        );
    }
}

#[test]
fn udp_length_must_cover_exactly_the_ipv4_payload() {
    for declared in [0u16, 1, 7, 8, 14, 16, u16::MAX] {
        let mut raw = bytes(ODD);
        raw[38..40].copy_from_slice(&declared.to_be_bytes());
        let expected = if declared < 8 {
            Error::InvalidUdpLength(declared)
        } else {
            Error::UdpLengthMismatch {
                declared,
                ip_payload: 15,
            }
        };
        assert_eq!(parse(&raw), Err(expected));
    }
    let mut raw = bytes(ODD);
    raw.push(0);
    raw[16..18].copy_from_slice(&36u16.to_be_bytes());
    fix_ip(&mut raw);
    assert_eq!(
        parse(&raw),
        Err(Error::UdpLengthMismatch {
            declared: 15,
            ip_payload: 16
        })
    );
}

#[test]
fn ethernet_minimum_padding_is_separate_and_extra_trailers_are_rejected() {
    let original = bytes(ODD);
    let mut padded = original.clone();
    padded.resize(60, 0xa5); // Receiver does not treat link padding as UDP data.
    let packet = parse(&padded).unwrap();
    assert_eq!(packet.payload(), b"nieuwOS");
    assert_eq!(packet.ipv4_packet(), &original[14..]);
    assert_eq!(packet.ethernet_padding(), &[0xa5; 11]);
    for length in 50..=65 {
        if length == 60 {
            continue;
        }
        let mut changed = original.clone();
        changed.resize(length, 0);
        assert_eq!(
            parse(&changed),
            Err(Error::UnexpectedEthernetLength {
                actual: length,
                datagram_end: 49,
                padded_end: 60
            })
        );
    }
}

#[test]
fn maximum_ipv4_length_is_bounded_without_udp_length_wraparound() {
    let payload = vec![0xff; MAX_IPV4_BYTES - 20 - 8];
    let mut raw = fixture(&payload);
    assert_eq!(raw.len(), 14 + MAX_IPV4_BYTES);
    let packet = parse(&raw).unwrap();
    assert_eq!(packet.ipv4_packet().len(), MAX_IPV4_BYTES);
    assert_eq!(packet.udp_packet().len(), MAX_IPV4_BYTES - 20);
    assert_eq!(packet.payload(), payload);
    raw[38..40].copy_from_slice(&u16::MAX.to_be_bytes());
    assert_eq!(
        parse(&raw),
        Err(Error::UdpLengthMismatch {
            declared: u16::MAX,
            ip_payload: MAX_IPV4_BYTES - 20
        })
    );
}

#[test]
fn source_port_zero_and_link_addresses_are_data_not_authority() {
    let mut raw = bytes(ABSENT);
    raw[34..36].fill(0); // RFC 768 explicitly permits an absent source port.
    raw[6..12].copy_from_slice(&[0xff; 6]);
    let packet = parse(&raw).unwrap();
    assert_eq!(packet.source_port(), 0);
    assert_eq!(packet.source_mac(), [0xff; 6]);
    assert_eq!(packet.checksum(), UdpChecksum::Absent);
    // No authenticated peer, socket handle or capability is returned by parse.
}
