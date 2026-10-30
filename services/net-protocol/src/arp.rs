//! Ethernet/IPv4 ARP and one solicited, expiring neighbor slot (RFC 826).
//!
//! ARP contains no authentication. A validated MAC/IP pair is only a link-layer
//! routing hint; it must never create a peer identity or application capability.
//! No device access, transmission, routing, or automatic retries occur here.

pub const FRAME_BYTES: usize = 42;
pub const PADDED_FRAME_BYTES: usize = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Request,
    Reply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Truncated,
    InvalidLength,
    UnsupportedFormat,
    UnsupportedOperation,
    InvalidMac,
    InvalidIpv4,
    SenderMismatch,
    TargetMismatch,
    NotRequest,
    ProbeNotSupported,
    OutputTooSmall,
    Busy,
    NoPendingRequest,
    Timeout,
    InvalidLifetime,
    ClockOverflow,
}

/// Configured interface identity, not a trusted identity of a remote peer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interface {
    pub mac: [u8; 6],
    pub ipv4: [u8; 4],
}

fn unicast_mac(mac: [u8; 6]) -> bool {
    mac != [0; 6] && mac[0] & 1 == 0
}

pub(crate) fn unicast_ipv4(ip: [u8; 4]) -> bool {
    // This profile does not support the 0/8, loopback, multicast or reserved
    // ranges. Directed subnet broadcasts require the caller's route/netmask.
    ip[0] != 0 && ip[0] != 127 && ip[0] < 224 && ip != [255; 4]
}

fn validate_interface(local: Interface) -> Result<(), Error> {
    if !unicast_mac(local.mac) {
        return Err(Error::InvalidMac);
    }
    if !unicast_ipv4(local.ipv4) {
        return Err(Error::InvalidIpv4);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Packet<'a> {
    frame: &'a [u8],
    operation: Operation,
}

impl Packet<'_> {
    pub fn operation(&self) -> Operation {
        self.operation
    }
    pub fn sender_mac(&self) -> [u8; 6] {
        self.frame[22..28].try_into().unwrap()
    }
    pub fn sender_ipv4(&self) -> [u8; 4] {
        self.frame[28..32].try_into().unwrap()
    }
    pub fn target_mac(&self) -> [u8; 6] {
        self.frame[32..38].try_into().unwrap()
    }
    pub fn target_ipv4(&self) -> [u8; 4] {
        self.frame[38..42].try_into().unwrap()
    }
    pub fn destination_mac(&self) -> [u8; 6] {
        self.frame[..6].try_into().unwrap()
    }
    pub fn is_probe(&self) -> bool {
        self.sender_ipv4() == [0; 4]
    }
}

/// Parse an Ethernet II ARP packet, with stripped padding or the complete
/// 60-byte Ethernet minimum, without FCS. Replies must address the same target
/// MAC in both headers. Probe requests are visible but are not cache entries.
pub fn parse(frame: &[u8]) -> Result<Packet<'_>, Error> {
    if frame.len() < FRAME_BYTES {
        return Err(Error::Truncated);
    }
    if frame.len() != FRAME_BYTES && frame.len() != PADDED_FRAME_BYTES {
        return Err(Error::InvalidLength);
    }
    if frame[12..22] != [0x08, 0x06, 0, 1, 0x08, 0, 6, 4, 0, 1]
        && frame[12..22] != [0x08, 0x06, 0, 1, 0x08, 0, 6, 4, 0, 2]
    {
        if frame[12..20] != [0x08, 0x06, 0, 1, 0x08, 0, 6, 4] {
            return Err(Error::UnsupportedFormat);
        }
        return Err(Error::UnsupportedOperation);
    }
    let operation = if frame[21] == 1 {
        Operation::Request
    } else {
        Operation::Reply
    };
    let packet = Packet { frame, operation };
    if !unicast_mac(packet.sender_mac()) {
        return Err(Error::InvalidMac);
    }
    if frame[6..12] != packet.sender_mac() {
        return Err(Error::SenderMismatch);
    }
    if !unicast_ipv4(packet.target_ipv4())
        || !(unicast_ipv4(packet.sender_ipv4())
            || operation == Operation::Request && packet.is_probe())
    {
        return Err(Error::InvalidIpv4);
    }
    if operation == Operation::Reply {
        if !unicast_mac(packet.target_mac()) {
            return Err(Error::InvalidMac);
        }
        if packet.target_mac() != packet.destination_mac() {
            return Err(Error::TargetMismatch);
        }
    } else if packet.destination_mac() != [255; 6] && !unicast_mac(packet.destination_mac()) {
        return Err(Error::InvalidMac);
    }
    Ok(packet)
}

fn encode(
    local: Interface,
    target_mac: [u8; 6],
    target_ipv4: [u8; 4],
    operation: Operation,
    output: &mut [u8],
) -> Result<usize, Error> {
    if output.len() < PADDED_FRAME_BYTES {
        return Err(Error::OutputTooSmall);
    }
    output[..PADDED_FRAME_BYTES].fill(0);
    output[..6].copy_from_slice(&if operation == Operation::Request {
        [255; 6]
    } else {
        target_mac
    });
    output[6..12].copy_from_slice(&local.mac);
    output[12..22].copy_from_slice(&[0x08, 0x06, 0, 1, 0x08, 0, 6, 4, 0, 1]);
    if operation == Operation::Reply {
        output[21] = 2;
    }
    output[22..28].copy_from_slice(&local.mac);
    output[28..32].copy_from_slice(&local.ipv4);
    output[32..38].copy_from_slice(&target_mac);
    output[38..42].copy_from_slice(&target_ipv4);
    Ok(PADDED_FRAME_BYTES)
}

/// Every failure leaves the output intact; success returns the exact frame
/// length, including zeroed link padding. This does not transmit the request.
pub fn build_request(
    local: Interface,
    target_ipv4: [u8; 4],
    output: &mut [u8],
) -> Result<usize, Error> {
    validate_interface(local)?;
    if !unicast_ipv4(target_ipv4) || target_ipv4 == local.ipv4 {
        return Err(Error::InvalidIpv4);
    }
    encode(local, [0; 6], target_ipv4, Operation::Request, output)
}

/// Reply only to a request for this configured interface. Address-conflict
/// probes require RFC 5227 conflict handling and are explicitly not answered by
/// this initial resolver. A parser success is not authority to impersonate IPs.
pub fn build_reply(
    local: Interface,
    request: &Packet<'_>,
    output: &mut [u8],
) -> Result<usize, Error> {
    validate_interface(local)?;
    if request.operation() != Operation::Request {
        return Err(Error::NotRequest);
    }
    if request.target_ipv4() != local.ipv4
        || (request.destination_mac() != [255; 6] && request.destination_mac() != local.mac)
    {
        return Err(Error::TargetMismatch);
    }
    if request.is_probe() {
        return Err(Error::ProbeNotSupported);
    }
    encode(
        local,
        request.sender_mac(),
        request.sender_ipv4(),
        Operation::Reply,
        output,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pending {
    ip: [u8; 4],
    deadline: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Neighbor {
    ip: [u8; 4],
    mac: [u8; 6],
    expires: u64,
}

/// Single-slot state for a configured interface. The network service owns the
/// monotonic clock, route/subnet checks, request transmission and retry policy.
/// Unsolicited, late, foreign-target and wrong-IP replies never change a cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NeighborSlot {
    local: Interface,
    pending: Option<Pending>,
    neighbor: Option<Neighbor>,
}

impl NeighborSlot {
    pub fn new(local: Interface) -> Result<Self, Error> {
        validate_interface(local)?;
        Ok(Self {
            local,
            pending: None,
            neighbor: None,
        })
    }

    pub fn begin(&mut self, ip: [u8; 4], now: u64, timeout: u64) -> Result<(), Error> {
        if !unicast_ipv4(ip) || ip == self.local.ipv4 {
            return Err(Error::InvalidIpv4);
        }
        if timeout == 0 {
            return Err(Error::InvalidLifetime);
        }
        let deadline = now.checked_add(timeout).ok_or(Error::ClockOverflow)?;
        if self.pending.is_some_and(|pending| now < pending.deadline) {
            return Err(Error::Busy);
        }
        self.pending = Some(Pending { ip, deadline });
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.pending = None;
    }

    pub fn accept_reply(
        &mut self,
        frame: &[u8],
        now: u64,
        lifetime: u64,
    ) -> Result<[u8; 6], Error> {
        let pending = self.pending.ok_or(Error::NoPendingRequest)?;
        if now >= pending.deadline {
            self.pending = None;
            return Err(Error::Timeout);
        }
        if lifetime == 0 {
            return Err(Error::InvalidLifetime);
        }
        let expires = now.checked_add(lifetime).ok_or(Error::ClockOverflow)?;
        let packet = parse(frame)?;
        if packet.operation() != Operation::Reply
            || packet.target_ipv4() != self.local.ipv4
            || packet.target_mac() != self.local.mac
            || packet.sender_ipv4() != pending.ip
        {
            return Err(Error::TargetMismatch);
        }
        self.neighbor = Some(Neighbor {
            ip: pending.ip,
            mac: packet.sender_mac(),
            expires,
        });
        self.pending = None;
        Ok(packet.sender_mac())
    }

    /// A routing hint, never authenticated identity. Expiration is exclusive.
    pub fn lookup(&self, ip: [u8; 4], now: u64) -> Option<[u8; 6]> {
        self.neighbor
            .filter(|neighbor| neighbor.ip == ip && now < neighbor.expires)
            .map(|neighbor| neighbor.mac)
    }
}
