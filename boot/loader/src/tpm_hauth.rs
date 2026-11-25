//! TEST-only REL-05 M1-HAUTH BOUND provisioning (B03, ADR-B03-REL05-TPM-20261125,
//! owner decision option 1): QEMU/swtpm only. First provisioning is
//! unauthorised here because S3b (EK, signed `ENROLL`), genesis, ACCEPTED and
//! the UEFI copy are out of scope; with ownerAuth already set the loader only
//! reads and verifies, and halts on anything it cannot bind. PCR 13 is
//! extended (`CLOSED`, every active bank through firmware) on every path that
//! reached the TPM, before the loader continues or stops. Markers carry
//! results only, never secret bytes.
use super::tpm_probe::{
    active_banks, codec_fail, exchange, locate, Fail, Submit, Tcg2, TCG2_GUID,
};
use super::Firmware;
use crate::serial;
use core::{ffi::c_void, mem::size_of, ptr};
use nieuw_os_tpm_crb::codec::{
    encode, Auth, Hierarchy, NvDefinition, NvIndex, Password, PolicySession, Request, Response,
    ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, MAX_COMMAND, MAX_NV_DATA, MAX_RESPONSE,
};
use nieuw_os_tpm_crb::transport::{BASE, BUFFER_OFFSET, BUFFER_SIZE};
use r_efi::{efi, protocols::rng};
use update_core::trial::{hauth_binding, LoaderRecord};
use update_core::trial_tpm::{self as rel05, NvSpec};

/// `TPMA_PERMANENT` (Part 2, 8.6).
const OWNER_AUTH_SET: u32 = 1 << 0;
const LOCKOUT_AUTH_SET: u32 = 1 << 2;
/// Exact "not defined" answer to an owner undefine: `TPM_RC_HANDLE` on handle 2.
const RC_UNDEFINED: u64 = 0x28b;
/// `TPM_RC_POLICY_FAIL` on session 1 (probe U1).
const RC_POLICY_FAIL: u64 = 0x99d;
/// `TPM_RC_BAD_AUTH` on session 1: a hierarchy authValue failure; the loader
/// reads the DA lockout counter around it.
const RC_BAD_AUTH: u64 = 0x9a2;
/// `TPM_PT_PERMANENT` and `TPM_PT_LOCKOUT_COUNTER`: `PT_VAR + 0` and `PT_VAR + 14`
/// (Part 2 `TPM_PT`; libtpms 0.10 `src/tpm2/TpmTypes.h`).
const PT_PERMANENT: u32 = 0x200;
const PT_LOCKOUT_COUNTER: u32 = 0x20e;
/// `EV_IPL` (TCG PC Client Platform Firmware Profile, OS-loader measurements).
const EV_IPL: u32 = 0x0d;

/// Zeroed with volatile writes when dropped, on every exit path.
struct Wiped<const N: usize>([u8; N]);
impl<const N: usize> Drop for Wiped<N> {
    fn drop(&mut self) {
        for byte in &mut self.0 {
            // SAFETY: Valid exclusive byte of a live local; volatile keeps the wipe.
            unsafe { ptr::write_volatile(byte, 0) };
        }
    }
}

/// `EFI_TCG2_EVENT` with its 14-byte packed header and the event string.
#[repr(C, packed)]
struct Event {
    size: u32,
    header_size: u32,
    header_version: u16,
    pcr_index: u32,
    event_type: u32,
    data: [u8; 22],
}

struct Tpm {
    protocol: *mut Tcg2,
    submit: Submit,
}

pub fn run(fw: &Firmware) {
    serial::write(b"LOADER-HAUTH:BEGIN\n");
    let Some(tpm) = Tpm::locate(fw).unwrap_or_else(|f| fail(f)) else {
        fail((b"TCG2_ABSENT", 0));
    };
    let mut secret = Wiped([0u8; MAX_NV_DATA]);
    let bound = bind(fw, &tpm, &mut secret.0);
    // Close the loader window first, whatever happened above.
    let extended = tpm.extend_closed();
    let result = extended
        .and(bound)
        .and_then(|()| after(fw, &tpm, &secret.0));
    drop(secret);
    if let Err(f) = result {
        if extended.is_ok() {
            // Observation only: the original failure is reported either way.
            let line: &[u8] = match tpm.pcr13() {
                Ok(digest) if digest == rel05::P_CLOSED => b"LOADER-HAUTH:HALT:PCR13:CLOSED\n",
                Ok(_) => b"LOADER-HAUTH:HALT:PCR13:OTHER\n",
                Err(_) => b"LOADER-HAUTH:HALT:PCR13:UNREADABLE\n",
            };
            serial::write(line);
        }
        fail(f);
    }
    serial::write(b"LOADER-HAUTH:PASS\n");
}

fn fail((code, detail): Fail) -> ! {
    serial::write(b"LOADER-HAUTH:FAIL:");
    serial::write(code);
    serial::write(b":");
    serial::hex(detail);
    serial::write(b"\n");
    serial::exit(0x11)
}

fn bind(fw: &Firmware, tpm: &Tpm, secret: &mut [u8; MAX_NV_DATA]) -> Result<(), Fail> {
    let permanent = tpm.property(PT_PERMANENT)?;
    serial::write(b"LOADER-HAUTH:PERMANENT:");
    serial::hex(u64::from(permanent));
    serial::write(b"\n");
    if tpm.pcr13()? != rel05::PCR_ZERO {
        return Err((b"PCR13_NONZERO", 0));
    }
    let hauth = index(&rel05::NV_HAUTH)?;
    let lrec = index(&rel05::NV_LREC)?;
    let none = Password::new(&[]).map_err(codec_fail)?;
    if permanent & OWNER_AUTH_SET == 0 {
        serial::write(b"LOADER-HAUTH:MODE:PROVISION\n");
        // Leftovers of a cut and owner squats; only "not defined" is tolerated.
        for index in [hauth, lrec] {
            match tpm.command(Request::NvUndefineSpace {
                index,
                owner_auth: none,
            }) {
                Err(f) if f == (&b"TPM_RC"[..], RC_UNDEFINED) => {}
                other => {
                    other?;
                }
            }
        }
        random(fw, secret)?;
        tpm.command(define(&rel05::NV_HAUTH)?)?;
        tpm.check_public(&rel05::NV_HAUTH, &rel05::NAME_HAUTH)?;
        tpm.command(Request::NvWriteOwner {
            index: hauth,
            owner_auth: none,
            data: secret,
        })?;
        tpm.command(define(&rel05::NV_LREC)?)?;
        let record = LoaderRecord {
            accepted: None,
            binding: hauth_binding(secret),
        }
        .encode();
        let session = tpm.policy(fw, rel05::TPM_CC_NV_WRITE)?;
        tpm.authorized(
            session,
            Request::NvWrite {
                index: lrec,
                auth: Auth::Policy(session),
                data: &record,
                offset: 0,
            },
        )?;
        tpm.command(Request::HierarchyChangeAuth {
            hierarchy: Hierarchy::Owner,
            auth: none,
            new_auth: Password::new(&secret[..32]).map_err(codec_fail)?,
        })?;
    } else {
        serial::write(b"LOADER-HAUTH:MODE:EXISTING\n");
    }
    // Every boot: a squatted HAUTH is never read into an authorisation.
    tpm.check_public(&rel05::NV_HAUTH, &rel05::NAME_HAUTH_WRITTEN)?;
    let session = tpm.policy(fw, rel05::TPM_CC_NV_READ)?;
    tpm.read_secret(session, hauth, secret)?;
    tpm.check_public(&rel05::NV_LREC, &rel05::NAME_LREC_WRITTEN)?;
    let Response::Data { len: 64, bytes } = tpm.command(Request::NvRead {
        index: lrec,
        auth: none,
        size: 64,
        offset: 0,
    })?
    else {
        return Err((b"LREC_READ", 0));
    };
    match LoaderRecord::decode(&bytes) {
        Some(LoaderRecord {
            accepted: None,
            binding,
        }) if binding == hauth_binding(secret) => {}
        _ => return Err((b"UNBOUND", 0)),
    }
    serial::write(b"LOADER-HAUTH:BOUND\n");
    if permanent & LOCKOUT_AUTH_SET == 0 {
        tpm.command(Request::HierarchyChangeAuth {
            hierarchy: Hierarchy::Lockout,
            auth: none,
            new_auth: Password::new(&secret[32..]).map_err(codec_fail)?,
        })?;
        serial::write(b"LOADER-HAUTH:LOCKOUT:SET\n");
    } else {
        serial::write(b"LOADER-HAUTH:LOCKOUT:KEPT\n");
    }
    Ok(())
}

/// After the extend: the window is closed and nothing secret is left behind.
fn after(fw: &Firmware, tpm: &Tpm, secret: &[u8; MAX_NV_DATA]) -> Result<(), Fail> {
    if tpm.pcr13()? != rel05::P_CLOSED {
        return Err((b"PCR13_NOT_CLOSED", 0));
    }
    serial::write(b"LOADER-HAUTH:PCR13:CLOSED\n");
    // Public read-back of every bank (U2); the runner checks each value. A bank
    // the firmware reports inactive is not read. Before the 0x20E reads below,
    // so the residue check still sees that answer last.
    let active = active_banks(tpm.protocol)?;
    for (bit, alg) in [(1, ALG_SHA1), (2, ALG_SHA256), (4, ALG_SHA384), (8, ALG_SHA512)] {
        let bank = if active & bit == 0 {
            None
        } else {
            match tpm.command(Request::PcrReadBank { pcr: 13, alg })? {
                Response::PcrBank { len, digest, .. } => Some((len, digest)),
                _ => return Err((b"PCR13_BANK", u64::from(alg))),
            }
        };
        serial::write(b"LOADER-HAUTH:PCR13:BANK:");
        serial::hex(u64::from(alg));
        serial::write(b":");
        match bank {
            Some((len, digest)) => serial::bytes_hex(&digest[..usize::from(len)]),
            None => serial::write(b"INACTIVE"),
        }
        serial::write(b"\n");
    }
    let hauth = index(&rel05::NV_HAUTH)?;
    let session = tpm.policy(fw, rel05::TPM_CC_NV_READ)?;
    let mut scratch = Wiped([0u8; MAX_NV_DATA]);
    match tpm.read_secret(session, hauth, &mut scratch.0) {
        Err(f) if f == (&b"TPM_RC"[..], RC_POLICY_FAIL) => {}
        Err(f) => return Err(f),
        Ok(()) => return Err((b"READ_ALLOWED", 0)),
    }
    serial::write(b"LOADER-HAUTH:READ:REFUSED\n");
    let failures = tpm.property(PT_LOCKOUT_COUNTER)?;
    match tpm.command(Request::NvUndefineSpace {
        index: hauth,
        owner_auth: Password::new(&[]).map_err(codec_fail)?,
    }) {
        Err(f) if f == (&b"TPM_RC"[..], RC_BAD_AUTH) => {}
        Err(f) => return Err(f),
        Ok(_) => return Err((b"OWNER_ALLOWED", 0)),
    }
    serial::write(b"LOADER-HAUTH:OWNER:REFUSED\n");
    let counted = tpm.property(PT_LOCKOUT_COUNTER)?;
    serial::write(b"LOADER-HAUTH:DA:");
    serial::hex(u64::from(failures));
    serial::write(b":");
    serial::hex(u64::from(counted));
    serial::write(b"\n");
    if counted != failures {
        return Err((b"DA_COUNTED", u64::from(counted)));
    }
    // The scan must see the real buffer: it still holds the last response, the
    // one-property `GetCapability` answer for 0x20E (byte 10 is moreData).
    const LAST: [u8; 23] = [
        0x80, 0x01, 0, 0, 0, 0x1b, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 1, 0, 0, 0x02, 0x0e,
    ];
    if (0..LAST.len()).any(|at| at != 10 && crb_byte(at) != LAST[at]) {
        return Err((b"RESIDUE_UNREADABLE", 0));
    }
    if residue(secret) {
        return Err((b"RESIDUE", 0));
    }
    serial::write(b"LOADER-HAUTH:RESIDUE:NONE\n");
    Ok(())
}

/// TEST QEMU CRB only: does either 32-byte half of the secret survive in the
/// shared command/response buffer after the loader's last TPM command?
fn residue(secret: &[u8; MAX_NV_DATA]) -> bool {
    secret.chunks(32).any(|half| {
        (0..=BUFFER_SIZE - half.len())
            .any(|at| (0..half.len()).all(|i| crb_byte(at + i) == half[i]))
    })
}

fn crb_byte(at: usize) -> u8 {
    assert!(at < BUFFER_SIZE);
    let buffer = (BASE as usize + BUFFER_OFFSET) as *const u8;
    // SAFETY: Read-only access inside the fixed QEMU CRB data buffer (bounded
    // above), identity mapped by firmware before EBS; no device side effect.
    unsafe { ptr::read_volatile(buffer.add(at)) }
}

fn index(spec: &NvSpec) -> Result<NvIndex, Fail> {
    NvIndex::new(spec.handle).map_err(codec_fail)
}

fn define(spec: &'static NvSpec) -> Result<Request<'static>, Fail> {
    Ok(Request::NvDefineSpace {
        owner_auth: Password::new(&[]).map_err(codec_fail)?,
        public: NvDefinition {
            index: index(spec)?,
            attributes: spec.attributes,
            auth_policy: spec.auth_policy.as_ref(),
            size: spec.size,
        },
    })
}

/// Firmware `EFI_RNG_PROTOCOL` (U25); all-zero output is refused.
fn random(fw: &Firmware, out: &mut [u8]) -> Result<(), Fail> {
    let protocol = locate::<rng::Protocol>(fw, rng::PROTOCOL_GUID)?.ok_or((b"RNG_ABSENT" as &[u8], 0))?;
    // SAFETY: Firmware RNG call before EBS; null selects the default algorithm
    // and the length bounds the caller's live buffer.
    let status =
        unsafe { ((*protocol).get_rng)(protocol, ptr::null_mut(), out.len(), out.as_mut_ptr()) };
    if status != efi::Status::SUCCESS {
        return Err((b"RNG", status.as_usize() as u64));
    }
    if out.iter().all(|byte| *byte == 0) {
        return Err((b"RNG_ZERO", 0));
    }
    Ok(())
}

impl Tpm {
    fn locate(fw: &Firmware) -> Result<Option<Self>, Fail> {
        let Some(protocol) = locate::<Tcg2>(fw, TCG2_GUID)? else {
            return Ok(None);
        };
        // SAFETY: Aligned, firmware-owned TCG2 instance; one pointer is copied.
        let submit = unsafe { (*protocol).submit_command }.ok_or((b"TCG2_TABLE" as &[u8], 0))?;
        Ok(Some(Self { protocol, submit }))
    }

    /// One command; both buffers are wiped because some frames carry secrets.
    fn command(&self, request: Request) -> Result<Response, Fail> {
        let mut buffer = Wiped([0u8; MAX_COMMAND]);
        let command = encode(request, &mut buffer.0).map_err(|_| (b"ENCODE" as &[u8], 0))?;
        let mut response = Wiped([0u8; MAX_RESPONSE]);
        let size = exchange(self.protocol, self.submit, command.bytes(), &mut response.0)?;
        command.decode(&response.0[..size]).map_err(codec_fail)
    }

    /// A policy-authorised command; a failure leaves the session loaded.
    fn authorized(&self, session: PolicySession, request: Request) -> Result<Response, Fail> {
        self.command(request).inspect_err(|_| self.flush(session))
    }

    fn flush(&self, session: PolicySession) {
        let _ = self.command(Request::FlushContext { session });
    }

    /// `PolicyPCR(13)` with the current value plus `PolicyCommandCode(code)`.
    fn policy(&self, fw: &Firmware, code: u32) -> Result<PolicySession, Fail> {
        let mut nonce = [0u8; 16];
        random(fw, &mut nonce)?;
        let Response::Session { handle, .. } = self.command(Request::StartPolicySession {
            nonce_caller: &nonce,
        })?
        else {
            return Err((b"SESSION", 0));
        };
        self.authorized(handle, Request::PolicyPcr { session: handle, pcr: 13 })?;
        self.authorized(handle, Request::PolicyCommandCode { session: handle, code })?;
        Ok(handle)
    }

    fn read_secret(
        &self,
        session: PolicySession,
        index: NvIndex,
        out: &mut [u8; MAX_NV_DATA],
    ) -> Result<(), Fail> {
        let mut buffer = Wiped([0u8; MAX_COMMAND]);
        let command = encode(Request::NvReadSecret { index, session }, &mut buffer.0)
            .map_err(|_| (b"ENCODE" as &[u8], 0))?;
        let mut response = Wiped([0u8; MAX_RESPONSE]);
        let size = exchange(self.protocol, self.submit, command.bytes(), &mut response.0)
            .inspect_err(|_| self.flush(session))?;
        command
            .decode_secret(&response.0[..size], out)
            .map_err(codec_fail)
            .inspect_err(|_| self.flush(session))
    }

    /// Fail-closed public area: frozen Name and the `NvSpec` fields.
    fn check_public(&self, spec: &NvSpec, name: &[u8; 34]) -> Result<(), Fail> {
        match self.command(Request::NvReadPublicArea { index: index(spec)? })? {
            Response::NvPublicArea(p)
                if p.name == *name
                    && spec.matches(
                        rel05::TPM_ALG_SHA256,
                        p.attributes,
                        p.auth_policy(),
                        p.size,
                    ) =>
            {
                Ok(())
            }
            _ => Err((b"PUBLIC", u64::from(spec.handle))),
        }
    }

    fn property(&self, id: u32) -> Result<u32, Fail> {
        match self.command(Request::Properties { first: id, count: 1 })? {
            Response::Properties {
                count: 1, values, ..
            } if values[0].property == id => Ok(values[0].value),
            _ => Err((b"PROPERTY", u64::from(id))),
        }
    }

    fn pcr13(&self) -> Result<[u8; 32], Fail> {
        match self.command(Request::PcrRead { pcr: 13 })? {
            Response::Pcr { digest, .. } => Ok(digest),
            _ => Err((b"PCR13", 0)),
        }
    }

    /// `HashLogExtendEvent(13, CLOSED)`: firmware hashes the event string in
    /// every active bank and logs it (U2; `after` reads every active bank back).
    fn extend_closed(&self) -> Result<(), Fail> {
        // SAFETY: Aligned, firmware-owned TCG2 instance; one pointer is copied.
        let extend =
            unsafe { (*self.protocol).hash_log_extend_event }.ok_or((b"TCG2_TABLE" as &[u8], 1))?;
        let mut event = Event {
            size: size_of::<Event>() as u32,
            header_size: 14,
            header_version: 1,
            pcr_index: rel05::PCR_PHASE,
            event_type: EV_IPL,
            data: [0; 22],
        };
        event.data.copy_from_slice(rel05::EVENT_CLOSED);
        // SAFETY: Firmware TCG2 call before EBS. Both pointers reference live
        // locals of the stated lengths; firmware only reads them.
        let status = unsafe {
            extend(
                self.protocol,
                0,
                rel05::EVENT_CLOSED.as_ptr() as u64,
                rel05::EVENT_CLOSED.len() as u64,
                ptr::addr_of_mut!(event).cast::<c_void>(),
            )
        };
        if status != efi::Status::SUCCESS {
            return Err((b"EXTEND", status.as_usize() as u64));
        }
        Ok(())
    }
}
