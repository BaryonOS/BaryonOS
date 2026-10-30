//! TPM2 command codecs, separate from CRB register I/O. Passwords are explicit
//! caller borrows; the encoder has no default credentials or crypto primitive.
pub const MAX_COMMAND: usize = 128;
pub const MAX_RESPONSE: usize = 512;
pub const MAX_PROPERTIES: usize = 16;
pub const NO_SESSIONS: u16 = 0x8001;
pub const SESSIONS: u16 = 0x8002;
pub const COUNTER_ATTRIBUTES: u32 = 0x0004_0014;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Input,
    Length,
    Protocol,
    Tpm(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NvIndex(u32);
impl NvIndex {
    pub fn new(value: u32) -> Result<Self, Error> {
        if (0x0100_0000..0x0200_0000).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::Input)
        }
    }
    pub const fn raw(self) -> u32 {
        self.0
    }
}
/// No Debug/Default: the bytes must not be logged or silently chosen. The
/// caller retains responsibility for entropy, protection and wiping its RAM.
#[derive(Clone, Copy)]
pub struct Password<'a>(&'a [u8]);
impl<'a> Password<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() <= 32 {
            Ok(Self(bytes))
        } else {
            Err(Error::Input)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Startup {
    Clear = 0,
    State = 1,
}
pub enum Request<'a> {
    Startup(Startup),
    SelfTest {
        full: bool,
    },
    Properties {
        first: u32,
        count: u8,
    },
    NvReadPublic {
        index: NvIndex,
    },
    NvDefineCounter {
        index: NvIndex,
        owner_auth: Password<'a>,
        index_auth: Password<'a>,
    },
    NvReadCounter {
        index: NvIndex,
        auth: Password<'a>,
    },
    NvIncrement {
        index: NvIndex,
        auth: Password<'a>,
    },
}
#[derive(Clone, Copy)]
pub(crate) enum Expectation {
    Empty,
    Properties { first: u32, count: u8 },
    Public { index: NvIndex },
    SessionEmpty,
    Counter,
}
pub struct Command<'a> {
    bytes: &'a [u8],
    pub(crate) expect: Expectation,
    pub(crate) sessions: bool,
    mutating: bool,
}
impl Command<'_> {
    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }
    /// True only for DefineCounter/Increment; false does not mean a command
    /// has no other TPM state or authorization side effects. Never replay
    /// automatically after an Unknown result.
    pub const fn is_nv_mutation(&self) -> bool {
        self.mutating
    }
}
struct Writer<'a> {
    bytes: &'a mut [u8],
    at: usize,
}
impl Writer<'_> {
    fn put(&mut self, data: &[u8]) {
        self.bytes[self.at..self.at + data.len()].copy_from_slice(data);
        self.at += data.len();
    }
    fn u16(&mut self, v: u16) {
        self.put(&v.to_be_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.put(&v.to_be_bytes());
    }
    fn auth(&mut self, p: Password<'_>) {
        self.u32(9 + p.0.len() as u32);
        self.u32(0x4000_0009);
        self.u16(0);
        self.put(&[0]);
        self.u16(p.0.len() as u16);
        self.put(p.0);
    }
    fn public(&mut self, index: NvIndex) {
        self.u16(14);
        self.u32(index.0);
        self.u16(0x000b);
        self.u32(COUNTER_ATTRIBUTES);
        self.u16(0);
        self.u16(8);
    }
}
/// All input/output bounds are checked before any output byte is changed.
/// The returned command borrows only the exact initialized request prefix.
pub fn encode<'a>(request: Request<'_>, out: &'a mut [u8]) -> Result<Command<'a>, Error> {
    let (len, cc, sessions, expect, mutating) = match &request {
        Request::Startup(_) => (12, 0x144, false, Expectation::Empty, false),
        Request::SelfTest { .. } => (11, 0x143, false, Expectation::Empty, false),
        Request::Properties { first, count } => {
            if *count == 0
                || usize::from(*count) > MAX_PROPERTIES
                || !(0x100..0x300).contains(first)
            {
                return Err(Error::Input);
            }
            (
                22,
                0x17a,
                false,
                Expectation::Properties {
                    first: *first,
                    count: *count,
                },
                false,
            )
        }
        Request::NvReadPublic { index } => (
            14,
            0x169,
            false,
            Expectation::Public { index: *index },
            false,
        ),
        Request::NvDefineCounter {
            owner_auth,
            index_auth,
            ..
        } => (
            45 + owner_auth.0.len() + index_auth.0.len(),
            0x12a,
            true,
            Expectation::SessionEmpty,
            true,
        ),
        Request::NvReadCounter { auth, .. } => {
            (35 + auth.0.len(), 0x14e, true, Expectation::Counter, false)
        }
        Request::NvIncrement { auth, .. } => (
            31 + auth.0.len(),
            0x134,
            true,
            Expectation::SessionEmpty,
            true,
        ),
    };
    if len > MAX_COMMAND || out.len() < len {
        return Err(Error::Length);
    }
    let mut w = Writer {
        bytes: &mut out[..len],
        at: 0,
    };
    w.u16(if sessions { SESSIONS } else { NO_SESSIONS });
    w.u32(len as u32);
    w.u32(cc);
    match request {
        Request::Startup(mode) => w.u16(mode as u16),
        Request::SelfTest { full } => w.put(&[u8::from(full)]),
        Request::Properties { first, count } => {
            w.u32(6);
            w.u32(first);
            w.u32(count.into());
        }
        Request::NvReadPublic { index } => w.u32(index.0),
        Request::NvDefineCounter {
            index,
            owner_auth,
            index_auth,
        } => {
            w.u32(0x4000_0001);
            w.auth(owner_auth);
            w.u16(index_auth.0.len() as u16);
            w.put(index_auth.0);
            w.public(index);
        }
        Request::NvReadCounter { index, auth } => {
            w.u32(index.0);
            w.u32(index.0);
            w.auth(auth);
            w.u16(8);
            w.u16(0);
        }
        Request::NvIncrement { index, auth } => {
            w.u32(index.0);
            w.u32(index.0);
            w.auth(auth);
        }
    }
    debug_assert_eq!(w.at, len);
    Ok(Command {
        bytes: &out[..len],
        expect,
        sessions,
        mutating,
    })
}
fn u16at(b: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(b[at..at + 2].try_into().unwrap())
}
fn u32at(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Property {
    pub property: u32,
    pub value: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    Empty,
    Properties {
        more: bool,
        count: u8,
        values: [Property; MAX_PROPERTIES],
    },
    NvPublic {
        index: NvIndex,
        attributes: u32,
        name: [u8; 34],
    },
    Counter(u64),
}
/// The wire size is trusted only after bounding it against both the device
/// buffer and our private response arena. No output is exposed on error.
pub(crate) fn response_size(header: &[u8], device_size: usize) -> Result<usize, Error> {
    if header.len() != 10 {
        return Err(Error::Length);
    }
    let size = u32at(header, 2) as usize;
    if !(10..=MAX_RESPONSE).contains(&size) || size > device_size {
        return Err(Error::Length);
    }
    Ok(size)
}
pub(crate) fn validate(
    expect: Expectation,
    sessions: bool,
    bytes: &[u8],
) -> Result<Response, Error> {
    if bytes.len() < 10 || bytes.len() > MAX_RESPONSE || u32at(bytes, 2) as usize != bytes.len() {
        return Err(Error::Length);
    }
    let tag = u16at(bytes, 0);
    let rc = u32at(bytes, 6);
    if rc != 0 {
        if tag != NO_SESSIONS || bytes.len() != 10 {
            return Err(Error::Protocol);
        }
        return Err(Error::Tpm(rc));
    }
    if tag != if sessions { SESSIONS } else { NO_SESSIONS } {
        return Err(Error::Protocol);
    }
    let body = if sessions {
        if bytes.len() < 19 {
            return Err(Error::Length);
        }
        let parameters = u32at(bytes, 10) as usize;
        let end = 14usize.checked_add(parameters).ok_or(Error::Length)?;
        // TPM_RS_PW responses have empty nonce/HMAC and continueSession=1.
        if end.checked_add(5) != Some(bytes.len()) || bytes[end..] != [0, 0, 1, 0, 0] {
            return Err(Error::Protocol);
        }
        &bytes[14..end]
    } else {
        &bytes[10..]
    };
    match expect {
        Expectation::Empty | Expectation::SessionEmpty => {
            if !body.is_empty() {
                return Err(Error::Length);
            }
            Ok(Response::Empty)
        }
        Expectation::Counter => {
            if body.len() != 10 || u16at(body, 0) != 8 {
                return Err(Error::Length);
            }
            Ok(Response::Counter(u64::from_be_bytes(
                body[2..10].try_into().unwrap(),
            )))
        }
        Expectation::Properties { first, count } => {
            if body.len() < 9 || body[0] > 1 || u32at(body, 1) != 6 {
                return Err(Error::Protocol);
            }
            let actual = u32at(body, 5) as usize;
            if actual > usize::from(count)
                || actual > MAX_PROPERTIES
                || body.len() != 9 + actual * 8
                || (actual == 0 && body[0] != 0)
            {
                return Err(Error::Length);
            }
            let mut values = [Property::default(); MAX_PROPERTIES];
            let mut last = None;
            for (i, value) in values.iter_mut().enumerate().take(actual) {
                let p = u32at(body, 9 + i * 8);
                if p < first || p >= 0x300 || last.is_some_and(|v| p <= v) {
                    return Err(Error::Protocol);
                }
                *value = Property {
                    property: p,
                    value: u32at(body, 13 + i * 8),
                };
                last = Some(p);
            }
            Ok(Response::Properties {
                more: body[0] != 0,
                count: actual as u8,
                values,
            })
        }
        Expectation::Public { index } => {
            if body.len() != 52
                || u16at(body, 0) != 14
                || u32at(body, 2) != index.0
                || u16at(body, 6) != 0x000b
                || u32at(body, 8) & !0x2000_0000 != COUNTER_ATTRIBUTES
                || u16at(body, 12) != 0
                || u16at(body, 14) != 8
                || u16at(body, 16) != 34
                || u16at(body, 18) != 0x000b
            {
                return Err(Error::Protocol);
            }
            let mut name = [0; 34];
            name.copy_from_slice(&body[18..52]);
            Ok(Response::NvPublic {
                index,
                attributes: u32at(body, 8),
                name,
            })
        }
    }
}
