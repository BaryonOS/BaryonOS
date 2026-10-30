//! Deliberately narrow RAW fixture codec. Real service/client codecs remain in
//! their own crates; host tests compare these exact test bytes against them.
use nieuw_os_sdk_runtime::Received;
use zeroize::Zeroizing;
pub const SIZE: usize = 256;
pub const TAG: u64 = 0x4b45_5931;
pub const RANDOM: u8 = 10;
pub const MAX: usize = 224;

pub fn request(op: u8, id: u64, count: usize) -> [u8; SIZE] {
    let mut frame = [0; SIZE];
    frame[0] = op;
    frame[4..12].copy_from_slice(&id.to_le_bytes());
    if op == RANDOM {
        frame[2..4].copy_from_slice(&16u16.to_le_bytes());
        frame[16..20].copy_from_slice(b"RND1");
        frame[20..22].copy_from_slice(&1u16.to_le_bytes());
        frame[22..24].copy_from_slice(&(count as u16).to_le_bytes());
    }
    frame
}

pub struct Reply {
    pub status: i32,
    bytes: Zeroizing<[u8; MAX]>,
    pub length: usize,
}
impl Reply {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}
pub fn reply(message: &Received, pid: u64, request: &[u8; SIZE], count: usize) -> Option<Reply> {
    if message.sender() != pid || message.tag() != TAG || message.cap_count() != 0 {
        return None;
    }
    let op = request[0];
    let p = message.payload();
    if p.len() != SIZE || p[0] != (op | 0x80) || p[1] != request[1] || p[4..12] != request[4..12] {
        return None;
    }
    let n = usize::from(u16::from_le_bytes([p[2], p[3]]));
    if n > 240 || p[16 + n..].iter().any(|b| *b != 0) {
        return None;
    }
    let status = i32::from_le_bytes(p[12..16].try_into().ok()?);
    let mut result = Reply {
        status,
        bytes: Zeroizing::new([0; MAX]),
        length: 0,
    };
    if status != 0 {
        return (n == 0).then_some(result);
    }
    if op != RANDOM
        || !(1..=MAX).contains(&count)
        || n != 16 + count
        || &p[16..20] != b"RND1"
        || p[20..22] != 1u16.to_le_bytes()
        || p[22..24] != (count as u16).to_le_bytes()
        || p[24..32].iter().any(|b| *b != 0)
    {
        return None;
    }
    result.length = count;
    result.bytes[..count].copy_from_slice(&p[32..32 + count]);
    Some(result)
}
