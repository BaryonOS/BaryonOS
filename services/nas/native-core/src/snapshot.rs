use crate::{
    Error, Namespace, NamespaceStatus, Object, Principal, State, MAX_CIPHERTEXT, MAX_QUOTA,
    NAMESPACES, OBJECTS,
};

const HEADER: usize = 64;
const NS_HEADER: usize = 128;
const OBJECT_BYTES: usize = 16 + MAX_CIPHERTEXT;
const NS_BYTES: usize = NS_HEADER + OBJECTS * OBJECT_BYTES;
pub const SNAPSHOT_BYTES: usize = HEADER + NAMESPACES * NS_BYTES;

pub(super) fn encode(state: &State, out: &mut [u8; SNAPSHOT_BYTES]) {
    out.fill(0);
    out[..8].copy_from_slice(b"NOSNAS01");
    out[8..10].copy_from_slice(&1u16.to_le_bytes());
    out[16..24].copy_from_slice(&state.generation.to_le_bytes());
    out[24..56].copy_from_slice(&state.admin.0);
    for (i, ns) in state.namespaces.iter().enumerate() {
        let Some(ns) = ns else { continue };
        let bytes = &mut out[HEADER + i * NS_BYTES..HEADER + (i + 1) * NS_BYTES];
        bytes[..32].copy_from_slice(&ns.id);
        bytes[32] = ns.status as u8;
        bytes[40..48].copy_from_slice(&ns.epoch.to_le_bytes());
        bytes[48..52].copy_from_slice(&ns.quota.to_le_bytes());
        bytes[56..88].copy_from_slice(&ns.owner.0);
        bytes[88..120].copy_from_slice(&ns.invitation);
        bytes[120..128].copy_from_slice(&ns.expires.to_le_bytes());
        for (j, object) in ns.objects.iter().enumerate() {
            let at = NS_HEADER + j * OBJECT_BYTES;
            bytes[at..at + 8].copy_from_slice(&object.revision.to_le_bytes());
            bytes[at + 8..at + 10].copy_from_slice(&object.length.to_le_bytes());
            bytes[at + 16..at + 16 + usize::from(object.length)]
                .copy_from_slice(&object.bytes[..usize::from(object.length)]);
        }
    }
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

pub(super) fn decode(bytes: &[u8; SNAPSHOT_BYTES]) -> Result<State, Error> {
    if &bytes[..8] != b"NOSNAS01"
        || bytes[8..10] != 1u16.to_le_bytes()
        || bytes[10..16] != [0; 6]
        || bytes[56..HEADER] != [0; 8]
    {
        return Err(Error::Corrupt);
    }
    let mut state = State {
        generation: u64_at(bytes, 16),
        admin: Principal(bytes[24..56].try_into().unwrap()),
        namespaces: [None; NAMESPACES],
    };
    if state.generation == 0 || !state.admin.valid() {
        return Err(Error::Corrupt);
    }
    let mut gap = false;
    for i in 0..NAMESPACES {
        let b = &bytes[HEADER + i * NS_BYTES..HEADER + (i + 1) * NS_BYTES];
        if b.iter().all(|v| *v == 0) {
            gap = true;
            continue;
        }
        if gap || b[33..40] != [0; 7] || b[52..56] != [0; 4] {
            return Err(Error::Corrupt);
        }
        let status = match b[32] {
            1 => NamespaceStatus::Offered,
            2 => NamespaceStatus::Active,
            3 => NamespaceStatus::Revoked,
            4 => NamespaceStatus::Evicted,
            _ => return Err(Error::Corrupt),
        };
        let mut ns = Namespace {
            id: b[..32].try_into().unwrap(),
            status,
            epoch: u64_at(b, 40),
            quota: u32::from_le_bytes(b[48..52].try_into().unwrap()),
            owner: Principal(b[56..88].try_into().unwrap()),
            invitation: b[88..120].try_into().unwrap(),
            expires: u64_at(b, 120),
            objects: [Object::EMPTY; OBJECTS],
        };
        if ns.id == [0; 32]
            || ns.epoch == 0
            || ns.quota == 0
            || ns.quota > MAX_QUOTA
            || !ns.owner.valid()
            || ns.owner == state.admin
            || state.namespaces[..i]
                .iter()
                .flatten()
                .any(|old| old.id == ns.id)
        {
            return Err(Error::Corrupt);
        }
        if status == NamespaceStatus::Offered {
            if ns.invitation == [0; 32]
                || ns.invitation == ns.id
                || ns.expires == 0
                || ns.epoch != 1
            {
                return Err(Error::Corrupt);
            }
        } else if ns.invitation != [0; 32] || ns.expires != 0 {
            return Err(Error::Corrupt);
        }
        if matches!(status, NamespaceStatus::Revoked | NamespaceStatus::Evicted) && ns.epoch < 2 {
            return Err(Error::Corrupt);
        }
        for (j, object) in ns.objects.iter_mut().enumerate() {
            let at = NS_HEADER + j * OBJECT_BYTES;
            object.revision = u64_at(b, at);
            object.length = u16::from_le_bytes(b[at + 8..at + 10].try_into().unwrap());
            let length = usize::from(object.length);
            if length > MAX_CIPHERTEXT
                || b[at + 10..at + 16] != [0; 6]
                || (length == 0) != (object.revision == 0)
                || b[at + 16 + length..at + OBJECT_BYTES]
                    .iter()
                    .any(|v| *v != 0)
                || (matches!(status, NamespaceStatus::Offered | NamespaceStatus::Evicted)
                    && length != 0)
            {
                return Err(Error::Corrupt);
            }
            object.bytes[..length].copy_from_slice(&b[at + 16..at + 16 + length]);
        }
        if ns.used() > ns.quota {
            return Err(Error::Corrupt);
        }
        state.namespaces[i] = Some(ns);
    }
    Ok(state)
}
