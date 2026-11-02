//! Bounded clipboard RPC over the existing input pair. A validated outstanding
//! shortcut offer is intent, never permission to reach another app's memory.
use super::{Delivery, Offer, ProtocolError, Session, Transport};
use nieuw_os_sdk_runtime::{Error as RuntimeError, Outgoing};
use ui_core::{KeyAction, PhysicalKey};
use zeroize::{Zeroize, Zeroizing};

pub const TAG: u64 = u64::from_le_bytes(*b"CLIPBRK1");
pub const HEADER: usize = 64;
pub const MAX_TEXT: usize = 192;
pub const MAX_PAYLOAD: usize = HEADER + MAX_TEXT;
const VERSION: u16 = 1;
const REPLY: u16 = 0x8000;

/// Private, exact UTF-8 clipboard bytes. No plaintext Debug or Clone exists.
pub struct Text {
    bytes: [u8; MAX_TEXT],
    length: usize,
}
impl Text {
    pub const fn empty() -> Self {
        Self {
            bytes: [0; MAX_TEXT],
            length: 0,
        }
    }
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.length]).expect("validated clipboard UTF-8")
    }
    pub const fn len(&self) -> usize {
        self.length
    }
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }
}
impl TryFrom<&str> for Text {
    type Error = Error;
    fn try_from(text: &str) -> Result<Self, Error> {
        if text.len() > MAX_TEXT {
            return Err(Error::TooLong);
        }
        let mut owned = Self::empty();
        owned.bytes[..text.len()].copy_from_slice(text.as_bytes());
        owned.length = text.len();
        Ok(owned)
    }
}
impl Drop for Text {
    fn drop(&mut self) {
        self.bytes.zeroize();
        self.length.zeroize();
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Input(super::Error),
    Denied,
    Empty,
    TooLong,
    InvalidText,
    Used,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireError {
    Malformed,
    TooLong,
    InvalidText,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Operation {
    Copy = 1,
    Paste = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Status {
    Ok = 0,
    Denied = 1,
    Empty = 2,
    TooLong = 3,
    InvalidText = 4,
    Used = 5,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub operation: Operation,
    pub request: u64,
    pub epoch: u64,
    pub sequence: u64,
    pub generation: u64,
}
pub struct Request<'a> {
    pub header: Header,
    pub text: &'a str,
}
pub struct Reply<'a> {
    pub header: Header,
    pub status: Status,
    pub text: &'a str,
}

fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn valid(header: Header) -> bool {
    header.request != 0 && header.epoch != 0 && header.sequence != 0 && header.generation != 0
}
fn decode(bytes: &[u8], reply: bool) -> Result<(Header, Status, &str), WireError> {
    if bytes.len() < HEADER
        || bytes.len() > MAX_PAYLOAD
        || &bytes[..4] != b"CLP1"
        || bytes[4..6] != VERSION.to_le_bytes()
        || bytes[44..HEADER].iter().any(|b| *b != 0)
    {
        return Err(WireError::Malformed);
    }
    let raw = u16::from_le_bytes([bytes[6], bytes[7]]);
    if (raw & REPLY != 0) != reply {
        return Err(WireError::Malformed);
    }
    let operation = match raw & !REPLY {
        1 => Operation::Copy,
        2 => Operation::Paste,
        _ => return Err(WireError::Malformed),
    };
    let header = Header {
        operation,
        request: word(bytes, 8),
        epoch: word(bytes, 16),
        sequence: word(bytes, 24),
        generation: word(bytes, 32),
    };
    let length = usize::from(u16::from_le_bytes([bytes[40], bytes[41]]));
    if length > MAX_TEXT {
        return Err(WireError::TooLong);
    }
    if !valid(header) || bytes.len() != HEADER + length {
        return Err(WireError::Malformed);
    }
    let status = match u16::from_le_bytes([bytes[42], bytes[43]]) {
        0 => Status::Ok,
        1 if reply => Status::Denied,
        2 if reply => Status::Empty,
        3 if reply => Status::TooLong,
        4 if reply => Status::InvalidText,
        5 if reply => Status::Used,
        _ => return Err(WireError::Malformed),
    };
    let text = core::str::from_utf8(&bytes[HEADER..]).map_err(|_| WireError::InvalidText)?;
    validate_body(header.operation, status, text, reply)?;
    Ok((header, status, text))
}
fn validate_body(op: Operation, status: Status, text: &str, reply: bool) -> Result<(), WireError> {
    if text.len() > MAX_TEXT {
        return Err(WireError::TooLong);
    }
    let requires_text = if reply {
        status == Status::Ok && op == Operation::Paste
    } else {
        op == Operation::Copy
    };
    if requires_text {
        if text.is_empty() {
            return Err(WireError::InvalidText);
        }
    } else if !text.is_empty() {
        return Err(WireError::Malformed);
    }
    if !reply && status != Status::Ok {
        return Err(WireError::Malformed);
    }
    Ok(())
}
fn encode(
    header: Header,
    status: Status,
    text: &str,
    reply: bool,
    out: &mut [u8; MAX_PAYLOAD],
) -> Result<usize, WireError> {
    out.zeroize();
    if !valid(header) {
        return Err(WireError::Malformed);
    }
    validate_body(header.operation, status, text, reply)?;
    out[..4].copy_from_slice(b"CLP1");
    out[4..6].copy_from_slice(&VERSION.to_le_bytes());
    out[6..8].copy_from_slice(
        &((header.operation as u16) | if reply { REPLY } else { 0 }).to_le_bytes(),
    );
    for (at, value) in [
        (8, header.request),
        (16, header.epoch),
        (24, header.sequence),
        (32, header.generation),
    ] {
        out[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    out[40..42].copy_from_slice(&(text.len() as u16).to_le_bytes());
    out[42..44].copy_from_slice(&(status as u16).to_le_bytes());
    out[HEADER..HEADER + text.len()].copy_from_slice(text.as_bytes());
    Ok(HEADER + text.len())
}
impl<'a> Request<'a> {
    pub fn decode(bytes: &'a [u8]) -> Result<Self, WireError> {
        let (header, _, text) = decode(bytes, false)?;
        Ok(Self { header, text })
    }
    pub fn encode(&self, out: &mut [u8; MAX_PAYLOAD]) -> Result<usize, WireError> {
        encode(self.header, Status::Ok, self.text, false, out)
    }
    pub fn outgoing(&self) -> Result<Outgoing, WireError> {
        let mut bytes = Zeroizing::new([0; MAX_PAYLOAD]);
        let n = self.encode(&mut bytes)?;
        Outgoing::new(TAG, &bytes[..n]).map_err(|_| WireError::Malformed)
    }
}
impl<'a> Reply<'a> {
    pub fn decode(bytes: &'a [u8]) -> Result<Self, WireError> {
        let (header, status, text) = decode(bytes, true)?;
        Ok(Self {
            header,
            status,
            text,
        })
    }
    pub fn encode(&self, out: &mut [u8; MAX_PAYLOAD]) -> Result<usize, WireError> {
        encode(self.header, self.status, self.text, true, out)
    }
    pub fn outgoing(&self) -> Result<Outgoing, WireError> {
        let mut bytes = Zeroizing::new([0; MAX_PAYLOAD]);
        let n = self.encode(&mut bytes)?;
        Outgoing::new(TAG, &bytes[..n]).map_err(|_| WireError::Malformed)
    }
}

impl<T: Transport> Session<T> {
    pub fn clipboard_copy(&mut self, offer: &Offer, text: &str) -> Result<(), Error> {
        self.clipboard_eligible(offer, Operation::Copy)?;
        if text.is_empty() {
            return Err(Error::Empty);
        }
        if text.len() > MAX_TEXT {
            return Err(Error::TooLong);
        }
        self.clipboard_exchange(offer, Operation::Copy, text)
            .map(|_| ())
    }
    pub fn clipboard_paste(&mut self, offer: &Offer) -> Result<Text, Error> {
        self.clipboard_eligible(offer, Operation::Paste)?;
        match self.clipboard_exchange(offer, Operation::Paste, "")? {
            Some(text) => Ok(text),
            None => Err(self.clipboard_protocol(None)),
        }
    }
    fn clipboard_eligible(&self, offer: &Offer, operation: Operation) -> Result<(), Error> {
        if self.closed {
            return Err(Error::Input(super::Error::Closed));
        }
        if self.outstanding.as_ref() != Some(offer) {
            return Err(Error::Input(super::Error::WrongOffer));
        }
        let expected = match operation {
            Operation::Copy => PhysicalKey::C,
            Operation::Paste => PhysicalKey::V,
        };
        match offer.event() {
            super::Event::Key(key)
                if key.key == expected
                    && key.action == KeyAction::Press
                    && key.modifiers.control()
                    && !key.modifiers.alt()
                    && !key.modifiers.super_key()
                    && !key.modifiers.shift() => {}
            _ => return Err(Error::Denied),
        }
        if self.clipboard_used {
            return Err(Error::Used);
        }
        Ok(())
    }
    fn clipboard_protocol(&mut self, cleanup: Option<RuntimeError>) -> Error {
        Error::Input(self.interrupted(Delivery::Enqueued, None, Some(ProtocolError::Body), cleanup))
    }
    fn clipboard_exchange(
        &mut self,
        offer: &Offer,
        operation: Operation,
        text: &str,
    ) -> Result<Option<Text>, Error> {
        let id = self.next_id;
        self.next_id = id.checked_add(1).ok_or_else(|| {
            Error::Input(super::Error::RequestExhausted {
                cleanup: self.close().err(),
            })
        })?;
        let header = Header {
            operation,
            request: id,
            epoch: self.binding.epoch,
            sequence: offer.sequence(),
            generation: offer.generation(),
        };
        let outgoing = Request { header, text }
            .outgoing()
            .map_err(|_| self.clipboard_protocol(None))?;
        self.clipboard_used = true;
        let mut queued = false;
        for _ in 0..self.budget {
            match self.transport.send(self.binding.send, &outgoing) {
                Ok(()) => {
                    queued = true;
                    break;
                }
                Err(error) if error.is_again() => {}
                Err(error) => {
                    let delivery = if matches!(error,RuntimeError::Kernel(code) if code<0) {
                        Delivery::NotEnqueued
                    } else {
                        Delivery::Unknown
                    };
                    return Err(Error::Input(self.interrupted(
                        delivery,
                        Some(error),
                        None,
                        None,
                    )));
                }
            }
            if let Err(error) = self.transport.yield_now() {
                return Err(Error::Input(self.interrupted(
                    Delivery::NotEnqueued,
                    Some(error),
                    None,
                    None,
                )));
            }
        }
        if !queued {
            return Err(Error::Input(self.interrupted(
                Delivery::NotEnqueued,
                None,
                None,
                None,
            )));
        }
        for _ in 0..self.budget {
            let message = match self.transport.receive(self.binding.receive) {
                Ok(message) => message,
                Err(error) if error.is_again() => {
                    if let Err(error) = self.transport.yield_now() {
                        return Err(Error::Input(self.interrupted(
                            Delivery::Enqueued,
                            Some(error),
                            None,
                            None,
                        )));
                    }
                    continue;
                }
                Err(error) => {
                    return Err(Error::Input(self.interrupted(
                        Delivery::Enqueued,
                        Some(error),
                        None,
                        None,
                    )))
                }
            };
            let cleanup = self.close_received_caps(&message);
            if message.cap_count() != 0
                || cleanup.is_some()
                || message.sender() != self.binding.service_pid
                || message.tag() != TAG
            {
                return Err(self.clipboard_protocol(cleanup));
            }
            let reply =
                Reply::decode(message.payload()).map_err(|_| self.clipboard_protocol(None))?;
            if reply.header != header {
                return Err(self.clipboard_protocol(None));
            }
            return match reply.status {
                Status::Ok if operation == Operation::Copy => Ok(None),
                Status::Ok => Text::try_from(reply.text).map(Some),
                Status::Denied => Err(Error::Denied),
                Status::Empty => Err(Error::Empty),
                Status::TooLong => Err(Error::TooLong),
                Status::InvalidText => Err(Error::InvalidText),
                Status::Used => Err(Error::Used),
            };
        }
        Err(Error::Input(self.interrupted(
            Delivery::Enqueued,
            None,
            None,
            None,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(operation: Operation) -> Header {
        Header {
            operation,
            request: 2,
            epoch: 3,
            sequence: 4,
            generation: 5,
        }
    }
    #[test]
    fn canonical_wire_positions_and_exact_utf8_are_preserved() {
        let request = Request {
            header: header(Operation::Copy),
            text: "é猫\0",
        };
        let mut bytes = [0xa5; MAX_PAYLOAD];
        let length = request.encode(&mut bytes).unwrap();
        assert_eq!(length, 70);
        assert_eq!(&bytes[..8], b"CLP1\x01\x00\x01\x00");
        for (at, value) in [(8, 2u64), (16, 3), (24, 4), (32, 5)] {
            assert_eq!(&bytes[at..at + 8], &value.to_le_bytes());
        }
        assert_eq!(&bytes[40..44], &[6, 0, 0, 0]);
        assert!(bytes[44..HEADER].iter().all(|b| *b == 0));
        assert_eq!(&bytes[HEADER..length], request.text.as_bytes());
        assert!(bytes[length..].iter().all(|b| *b == 0));
        let decoded = Request::decode(&bytes[..length]).unwrap();
        assert_eq!(decoded.header, request.header);
        assert_eq!(decoded.text, request.text);
        let outgoing = Reply {
            header: header(Operation::Paste),
            status: Status::Ok,
            text: request.text,
        }
        .outgoing()
        .unwrap();
        assert_eq!(&outgoing.as_bytes()[64 + 6..64 + 8], &[2, 0x80]);
        let decoded = Reply::decode(&outgoing.as_bytes()[64..64 + length]).unwrap();
        let text = Text::try_from(decoded.text).unwrap();
        assert_eq!(text.as_str(), request.text);
        assert_eq!(text.len(), 6);
    }
    #[test]
    fn maximum_body_is_in_bytes_and_oversize_encoding_clears_scratch() {
        let mut utf8 = [0; MAX_TEXT];
        for pair in utf8.chunks_exact_mut(2) {
            pair.copy_from_slice(&[0xc3, 0xa9]);
        }
        let text = core::str::from_utf8(&utf8).unwrap();
        let mut bytes = [0xa5; MAX_PAYLOAD];
        assert_eq!(
            Request {
                header: header(Operation::Copy),
                text
            }
            .encode(&mut bytes),
            Ok(MAX_PAYLOAD)
        );
        assert_eq!(Request::decode(&bytes).unwrap().text.as_bytes(), utf8);
        let oversized = [b'a'; MAX_TEXT + 1];
        let text = core::str::from_utf8(&oversized).unwrap();
        assert_eq!(
            Request {
                header: header(Operation::Copy),
                text
            }
            .encode(&mut bytes),
            Err(WireError::TooLong)
        );
        assert!(bytes.iter().all(|b| *b == 0));
        assert_eq!(Text::try_from(text).err(), Some(Error::TooLong));
    }
    #[test]
    fn malformed_header_body_and_status_cannot_be_accepted() {
        let outgoing = Request {
            header: header(Operation::Copy),
            text: "x",
        }
        .outgoing()
        .unwrap();
        let payload = &outgoing.as_bytes()[64..64 + HEADER + 1];
        for (index, value) in [
            (0, b'X'),
            (4, 2),
            (6, 3),
            (7, 0x80),
            (8, 0),
            (16, 0),
            (24, 0),
            (32, 0),
            (40, 0),
            (42, 1),
            (44, 1),
            (63, 1),
        ] {
            let mut bad = [0; HEADER + 1];
            bad.copy_from_slice(payload);
            bad[index] = value;
            assert!(Request::decode(&bad).is_err(), "header index {index}");
        }
        let mut bad = [0; HEADER + 1];
        bad.copy_from_slice(payload);
        bad[HEADER] = 0xff;
        assert_eq!(Request::decode(&bad).err(), Some(WireError::InvalidText));
        bad.copy_from_slice(payload);
        bad[40] = 193;
        assert_eq!(Request::decode(&bad).err(), Some(WireError::TooLong));
        assert!(Request::decode(&payload[..HEADER - 1]).is_err());
        assert!(Request::decode(&payload[..HEADER]).is_err());
        let mut bytes = [0; MAX_PAYLOAD];
        for (op, status, text) in [
            (Operation::Paste, Status::Ok, ""),
            (Operation::Copy, Status::Ok, "x"),
            (Operation::Paste, Status::Denied, "x"),
        ] {
            assert!(Reply {
                header: header(op),
                status,
                text
            }
            .encode(&mut bytes)
            .is_err());
        }
        assert_eq!(
            Request {
                header: header(Operation::Copy),
                text: ""
            }
            .encode(&mut bytes),
            Err(WireError::InvalidText)
        );
        assert!(Text::empty().is_empty());
        assert!(Text::try_from("").unwrap().is_empty());
    }
}
