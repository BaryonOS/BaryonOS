#![no_std]
#![forbid(unsafe_code)]
//! Wire protocol of the K05 acceptance programs (supervisor, worker, client).
//! It is an app-level agreement carried in ordinary Message320 payloads; the
//! kernel knows nothing about it. Every field is a little-endian u64 so the
//! encoding is canonical and host-testable.

/// Message tag shared by all K05 traffic ("K05").
pub const TAG: u64 = 0x004b_3035;
pub const SIZE: usize = 32;

/// Operations. Each payload is exactly `SIZE` bytes: op, then three words.
pub const REGISTER: u64 = 1; // worker -> supervisor, cap0 = worker ingress (SEND|TRANSFER); a = instance
pub const CLIENT_HELLO: u64 = 2; // client -> supervisor, cap0 = client reply endpoint (SEND|TRANSFER)
pub const WORKER_AVAILABLE: u64 = 3; // supervisor -> client, cap0 = worker ingress (SEND); a = instance
pub const REPLY_TARGET: u64 = 4; // supervisor -> worker, cap0 = client reply endpoint (SEND)
pub const REQUEST: u64 = 5; // client -> worker; a = seq
pub const REPLY: u64 = 6; // worker -> client; a = seq, b = instance, c = served
pub const PROGRESS: u64 = 7; // client -> supervisor; a = replies, b = closed observations, c = instances
pub const DENIED_REPORT: u64 = 8; // child -> supervisor; a = matched, b = total
pub const DONE: u64 = 9; // client -> supervisor; a = replies, b = closed, c = instances

/// Modes passed as startup word 0 to children.
pub const WORKER_SERVE: u64 = 0;
pub const WORKER_CRASH_AFTER_8: u64 = 1;
pub const WORKER_IDLE: u64 = 2;
pub const WORKER_DENIED_CHECKS: u64 = 3;
pub const CLIENT_RUN_24: u64 = 0;
/// Same workload, with the registry supplied only in startup cap1.
pub const CLIENT_SECOND_CAP: u64 = 4;
pub const CLIENT_DENIED_CHECKS: u64 = 3;

/// Supervisor cases (startup word 0 / 112 of the supervisor itself).
pub const CASE_NORMAL: u64 = 0;
pub const CASE_CRASH: u64 = 1;
pub const CASE_DENIED: u64 = 2;
pub const CASE_PARENT_EXIT: u64 = 3;
pub const CASE_QUOTA: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub op: u64,
    pub a: u64,
    pub b: u64,
    pub c: u64,
}

impl Frame {
    pub const fn new(op: u64, a: u64, b: u64, c: u64) -> Self {
        Self { op, a, b, c }
    }
    pub fn encode(self) -> [u8; SIZE] {
        let mut bytes = [0; SIZE];
        for (i, word) in [self.op, self.a, self.b, self.c].into_iter().enumerate() {
            bytes[i * 8..i * 8 + 8].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
    /// Reject anything but an exactly sized payload with a known operation.
    pub fn decode(payload: &[u8]) -> Option<Self> {
        if payload.len() != SIZE {
            return None;
        }
        let word = |i: usize| {
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&payload[i * 8..i * 8 + 8]);
            u64::from_le_bytes(bytes)
        };
        let frame = Self::new(word(0), word(1), word(2), word(3));
        if !(REGISTER..=DONE).contains(&frame.op) {
            return None;
        }
        Some(frame)
    }
}

/// Expected kernel error codes for the denied checks, shared by the three
/// programs so the supervisor can total them. Each entry is (label, code).
pub const EBADHANDLE: i64 = -4;
pub const EACCESS: i64 = -5;
pub const ETYPE: i64 = -6;
pub const EQUOTA: i64 = -7;
pub const ECLOSED: i64 = -9;
pub const EINVAL: i64 = -1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip_and_reject_bad_sizes_or_ops() {
        let frame = Frame::new(REPLY, 7, 2, 9);
        assert_eq!(Frame::decode(&frame.encode()), Some(frame));
        assert_eq!(Frame::decode(&frame.encode()[..31]), None);
        assert_eq!(Frame::decode(&[0; 33]), None);
        let mut bad = Frame::new(0, 0, 0, 0).encode();
        assert_eq!(Frame::decode(&bad), None);
        bad[..8].copy_from_slice(&(DONE + 1).to_le_bytes());
        assert_eq!(Frame::decode(&bad), None);
        assert_eq!(TAG, 0x4b3035);
    }
}
