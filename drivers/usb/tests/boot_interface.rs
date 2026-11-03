use nieuw_os_usb_core::boot_interface::{select, Error, Protocol, Speed};

fn configuration(body: &[u8]) -> Vec<u8> {
    let length = u16::try_from(body.len() + 9).unwrap().to_le_bytes();
    let mut bytes = vec![9, 2, length[0], length[1], 2, 1, 0, 0x80, 50];
    bytes.extend_from_slice(body);
    bytes
}

const KEYBOARD: [u8; 9] = [9, 4, 0, 0, 1, 3, 1, 1, 0];
const INPUT: [u8; 7] = [7, 5, 0x81, 3, 8, 0, 10];

#[test]
fn selects_interface_level_boot_identity_and_its_own_input_endpoint() {
    let bytes = configuration(
        &[
            KEYBOARD.as_slice(),
            &[9, 0x21, 0x11, 1, 0, 1, 0x22, 63, 0],
            INPUT.as_slice(),
            &[9, 4, 1, 0, 1, 3, 1, 2, 0],
            &[7, 5, 0x82, 3, 4, 0, 7],
        ]
        .concat(),
    );
    let keyboard = select(&bytes, 0, Speed::Low).unwrap();
    assert_eq!(keyboard.protocol, Protocol::Keyboard);
    assert_eq!(keyboard.input_endpoint, 0x81);
    assert_eq!(keyboard.max_packet_size, 8);
    let mouse = select(&bytes, 1, Speed::Full).unwrap();
    assert_eq!(mouse.protocol, Protocol::Mouse);
    assert_eq!(mouse.input_endpoint, 0x82);
    assert_eq!(mouse.interval_ms, 7);
    assert_eq!(select(&bytes, 2, Speed::Full), Err(Error::MissingInterface));
}

#[test]
fn endpoint_from_another_interface_cannot_complete_the_selected_interface() {
    let bytes = configuration(
        &[
            KEYBOARD.as_slice(),
            &[9, 4, 1, 0, 1, 3, 1, 2, 0],
            INPUT.as_slice(),
        ]
        .concat(),
    );
    assert_eq!(select(&bytes, 0, Speed::Full), Err(Error::EndpointCount));
}

#[test]
fn a_valid_prefix_cannot_hide_a_malformed_tail_or_duplicate_interface() {
    let bytes = configuration(&[KEYBOARD.as_slice(), INPUT.as_slice(), &[1, 0xff]].concat());
    assert!(matches!(
        select(&bytes, 0, Speed::Full),
        Err(Error::Framing(_))
    ));
    let bytes = configuration(
        &[
            KEYBOARD.as_slice(),
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
