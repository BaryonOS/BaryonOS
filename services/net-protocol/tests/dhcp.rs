use nieuw_os_net_protocol::dhcp::{self, Client, Error, MessageType, MAX_MESSAGE_BYTES};

const XID: u32 = 0x1234_5678;
const MAC: [u8; 6] = [2, 0, 0, 0, 0, 7];
const OFFERED: [u8; 4] = [192, 0, 2, 42];
const SERVER: [u8; 4] = [192, 0, 2, 1];
const CLIENT: Client = Client {
    transaction_id: XID,
    mac: MAC,
};

fn option(message: &mut Vec<u8>, code: u8, value: &[u8]) {
    message.push(code);
    message.push(value.len() as u8);
    message.extend_from_slice(value);
}

fn reply(kind: u8) -> Vec<u8> {
    let mut message = vec![0; 240];
    message[0] = 2;
    message[1] = 1;
    message[2] = 6;
    message[4..8].copy_from_slice(&XID.to_be_bytes());
    message[16..20].copy_from_slice(&OFFERED);
    message[28..34].copy_from_slice(&MAC);
    message[236..240].copy_from_slice(&[99, 130, 83, 99]);
    option(&mut message, 53, &[kind]);
    option(&mut message, 54, &SERVER);
    if kind != 6 {
        option(&mut message, 51, &3600u32.to_be_bytes());
        option(&mut message, 1, &[255, 255, 255, 0]);
        option(&mut message, 3, &SERVER);
        option(&mut message, 6, &[192, 0, 2, 53]);
        option(&mut message, 58, &1800u32.to_be_bytes());
        option(&mut message, 59, &3150u32.to_be_bytes());
    }
    message.push(255);
    message
}

fn options(message: &[u8]) -> Vec<(u8, Vec<u8>)> {
    let mut parsed = Vec::new();
    let mut cursor = 240;
    while cursor < message.len() {
        let code = message[cursor];
        cursor += 1;
        if code == 255 {
            break;
        }
        if code == 0 {
            continue;
        }
        let length = usize::from(message[cursor]);
        cursor += 1;
        parsed.push((code, message[cursor..cursor + length].to_vec()));
        cursor += length;
    }
    parsed
}

fn option_range(message: &[u8], wanted: u8) -> (usize, usize) {
    let mut cursor = 240;
    loop {
        let code = message[cursor];
        if code == 255 {
            panic!("option {wanted} not found");
        }
        if code == 0 {
            cursor += 1;
            continue;
        }
        let start = cursor;
        let length = usize::from(message[cursor + 1]);
        cursor += length + 2;
        if code == wanted {
            return (start, cursor);
        }
    }
}

fn insert_before_end(message: &mut Vec<u8>, code: u8, value: &[u8]) {
    let mut cursor = 240;
    loop {
        let current = message[cursor];
        if current == 255 {
            let mut inserted = Vec::with_capacity(value.len() + 3);
            inserted.push(code);
            inserted.push(value.len() as u8);
            inserted.extend_from_slice(value);
            inserted.push(255);
            message.splice(cursor..cursor + 1, inserted);
            return;
        }
        cursor += if current == 0 {
            1
        } else {
            usize::from(message[cursor + 1]) + 2
        };
    }
}

#[test]
fn parses_offer_and_bounded_network_options() {
    let parsed = dhcp::parse_reply(&reply(2), CLIENT).unwrap();
    assert_eq!(parsed.message_type, MessageType::Offer);
    assert_eq!(parsed.offered_address, OFFERED);
    assert_eq!(parsed.server_identifier, SERVER);
    assert_eq!(parsed.lease_seconds, Some(3600));
    assert_eq!(parsed.subnet_mask, Some([255, 255, 255, 0]));
    assert_eq!(parsed.routers(), &[SERVER]);
    assert_eq!(parsed.dns_servers(), &[[192, 0, 2, 53]]);
    assert_eq!(parsed.renewal_seconds, Some(1800));
    assert_eq!(parsed.rebinding_seconds, Some(3150));
}

#[test]
fn accepts_ack_and_nak_with_their_distinct_required_fields() {
    let ack = dhcp::parse_reply(&reply(5), CLIENT).unwrap();
    assert_eq!(ack.message_type, MessageType::Ack);
    assert_eq!(ack.lease_seconds, Some(3600));

    let nak = dhcp::parse_reply(&reply(6), CLIENT).unwrap();
    assert_eq!(nak.message_type, MessageType::Nak);
    assert_eq!(nak.lease_seconds, None);
}

#[test]
fn rejects_wrong_transaction_hardware_operation_and_cookie() {
    let mut message = reply(2);
    let mut wrong = CLIENT;
    wrong.transaction_id += 1;
    assert_eq!(
        dhcp::parse_reply(&message, wrong),
        Err(Error::WrongTransaction)
    );

    wrong = CLIENT;
    wrong.mac[5] += 1;
    assert_eq!(
        dhcp::parse_reply(&message, wrong),
        Err(Error::WrongHardwareAddress)
    );

    message[0] = 1;
    assert_eq!(
        dhcp::parse_reply(&message, CLIENT),
        Err(Error::InvalidOperation)
    );
    message[0] = 2;
    message[236] = 0;
    assert_eq!(
        dhcp::parse_reply(&message, CLIENT),
        Err(Error::InvalidCookie)
    );
}

#[test]
fn rejects_truncation_at_each_wire_boundary_and_missing_end() {
    let message = reply(2);
    for end in 0..message.len() {
        assert!(dhcp::parse_reply(&message[..end], CLIENT).is_err(), "{end}");
    }
    let mut no_end = message;
    no_end.pop();
    assert_eq!(dhcp::parse_reply(&no_end, CLIENT), Err(Error::MissingEnd));
}

#[test]
fn rejects_duplicate_required_options_and_malformed_tlvs() {
    let mut duplicate = reply(2);
    insert_before_end(&mut duplicate, 54, &SERVER);
    assert_eq!(
        dhcp::parse_reply(&duplicate, CLIENT),
        Err(Error::DuplicateOption(54))
    );

    let mut short = reply(2);
    let (start, _) = option_range(&short, 51);
    short[start + 1] = 2;
    assert_eq!(
        dhcp::parse_reply(&short, CLIENT),
        Err(Error::InvalidOptionLength(51))
    );

    let mut overrun = reply(2);
    overrun.truncate(overrun.len() - 1);
    overrun.extend_from_slice(&[12, 9, 1]);
    assert_eq!(
        dhcp::parse_reply(&overrun, CLIENT),
        Err(Error::TruncatedOption)
    );
}

#[test]
fn rejects_unsupported_overload_bad_padding_and_bad_network_values() {
    let mut overload = reply(2);
    insert_before_end(&mut overload, 52, &[1]);
    assert_eq!(
        dhcp::parse_reply(&overload, CLIENT),
        Err(Error::UnsupportedOptionOverload)
    );

    let mut padding = reply(2);
    padding.extend_from_slice(&[0, 1]);
    assert_eq!(
        dhcp::parse_reply(&padding, CLIENT),
        Err(Error::InvalidPadding)
    );

    let mut bad_mask = reply(2);
    let (start, _) = option_range(&bad_mask, 1);
    bad_mask[start + 2..start + 6].copy_from_slice(&[255, 0, 255, 0]);
    assert_eq!(
        dhcp::parse_reply(&bad_mask, CLIENT),
        Err(Error::InvalidSubnetMask)
    );

    let mut bad_lease = reply(2);
    let (start, _) = option_range(&bad_lease, 51);
    bad_lease[start + 2..start + 6].copy_from_slice(&0u32.to_be_bytes());
    assert_eq!(
        dhcp::parse_reply(&bad_lease, CLIENT),
        Err(Error::InvalidLeaseTime)
    );

    let mut bad_dns = reply(2);
    let (start, _) = option_range(&bad_dns, 6);
    bad_dns[start + 2..start + 6].copy_from_slice(&[0, 0, 0, 0]);
    assert_eq!(
        dhcp::parse_reply(&bad_dns, CLIENT),
        Err(Error::InvalidAddress)
    );

    let mut too_many_routers = reply(2);
    let (start, end) = option_range(&too_many_routers, 3);
    let mut replacement = vec![3, 20];
    replacement.extend_from_slice(&[
        192, 0, 2, 1, 192, 0, 2, 2, 192, 0, 2, 3, 192, 0, 2, 4, 192, 0, 2, 5,
    ]);
    too_many_routers.splice(start..end, replacement);
    assert_eq!(
        dhcp::parse_reply(&too_many_routers, CLIENT),
        Err(Error::TooManyAddresses)
    );
}

#[test]
fn rejects_missing_fields_invalid_types_and_bad_lease_timers() {
    let mut no_server = reply(2);
    no_server.truncate(240);
    option(&mut no_server, 53, &[2]);
    option(&mut no_server, 51, &3600u32.to_be_bytes());
    no_server.push(255);
    assert_eq!(
        dhcp::parse_reply(&no_server, CLIENT),
        Err(Error::MissingServerIdentifier)
    );

    let mut no_lease = reply(2);
    let (start, end) = option_range(&no_lease, 51);
    no_lease.drain(start..end);
    assert_eq!(
        dhcp::parse_reply(&no_lease, CLIENT),
        Err(Error::MissingLeaseTime)
    );

    let mut invalid_type = reply(2);
    invalid_type[242] = 1;
    assert_eq!(
        dhcp::parse_reply(&invalid_type, CLIENT),
        Err(Error::InvalidMessageType(1))
    );

    let mut invalid_timers = reply(2);
    let renewal = invalid_timers
        .windows(2)
        .position(|window| window == [58, 4])
        .unwrap();
    invalid_timers[renewal + 2..renewal + 6].copy_from_slice(&4000u32.to_be_bytes());
    assert_eq!(
        dhcp::parse_reply(&invalid_timers, CLIENT),
        Err(Error::InvalidLeaseTime)
    );
}

#[test]
fn builds_discover_selecting_and_renewal_messages() {
    let mut output = [0xA5; MAX_MESSAGE_BYTES];
    let length = dhcp::build_discover(&mut output, XID, MAC, 9).unwrap();
    assert_eq!(&output[..4], &[1, 1, 6, 0]);
    assert_eq!(&output[4..8], &XID.to_be_bytes());
    assert_eq!(&output[8..10], &9u16.to_be_bytes());
    assert_eq!(&output[10..12], &0x8000u16.to_be_bytes());
    assert_eq!(&output[28..34], &MAC);
    assert_eq!(&output[236..240], &[99, 130, 83, 99]);
    let discover = options(&output[..length]);
    assert_eq!(discover[0], (53, vec![1]));
    assert_eq!(discover[1], (61, [vec![1], MAC.to_vec()].concat()));
    assert_eq!(discover[2], (55, vec![1, 3, 6, 51, 54, 58, 59]));
    assert_eq!(
        discover[3],
        (57, (MAX_MESSAGE_BYTES as u16).to_be_bytes().to_vec())
    );

    let requested = [192, 0, 2, 42];
    let length =
        dhcp::build_selecting_request(&mut output, XID, MAC, 9, requested, SERVER).unwrap();
    assert_eq!(&output[10..12], &[0x80, 0]);
    let selecting = options(&output[..length]);
    assert_eq!(selecting[0], (53, vec![3]));
    assert_eq!(selecting[2], (50, requested.to_vec()));
    assert_eq!(selecting[3], (54, SERVER.to_vec()));

    let local_address = [192, 0, 2, 42];
    let length = dhcp::build_renewal_request(&mut output, XID, MAC, 15, local_address).unwrap();
    assert_eq!(&output[10..12], &[0, 0]);
    assert_eq!(&output[12..16], &local_address);
    let renewal = options(&output[..length]);
    assert_eq!(renewal[0], (53, vec![3]));
    assert!(!renewal.iter().any(|(code, _)| matches!(code, 50 | 54)));
}

#[test]
fn failed_builds_leave_output_unchanged() {
    let mut output = [0xA5; 64];
    assert_eq!(
        dhcp::build_discover(&mut output, XID, MAC, 0),
        Err(Error::OutputTooSmall)
    );
    assert!(output.iter().all(|byte| *byte == 0xA5));

    let mut output = [0x5A; MAX_MESSAGE_BYTES];
    assert_eq!(
        dhcp::build_selecting_request(&mut output, XID, MAC, 0, [0; 4], SERVER),
        Err(Error::InvalidAddress)
    );
    assert!(output.iter().all(|byte| *byte == 0x5A));

    let mut output = [0x3C; MAX_MESSAGE_BYTES];
    assert_eq!(
        dhcp::build_discover(&mut output, XID, [1, 0, 0, 0, 0, 7], 0),
        Err(Error::InvalidMac)
    );
    assert!(output.iter().all(|byte| *byte == 0x3C));

    let mut oversized = reply(2);
    oversized.resize(MAX_MESSAGE_BYTES + 1, 0);
    assert_eq!(
        dhcp::parse_reply(&oversized, CLIENT),
        Err(Error::MessageTooLarge)
    );
}
