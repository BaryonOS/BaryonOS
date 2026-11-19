//! Canonical queue bytes, not authority, authenticated storage or freshness.
use super::{Checkpoint, Edit, State, Version, MAX_CIPHERTEXT, OBJECTS, PENDING};
use nieuw_os_nas_core::Error;

const HEADER: usize = 16;
const SLOT: usize = 64 + 2 * MAX_CIPHERTEXT;
pub const CHECKPOINT_BYTES: usize = HEADER + PENDING * SLOT;

impl Checkpoint {
    /// Encode retained bytes; all lifecycle states normalize to NeedsReconcile.
    /// The caller must securely persist the complete buffer before reporting save.
    pub fn encode(&self, out: &mut [u8; CHECKPOINT_BYTES]) {
        out.fill(0);
        out[..8].copy_from_slice(b"NOSNCP01");
        out[8..10].copy_from_slice(&1u16.to_le_bytes());
        for (i, edit) in self.edits.iter().enumerate() {
            let Some(edit) = edit else { continue };
            let b = &mut out[HEADER + i * SLOT..HEADER + (i + 1) * SLOT];
            b[0] = 1; // NeedsReconcile; no persisted publication acknowledgement.
            b[2..4].copy_from_slice(&(edit.object as u16).to_le_bytes());
            b[8..40].copy_from_slice(&edit.namespace);
            b[40..48].copy_from_slice(&edit.base.to_le_bytes());
            b[56..58].copy_from_slice(&(edit.local.length as u16).to_le_bytes());
            b[64..64 + edit.local.length].copy_from_slice(edit.local_ciphertext());
            if let Some(remote) = &edit.remote {
                b[1] = 1;
                b[48..56].copy_from_slice(&remote.revision.to_le_bytes());
                b[58..60].copy_from_slice(&(remote.length as u16).to_le_bytes());
                b[64 + MAX_CIPHERTEXT..64 + MAX_CIPHERTEXT + remote.length]
                    .copy_from_slice(&remote.bytes[..remote.length]);
            }
        }
    }

    /// Strict all-or-error decode; nothing in these bytes grants a live ticket.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != CHECKPOINT_BYTES
            || &bytes[..8] != b"NOSNCP01"
            || bytes[8..10] != 1u16.to_le_bytes()
            || bytes[10..HEADER] != [0; 6]
        {
            return Err(Error::Corrupt);
        }
        let mut edits: [Option<Edit>; PENDING] = core::array::from_fn(|_| None);
        for i in 0..PENDING {
            let b = &bytes[HEADER + i * SLOT..HEADER + (i + 1) * SLOT];
            if b.iter().all(|v| *v == 0) {
                continue;
            }
            let object = usize::from(u16::from_le_bytes(b[2..4].try_into().unwrap()));
            let namespace: [u8; 32] = b[8..40].try_into().unwrap();
            let base = u64::from_le_bytes(b[40..48].try_into().unwrap());
            let revision = base.checked_add(1).ok_or(Error::Corrupt)?;
            let local_len = usize::from(u16::from_le_bytes(b[56..58].try_into().unwrap()));
            let remote_len = usize::from(u16::from_le_bytes(b[58..60].try_into().unwrap()));
            let remote_revision = u64::from_le_bytes(b[48..56].try_into().unwrap());
            if b[0] != 1
                || b[1] > 1
                || b[4..8] != [0; 4]
                || b[60..64] != [0; 4]
                || object >= OBJECTS
                || namespace == [0; 32]
                || local_len == 0
                || local_len > MAX_CIPHERTEXT
                || remote_len > MAX_CIPHERTEXT
                || edits[..i]
                    .iter()
                    .flatten()
                    .any(|e| e.namespace == namespace && e.object == object)
            {
                return Err(Error::Corrupt);
            }
            let local = &b[64..64 + MAX_CIPHERTEXT];
            let remote = &b[64 + MAX_CIPHERTEXT..];
            if local[local_len..].iter().any(|v| *v != 0)
                || remote[remote_len..].iter().any(|v| *v != 0)
                || (b[1] == 0 && (remote_revision != 0 || remote_len != 0))
                || (b[1] == 1 && (remote_revision == 0) != (remote_len == 0))
            {
                return Err(Error::Corrupt);
            }
            edits[i] = Some(Edit {
                namespace,
                object,
                base,
                state: State::NeedsReconcile,
                local: Version {
                    revision,
                    length: local_len,
                    bytes: local.try_into().unwrap(),
                },
                remote: (b[1] == 1).then(|| Version {
                    revision: remote_revision,
                    length: remote_len,
                    bytes: remote.try_into().unwrap(),
                }),
            });
        }
        Ok(Checkpoint { edits })
    }
}
