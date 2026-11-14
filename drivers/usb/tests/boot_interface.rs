use nieuw_os_usb_core::boot_interface::{select, Error, Protocol, Speed};

fn configuration(body: &[u8]) -> Vec<u8> {
    let length = u16::try_from(body.len() + 9).unwrap().to_le_bytes();
    let mut bytes = vec![9, 2, length[0], length[1], 2, 1, 0, 0x80, 50];
    bytes.extend_from_slice(body);
    bytes
}

const KEYBOARD: [u8; 9] = [9, 4, 0, 0, 1, 3, 1, 1, 0];
const HID: [u8; 9] = [9, 0x21, 0x11, 1, 0, 1, 0x22, 63, 0];
const INPUT: [u8; 7] = [7, 5, 0x81, 3, 8, 0, 10];

#[test]
fn boot_binding_requires_its_own_hid_descriptor_before_endpoints() {
    let missing = configuration(&[KEYBOARD.as_slice(), INPUT.as_slice()].concat());
    assert!(select(&missing, 0, Speed::Full).is_err());

    let mut other = KEYBOARD;
    other[2] = 1;
    for body in [
        [KEYBOARD.as_slice(), INPUT.as_slice(), HID.as_slice()].concat(),
        [
            KEYBOARD.as_slice(),
            HID.as_slice(),
            HID.as_slice(),
            INPUT.as_slice(),
        ]
        .concat(),
        [
            other.as_slice(),
            HID.as_slice(),
            INPUT.as_slice(),
            KEYBOARD.as_slice(),
            INPUT.as_slice(),
        ]
        .concat(),
    ] {
        assert_eq!(
            select(&configuration(&body), 0, Speed::Full),
            Err(Error::HidDescriptor)
        );
    }

    let mut alternate = KEYBOARD;
    alternate[3] = 1;
    let body = [
        alternate.as_slice(),
        HID.as_slice(),
        INPUT.as_slice(),
        KEYBOARD.as_slice(),
        INPUT.as_slice(),
    ]
    .concat();
    assert_eq!(
        select(&configuration(&body), 0, Speed::Full),
        Err(Error::HidDescriptor)
    );
}

#[test]
fn hid_shape_count_and_report_header_are_bounded() {
    let check = |hid: &[u8]| {
        select(
            &configuration(&[KEYBOARD.as_slice(), hid, INPUT.as_slice()].concat()),
            0,
            Speed::Full,
        )
    };
    for value in 0..=u8::MAX {
        let mut hid = HID;
        hid[5] = value;
        assert_eq!(
            check(&hid).map(|_| ()),
            if value == 1 {
                Ok(())
            } else {
                Err(Error::HidDescriptor)
            }
        );
        hid = HID;
        hid[6] = value;
        assert_eq!(
            check(&hid).map(|_| ()),
            if value == 0x22 {
                Ok(())
            } else {
                Err(Error::HidDescriptor)
            }
        );
    }
    for length in 2..=u8::MAX {
        let mut hid = vec![0; usize::from(length)];
        hid[0] = length;
        hid[1] = 0x21;
        if length >= 9 {
            hid[2..9].copy_from_slice(&HID[2..9]);
        }
        assert_eq!(
            check(&hid).map(|_| ()),
            if length == 9 {
                Ok(())
            } else {
                Err(Error::HidDescriptor)
            }
        );
    }
    for length in 0..=u16::MAX {
        let mut hid = HID;
        hid[7..9].copy_from_slice(&length.to_le_bytes());
        assert_eq!(
            check(&hid).map(|binding| binding.report_descriptor_length),
            if length == 0 {
                Err(Error::HidDescriptor)
            } else {
                Ok(length)
            }
        );
    }
    // Optional class descriptors retain their declared framing. Their contents
    // and HID version/country policy remain the enumeration owner's concern.
    for count in 2..=83u8 {
        let mut hid = HID.to_vec();
        for _ in 1..count {
            hid.extend_from_slice(&[0x23, 7, 0]);
        }
        hid[0] = u8::try_from(hid.len()).unwrap();
        hid[5] = count;
        assert_eq!(check(&hid).unwrap().report_descriptor_length, 63);
    }
}

#[test]
fn selects_interface_level_boot_identity_and_its_own_input_endpoint() {
    let bytes = configuration(
        &[
            KEYBOARD.as_slice(),
            HID.as_slice(),
            INPUT.as_slice(),
            &[9, 4, 1, 0, 1, 3, 1, 2, 0],
            &[9, 0x21, 0x11, 1, 0, 1, 0x22, 50, 0],
            &[7, 5, 0x82, 3, 4, 0, 7],
        ]
        .concat(),
    );
    let keyboard = select(&bytes, 0, Speed::Low).unwrap();
    assert_eq!(keyboard.protocol, Protocol::Keyboard);
    assert_eq!(keyboard.input_endpoint, 0x81);
    assert_eq!(keyboard.max_packet_size, 8);
    assert_eq!(keyboard.report_descriptor_length, 63);
    let mouse = select(&bytes, 1, Speed::Full).unwrap();
    assert_eq!(mouse.protocol, Protocol::Mouse);
    assert_eq!(mouse.input_endpoint, 0x82);
    assert_eq!(mouse.interval_ms, 7);
    assert_eq!(mouse.report_descriptor_length, 50);
    assert_eq!(select(&bytes, 2, Speed::Full), Err(Error::MissingInterface));
}

#[test]
fn endpoint_from_another_interface_cannot_complete_the_selected_interface() {
    let bytes = configuration(
        &[
            KEYBOARD.as_slice(),
            HID.as_slice(),
            &[9, 4, 1, 0, 1, 3, 1, 2, 0],
            INPUT.as_slice(),
        ]
        .concat(),
    );
    assert_eq!(select(&bytes, 0, Speed::Full), Err(Error::EndpointCount));
}

#[test]
fn a_valid_prefix_cannot_hide_a_malformed_tail_or_duplicate_interface() {
    let bytes = configuration(
        &[
            KEYBOARD.as_slice(),
            HID.as_slice(),
            INPUT.as_slice(),
            &[1, 0xff],
        ]
        .concat(),
    );
    assert!(matches!(
        select(&bytes, 0, Speed::Full),
        Err(Error::Framing(_))
    ));
    let bytes = configuration(
        &[
            KEYBOARD.as_slice(),
            HID.as_slice(),
            INPUT.as_slice(),
            KEYBOARD.as_slice(),
            INPUT.as_slice(),
        ]
        .concat(),
    );
    assert_eq!(
        select(&bytes, 0, Speed::Full),
        Err(Error::DuplicateInterface)
    );
}

fn single(interface: &[u8], endpoints: &[u8]) -> Vec<u8> {
    configuration(&[interface, HID.as_slice(), endpoints].concat())
}

#[test]
fn every_endpoint_address_and_attribute_is_bounded_and_direction_is_not_inferred() {
    for speed in [Speed::Low, Speed::Full] {
        for value in 0..=u8::MAX {
            let mut input = INPUT;
            input[2] = value;
            let result = select(&single(&KEYBOARD, &input), 0, speed);
            let expected = if value & 0x70 != 0 || value & 15 == 0 {
                Err(Error::Endpoint)
            } else if value & 0x80 == 0 {
                Err(Error::MissingInput)
            } else {
                Ok(value)
            };
            assert_eq!(result.map(|binding| binding.input_endpoint), expected);
            input = INPUT;
            input[3] = value;
            assert_eq!(
                select(&single(&KEYBOARD, &input), 0, speed).map(|_| ()),
                if value == 3 {
                    Ok(())
                } else {
                    Err(Error::Endpoint)
                }
            );
        }
    }
}

#[test]
fn all_packet_words_and_intervals_obey_speed_without_confusing_packets_with_reports() {
    for speed in [Speed::Low, Speed::Full] {
        for protocol in [1, 2] {
            let mut interface = KEYBOARD;
            interface[7] = protocol;
            let maximum = if speed == Speed::Low { 8 } else { 64 };
            for packet in 0..=u16::MAX {
                let mut input = INPUT;
                input[4..6].copy_from_slice(&packet.to_le_bytes());
                assert_eq!(
                    select(&single(&interface, &input), 0, speed)
                        .map(|binding| binding.max_packet_size),
                    if (1..=maximum).contains(&packet) {
                        Ok(packet)
                    } else {
                        Err(Error::Endpoint)
                    }
                );
            }
        }
        for interval in 0..=u8::MAX {
            let mut input = INPUT;
            input[6] = interval;
            assert_eq!(
                select(&single(&KEYBOARD, &input), 0, speed).map(|binding| binding.interval_ms),
                match (speed, interval) {
                    (_, 0) | (Speed::Low, 1..=9) => Err(Error::Endpoint),
                    _ => Ok(interval),
                }
            );
        }
    }
}

#[test]
fn endpoint_count_duplicate_and_ambiguous_input_are_distinct_failures() {
    for expected in 0..=u8::MAX {
        let mut interface = KEYBOARD;
        interface[4] = expected;
        assert_eq!(
            select(&single(&interface, &INPUT), 0, Speed::Full).map(|_| ()),
            if expected == 1 {
                Ok(())
            } else {
                Err(Error::EndpointCount)
            }
        );
    }
    let mut interface = KEYBOARD;
    interface[4] = 2;
    let mut other = INPUT;
    other[2] = 0x82;
    assert_eq!(
        select(
            &single(&interface, &[INPUT, INPUT].concat()),
            0,
            Speed::Full
        ),
        Err(Error::DuplicateEndpoint)
    );
    assert_eq!(
        select(
            &single(&interface, &[INPUT, other].concat()),
            0,
            Speed::Full
        ),
        Err(Error::AmbiguousInput)
    );
    other[2] = 1;
    let binding = select(&single(&interface, &[other, INPUT].concat()), 0, Speed::Low).unwrap();
    assert_eq!(binding.input_endpoint, 0x81);
}

#[test]
fn only_requested_alternate_zero_can_bind_and_endpoints_do_not_bleed_across_alternates() {
    for alternate in 1..=u8::MAX {
        let mut other = KEYBOARD;
        other[3] = alternate;
        assert_eq!(
            select(&single(&other, &INPUT), 0, Speed::Full),
            Err(Error::MissingInterface)
        );
        let bytes = configuration(
            &[
                KEYBOARD.as_slice(),
                HID.as_slice(),
                INPUT.as_slice(),
                other.as_slice(),
                &[7, 5, 0x82, 3, 64, 0, 1],
            ]
            .concat(),
        );
        assert_eq!(select(&bytes, 0, Speed::Full).unwrap().input_endpoint, 0x81);
        let bytes = configuration(
            &[
                KEYBOARD.as_slice(),
                HID.as_slice(),
                other.as_slice(),
                INPUT.as_slice(),
            ]
            .concat(),
        );
        assert_eq!(select(&bytes, 0, Speed::Full), Err(Error::EndpointCount));
    }
    for field in 5..=7 {
        for value in 0..=u8::MAX {
            let mut interface = KEYBOARD;
            interface[field] = value;
            let boot =
                interface[5] == 3 && interface[6] == 1 && (interface[7] == 1 || interface[7] == 2);
            assert_eq!(
                select(&single(&interface, &INPUT), 0, Speed::Full).map(|_| ()),
                if boot { Ok(()) } else { Err(Error::NotBoot) }
            );
        }
    }
}

#[test]
fn configuration_header_and_all_transfer_cuts_cannot_return_a_partial_match() {
    let valid = single(&KEYBOARD, &INPUT);
    for length in 0..valid.len() {
        assert_eq!(
            select(&valid[..length], 0, Speed::Full),
            Err(Error::Configuration)
        );
    }
    for value in 0..=u16::MAX {
        let mut bytes = valid.clone();
        bytes[2..4].copy_from_slice(&value.to_le_bytes());
        assert_eq!(
            select(&bytes, 0, Speed::Full).map(|_| ()),
            if usize::from(value) == bytes.len() {
                Ok(())
            } else {
                Err(Error::Configuration)
            }
        );
    }
    for offset in [0, 1, 4, 5] {
        let mut bytes = valid.clone();
        bytes[offset] = 0;
        assert_eq!(select(&bytes, 0, Speed::Full), Err(Error::Configuration));
    }
    let bytes = configuration(
        &[
            KEYBOARD.as_slice(),
            HID.as_slice(),
            INPUT.as_slice(),
            &[9, 2, 9, 0, 1, 1, 0, 0x80, 50],
        ]
        .concat(),
    );
    assert_eq!(select(&bytes, 0, Speed::Full), Err(Error::Configuration));
}

#[test]
fn malformed_interface_and_endpoint_shapes_are_not_unknown_descriptors() {
    for length in 2..=u8::MAX {
        let mut interface = vec![0; usize::from(length)];
        interface[0] = length;
        interface[1] = 4;
        if length != 9 {
            assert_eq!(
                select(&configuration(&interface), 0, Speed::Full),
                Err(Error::Interface)
            );
        }
        let mut endpoint = vec![0; usize::from(length)];
        endpoint[0] = length;
        endpoint[1] = 5;
        if length != 7 {
            assert_eq!(
                select(&single(&KEYBOARD, &endpoint), 0, Speed::Full),
                Err(Error::Endpoint)
            );
        }
    }
}
