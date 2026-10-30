//! Local IPC acceptance probe, NOT an interoperable NAS network protocol.
//! Sender and ingress role come from the trusted kernel adapter, never bytes.
#![no_std]
#![forbid(unsafe_code)]

use nieuw_os_nas_core::{
    Error, Invitation, Principal, Service, Session, Storage, Upload, Usage, TOKEN_BYTES,
};

pub const SIZE: usize = 256;
pub const DATA: usize = 104;
pub const TAG: u64 = 0x4e41_5350_0000_0001;
pub const REPLY: u16 = 0x8000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum Op {
    Invite = 1,
    Accept = 2,
    Open = 3,
    Stage = 4,
    Commit = 5,
    Read = 6,
    Usage = 7,
    SetQuota = 8,
    Revoke = 9,
    Evict = 10,
    Shutdown = 11,
    UsageAt = 12,
    Close = 13,
}

impl Op {
    fn decode(raw: u16) -> Result<Self, Error> {
        match raw {
            1 => Ok(Self::Invite),
            2 => Ok(Self::Accept),
            3 => Ok(Self::Open),
            4 => Ok(Self::Stage),
            5 => Ok(Self::Commit),
            6 => Ok(Self::Read),
            7 => Ok(Self::Usage),
            8 => Ok(Self::SetQuota),
            9 => Ok(Self::Revoke),
            10 => Ok(Self::Evict),
            11 => Ok(Self::Shutdown),
            12 => Ok(Self::UsageAt),
            13 => Ok(Self::Close),
            _ => Err(Error::Invalid),
        }
    }
}

/// No Debug: invitation nonces, session/upload handles and ciphertext stay
/// out of ordinary logs. The explicit length bounds all payload access.
pub struct Frame {
    pub op: Op,
    pub request: u64,
    pub status: Option<Error>,
    pub namespace: [u8; 32],
    pub session: [u8; TOKEN_BYTES],
    pub upload: [u8; TOKEN_BYTES],
    pub revision: u64,
    pub value: u64,
    pub length: usize,
    pub data: [u8; DATA],
}

impl Frame {
    pub const fn new(op: Op, request: u64) -> Self {
        Self {
            op,
            request,
            status: None,
            namespace: [0; 32],
            session: [0; TOKEN_BYTES],
            upload: [0; TOKEN_BYTES],
            revision: 0,
            value: 0,
            length: 0,
            data: [0; DATA],
        }
    }

    pub fn encode(&self, reply: bool) -> Result<[u8; SIZE], Error> {
        if self.request == 0
            || self.length > DATA
            || self.data[self.length..].iter().any(|b| *b != 0)
            || (!reply && self.status.is_some())
        {
            return Err(Error::Invalid);
        }
        if reply {
            self.validate_reply()?;
        } else {
            self.validate_request()?;
        }
        let mut bytes = [0; SIZE];
        bytes[..4].copy_from_slice(b"NASP");
        bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
        bytes[6..8]
            .copy_from_slice(&(self.op as u16 | if reply { REPLY } else { 0 }).to_le_bytes());
        bytes[8..16].copy_from_slice(&self.request.to_le_bytes());
        bytes[16..20].copy_from_slice(&status_code(self.status).to_le_bytes());
        bytes[20..22].copy_from_slice(&(self.length as u16).to_le_bytes());
        bytes[24..56].copy_from_slice(&self.namespace);
        bytes[56..96].copy_from_slice(&self.session);
        bytes[96..136].copy_from_slice(&self.upload);
        bytes[136..144].copy_from_slice(&self.revision.to_le_bytes());
        bytes[144..152].copy_from_slice(&self.value.to_le_bytes());
        bytes[152..].copy_from_slice(&self.data);
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8; SIZE], reply: bool) -> Result<Self, Error> {
        let raw = u16::from_le_bytes(bytes[6..8].try_into().unwrap());
        let request = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let length = usize::from(u16::from_le_bytes(bytes[20..22].try_into().unwrap()));
        if &bytes[..4] != b"NASP"
            || bytes[4..6] != 1u16.to_le_bytes()
            || (raw & REPLY != 0) != reply
            || request == 0
            || length > DATA
            || bytes[22..24] != [0; 2]
            || bytes[152 + length..].iter().any(|b| *b != 0)
        {
            return Err(Error::Invalid);
        }
        let mut out = Self::new(Op::decode(raw & !REPLY)?, request);
        out.status = decode_status(i32::from_le_bytes(bytes[16..20].try_into().unwrap()))?;
        if !reply && out.status.is_some() {
            return Err(Error::Invalid);
        }
        out.namespace.copy_from_slice(&bytes[24..56]);
        out.session.copy_from_slice(&bytes[56..96]);
        out.upload.copy_from_slice(&bytes[96..136]);
        out.revision = u64::from_le_bytes(bytes[136..144].try_into().unwrap());
        out.value = u64::from_le_bytes(bytes[144..152].try_into().unwrap());
        out.length = length;
        out.data.copy_from_slice(&bytes[152..]);
        if reply {
            out.validate_reply()?;
        } else {
            out.validate_request()?;
        }
        Ok(out)
    }

    fn validate_request(&self) -> Result<(), Error> {
        use Op::*;
        let namespace = matches!(self.op, Accept | Open | Usage | SetQuota | Revoke | Evict);
        let session = matches!(self.op, Stage | Commit | Read | Close);
        let upload = self.op == Commit;
        if (self.namespace != [0; 32]) != namespace
            || (self.session != [0; TOKEN_BYTES]) != session
            || (self.upload != [0; TOKEN_BYTES]) != upload
            || (!matches!(self.op, Stage | Read) && self.revision != 0)
            || (!matches!(self.op, Invite | Accept | Stage | Read | SetQuota | UsageAt)
                && self.value != 0)
            || match self.op {
                Accept => self.length != 32,
                Stage => self.length == 0,
                _ => self.length != 0,
            }
        {
            return Err(Error::Invalid);
        }
        if session {
            Session::from_bytes(&self.session)?;
        }
        if upload {
            Upload::from_bytes(&self.upload)?;
        }
        Ok(())
    }

    fn validate_reply(&self) -> Result<(), Error> {
        use Op::*;
        if self.status.is_some() {
            return if self.namespace == [0; 32]
                && self.session == [0; TOKEN_BYTES]
                && self.upload == [0; TOKEN_BYTES]
                && self.revision == 0
                && self.value == 0
                && self.length == 0
            {
                Ok(())
            } else {
                Err(Error::Invalid)
            };
        }
        if (self.namespace != [0; 32]) != matches!(self.op, Invite | Usage | UsageAt)
            || (self.session != [0; TOKEN_BYTES]) != (self.op == Open)
            || (self.upload != [0; TOKEN_BYTES]) != (self.op == Stage)
            || (self.revision != 0) != matches!(self.op, Commit | Usage | UsageAt)
            || (self.value != 0) != matches!(self.op, Invite | Usage | UsageAt)
            || match self.op {
                Invite => self.length != 32 || self.data[..32] == [0; 32],
                Read => self.length == 0,
                Usage | UsageAt => self.length != 16,
                _ => self.length != 0,
            }
        {
            return Err(Error::Invalid);
        }
        if self.op == Open {
            Session::from_bytes(&self.session)?;
        }
        if self.op == Stage {
            Upload::from_bytes(&self.upload)?;
        }
        if matches!(self.op, Usage | UsageAt) {
            let used = u32::from_le_bytes(self.data[4..8].try_into().unwrap());
            let reserved = u32::from_le_bytes(self.data[8..12].try_into().unwrap());
            if !(1..=4).contains(&self.data[0])
                || self.data[1..4] != [0; 3]
                || self.data[12..16] != [0; 4]
                || self.value > u64::from(nieuw_os_nas_core::MAX_QUOTA)
                || u64::from(used) + u64::from(reserved) > self.value
                || matches!(self.data[0], 1 | 4) && used != 0
                || self.data[0] != 2 && reserved != 0
            {
                return Err(Error::Invalid);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum Ingress {
    Administrator,
    Guest,
}

pub struct LocalBinding {
    pub admin_pid: u64,
    pub guest_pid: u64,
}

pub struct RequestContext<'a> {
    pub binding: &'a LocalBinding,
    pub ingress: Ingress,
    pub sender: u64,
    pub now: u64,
    pub recover: bool,
}

impl LocalBinding {
    pub fn new(admin_pid: u64, guest_pid: u64) -> Result<Self, Error> {
        if admin_pid == 0 || guest_pid == 0 || admin_pid == guest_pid {
            return Err(Error::Invalid);
        }
        Ok(Self {
            admin_pid,
            guest_pid,
        })
    }

    /// Process identity in this local probe, not a cryptographic peer identity.
    pub fn principal(pid: u64) -> Principal {
        let mut bytes = [0; 32];
        bytes[..8].copy_from_slice(b"NOSPROC1");
        bytes[8..16].copy_from_slice(&pid.to_le_bytes());
        Principal(bytes)
    }

    pub fn caller(&self, ingress: Ingress, actual_sender: u64) -> Result<Principal, Error> {
        let expected = match ingress {
            Ingress::Administrator => self.admin_pid,
            Ingress::Guest => self.guest_pid,
        };
        if actual_sender != expected {
            return Err(Error::Denied);
        }
        Ok(Self::principal(expected))
    }
}

/// A malformed envelope or wrong actual sender returns an explicit error
/// without touching authority. Ordinary operation errors are canonical replies.
pub fn dispatch<S: Storage>(
    service: &mut Service<'_, S>,
    context: &RequestContext<'_>,
    bytes: &[u8; SIZE],
    entropy: &mut impl nieuw_os_nas_core::Entropy,
) -> Result<([u8; SIZE], bool), Error> {
    let caller = context.binding.caller(context.ingress, context.sender)?;
    let request = Frame::decode(bytes, false)?;
    let mut reply = Frame::new(request.op, request.request);
    let operation = (|| {
        match request.op {
            Op::Invite => {
                let invitation = service.invite(
                    caller,
                    LocalBinding::principal(context.binding.guest_pid),
                    u32::try_from(request.value).map_err(|_| Error::Invalid)?,
                    context.now,
                    6000,
                    entropy,
                )?;
                reply.namespace = invitation.namespace;
                reply.value = invitation.expires;
                reply.length = 32;
                reply.data[..32].copy_from_slice(&invitation.nonce);
            }
            Op::Accept => {
                // This probe has no trusted cross-boot expiry source. It never
                // treats reset PIT ticks as proof that an old offer is valid.
                if context.recover {
                    return Err(Error::Expired);
                }
                service.accept(
                    caller,
                    &Invitation {
                        namespace: request.namespace,
                        recipient: caller,
                        nonce: request.data[..32].try_into().unwrap(),
                        expires: request.value,
                    },
                    context.now,
                )?;
            }
            Op::Open => {
                reply.session = service.open(caller, &request.namespace)?.to_bytes();
            }
            Op::Stage => {
                reply.upload = service
                    .stage(
                        caller,
                        Session::from_bytes(&request.session)?,
                        usize::try_from(request.value).map_err(|_| Error::Invalid)?,
                        request.revision,
                        &request.data[..request.length],
                    )?
                    .to_bytes();
            }
            Op::Commit => {
                reply.revision = service.commit(
                    caller,
                    Session::from_bytes(&request.session)?,
                    Upload::from_bytes(&request.upload)?,
                )?;
            }
            Op::Read => {
                reply.length = service.read(
                    caller,
                    Session::from_bytes(&request.session)?,
                    usize::try_from(request.value).map_err(|_| Error::Invalid)?,
                    request.revision,
                    &mut reply.data,
                )?;
            }
            Op::Usage => put_usage(&mut reply, service.usage(caller, &request.namespace)?),
            Op::UsageAt => put_usage(
                &mut reply,
                service.usage_at(
                    caller,
                    usize::try_from(request.value).map_err(|_| Error::Invalid)?,
                )?,
            ),
            Op::SetQuota => service.set_quota(
                caller,
                &request.namespace,
                u32::try_from(request.value).map_err(|_| Error::Invalid)?,
            )?,
            Op::Revoke => service.revoke(caller, &request.namespace)?,
            Op::Evict => service.evict(caller, &request.namespace)?,
            Op::Close => service.close(caller, Session::from_bytes(&request.session)?)?,
            Op::Shutdown => {
                // Check the core's administrator authority even if there is
                // no namespace yet; NotFound does not authorize a guest.
                match service.usage_at(caller, 0) {
                    Ok(_) | Err(Error::NotFound) => (),
                    Err(error) => return Err(error),
                }
            }
        }
        Ok::<(), Error>(())
    })();
    if let Err(error) = operation {
        reply = Frame::new(request.op, request.request);
        reply.status = Some(error);
    }
    let shutdown = request.op == Op::Shutdown && reply.status.is_none();
    Ok((reply.encode(true)?, shutdown))
}

fn put_usage(reply: &mut Frame, usage: Usage) {
    reply.namespace = usage.namespace;
    reply.revision = usage.epoch;
    reply.value = u64::from(usage.quota);
    reply.length = 16;
    reply.data[0] = usage.status as u8;
    reply.data[4..8].copy_from_slice(&usage.used.to_le_bytes());
    reply.data[8..12].copy_from_slice(&usage.reserved.to_le_bytes());
}

fn status_code(error: Option<Error>) -> i32 {
    match error {
        None => 0,
        Some(Error::Denied) => -1,
        Some(Error::Invalid) => -2,
        Some(Error::Full) => -3,
        Some(Error::Quota) => -4,
        Some(Error::Conflict) => -5,
        Some(Error::Expired) => -6,
        Some(Error::NotFound) => -7,
        Some(Error::Stale) => -8,
        Some(Error::Entropy) => -9,
        Some(Error::Corrupt) => -10,
        Some(Error::Io) => -11,
        Some(Error::Unknown) => -12,
        Some(Error::ReopenRequired) => -13,
    }
}

fn decode_status(code: i32) -> Result<Option<Error>, Error> {
    match code {
        0 => Ok(None),
        -1 => Ok(Some(Error::Denied)),
        -2 => Ok(Some(Error::Invalid)),
        -3 => Ok(Some(Error::Full)),
        -4 => Ok(Some(Error::Quota)),
        -5 => Ok(Some(Error::Conflict)),
        -6 => Ok(Some(Error::Expired)),
        -7 => Ok(Some(Error::NotFound)),
        -8 => Ok(Some(Error::Stale)),
        -9 => Ok(Some(Error::Entropy)),
        -10 => Ok(Some(Error::Corrupt)),
        -11 => Ok(Some(Error::Io)),
        -12 => Ok(Some(Error::Unknown)),
        -13 => Ok(Some(Error::ReopenRequired)),
        _ => Err(Error::Invalid),
    }
}
