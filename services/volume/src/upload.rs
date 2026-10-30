//! Opt-in v7 envelope for service-issued, principal-bound upload tokens.
//! Ordinary operations keep the v6 payload/semantics. The four bytes at
//! offset 4 carry a token only for a successful BEGIN reply and subsequent
//! APPEND/COMMIT/ABORT. A token is correlation within granted domain authority,
//! never an object capability or a caller-chosen authorization identity.
use crate::{client, get32, put32, wire, Status};

pub const VERSION: u16 = 7;
pub const TAG: u64 = 0x4d36_454c_4b00_0007;

pub fn token(body: &[u8; wire::SIZE]) -> u32 {
    get32(body, 4)
}

pub fn encode(header: &wire::Header, data: &[u8], token: u32) -> [u8; wire::SIZE] {
    wrap(wire::encode(header, data), token)
}

pub fn wrap(mut body: [u8; wire::SIZE], token: u32) -> [u8; wire::SIZE] {
    body[..2].copy_from_slice(&VERSION.to_le_bytes());
    put32(&mut body, 4, token);
    body
}

fn legacy(body: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
    if u16::from_le_bytes([body[0], body[1]]) != VERSION {
        return Err(Status::Invalid);
    }
    let mut canonical = *body;
    canonical[..2].copy_from_slice(&wire::VERSION.to_le_bytes());
    canonical[4..8].fill(0);
    Ok(canonical)
}

pub fn validate_request(
    body: &[u8; wire::SIZE],
    epoch: u64,
    domain: usize,
) -> Result<(wire::Header, u32, [u8; wire::SIZE]), Status> {
    let canonical = legacy(body)?;
    let h = wire::validate_request(&canonical, epoch, domain)?;
    let token = token(body);
    let required = matches!(h.operation, wire::APPEND | wire::COMMIT | wire::ABORT);
    if (token != 0) != required {
        return Err(Status::Invalid);
    }
    Ok((h, token, canonical))
}

pub fn validate_reply(
    request: &wire::Header,
    request_token: u32,
    body: &[u8; wire::SIZE],
    epoch: u64,
) -> Result<wire::Header, Status> {
    let canonical = legacy(body).map_err(|_| Status::Protocol)?;
    let h = client::validate_reply(request, &canonical, epoch)?;
    let response_token = token(body);
    if request.operation == wire::BEGIN && h.status == Status::Ok as i32 {
        if response_token == 0 {
            return Err(Status::Protocol);
        }
    } else if response_token != request_token {
        return Err(Status::Protocol);
    }
    Ok(h)
}
