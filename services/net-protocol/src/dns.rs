//! Bounded DNS A/AAAA queries and validated UDP replies (RFC 1035/2181).
//!
//! No allocator, sockets, DNSSEC validation or authority is provided here.
//! Transaction IDs and ports must come from the authorized resolver's entropy
//! source. DNS answers are routing data; TLS must still authenticate a peer.

use crate::{arp::unicast_ipv4, UdpChecksum, UdpDatagram};

pub const MAX_MESSAGE_BYTES: usize = 4096;
pub const MAX_RECORDS: usize = 32;
pub const MAX_ADDRESSES: usize = 8;
pub const MAX_CNAME_HOPS: usize = 8;
pub const MAX_POINTERS: usize = 32;
/// Conservative local cache cap, in seconds, not a modification of wire TTLs.
pub const MAX_CACHE_TTL: u32 = 86_400;
const HEADER: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Truncated,
    MessageTooLarge,
    InvalidName,
    NameTooLong,
    InvalidPointer,
    CompressionLoop,
    CompressionLimit,
    WrongId,
    InvalidFlags,
    TruncatedResponse,
    ResponseCode(u8),
    WrongQuestion,
    TooManyRecords,
    InvalidRecord,
    UnsupportedEdns,
    TrailingBytes,
    OutputTooSmall,
    NoAddress,
    ConflictingAnswers,
    CnameLoop,
    TooManyAliases,
    TooManyAddresses,
    InvalidEndpoint,
    WrongEndpoint,
    ChecksumRequired,
    InvalidLifetime,
    ClockOverflow,
    Timeout,
    NotPending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum QueryType {
    A = 1,
    Aaaa = 28,
}

/// Canonical uncompressed wire name; ASCII letters compare case-insensitively.
/// Query construction accepts ASCII hostnames (IDNA conversion is separate).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Name {
    bytes: [u8; 255],
    len: u8,
}

impl Name {
    const fn root() -> Self {
        Self {
            bytes: [0; 255],
            len: 1,
        }
    }

    pub fn from_ascii(hostname: &str) -> Result<Self, Error> {
        if hostname == "." {
            return Ok(Self::root());
        }
        let text = hostname.strip_suffix('.').unwrap_or(hostname);
        if text.is_empty() {
            return Err(Error::InvalidName);
        }
        let mut name = Self::root();
        let mut end = 0;
        for label in text.as_bytes().split(|&byte| byte == b'.') {
            if label.is_empty()
                || label.len() > 63
                || !label[0].is_ascii_alphanumeric()
                || !label[label.len() - 1].is_ascii_alphanumeric()
                || !label
                    .iter()
                    .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
            {
                return Err(Error::InvalidName);
            }
            if end + 1 + label.len() >= name.bytes.len() {
                return Err(Error::NameTooLong);
            }
            name.bytes[end] = label.len() as u8;
            end += 1;
            for byte in label {
                name.bytes[end] = byte.to_ascii_lowercase();
                end += 1;
            }
        }
        // The buffer starts zeroed, including the final root label.
        name.len = (end + 1) as u8;
        Ok(name)
    }

    pub fn wire(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len)]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Question {
    pub id: u16,
    pub name: Name,
    pub kind: QueryType,
}

/// One IN question, recursion desired, no EDNS/DNSSEC flags. All failures are
/// detected before output is modified. Tail bytes are never transmitted.
pub fn build_query(question: Question, output: &mut [u8]) -> Result<usize, Error> {
    let size = HEADER + question.name.wire().len() + 4;
    if output.len() < size {
        return Err(Error::OutputTooSmall);
    }
    output[..size].fill(0);
    output[..2].copy_from_slice(&question.id.to_be_bytes());
    output[2] = 1; // RD, standard query.
    output[5] = 1; // QDCOUNT.
    let end = HEADER + question.name.wire().len();
    output[HEADER..end].copy_from_slice(question.name.wire());
    output[end..end + 2].copy_from_slice(&(question.kind as u16).to_be_bytes());
    output[end + 2..end + 4].copy_from_slice(&1u16.to_be_bytes());
    Ok(size)
}

fn word(message: &[u8], at: usize) -> Result<u16, Error> {
    let data = message.get(at..at + 2).ok_or(Error::Truncated)?;
    Ok(u16::from_be_bytes([data[0], data[1]]))
}

fn dword(message: &[u8], at: usize) -> Result<u32, Error> {
    let data = message.get(at..at + 4).ok_or(Error::Truncated)?;
    Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
}

/// Decode a name whose original encoding must end within `encoded_end`.
/// Backward compression can refer outside that region, but never into the
/// header, forward, or outside the message. Track targets to catch label/pointer
/// cycles, not just chains of adjacent pointers. Both name and work are bounded.
fn decode_name(message: &[u8], start: usize, encoded_end: usize) -> Result<(Name, usize), Error> {
    let mut cursor = start;
    let mut end = None;
    let mut limit = encoded_end.min(message.len());
    let mut name = Name::root();
    let mut written = 0;
    let mut targets = [0usize; MAX_POINTERS];
    let mut target_count = 0;
    loop {
        if cursor >= limit {
            return Err(Error::Truncated);
        }
        let length = message[cursor];
        if length & 0xc0 == 0xc0 {
            if cursor + 1 >= limit {
                return Err(Error::Truncated);
            }
            let target = (usize::from(length & 0x3f) << 8) | usize::from(message[cursor + 1]);
            if target < HEADER || target >= cursor || target >= message.len() {
                return Err(Error::InvalidPointer);
            }
            if targets[..target_count].contains(&target) {
                return Err(Error::CompressionLoop);
            }
            if target_count == MAX_POINTERS {
                return Err(Error::CompressionLimit);
            }
            targets[target_count] = target;
            target_count += 1;
            end.get_or_insert(cursor + 2);
            cursor = target;
            limit = message.len();
            continue;
        }
        if length & 0xc0 != 0 {
            return Err(Error::InvalidName);
        }
        cursor += 1;
        if length == 0 {
            if written >= name.bytes.len() {
                return Err(Error::NameTooLong);
            }
            name.bytes[written] = 0;
            name.len = (written + 1) as u8;
            return Ok((name, end.unwrap_or(cursor)));
        }
        let length = usize::from(length);
        if cursor + length > limit {
            return Err(Error::Truncated);
        }
        if written + 1 + length >= name.bytes.len() {
            return Err(Error::NameTooLong);
        }
        name.bytes[written] = length as u8;
        written += 1;
        for byte in &message[cursor..cursor + length] {
            name.bytes[written] = byte.to_ascii_lowercase();
            written += 1;
        }
        cursor += length;
    }
}

/// A validated, borrowed encoding. The immutable message must outlive the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameRef<'a> {
    message: &'a [u8],
    start: usize,
    end: usize,
}

impl NameRef<'_> {
    pub fn to_name(&self) -> Result<Name, Error> {
        decode_name(self.message, self.start, self.end).map(|(name, _)| name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Answer,
    Authority,
    Additional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordData<'a> {
    A([u8; 4]),
    Aaaa([u8; 16]),
    Cname(NameRef<'a>),
    /// Other classes/types remain opaque and never supply resolved addresses.
    Opaque(&'a [u8]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record<'a> {
    pub owner: NameRef<'a>,
    pub section: Section,
    pub class: u16,
    pub kind: u16,
    /// RFC 2181: high-bit wire TTL means zero. Otherwise capped for caching.
    pub cache_ttl: u32,
    pub data: RecordData<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response<'a> {
    question: Question,
    records: [Option<Record<'a>>; MAX_RECORDS],
    count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Address {
    Ipv4([u8; 4]),
    Ipv6([u8; 16]),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub canonical_name: Name,
    /// Minimum of the relevant alias/address TTLs, capped at MAX_CACHE_TTL.
    /// Zero may be used for this response, but must not be cached.
    pub cache_ttl: u32,
    addresses: [Option<Address>; MAX_ADDRESSES],
    count: usize,
}

impl Resolution {
    pub fn addresses(&self) -> impl Iterator<Item = &Address> {
        self.addresses[..self.count].iter().flatten()
    }
}

impl<'a> Response<'a> {
    pub fn records(&self) -> impl Iterator<Item = &Record<'a>> {
        self.records[..self.count].iter().flatten()
    }

    /// Only IN answers rooted in the exact requested name may provide addresses.
    /// Additional-section glue and unrelated answers never enter the result.
    /// CNAMEs are followed independently of record order, with cycle/work bounds.
    pub fn resolve(&self) -> Result<Resolution, Error> {
        let mut current = self.question.name;
        let mut seen = [Name::root(); MAX_CNAME_HOPS + 1];
        let mut path_ttl = MAX_CACHE_TTL;
        for hop in 0..=MAX_CNAME_HOPS {
            if seen[..hop].contains(&current) {
                return Err(Error::CnameLoop);
            }
            seen[hop] = current;
            let mut result = Resolution {
                canonical_name: current,
                cache_ttl: path_ttl,
                addresses: [None; MAX_ADDRESSES],
                count: 0,
            };
            let mut alias = None;
            let mut alias_ttl = MAX_CACHE_TTL;
            let mut has_address = false;
            for record in self.records() {
                if record.section != Section::Answer
                    || record.class != 1
                    || record.owner.to_name()? != current
                {
                    continue;
                }
                has_address |= matches!(record.data, RecordData::A(_) | RecordData::Aaaa(_));
                let address = match (record.data, self.question.kind) {
                    (RecordData::A(ip), QueryType::A) => Some(Address::Ipv4(ip)),
                    (RecordData::Aaaa(ip), QueryType::Aaaa) => Some(Address::Ipv6(ip)),
                    (RecordData::Cname(target), _) => {
                        let target = target.to_name()?;
                        if alias.is_some_and(|previous| previous != target) {
                            return Err(Error::ConflictingAnswers);
                        }
                        alias = Some(target);
                        alias_ttl = alias_ttl.min(record.cache_ttl);
                        None
                    }
                    _ => None,
                };
                if let Some(address) = address {
                    result.cache_ttl = result.cache_ttl.min(record.cache_ttl);
                    if result.addresses[..result.count].contains(&Some(address)) {
                        continue;
                    }
                    if result.count == MAX_ADDRESSES {
                        return Err(Error::TooManyAddresses);
                    }
                    result.addresses[result.count] = Some(address);
                    result.count += 1;
                }
            }
            if let Some(target) = alias {
                if has_address {
                    return Err(Error::ConflictingAnswers);
                }
                if hop == MAX_CNAME_HOPS {
                    return Err(Error::TooManyAliases);
                }
                current = target;
                path_ttl = path_ttl.min(alias_ttl);
            } else if result.count != 0 {
                return Ok(result);
            } else {
                return Err(Error::NoAddress);
            }
        }
        Err(Error::TooManyAliases)
    }
}

/// Fully validate the question, declared records and message bounds before
/// exposing any record. A truncated UDP reply needs a separate TCP retry; this
/// module never accepts its partial answers or silently changes transport.
pub fn parse_response(message: &[u8], expected: Question) -> Result<Response<'_>, Error> {
    if message.len() < HEADER {
        return Err(Error::Truncated);
    }
    if message.len() > MAX_MESSAGE_BYTES {
        return Err(Error::MessageTooLarge);
    }
    if word(message, 0)? != expected.id {
        return Err(Error::WrongId);
    }
    let flags = word(message, 2)?;
    if flags & 0xf840 != 0x8000 {
        return Err(Error::InvalidFlags);
    }
    if flags & 0x0200 != 0 {
        return Err(Error::TruncatedResponse);
    }
    if word(message, 4)? != 1 {
        return Err(Error::WrongQuestion);
    }
    let (name, question_end) = decode_name(message, HEADER, message.len())?;
    if name != expected.name
        || word(message, question_end)? != expected.kind as u16
        || word(message, question_end + 2)? != 1
    {
        return Err(Error::WrongQuestion);
    }
    let rcode = (flags & 15) as u8;
    if rcode != 0 {
        return Err(Error::ResponseCode(rcode));
    }
    let answers = usize::from(word(message, 6)?);
    let authority = usize::from(word(message, 8)?);
    let additional = usize::from(word(message, 10)?);
    let count = answers + authority + additional;
    if count > MAX_RECORDS {
        return Err(Error::TooManyRecords);
    }
    let mut response = Response {
        question: expected,
        records: [None; MAX_RECORDS],
        count,
    };
    let mut cursor = question_end + 4;
    for index in 0..count {
        let start = cursor;
        let (_, header) = decode_name(message, start, message.len())?;
        let kind = word(message, header)?;
        let class = word(message, header + 2)?;
        let ttl = dword(message, header + 4)?;
        let size = usize::from(word(message, header + 8)?);
        let data_start = header + 10;
        let data_end = data_start + size;
        let data = message.get(data_start..data_end).ok_or(Error::Truncated)?;
        let data = match (class, kind) {
            // No OPT was sent in build_query. Extended rcodes/options must not
            // be misinterpreted as an ordinary successful DNS response.
            (_, 41) => return Err(Error::UnsupportedEdns),
            (1, 1) => RecordData::A(data.try_into().map_err(|_| Error::InvalidRecord)?),
            (1, 28) => RecordData::Aaaa(data.try_into().map_err(|_| Error::InvalidRecord)?),
            (1, 5) => {
                let (_, end) = decode_name(message, data_start, data_end)?;
                if end != data_end {
                    return Err(Error::InvalidRecord);
                }
                RecordData::Cname(NameRef {
                    message,
                    start: data_start,
                    end: data_end,
                })
            }
            _ => RecordData::Opaque(data),
        };
        response.records[index] = Some(Record {
            owner: NameRef {
                message,
                start,
                end: header,
            },
            section: if index < answers {
                Section::Answer
            } else if index < answers + authority {
                Section::Authority
            } else {
                Section::Additional
            },
            class,
            kind,
            cache_ttl: if ttl & 0x8000_0000 == 0 {
                ttl.min(MAX_CACHE_TTL)
            } else {
                0
            },
            data,
        });
        cursor = data_end;
    }
    if cursor != message.len() {
        return Err(Error::TrailingBytes);
    }
    Ok(response)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub address: [u8; 4],
    pub port: u16,
}

/// A single outstanding UDP lookup, bound to both endpoints and a monotonic
/// deadline. Accepted replies consume it; malformed/unrelated replies do not.
/// Caller controls cancellation, retry/TCP policy and all network capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingQuery {
    question: Question,
    local: Endpoint,
    server: Endpoint,
    deadline: u64,
    pending: bool,
}

impl PendingQuery {
    pub fn new(
        question: Question,
        local: Endpoint,
        server: Endpoint,
        now: u64,
        timeout: u64,
    ) -> Result<Self, Error> {
        if !unicast_ipv4(local.address)
            || !unicast_ipv4(server.address)
            || local.port == 0
            || server.port == 0
        {
            return Err(Error::InvalidEndpoint);
        }
        if timeout == 0 {
            return Err(Error::InvalidLifetime);
        }
        Ok(Self {
            question,
            local,
            server,
            deadline: now.checked_add(timeout).ok_or(Error::ClockOverflow)?,
            pending: true,
        })
    }

    pub fn cancel(&mut self) {
        self.pending = false;
    }

    pub fn accept<'a>(
        &mut self,
        packet: &UdpDatagram<'a>,
        now: u64,
    ) -> Result<Response<'a>, Error> {
        if !self.pending {
            return Err(Error::NotPending);
        }
        if now >= self.deadline {
            self.pending = false;
            return Err(Error::Timeout);
        }
        if packet.source_address() != self.server.address
            || packet.source_port() != self.server.port
            || packet.destination_address() != self.local.address
            || packet.destination_port() != self.local.port
        {
            return Err(Error::WrongEndpoint);
        }
        if packet.checksum() == UdpChecksum::Absent {
            return Err(Error::ChecksumRequired);
        }
        let response = parse_response(packet.payload(), self.question)?;
        self.pending = false;
        Ok(response)
    }
}
