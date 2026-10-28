#![no_std]
#![forbid(unsafe_code)]
//! Bounded Ethernet II / IPv4 / UDP header parsing, with no I/O or authority.
//!
//! Input is immutable driver-owned bytes beginning at the destination MAC,
//! without FCS. Link padding may be stripped or present to the 60-byte Ethernet
//! minimum; it is exposed separately, never included in the UDP payload.
//! Only IPv4 IHL=5, unfragmented UDP is supported. This is not a network stack.
//! The Internet checksum detects some errors; it does not authenticate a peer.

pub const ETHERNET_HEADER_BYTES: usize = 14;
pub const IPV4_HEADER_BYTES: usize = 20;
pub const UDP_HEADER_BYTES: usize = 8;
pub const MAX_IPV4_BYTES: usize = u16::MAX as usize;
const MIN_ETHERNET_DATA_BYTES: usize = 46;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Ethernet,
    Ipv4,
    Udp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    EtherType(u16),
    Vlan(u16),
    IpVersion(u8),
    Ipv4Options(u8),
    Ipv4Fragmentation(u16),
    IpProtocol(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Truncated {
        layer: Layer,
        needed: usize,
        available: usize,
    },
    Unsupported(Unsupported),
    InvalidIpv4HeaderLength(u8),
    InvalidIpv4TotalLength(u16),
    InvalidIpv4Flags(u16),
    InvalidIpv4Checksum,
    InvalidUdpLength(u16),
    UdpLengthMismatch {
        declared: u16,
        ip_payload: usize,
    },
    InvalidUdpChecksum,
    UnexpectedEthernetLength {
        actual: usize,
        datagram_end: usize,
        padded_end: usize,
    },
    LengthOverflow,
}

/// A zero checksum is permitted by UDP over IPv4, but is not a successful
/// checksum verification. `Valid` means only the 16-bit error check passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UdpChecksum {
    Absent,
    Valid(u16),
}

/// All slices borrow the unmodified input. Addresses and ports are untrusted
/// packet fields, not authenticated identities, endpoint grants or rights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UdpDatagram<'a> {
    frame: &'a [u8],
    ip: &'a [u8],
    udp: &'a [u8],
    checksum: UdpChecksum,
}

impl<'a> UdpDatagram<'a> {
    pub fn frame(&self) -> &'a [u8] {
        self.frame
    }
    pub fn ethernet_header(&self) -> &'a [u8] {
        &self.frame[..ETHERNET_HEADER_BYTES]
    }
    pub fn ipv4_packet(&self) -> &'a [u8] {
        self.ip
    }
    pub fn udp_packet(&self) -> &'a [u8] {
        self.udp
    }
    pub fn payload(&self) -> &'a [u8] {
        &self.udp[UDP_HEADER_BYTES..]
    }
    pub fn ethernet_padding(&self) -> &'a [u8] {
        &self.frame[ETHERNET_HEADER_BYTES + self.ip.len()..]
    }
    pub fn source_mac(&self) -> [u8; 6] {
        let mut address = [0; 6];
        address.copy_from_slice(&self.frame[6..12]);
        address
    }
    pub fn destination_mac(&self) -> [u8; 6] {
        let mut address = [0; 6];
        address.copy_from_slice(&self.frame[..6]);
        address
    }
    pub fn source_address(&self) -> [u8; 4] {
        let mut address = [0; 4];
        address.copy_from_slice(&self.ip[12..16]);
        address
    }
    pub fn destination_address(&self) -> [u8; 4] {
        let mut address = [0; 4];
        address.copy_from_slice(&self.ip[16..20]);
        address
    }
    pub fn source_port(&self) -> u16 {
        word(self.udp, 0)
    }
    pub fn destination_port(&self) -> u16 {
        word(self.udp, 2)
    }
    pub fn ttl(&self) -> u8 {
        self.ip[8]
    }
    pub fn dont_fragment(&self) -> bool {
        word(self.ip, 6) & 0x4000 != 0
    }
    pub fn checksum(&self) -> UdpChecksum {
        self.checksum
    }
}

fn require_length(bytes: &[u8], needed: usize, layer: Layer) -> Result<(), Error> {
    if bytes.len() < needed {
        Err(Error::Truncated {
            layer,
            needed,
            available: bytes.len(),
        })
    } else {
        Ok(())
    }
}

// Called only after the enclosing header has been length checked.
fn word(header: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([header[offset], header[offset + 1]])
}

/// Validate one FCS-stripped Ethernet frame or a driver-stripped padding view.
/// No allocation, retained mutable access, logging, transport or capability
/// lookup occurs. Work is bounded by the 16-bit IPv4 length. MTU enforcement,
/// TTL/routing policy, FCS/offload status and stable DMA-buffer ownership belong
/// to the future driver/service, not this borrowed header parser.
pub fn parse(frame: &[u8]) -> Result<UdpDatagram<'_>, Error> {
    require_length(frame, ETHERNET_HEADER_BYTES, Layer::Ethernet)?;
    match word(frame, 12) {
        0x0800 => (),
        tag @ (0x8100 | 0x88a8) => return Err(Error::Unsupported(Unsupported::Vlan(tag))),
        other => return Err(Error::Unsupported(Unsupported::EtherType(other))),
    }
    let data = &frame[ETHERNET_HEADER_BYTES..];
    require_length(data, IPV4_HEADER_BYTES, Layer::Ipv4)?;
    let version = data[0] >> 4;
    if version != 4 {
        return Err(Error::Unsupported(Unsupported::IpVersion(version)));
    }
    let ihl = data[0] & 15;
    if ihl < 5 {
        return Err(Error::InvalidIpv4HeaderLength(ihl));
    }
    if ihl != 5 {
        return Err(Error::Unsupported(Unsupported::Ipv4Options(ihl)));
    }
    let declared = word(data, 2);
    let total = usize::from(declared);
    if total < IPV4_HEADER_BYTES {
        return Err(Error::InvalidIpv4TotalLength(declared));
    }
    require_length(data, total, Layer::Ipv4)?;
    let ip = &data[..total];
    if internet_checksum(&[&ip[..IPV4_HEADER_BYTES]]) != 0 {
        return Err(Error::InvalidIpv4Checksum);
    }
    let flags_offset = word(ip, 6);
    if flags_offset & 0x8000 != 0 {
        return Err(Error::InvalidIpv4Flags(flags_offset));
    }
    if flags_offset & 0x3fff != 0 {
        return Err(Error::Unsupported(Unsupported::Ipv4Fragmentation(
            flags_offset,
        )));
    }
    if ip[9] != 17 {
        return Err(Error::Unsupported(Unsupported::IpProtocol(ip[9])));
    }
    let udp = &ip[IPV4_HEADER_BYTES..];
    require_length(udp, UDP_HEADER_BYTES, Layer::Udp)?;
    let udp_length = word(udp, 4);
    if usize::from(udp_length) < UDP_HEADER_BYTES {
        return Err(Error::InvalidUdpLength(udp_length));
    }
    if usize::from(udp_length) != udp.len() {
        return Err(Error::UdpLengthMismatch {
            declared: udp_length,
            ip_payload: udp.len(),
        });
    }
    let transmitted = word(udp, 6);
    let checksum = if transmitted == 0 {
        UdpChecksum::Absent
    } else {
        let pseudo_suffix = [0, ip[9], udp[4], udp[5]];
        if internet_checksum(&[&ip[12..20], &pseudo_suffix, udp]) != 0 {
            return Err(Error::InvalidUdpChecksum);
        }
        UdpChecksum::Valid(transmitted)
    };
    let datagram_end = ETHERNET_HEADER_BYTES
        .checked_add(total)
        .ok_or(Error::LengthOverflow)?;
    let padded_end = ETHERNET_HEADER_BYTES
        .checked_add(total.max(MIN_ETHERNET_DATA_BYTES))
        .ok_or(Error::LengthOverflow)?;
    if frame.len() != datagram_end && frame.len() != padded_end {
        return Err(Error::UnexpectedEthernetLength {
            actual: frame.len(),
            datagram_end,
            padded_end,
        });
    }
    Ok(UdpDatagram {
        frame,
        ip,
        udp,
        checksum,
    })
}

// RFC 1071 one's-complement addition. Fold after every word: the accumulator
// remains <= 0xffff between additions and cannot overflow u32. An odd final
// octet is the high byte of a zero-padded word, including across chunk borders.
// This is error detection, not cryptography or authentication.
fn internet_checksum(chunks: &[&[u8]]) -> u16 {
    let mut sum = 0u32;
    let mut high = None;
    for chunk in chunks {
        for &octet in *chunk {
            if let Some(first) = high.take() {
                sum += u32::from(u16::from_be_bytes([first, octet]));
                sum = (sum & 0xffff) + (sum >> 16);
            } else {
                high = Some(octet);
            }
        }
    }
    if let Some(last) = high {
        sum += u32::from(last) << 8;
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

#[cfg(test)]
mod checksum_tests {
    use super::internet_checksum;

    #[test]
    fn rfc1071_known_sum_and_every_chunk_boundary() {
        let bytes = [0x00, 0x01, 0xf2, 0x03, 0xf4, 0xf5, 0xf6, 0xf7];
        // RFC 1071 section 3 gives the folded sum 0xddf2; complement 0x220d.
        for split in 0..=bytes.len() {
            assert_eq!(
                internet_checksum(&[&bytes[..split], &bytes[split..]]),
                0x220d
            );
        }
    }

    #[test]
    fn odd_octet_uses_high_byte_and_end_around_carry() {
        assert_eq!(internet_checksum(&[&[0x12], &[0x34, 0x56]]), 0x97cb);
        assert_eq!(internet_checksum(&[&[0xff; 257]]), 0x00ff);
        assert_eq!(internet_checksum(&[&[]]), 0xffff);
    }
}
