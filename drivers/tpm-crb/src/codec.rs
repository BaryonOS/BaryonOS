//! TPM2 command codecs, separate from CRB register I/O. Passwords are explicit
//! caller borrows; the encoder has no default credentials or crypto primitive.
pub const MAX_COMMAND: usize = 128;
pub const MAX_RESPONSE: usize = 512;
pub const MAX_PROPERTIES: usize = 16;
pub const NO_SESSIONS: u16 = 0x8001;
pub const SESSIONS: u16 = 0x8002;
pub const COUNTER_ATTRIBUTES: u32 = 0x0004_0014;
/// Largest NV payload any REL-05 index carries (section 6.1: 64-byte records).
pub const MAX_NV_DATA: usize = 64;
/// SHA-256 digest size; the only hash this codec speaks.
pub const DIGEST: usize = 32;
/// Highest PCR index addressable by the 3-byte selection (PC client: 24 PCRs).
pub const MAX_PCR: u8 = 23;
const ALG_SHA256: u16 = 0x000b;
const ALG_NULL: u16 = 0x0010;
const RH_NULL: u32 = 0x4000_0007;
const RH_OWNER: u32 = 0x4000_0001;
const SE_POLICY: u8 = 0x01;
const NT_ORDINARY: u32 = 0;
const NT_COUNTER: u32 = 1;
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
/// Handle of a policy session returned by `StartAuthSession` (Part 2: HR_POLICY_SESSION).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicySession(u32);
impl PolicySession {
    pub fn new(value: u32) -> Result<Self, Error> {
        if value >> 24 == 0x03 {
            Ok(Self(value))
        } else {
            Err(Error::Input)
        }
    }
    pub const fn raw(self) -> u32 {
        self.0
    }
}
/// Authorization for one command. A policy session is sent with an empty
/// nonce, empty HMAC and `continueSession` clear, so the TPM flushes it after
/// this single use; REL-05 policies contain no PolicyAuthValue/PolicyPassword.
#[derive(Clone, Copy)]
pub enum Auth<'a> {
    Password(Password<'a>),
    Policy(PolicySession),
}
impl Auth<'_> {
    const fn len(&self) -> usize {
        match self {
            Self::Password(p) => 13 + p.0.len(),
            Self::Policy(_) => 13,
        }
    }
    const fn tail(&self) -> Tail {
        match self {
            Self::Password(_) => Tail::Password,
            Self::Policy(_) => Tail::Policy,
        }
    }
}
/// `TPMS_NV_PUBLIC` input for `NV_DefineSpace` (nameAlg SHA-256, empty index
/// authValue). Type comes from the TPM_NT field of `attributes`: ordinary
/// (1..=64 bytes) or counter (exactly 8); anything else is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NvDefinition<'a> {
    pub index: NvIndex,
    pub attributes: u32,
    pub auth_policy: Option<&'a [u8; DIGEST]>,
    pub size: u16,
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
    /// Policy session, unsalted (`tpmKey` = NULL) and unbound (`bind` = NULL),
    /// no parameter encryption, SHA-256. `nonce_caller` is 16..=32 caller bytes;
    /// their entropy is the caller's responsibility.
    StartPolicySession {
        nonce_caller: &'a [u8],
    },
    /// `PolicyPCR` with an empty `pcrDigest`: the TPM uses the current value of
    /// the one selected SHA-256 PCR.
    PolicyPcr {
        session: PolicySession,
        pcr: u8,
    },
    PolicyCommandCode {
        session: PolicySession,
        code: u32,
    },
    /// Owner-authorized `NV_DefineSpace` with an explicit public area.
    NvDefineSpace {
        owner_auth: Password<'a>,
        public: NvDefinition<'a>,
    },
    /// `NV_Write` with `authHandle` = the index (policy or AUTHWRITE password).
    NvWrite {
        index: NvIndex,
        auth: Auth<'a>,
        data: &'a [u8],
        offset: u16,
    },
    /// `NV_Read` of 1..=64 bytes with `authHandle` = the index (AUTHREAD).
    NvRead {
        index: NvIndex,
        auth: Password<'a>,
        size: u16,
        offset: u16,
    },
    PcrRead {
        pcr: u8,
    },
    /// `PCR_Extend` with exactly one SHA-256 digest.
    PcrExtend {
        pcr: u8,
        auth: Password<'a>,
        digest: &'a [u8; DIGEST],
    },
    /// `NV_ReadPublic` decoded into type, attributes, size and `authPolicy`.
    NvReadPublicArea {
        index: NvIndex,
    },
}
/// Response authorization trailer the decoder demands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tail {
    None,
    /// TPM_RS_PW: empty nonce, continueSession=1, empty HMAC.
    Password,
    /// Policy session sent with continueSession=0: fresh 32-byte nonceTPM,
    /// attributes 0, empty HMAC *[unverified against swtpm, S4/S5]*.
    Policy,
}
#[derive(Clone, Copy)]
pub(crate) enum Expectation {
    Empty,
    Properties { first: u32, count: u8 },
    Public { index: NvIndex },
    SessionEmpty,
    Counter,
    Session,
    Data { size: u16 },
    Pcr { pcr: u8 },
    PublicArea { index: NvIndex },
}
pub struct Command<'a> {
    bytes: &'a [u8],
    pub(crate) expect: Expectation,
    pub(crate) sessions: Tail,
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
    /// Decode a complete response to this command (for transports other than
    /// [`crate::transport`], e.g. a firmware TCG2 SubmitCommand). Same checks
    /// as the CRB path; nothing is returned on error.
    pub fn decode(&self, response: &[u8]) -> Result<Response, Error> {
        validate(self.expect, self.sessions, response)
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
    fn authorize(&mut self, a: Auth<'_>) {
        match a {
            Auth::Password(p) => self.auth(p),
            Auth::Policy(s) => {
                self.u32(9);
                self.u32(s.0);
                self.u16(0);
                self.put(&[0]);
                self.u16(0);
            }
        }
    }
    fn tpm2b(&mut self, data: &[u8]) {
        self.u16(data.len() as u16);
        self.put(data);
    }
    fn selection(&mut self, pcr: u8) {
        self.put(&selection(pcr));
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
        Request::Startup(_) => (12, 0x144, Tail::None, Expectation::Empty, false),
        Request::SelfTest { .. } => (11, 0x143, Tail::None, Expectation::Empty, false),
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
                Tail::None,
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
            Tail::None,
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
            Tail::Password,
            Expectation::SessionEmpty,
            true,
        ),
        Request::NvReadCounter { auth, .. } => (
            35 + auth.0.len(),
            0x14e,
            Tail::Password,
            Expectation::Counter,
            false,
        ),
        Request::NvIncrement { auth, .. } => (
            31 + auth.0.len(),
            0x134,
            Tail::Password,
            Expectation::SessionEmpty,
            true,
        ),
        Request::StartPolicySession { nonce_caller } => {
            if !(16..=DIGEST).contains(&nonce_caller.len()) {
                return Err(Error::Input);
            }
            (
                27 + nonce_caller.len(),
                0x176,
                Tail::None,
                Expectation::Session,
                false,
            )
        }
        Request::PolicyPcr { pcr, .. } => {
            check_pcr(*pcr)?;
            (26, 0x17f, Tail::None, Expectation::Empty, false)
        }
        Request::PolicyCommandCode { .. } => (18, 0x16c, Tail::None, Expectation::Empty, false),
        Request::NvDefineSpace { owner_auth, public } => {
            let size_ok = match (public.attributes >> 4) & 0xf {
                NT_ORDINARY => (1..=MAX_NV_DATA as u16).contains(&public.size),
                NT_COUNTER => public.size == 8,
                _ => false,
            };
            if !size_ok {
                return Err(Error::Input);
            }
            (
                45 + owner_auth.0.len() + public.auth_policy.map_or(0, |_| DIGEST),
                0x12a,
                Tail::Password,
                Expectation::SessionEmpty,
                true,
            )
        }
        Request::NvWrite { auth, data, .. } => {
            if data.is_empty() || data.len() > MAX_NV_DATA {
                return Err(Error::Input);
            }
            (
                22 + auth.len() + data.len(),
                0x137,
                auth.tail(),
                Expectation::SessionEmpty,
                true,
            )
        }
        Request::NvRead { auth, size, .. } => {
            if !(1..=MAX_NV_DATA as u16).contains(size) {
                return Err(Error::Input);
            }
            (
                35 + auth.0.len(),
                0x14e,
                Tail::Password,
                Expectation::Data { size: *size },
                false,
            )
        }
        Request::PcrRead { pcr } => {
            check_pcr(*pcr)?;
            (20, 0x17e, Tail::None, Expectation::Pcr { pcr: *pcr }, false)
        }
        Request::PcrExtend { pcr, auth, .. } => {
            check_pcr(*pcr)?;
            (
                65 + auth.0.len(),
                0x182,
                Tail::Password,
                Expectation::SessionEmpty,
                false,
            )
        }
        Request::NvReadPublicArea { index } => (
            14,
            0x169,
            Tail::None,
            Expectation::PublicArea { index: *index },
            false,
        ),
    };
    if len > MAX_COMMAND || out.len() < len {
        return Err(Error::Length);
    }
    let mut w = Writer {
        bytes: &mut out[..len],
        at: 0,
    };
    w.u16(if sessions == Tail::None {
        NO_SESSIONS
    } else {
        SESSIONS
    });
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
        Request::StartPolicySession { nonce_caller } => {
            w.u32(RH_NULL);
            w.u32(RH_NULL);
            w.tpm2b(nonce_caller);
            w.u16(0);
            w.put(&[SE_POLICY]);
            w.u16(ALG_NULL);
            w.u16(ALG_SHA256);
        }
        Request::PolicyPcr { session, pcr } => {
            w.u32(session.0);
            w.u16(0);
            w.selection(pcr);
        }
        Request::PolicyCommandCode { session, code } => {
            w.u32(session.0);
            w.u32(code);
        }
        Request::NvDefineSpace { owner_auth, public } => {
            w.u32(RH_OWNER);
            w.auth(owner_auth);
            w.u16(0);
            let policy = public.auth_policy.map_or(&[][..], |p| &p[..]);
            w.u16(14 + policy.len() as u16);
            w.u32(public.index.0);
            w.u16(ALG_SHA256);
            w.u32(public.attributes);
            w.tpm2b(policy);
            w.u16(public.size);
        }
        Request::NvWrite {
            index,
            auth,
            data,
            offset,
        } => {
            w.u32(index.0);
            w.u32(index.0);
            w.authorize(auth);
            w.tpm2b(data);
            w.u16(offset);
        }
        Request::NvRead {
            index,
            auth,
            size,
            offset,
        } => {
            w.u32(index.0);
            w.u32(index.0);
            w.auth(auth);
            w.u16(size);
            w.u16(offset);
        }
        Request::PcrRead { pcr } => w.selection(pcr),
        Request::PcrExtend { pcr, auth, digest } => {
            w.u32(pcr.into());
            w.auth(auth);
            w.u32(1);
            w.u16(ALG_SHA256);
            w.put(digest);
        }
        Request::NvReadPublicArea { index } => w.u32(index.0),
    }
    debug_assert_eq!(w.at, len);
    Ok(Command {
        bytes: &out[..len],
        expect,
        sessions,
        mutating,
    })
}
fn check_pcr(pcr: u8) -> Result<(), Error> {
    if pcr <= MAX_PCR {
        Ok(())
    } else {
        Err(Error::Input)
    }
}
/// `TPML_PCR_SELECTION` {count 1, SHA-256, sizeofSelect 3, one bit}.
fn selection(pcr: u8) -> [u8; 10] {
    let mut s = [0, 0, 0, 1, 0x00, 0x0b, 3, 0, 0, 0];
    s[7 + usize::from(pcr / 8)] = 1 << (pcr % 8);
    s
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
    Session {
        handle: PolicySession,
        nonce_tpm: [u8; DIGEST],
    },
    Data {
        len: u8,
        bytes: [u8; MAX_NV_DATA],
    },
    Pcr {
        update_counter: u32,
        digest: [u8; DIGEST],
    },
    NvPublicArea(NvPublicArea),
}
/// Decoded `NV_ReadPublic`. nameAlg is SHA-256 by construction; the name is
/// returned unverified (this codec has no hash), so a caller that relies on
/// it must recompute `nameAlg ‖ H(nvPublic)` itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NvPublicArea {
    pub index: NvIndex,
    pub attributes: u32,
    pub size: u16,
    auth_policy: [u8; DIGEST],
    auth_policy_len: u8,
    pub name: [u8; 34],
}
impl NvPublicArea {
    /// TPM_NT field of the attributes (0 ordinary, 1 counter, ...).
    pub const fn nv_type(&self) -> u32 {
        (self.attributes >> 4) & 0xf
    }
    /// Empty or exactly 32 bytes.
    pub fn auth_policy(&self) -> &[u8] {
        &self.auth_policy[..usize::from(self.auth_policy_len)]
    }
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
    sessions: Tail,
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
    if tag
        != if sessions == Tail::None {
            NO_SESSIONS
        } else {
            SESSIONS
        }
    {
        return Err(Error::Protocol);
    }
    let trailer = match sessions {
        Tail::None => 0,
        Tail::Password => 5,
        Tail::Policy => 5 + DIGEST,
    };
    let body = if trailer == 0 {
        &bytes[10..]
    } else {
        if bytes.len() < 14 + trailer {
            return Err(Error::Length);
        }
        let parameters = u32at(bytes, 10) as usize;
        let end = 14usize.checked_add(parameters).ok_or(Error::Length)?;
        if end.checked_add(trailer) != Some(bytes.len()) {
            return Err(Error::Protocol);
        }
        let t = &bytes[end..];
        let ok = match sessions {
            // TPM_RS_PW responses have empty nonce/HMAC and continueSession=1.
            Tail::Password => t == [0, 0, 1, 0, 0],
            Tail::Policy => u16at(t, 0) == DIGEST as u16 && t[2 + DIGEST..] == [0, 0, 0],
            Tail::None => false,
        };
        if !ok {
            return Err(Error::Protocol);
        }
        &bytes[14..end]
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
        Expectation::Session => {
            if body.len() != 6 + DIGEST || u16at(body, 4) != DIGEST as u16 {
                return Err(Error::Length);
            }
            let handle = PolicySession::new(u32at(body, 0)).map_err(|_| Error::Protocol)?;
            let mut nonce_tpm = [0; DIGEST];
            nonce_tpm.copy_from_slice(&body[6..]);
            Ok(Response::Session { handle, nonce_tpm })
        }
        Expectation::Data { size } => {
            if body.len() != 2 + usize::from(size) || u16at(body, 0) != size {
                return Err(Error::Length);
            }
            let mut bytes = [0; MAX_NV_DATA];
            bytes[..usize::from(size)].copy_from_slice(&body[2..]);
            Ok(Response::Data {
                len: size as u8,
                bytes,
            })
        }
        Expectation::Pcr { pcr } => {
            if body.len() != 20 + DIGEST {
                return Err(Error::Length);
            }
            // An unallocated bank comes back as an empty selection: refuse it.
            if body[4..14] != selection(pcr)
                || u32at(body, 14) != 1
                || u16at(body, 18) != DIGEST as u16
            {
                return Err(Error::Protocol);
            }
            let mut digest = [0; DIGEST];
            digest.copy_from_slice(&body[20..]);
            Ok(Response::Pcr {
                update_counter: u32at(body, 0),
                digest,
            })
        }
        Expectation::PublicArea { index } => {
            if body.len() < 16 {
                return Err(Error::Length);
            }
            let public = usize::from(u16at(body, 0));
            let policy = usize::from(u16at(body, 12));
            if (policy != 0 && policy != DIGEST)
                || public != 14 + policy
                || body.len() != 2 + public + 2 + 34
            {
                return Err(Error::Length);
            }
            let name = &body[2 + public..];
            if u32at(body, 2) != index.0
                || u16at(body, 6) != ALG_SHA256
                || u16at(name, 0) != 34
                || u16at(name, 2) != ALG_SHA256
            {
                return Err(Error::Protocol);
            }
            let mut auth_policy = [0; DIGEST];
            auth_policy[..policy].copy_from_slice(&body[14..14 + policy]);
            let mut out = [0; 34];
            out.copy_from_slice(&name[2..]);
            Ok(Response::NvPublicArea(NvPublicArea {
                index,
                attributes: u32at(body, 8),
                size: u16at(body, 14 + policy),
                auth_policy,
                auth_policy_len: policy as u8,
                name: out,
            }))
        }
    }
}
