//! Small M5 read-only application protocol. This codec has no device access.
//! The service binds each request endpoint to a fixed sector; offsets here are
//! relative bytes inside that endpoint's 512-byte scope, never physical LBAs.

pub const SIZE: usize = 192;
pub const SLICE: usize = 128;
pub const VERSION: u16 = 1;
pub const READ: u16 = 1;
pub const WRITE: u16 = 2;
pub const TAG: u64 = 0x4d35_424c_4b00_0001;
pub const ANNOUNCE: u64 = 0x4d35_414e_4e00_0001;
pub const REBIND: u64 = 0x4d35_4249_4e00_0001;
pub const FNV_OFFSET: u64 = 14695981039346656037;
pub const FNV_PRIME: u64 = 1099511628211;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    Invalid = -1,
    NoSys = -2,
    Range = -3,
    Access = -5,
    Stale = -10,
    Io = -12,
    Timeout = -13,
    NoDevice = -14,
    DeviceProtocol = -15,
}
impl Status {
    pub fn from_code(code: i64) -> Self {
        match code {
            0 => Self::Ok,
            -1 => Self::Invalid,
            -2 => Self::NoSys,
            -3 => Self::Range,
            -5 => Self::Access,
            -10 => Self::Stale,
            -12 => Self::Io,
            -13 => Self::Timeout,
            -14 => Self::NoDevice,
            _ => Self::DeviceProtocol,
        }
    }
}

/// All fields are plain integers. Reading a header does not validate its policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub operation: u16,
    pub request_id: u64,
    pub epoch: u64,
    pub offset: u64,
    pub length: u32,
    pub status: i32,
}

pub fn get64(b: &[u8], at: usize) -> u64 {
    let mut value = 0u64;
    for i in 0..8 {
        value |= u64::from(b[at + i]) << (i * 8);
    }
    value
}
pub fn put64(b: &mut [u8], at: usize, value: u64) {
    for i in 0..8 {
        b[at + i] = (value >> (i * 8)) as u8;
    }
}
pub fn get32(b: &[u8], at: usize) -> u32 {
    let mut value = 0u32;
    for i in 0..4 {
        value |= u32::from(b[at + i]) << (i * 8);
    }
    value
}
pub fn put32(b: &mut [u8], at: usize, value: u32) {
    for i in 0..4 {
        b[at + i] = (value >> (i * 8)) as u8;
    }
}
fn get16(b: &[u8], at: usize) -> u16 {
    u16::from(b[at]) | (u16::from(b[at + 1]) << 8)
}
fn put16(b: &mut [u8], at: usize, value: u16) {
    b[at] = value as u8;
    b[at + 1] = (value >> 8) as u8;
}

pub fn header(bytes: &[u8; SIZE]) -> Header {
    Header {
        operation: get16(bytes, 2),
        request_id: get64(bytes, 8),
        epoch: get64(bytes, 16),
        offset: get64(bytes, 24),
        length: get32(bytes, 32),
        status: get32(bytes, 36) as i32,
    }
}
fn common_valid(bytes: &[u8; SIZE]) -> bool {
    get16(bytes, 0) == VERSION
        && get32(bytes, 4) == 0
        && get64(bytes, 40) == 0
        && bytes[176..].iter().all(|b| *b == 0)
}
pub fn request(
    request_id: u64,
    epoch: u64,
    offset: u64,
    length: u32,
    operation: u16,
) -> [u8; SIZE] {
    let mut bytes = [0; SIZE];
    put16(&mut bytes, 0, VERSION);
    put16(&mut bytes, 2, operation);
    put64(&mut bytes, 8, request_id);
    put64(&mut bytes, 16, epoch);
    put64(&mut bytes, 24, offset);
    put32(&mut bytes, 32, length);
    bytes
}
pub fn validate_request(bytes: &[u8; SIZE], epoch: u64) -> Result<Header, Status> {
    let h = header(bytes);
    if !common_valid(bytes) || h.status != 0 || bytes[48..176].iter().any(|b| *b != 0) {
        return Err(Status::Invalid);
    }
    if h.epoch != epoch {
        return Err(Status::Stale);
    }
    if h.operation != READ {
        return Err(Status::NoSys);
    }
    if h.length == 0
        || h.length > SLICE as u32
        || h.offset
            .checked_add(u64::from(h.length))
            .filter(|end| *end <= 512)
            .is_none()
    {
        return Err(Status::Range);
    }
    Ok(h)
}
pub fn reply(h: Header, epoch: u64, status: Status, data: &[u8]) -> Result<[u8; SIZE], Status> {
    if data.len() > SLICE || status != Status::Ok && !data.is_empty() {
        return Err(Status::Invalid);
    }
    let mut bytes = request(
        h.request_id,
        epoch,
        h.offset,
        data.len() as u32,
        h.operation,
    );
    put32(&mut bytes, 36, status as i32 as u32);
    bytes[48..48 + data.len()].copy_from_slice(data);
    Ok(bytes)
}
pub fn validate_reply(
    bytes: &[u8; SIZE],
    request: Header,
    service_epoch: u64,
) -> Result<Header, Status> {
    let h = header(bytes);
    if !common_valid(bytes)
        || h.epoch != service_epoch
        || h.request_id != request.request_id
        || h.operation != request.operation
        || h.offset != request.offset
        || h.length > SLICE as u32
    {
        return Err(Status::Invalid);
    }
    if h.status == 0 {
        if h.length != request.length {
            return Err(Status::Invalid);
        }
    } else if h.length != 0 {
        return Err(Status::Invalid);
    }
    if bytes[48 + h.length as usize..176].iter().any(|b| *b != 0) {
        return Err(Status::Invalid);
    }
    Ok(h)
}
/// Diagnostic test checksum only; never authentication or a cryptographic hash.
pub fn checksum_update(mut state: u64, data: &[u8]) -> u64 {
    for byte in data {
        state = (state ^ u64::from(*byte)).wrapping_mul(FNV_PRIME);
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    fn good() -> [u8; SIZE] {
        request(17, 1, 384, 128, READ)
    }
    #[test]
    fn exact_header_offsets_and_valid_boundary() {
        let b = good();
        let h = validate_request(&b, 1).unwrap();
        assert_eq!(b.len(), 192);
        assert_eq!(get64(&b, 8), 17);
        assert_eq!((h.offset, h.length), (384, 128));
        assert!(b[48..].iter().all(|b| *b == 0));
    }
    #[test]
    fn bad_version_flags_status_reserved_and_request_data() {
        for offset in [0, 4, 7, 36, 40, 47, 48, 175, 176, 191] {
            let mut b = good();
            b[offset] ^= 2;
            assert_eq!(
                validate_request(&b, 1),
                Err(Status::Invalid),
                "offset {offset}"
            );
        }
    }
    #[test]
    fn unsupported_write_and_unknown_opcode_have_no_read_request() {
        for op in [WRITE, 0, 3, u16::MAX] {
            assert_eq!(
                validate_request(&request(1, 1, 0, 128, op), 1),
                Err(Status::NoSys)
            );
        }
    }
    #[test]
    fn stale_epoch_is_rejected_without_returning_a_valid_request() {
        assert_eq!(validate_request(&good(), 2), Err(Status::Stale));
    }
    #[test]
    fn zero_oversized_overflow_and_cross_scope_ranges() {
        for (offset, length) in [
            (0, 0),
            (0, 129),
            (0, u32::MAX),
            (512, 1),
            (511, 2),
            (u64::MAX, 1),
        ] {
            assert_eq!(
                validate_request(&request(1, 1, offset, length, READ), 1),
                Err(Status::Range)
            );
        }
    }
    #[test]
    fn reply_uses_only_supplied_slice_and_zeroes_all_unused_bytes() {
        let req = header(&good());
        let bytes = reply(req, 1, Status::Ok, &[0xa5; 128]).unwrap();
        assert!(validate_reply(&bytes, req, 1).is_ok());
        assert_eq!(&bytes[48..176], &[0xa5; 128]);
        assert!(bytes[176..].iter().all(|b| *b == 0));
    }
    #[test]
    fn error_reply_is_empty_and_cannot_smuggle_old_data() {
        let req = header(&good());
        let bytes = reply(req, 1, Status::Io, &[]).unwrap();
        assert_eq!(validate_reply(&bytes, req, 1).unwrap().status, -12);
        assert!(bytes[48..].iter().all(|b| *b == 0));
        assert_eq!(reply(req, 1, Status::Io, &[1]), Err(Status::Invalid));
    }
    #[test]
    fn reply_correlation_epoch_and_tail_are_checked() {
        let req = header(&good());
        for offset in [0, 2, 4, 8, 16, 24, 40, 176, 191] {
            let mut b = reply(req, 1, Status::Ok, &[3; 128]).unwrap();
            b[offset] ^= 1;
            assert_eq!(validate_reply(&b, req, 1), Err(Status::Invalid));
        }
        let mut b = reply(req, 1, Status::Io, &[]).unwrap();
        b[48] = 1;
        assert_eq!(validate_reply(&b, req, 1), Err(Status::Invalid));
    }
    #[test]
    fn four_slices_accumulate_same_diagnostic_checksum_as_one_sector() {
        let mut sector = [0; 512];
        for (i, b) in sector.iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }
        let mut sliced = FNV_OFFSET;
        for chunk in sector.chunks(128) {
            sliced = checksum_update(sliced, chunk);
        }
        assert_eq!(sliced, checksum_update(FNV_OFFSET, &sector));
        assert_ne!(sliced, FNV_OFFSET);
    }
}
