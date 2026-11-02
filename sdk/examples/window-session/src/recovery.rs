//! Pure init policy transitions. A complete old readiness mask is never
//! permission to rebind before a successful, validated replacement install.
use crate::boot::READY;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Group {
    pub generation: u64,
    pub surface_epoch: u64,
    pub input_epoch: u64,
}
impl Group {
    fn valid(self) -> bool {
        self.generation != 0 && self.surface_epoch != 0 && self.input_epoch != 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Pause,
    Ready,
    RebindAck,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recovery {
    old: Group,
    installed: Option<Group>,
    stage: Stage,
}
impl Recovery {
    pub fn begin(old: Group) -> Result<Self, Invalid> {
        if !old.valid() {
            return Err(Invalid);
        }
        Ok(Self {
            old,
            installed: None,
            stage: Stage::Pause,
        })
    }
    pub fn awaiting_pause(&self) -> bool {
        self.stage == Stage::Pause
    }
    pub fn awaiting_ready(&self) -> bool {
        self.stage == Stage::Ready
    }
    pub fn awaiting_rebind_ack(&self) -> bool {
        self.stage == Stage::RebindAck
    }
    pub fn can_restart(&self, pause: u8) -> bool {
        self.awaiting_pause() && pause == 3
    }
    /// Init calls only after the real construction syscall returns and every
    /// process/capability identity check on the new private boot record passes.
    pub fn installed(&mut self, fresh: Group, pause: u8) -> Result<(), Invalid> {
        if !self.can_restart(pause)
            || !fresh.valid()
            || fresh.generation <= self.old.generation
            || fresh.surface_epoch == self.old.surface_epoch
            || fresh.input_epoch == self.old.input_epoch
        {
            return Err(Invalid);
        }
        self.installed = Some(fresh);
        self.stage = Stage::Ready;
        Ok(())
    }
    pub fn can_rebind(&self, current: Group, mask: u8, pause: u8) -> bool {
        self.awaiting_ready()
            && self.installed == Some(current)
            && current.generation > self.old.generation
            && mask == 31
            && pause == 0
    }
    pub fn rebind_sent(&mut self, current: Group, mask: u8, pause: u8) -> Result<(), Invalid> {
        if !self.can_rebind(current, mask, pause) {
            return Err(Invalid);
        }
        self.stage = Stage::RebindAck;
        Ok(())
    }
    pub fn complete(&self, rebound: u8) -> bool {
        self.awaiting_rebind_ack() && rebound == 3
    }
}

fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
/// `sender_index` comes from matching the kernel-supplied sender PID against
/// the current boot's five distinct process identities, never payload claims.
pub fn ready_bit(
    current: Group,
    sender_index: Option<usize>,
    tag: u64,
    bytes: &[u8],
) -> Option<u8> {
    if !current.valid()
        || tag != READY
        || bytes.len() != 32
        || &bytes[..4] != b"IPD1"
        || bytes[4..6] != 1u16.to_le_bytes()
        || word(bytes, 16) != current.generation
    {
        return None;
    }
    let role = u16::from_le_bytes([bytes[6], bytes[7]]);
    let (expected, epoch, bit) = match role {
        1 => (3, current.input_epoch, 8),
        2 => (4, current.input_epoch, 16),
        3 => (2, current.surface_epoch, 4),
        _ => return None,
    };
    if sender_index != Some(expected)
        || word(bytes, 8) != epoch
        || (role == 2 && word(bytes, 24) != 0)
        || (role != 2 && word(bytes, 24) == 0)
    {
        return None;
    }
    Some(bit)
}
