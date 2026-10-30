//! Bounded DHCPv4 BOOTP message construction and reply parsing (RFC 2131/2132).
//!
//! This module has no socket, timer, route, interface or authority. A matching
//! transaction is not server authentication and must not by itself configure
//! an interface or grant network access.

use crate::arp::unicast_ipv4;

const BOOTP_BYTES: usize = 236;
const OPTIONS_OFFSET: usize = 240;
const COOKIE: [u8; 4] = [99, 130, 83, 99];
pub const MAX_MESSAGE_BYTES: usize = 576;
pub const MAX_LIST_ADDRESSES: usize = 4;
const PARAMETER_REQUEST_LIST: [u8; 7] = [1, 3, 6, 51, 54, 58, 59];
const DHCP_OPTION_BYTES: usize = 3 + 9 + 9 + 4 + 1;
const SELECT_OPTION_BYTES: usize = DHCP_OPTION_BYTES + 6 + 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    Offer,
    Ack,
    Nak,
}

#[derive(Clone, Copy)]
enum RequestType {
    Discover,
    Request,
}

impl RequestType {
    const fn option_code(self) -> u8 {
        match self {
            Self::Discover => 1,
            Self::Request => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Truncated,
    MessageTooLarge,
    InvalidOperation,
    UnsupportedHardware,
    WrongTransaction,
    WrongHardwareAddress,
    InvalidCookie,
    TruncatedOption,
    MissingEnd,
    InvalidPadding,
    DuplicateOption(u8),
    InvalidOptionLength(u8),
    UnsupportedOptionOverload,
    InvalidMessageType(u8),
    MissingMessageType,
    MissingServerIdentifier,
    MissingLeaseTime,
    InvalidServerIdentifier,
    InvalidSubnetMask,
    InvalidAddress,
    InvalidLeaseTime,
    TooManyAddresses,
    OutputTooSmall,
    InvalidMac,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Client {
    pub transaction_id: u32,
    pub mac: [u8; 6],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub message_type: MessageType,
    pub offered_address: [u8; 4],
    pub server_identifier: [u8; 4],
    pub lease_seconds: Option<u32>,
    pub subnet_mask: Option<[u8; 4]>,
    routers: [[u8; 4]; MAX_LIST_ADDRESSES],
    router_count: usize,
    dns_servers: [[u8; 4]; MAX_LIST_ADDRESSES],
    dns_count: usize,
    pub renewal_seconds: Option<u32>,
    pub rebinding_seconds: Option<u32>,
}

impl Reply {
    pub fn routers(&self) -> &[[u8; 4]] {
        &self.routers[..self.router_count]
    }

    pub fn dns_servers(&self) -> &[[u8; 4]] {
        &self.dns_servers[..self.dns_count]
    }
}

#[derive(Default)]
struct Options {
    seen: u16,
    message_type: Option<MessageType>,
    server_identifier: Option<[u8; 4]>,
    lease_seconds: Option<u32>,
    subnet_mask: Option<[u8; 4]>,
    routers: [[u8; 4]; MAX_LIST_ADDRESSES],
    router_count: usize,
    dns_servers: [[u8; 4]; MAX_LIST_ADDRESSES],
    dns_count: usize,
    renewal_seconds: Option<u32>,
    rebinding_seconds: Option<u32>,
}

impl Options {
    fn mark(&mut self, code: u8) -> Result<(), Error> {
        let bit = match code {
            1 => 0,
            3 => 1,
            6 => 2,
            51 => 3,
            53 => 4,
            54 => 5,
            58 => 6,
            59 => 7,
            _ => return Ok(()),
        };
        let mask = 1u16 << bit;
        if self.seen & mask != 0 {
            return Err(Error::DuplicateOption(code));
        }
        self.seen |= mask;
        Ok(())
    }
}

fn dword(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn option_address(code: u8, value: &[u8]) -> Result<[u8; 4], Error> {
    value
        .try_into()
        .map_err(|_| Error::InvalidOptionLength(code))
}

fn option_time(code: u8, value: &[u8]) -> Result<u32, Error> {
    if value.len() != 4 {
        return Err(Error::InvalidOptionLength(code));
    }
    Ok(dword(value))
}

fn valid_mask(mask: [u8; 4]) -> bool {
    let inverse = !u32::from_be_bytes(mask);
    inverse & inverse.wrapping_add(1) == 0
}

fn address_list<const N: usize>(
    code: u8,
    value: &[u8],
    output: &mut [[u8; 4]; N],
    count: &mut usize,
) -> Result<(), Error> {
    if value.is_empty() || !value.len().is_multiple_of(4) {
        return Err(Error::InvalidOptionLength(code));
    }
    let entries = value.len() / 4;
    if entries > N {
        return Err(Error::TooManyAddresses);
    }
    for (index, address) in value.chunks_exact(4).enumerate() {
        let address: [u8; 4] = address.try_into().unwrap();
        if !unicast_ipv4(address) {
            return Err(Error::InvalidAddress);
        }
        output[index] = address;
    }
    *count = entries;
    Ok(())
}

/// Parse one server reply after the caller has checked the UDP endpoints and
/// interface scope. Only the fixed BOOTP header and primary options field are
/// supported; option overload is explicitly rejected.
pub fn parse_reply(message: &[u8], client: Client) -> Result<Reply, Error> {
    if !valid_mac(client.mac) {
        return Err(Error::InvalidMac);
    }
    if message.len() < OPTIONS_OFFSET {
        return Err(Error::Truncated);
    }
    if message.len() > MAX_MESSAGE_BYTES {
        return Err(Error::MessageTooLarge);
    }
    if message[0] != 2 {
        return Err(Error::InvalidOperation);
    }
    if message[1] != 1 || message[2] != 6 {
        return Err(Error::UnsupportedHardware);
    }
    if dword(&message[4..8]) != client.transaction_id {
        return Err(Error::WrongTransaction);
    }
    if message[28..34] != client.mac {
        return Err(Error::WrongHardwareAddress);
    }
    if message[BOOTP_BYTES..OPTIONS_OFFSET] != COOKIE {
        return Err(Error::InvalidCookie);
    }

    let mut options = Options::default();
    let mut cursor = OPTIONS_OFFSET;
    let mut ended = false;
    while cursor < message.len() {
        let code = message[cursor];
        cursor += 1;
        match code {
            0 => continue,
            255 => {
                ended = true;
                break;
            }
            _ => {}
        }
        let length = *message.get(cursor).ok_or(Error::TruncatedOption)? as usize;
        cursor += 1;
        let end = cursor.checked_add(length).ok_or(Error::TruncatedOption)?;
        let value = message.get(cursor..end).ok_or(Error::TruncatedOption)?;
        cursor = end;
        options.mark(code)?;
        match code {
            1 => {
                let mask = option_address(code, value)?;
                if !valid_mask(mask) {
                    return Err(Error::InvalidSubnetMask);
                }
                options.subnet_mask = Some(mask);
            }
            3 => address_list(code, value, &mut options.routers, &mut options.router_count)?,
            6 => address_list(
                code,
                value,
                &mut options.dns_servers,
                &mut options.dns_count,
            )?,
            51 => {
                let seconds = option_time(code, value)?;
                if seconds == 0 {
                    return Err(Error::InvalidLeaseTime);
                }
                options.lease_seconds = Some(seconds);
            }
            52 => return Err(Error::UnsupportedOptionOverload),
            53 => {
                if value.len() != 1 {
                    return Err(Error::InvalidOptionLength(code));
                }
                options.message_type = Some(match value[0] {
                    2 => MessageType::Offer,
                    5 => MessageType::Ack,
                    6 => MessageType::Nak,
                    other => return Err(Error::InvalidMessageType(other)),
                });
            }
            54 => {
                let address = option_address(code, value)?;
                if !unicast_ipv4(address) {
                    return Err(Error::InvalidServerIdentifier);
                }
                options.server_identifier = Some(address);
            }
            58 => options.renewal_seconds = Some(option_time(code, value)?),
            59 => options.rebinding_seconds = Some(option_time(code, value)?),
            _ => {}
        }
    }
    if !ended {
        return Err(Error::MissingEnd);
    }
    if message[cursor..].iter().any(|byte| *byte != 0) {
        return Err(Error::InvalidPadding);
    }

    let message_type = options.message_type.ok_or(Error::MissingMessageType)?;
    let server_identifier = options
        .server_identifier
        .ok_or(Error::MissingServerIdentifier)?;
    let lease_seconds = options.lease_seconds;
    if message_type != MessageType::Nak && lease_seconds.is_none() {
        return Err(Error::MissingLeaseTime);
    }
    if let Some(lease) = lease_seconds {
        let renewal = options
            .renewal_seconds
            .unwrap_or_else(|| (u64::from(lease) / 2) as u32);
        let rebinding = options
            .rebinding_seconds
            .unwrap_or_else(|| (u64::from(lease) * 7 / 8) as u32);
        if renewal == 0 || renewal >= rebinding || rebinding >= lease {
            return Err(Error::InvalidLeaseTime);
        }
    }

    Ok(Reply {
        message_type,
        offered_address: message[16..20].try_into().unwrap(),
        server_identifier,
        lease_seconds,
        subnet_mask: options.subnet_mask,
        routers: options.routers,
        router_count: options.router_count,
        dns_servers: options.dns_servers,
        dns_count: options.dns_count,
        renewal_seconds: options.renewal_seconds,
        rebinding_seconds: options.rebinding_seconds,
    })
}

fn valid_mac(mac: [u8; 6]) -> bool {
    mac != [0; 6] && mac[0] & 1 == 0
}

fn put_option(output: &mut [u8], cursor: &mut usize, code: u8, value: &[u8]) {
    output[*cursor] = code;
    output[*cursor + 1] = value.len() as u8;
    output[*cursor + 2..*cursor + 2 + value.len()].copy_from_slice(value);
    *cursor += value.len() + 2;
}

fn build_request(
    output: &mut [u8],
    transaction_id: u32,
    mac: [u8; 6],
    elapsed_seconds: u16,
    client_address: [u8; 4],
    broadcast: bool,
    request_type: RequestType,
    selected: Option<([u8; 4], [u8; 4])>,
) -> Result<usize, Error> {
    if !valid_mac(mac) {
        return Err(Error::InvalidMac);
    }
    let option_bytes = if selected.is_some() {
        SELECT_OPTION_BYTES
    } else {
        DHCP_OPTION_BYTES
    };
    let message_bytes = OPTIONS_OFFSET + option_bytes;
    if output.len() < message_bytes {
        return Err(Error::OutputTooSmall);
    }

    output[..message_bytes].fill(0);
    output[0] = 1;
    output[1] = 1;
    output[2] = 6;
    output[4..8].copy_from_slice(&transaction_id.to_be_bytes());
    output[8..10].copy_from_slice(&elapsed_seconds.to_be_bytes());
    if broadcast {
        output[10..12].copy_from_slice(&0x8000u16.to_be_bytes());
    }
    output[12..16].copy_from_slice(&client_address);
    output[28..34].copy_from_slice(&mac);
    output[BOOTP_BYTES..OPTIONS_OFFSET].copy_from_slice(&COOKIE);

    let mut cursor = OPTIONS_OFFSET;
    put_option(output, &mut cursor, 53, &[request_type.option_code()]);
    let mut client_identifier = [1; 7];
    client_identifier[1..].copy_from_slice(&mac);
    put_option(output, &mut cursor, 61, &client_identifier);
    if let Some((requested_address, server_identifier)) = selected {
        put_option(output, &mut cursor, 50, &requested_address);
        put_option(output, &mut cursor, 54, &server_identifier);
    }
    put_option(output, &mut cursor, 55, &PARAMETER_REQUEST_LIST);
    put_option(
        output,
        &mut cursor,
        57,
        &(MAX_MESSAGE_BYTES as u16).to_be_bytes(),
    );
    output[cursor] = 255;
    debug_assert_eq!(cursor + 1, message_bytes);
    Ok(message_bytes)
}

/// Build DHCPDISCOVER. The transaction ID must come from the caller's
/// authorized entropy source; this function does not generate randomness.
pub fn build_discover(
    output: &mut [u8],
    transaction_id: u32,
    mac: [u8; 6],
    elapsed_seconds: u16,
) -> Result<usize, Error> {
    build_request(
        output,
        transaction_id,
        mac,
        elapsed_seconds,
        [0; 4],
        true,
        RequestType::Discover,
        None,
    )
}

/// Build the selecting-state DHCPREQUEST for one received DHCPOFFER.
pub fn build_selecting_request(
    output: &mut [u8],
    transaction_id: u32,
    mac: [u8; 6],
    elapsed_seconds: u16,
    requested_address: [u8; 4],
    server_identifier: [u8; 4],
) -> Result<usize, Error> {
    if !unicast_ipv4(requested_address) || !unicast_ipv4(server_identifier) {
        return Err(Error::InvalidAddress);
    }
    build_request(
        output,
        transaction_id,
        mac,
        elapsed_seconds,
        [0; 4],
        true,
        RequestType::Request,
        Some((requested_address, server_identifier)),
    )
}

/// Build a renewing-state DHCPREQUEST. The address belongs in `ciaddr`; the
/// caller must unicast this to the selected server and enforce lease deadlines.
pub fn build_renewal_request(
    output: &mut [u8],
    transaction_id: u32,
    mac: [u8; 6],
    elapsed_seconds: u16,
    client_address: [u8; 4],
) -> Result<usize, Error> {
    if !unicast_ipv4(client_address) {
        return Err(Error::InvalidAddress);
    }
    build_request(
        output,
        transaction_id,
        mac,
        elapsed_seconds,
        client_address,
        false,
        RequestType::Request,
        None,
    )
}
