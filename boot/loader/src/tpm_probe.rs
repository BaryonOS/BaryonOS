//! TEST-only REL-05 loader probe (B03 S6 step 1): firmware TCG2 `SubmitCommand`
//! and `EFI_RNG_PROTOCOL` before ExitBootServices. Read-only TPM commands, no
//! NV, PCR or hierarchy change; RNG output is checked, wiped and never printed.
use super::{aligned, Firmware};
use crate::serial;
use core::{ffi::c_void, ptr};
use nieuw_os_tpm_crb::codec::{encode, Request, Response, MAX_COMMAND, MAX_RESPONSE};
use r_efi::{efi, protocols::rng};

const TCG2_GUID: efi::Guid = efi::Guid::from_fields(
    0x607f_766c,
    0x7455,
    0x42be,
    0x93,
    0x0b,
    &[0xe4, 0xd7, 0x6d, 0xb2, 0x72, 0x0f],
);
const TPM_PT_PERMANENT: u32 = 0x200;

type Submit = unsafe extern "efiapi" fn(*mut Tcg2, u32, *mut u8, u32, *mut u8) -> efi::Status;
type Banks = unsafe extern "efiapi" fn(*mut Tcg2, *mut u32) -> efi::Status;

/// `EFI_TCG2_PROTOCOL` (TCG EFI Protocol Specification for TPM 2.0). r-efi
/// 7.2.0 has no binding (U19); only the two members called here are typed.
#[repr(C)]
struct Tcg2 {
    get_capability: *const c_void,
    get_event_log: *const c_void,
    hash_log_extend_event: *const c_void,
    submit_command: Option<Submit>,
    get_active_pcr_banks: Option<Banks>,
    set_active_pcr_banks: *const c_void,
    get_result_of_set_active_pcr_banks: *const c_void,
}

type Fail = (&'static [u8], u64);

pub fn probe(fw: &Firmware) {
    serial::init();
    serial::write(b"\n");
    if let Err((code, detail)) = tcg2(fw) {
        serial::write(b"LOADER-TPM:FAIL:");
        serial::write(code);
        serial::write(b":");
        serial::hex(detail);
        serial::write(b"\n");
    }
    serial::write(b"LOADER-TPM:RNG:");
    serial::write(random(fw));
    serial::write(b"\n");
}

/// `Ok(None)` only for NOT_FOUND; any other failure or a bad pointer is an error.
fn locate<T>(fw: &Firmware, guid: efi::Guid) -> Result<Option<*mut T>, Fail> {
    let mut guid = guid;
    let mut raw: *mut c_void = ptr::null_mut();
    // SAFETY: Live BootServices before EBS; valid GUID and output slot. Firmware
    // owns the protocol and is trusted for its function pointers until EBS.
    let status = unsafe { (fw.services().locate_protocol)(&mut guid, ptr::null_mut(), &mut raw) };
    if status == efi::Status::NOT_FOUND {
        return Ok(None);
    }
    let protocol = raw.cast::<T>();
    if status != efi::Status::SUCCESS || !aligned(protocol) {
        return Err((b"LOCATE", status.as_usize() as u64));
    }
    Ok(Some(protocol))
}

fn tcg2(fw: &Firmware) -> Result<(), Fail> {
    let Some(protocol) = locate::<Tcg2>(fw, TCG2_GUID)? else {
        serial::write(b"LOADER-TPM:TCG2:ABSENT\n");
        return Ok(());
    };
    serial::write(b"LOADER-TPM:TCG2:PRESENT\n");
    // SAFETY: Aligned, firmware-owned TCG2 instance; only two pointers are copied.
    let (submit, banks) = unsafe { ((*protocol).submit_command, (*protocol).get_active_pcr_banks) };
    let (submit, banks) = submit.zip(banks).ok_or((b"TCG2_TABLE" as &[u8], 0))?;
    let mut active = 0u32;
    // SAFETY: Firmware TCG2 call before EBS with a valid local output.
    let status = unsafe { banks(protocol, &mut active) };
    if status != efi::Status::SUCCESS {
        return Err((b"BANKS", status.as_usize() as u64));
    }
    serial::write(b"LOADER-TPM:BANKS:");
    serial::hex(u64::from(active));
    serial::write(b"\n");

    let request = Request::Properties {
        first: TPM_PT_PERMANENT,
        count: 1,
    };
    match command(protocol, submit, request)? {
        Response::Properties {
            count: 1, values, ..
        } if values[0].property == TPM_PT_PERMANENT => {
            serial::write(b"LOADER-TPM:PERMANENT:");
            serial::hex(u64::from(values[0].value));
            serial::write(b"\n");
        }
        _ => return Err((b"PERMANENT", 0)),
    }
    match command(protocol, submit, Request::PcrRead { pcr: 13 })? {
        Response::Pcr { digest, .. } => serial::write(if digest == [0; 32] {
            b"LOADER-TPM:PCR13:ZERO\n"
        } else {
            b"LOADER-TPM:PCR13:NONZERO\n"
        }),
        _ => return Err((b"PCR13", 0)),
    }
    serial::write(b"LOADER-TPM:PASS\n");
    Ok(())
}

/// One read-only command through firmware `SubmitCommand`, decoded by the codec.
fn command(protocol: *mut Tcg2, submit: Submit, request: Request) -> Result<Response, Fail> {
    let mut buffer = [0u8; MAX_COMMAND];
    let command = encode(request, &mut buffer).map_err(|_| (b"ENCODE" as &[u8], 0))?;
    let bytes = command.bytes();
    let mut response = [0u8; MAX_RESPONSE];
    // SAFETY: Firmware TCG2 call before EBS. The input block is only read
    // (`IN UINT8 *`); both lengths bound live local buffers.
    let status = unsafe {
        submit(
            protocol,
            bytes.len() as u32,
            bytes.as_ptr().cast_mut(),
            response.len() as u32,
            response.as_mut_ptr(),
        )
    };
    if status != efi::Status::SUCCESS {
        return Err((b"SUBMIT", status.as_usize() as u64));
    }
    // The header size selects the response; the codec checks it again.
    let size = u32::from_be_bytes([response[2], response[3], response[4], response[5]]) as usize;
    let body = response
        .get(..size)
        .ok_or((b"RESPONSE_SIZE" as &[u8], size as u64))?;
    command.decode(body).map_err(|error| match error {
        nieuw_os_tpm_crb::codec::Error::Tpm(rc) => (b"TPM_RC" as &[u8], u64::from(rc)),
        _ => (b"DECODE" as &[u8], 0),
    })
}

/// U25 observation: does firmware supply EFI_RNG_PROTOCOL in this profile?
fn random(fw: &Firmware) -> &'static [u8] {
    let protocol = match locate::<rng::Protocol>(fw, rng::PROTOCOL_GUID) {
        Ok(Some(protocol)) => protocol,
        Ok(None) => return b"ABSENT",
        Err(_) => return b"FAIL",
    };
    let mut bytes = [0u8; 32];
    // SAFETY: Firmware RNG call before EBS; null selects the default algorithm
    // and the length bounds the live local buffer.
    let status = unsafe {
        ((*protocol).get_rng)(protocol, ptr::null_mut(), bytes.len(), bytes.as_mut_ptr())
    };
    let filled = status == efi::Status::SUCCESS && bytes.iter().any(|byte| *byte != 0);
    for byte in &mut bytes {
        // SAFETY: Valid exclusive byte of a live local; volatile keeps the wipe.
        unsafe { ptr::write_volatile(byte, 0) };
    }
    if filled {
        b"PASS"
    } else {
        b"FAIL"
    }
}
