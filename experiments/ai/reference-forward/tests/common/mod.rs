//! Host-side helpers shared by the tokenizer tests.

use baryon_ai_reference_forward::MAGIC;

/// Serialize a tokenizer table: header, byte ids, merges in rank order, then
/// the ranks sorted by their pair.
pub fn blob(byte_ids: &[u32; 256], merges: &[(u32, u32)]) -> Vec<u8> {
    let mut out = MAGIC.to_vec();
    out.extend((merges.len() as u32).to_le_bytes());
    for id in byte_ids {
        out.extend(id.to_le_bytes());
    }
    for (left, right) in merges {
        out.extend(left.to_le_bytes());
        out.extend(right.to_le_bytes());
    }
    let mut sorted: Vec<u32> = (0..merges.len() as u32).collect();
    sorted.sort_by_key(|&rank| merges[rank as usize]);
    for rank in sorted {
        out.extend(rank.to_le_bytes());
    }
    out
}
