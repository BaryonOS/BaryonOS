//! Pure, bounded host model of the transport/reset/epoch seam in
//! `TRANSPORT.md` §4–§6 (TASK-H03-TRANSPORT-MODEL-1).
//!
//! No I/O, no device, no DMA. "Status 0 read back" and "BME off read back"
//! are model inputs, i.e. assumptions about a conforming device; this model
//! does not prove real DMA quiescence or a device fence.

pub const SLOTS: usize = 4;
pub const TX_STATUS_LEN: u32 = 8;
pub const MAX_FENCE_BUDGET: u64 = 1_000_000;

const STATUS_OK: u32 = 0x8000;
const STATUS_IO_ERR: u32 = 0x8002;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Offline,
    Ready,
    Configured,
    Prepared,
    Running,
    Stopped,
    Fencing,
    Quarantined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Free,
    Published,
}

/// Lease token re-authorised on every submit and completion delivery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lease {
    pub epoch: u64,
    pub broker_pid: u32,
    pub cap_generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    BadState,
    Revoked,
    StaleLease,
    Again,
    UnknownSlot,
    RxForbidden,
    DoubleRelease,
    BadBudget,
    Frozen,
    Quarantined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion {
    Ok,
    IoError,
    /// Contract violation detected; a fence has been started.
    Fenced,
}

#[derive(Debug)]
pub struct Model {
    state: State,
    lease: Lease,
    slots: [Slot; SLOTS],
    revoked: bool,
    released: bool,
    deadline: u64,
    status_zero: bool,
    bme_off: bool,
    used_snapshot: Option<u16>,
}

impl Model {
    pub fn new(lease: Lease) -> Self {
        Self {
            state: State::Offline,
            lease,
            slots: [Slot::Free; SLOTS],
            revoked: false,
            released: false,
            deadline: 0,
            status_zero: false,
            bme_off: false,
            used_snapshot: None,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn lease(&self) -> Lease {
        self.lease
    }

    pub fn slot(&self, index: usize) -> Option<Slot> {
        self.slots.get(index).copied()
    }

    pub fn published(&self) -> usize {
        self.slots.iter().filter(|s| **s == Slot::Published).count()
    }

    fn live(&self) -> Result<(), Reject> {
        match self.state {
            State::Quarantined => Err(Reject::Quarantined),
            State::Fencing => Err(Reject::Frozen),
            _ => Ok(()),
        }
    }

    fn check_lease(&self, lease: Lease) -> Result<(), Reject> {
        if lease == self.lease {
            Ok(())
        } else {
            Err(Reject::StaleLease)
        }
    }

    fn step(&mut self, from: State, to: State) -> Result<(), Reject> {
        self.live()?;
        if self.revoked {
            return Err(Reject::Revoked);
        }
        if self.state != from {
            return Err(Reject::BadState);
        }
        self.state = to;
        Ok(())
    }

    /// Full §5.14.5 init after a fence. A new lease is mandatory: old epoch,
    /// PID or capability generation must not be carried over.
    pub fn init(&mut self, lease: Lease) -> Result<(), Reject> {
        self.live()?;
        if self.state != State::Offline {
            return Err(Reject::BadState);
        }
        if lease.epoch != self.lease.epoch || lease.cap_generation <= self.lease.cap_generation {
            return Err(Reject::StaleLease);
        }
        self.used_snapshot = None;
        self.lease = lease;
        self.revoked = false;
        self.released = false;
        self.state = State::Ready;
        Ok(())
    }

    pub fn configure(&mut self) -> Result<(), Reject> {
        self.step(State::Ready, State::Configured)?;
        self.released = false;
        Ok(())
    }

    pub fn prepare(&mut self) -> Result<(), Reject> {
        self.step(State::Configured, State::Prepared)
    }

    pub fn start(&mut self) -> Result<(), Reject> {
        self.live()?;
        if self.revoked {
            return Err(Reject::Revoked);
        }
        match self.state {
            State::Prepared | State::Stopped => {
                self.state = State::Running;
                Ok(())
            }
            _ => Err(Reject::BadState),
        }
    }

    pub fn stop(&mut self) -> Result<(), Reject> {
        self.step(State::Running, State::Stopped)
    }

    /// F19: one RELEASE per stream lifetime, refused before sending.
    pub fn release(&mut self) -> Result<(), Reject> {
        self.live()?;
        if self.released {
            return Err(Reject::DoubleRelease);
        }
        if self.state != State::Stopped {
            return Err(Reject::BadState);
        }
        self.released = true;
        self.state = State::Ready;
        Ok(())
    }

    /// Publish one TX period. Returns the slot / descriptor id.
    pub fn submit(&mut self, lease: Lease) -> Result<usize, Reject> {
        if self.revoked {
            return Err(Reject::Revoked);
        }
        self.live()?;
        self.check_lease(lease)?;
        if !matches!(self.state, State::Prepared | State::Running) {
            return Err(Reject::BadState);
        }
        let index = self
            .slots
            .iter()
            .position(|s| *s == Slot::Free)
            .ok_or(Reject::Again)?;
        self.slots[index] = Slot::Published;
        Ok(index)
    }

    /// I5: the rx queue never receives buffers.
    pub fn post_rx(&mut self) -> Result<(), Reject> {
        Err(Reject::RxForbidden)
    }

    /// Used-ring entry for `id`. Delivery to the broker re-authorises the
    /// lease; a stale lease is refused without touching any state.
    pub fn complete(
        &mut self,
        lease: Lease,
        id: u32,
        used_len: u32,
        status: u32,
    ) -> Result<Completion, Reject> {
        match self.state {
            State::Quarantined => return Err(Reject::Quarantined),
            // Ownership stays uncertain until both ACKs: no selective release.
            State::Fencing => return Err(Reject::Frozen),
            _ => {}
        }
        if self.used_snapshot.is_some() {
            // F11: completion after both ACKs.
            self.state = State::Quarantined;
            return Err(Reject::Quarantined);
        }
        self.check_lease(lease)?;
        let index = usize::try_from(id).map_err(|_| Reject::UnknownSlot)?;
        let published = self.slots.get(index).copied() == Some(Slot::Published);
        if !published {
            // F7/F8 and double completion: never free a slot twice.
            self.begin_fence_internal();
            return Err(Reject::UnknownSlot);
        }
        if used_len != TX_STATUS_LEN {
            self.begin_fence_internal();
            return Ok(Completion::Fenced);
        }
        match status {
            STATUS_OK => {
                self.slots[index] = Slot::Free;
                Ok(Completion::Ok)
            }
            STATUS_IO_ERR => {
                self.slots[index] = Slot::Free;
                if self.state == State::Running {
                    self.state = State::Stopped;
                }
                Ok(Completion::IoError)
            }
            _ => {
                self.begin_fence_internal();
                Ok(Completion::Fenced)
            }
        }
    }

    /// Internally detected faults freeze immediately; the caller must arm the
    /// deadline with `fence` before the next `tick`, otherwise it quarantines.
    fn begin_fence_internal(&mut self) {
        self.state = State::Fencing;
        self.status_zero = false;
        self.bme_off = false;
        self.deadline = 0;
    }

    /// Begin (or re-arm) a fence with a monotone deadline. All publication,
    /// control and completion work is frozen from here on.
    pub fn fence(&mut self, now: u64, budget: u64) -> Result<(), Reject> {
        if self.state == State::Quarantined {
            return Err(Reject::Quarantined);
        }
        if budget == 0 || budget > MAX_FENCE_BUDGET {
            return Err(Reject::BadBudget);
        }
        let Some(deadline) = now.checked_add(budget) else {
            self.state = State::Quarantined;
            return Err(Reject::Quarantined);
        };
        if self.state != State::Fencing {
            self.begin_fence_internal();
        }
        if self.deadline == 0 {
            self.deadline = deadline;
        }
        Ok(())
    }

    /// Revoke: refuse new submits, then fence. No replay.
    pub fn revoke(&mut self, now: u64, budget: u64) -> Result<(), Reject> {
        if self.state == State::Quarantined {
            return Err(Reject::Quarantined);
        }
        self.revoked = true;
        if let Err(error) = self.fence(now, budget) {
            // Revocation cannot leave a running transport when its deadline
            // is invalid. Keep every published slot pinned without a fence.
            self.state = State::Quarantined;
            return Err(error);
        }
        Ok(())
    }

    /// Read-back of the device status register.
    pub fn observe_status(&mut self, now: u64, status: u8) -> Result<(), Reject> {
        self.fence_input(now)?;
        if status == 0 {
            self.status_zero = true;
        } else if self.status_zero {
            // Contradicting read-back.
            self.state = State::Quarantined;
            return Err(Reject::Quarantined);
        }
        self.try_ack()
    }

    /// Read-back of PCI command.BME for the same device function.
    pub fn observe_bme(&mut self, now: u64, enabled: bool) -> Result<(), Reject> {
        self.fence_input(now)?;
        if !enabled {
            self.bme_off = true;
        } else if self.bme_off {
            self.state = State::Quarantined;
            return Err(Reject::Quarantined);
        }
        self.try_ack()
    }

    /// Explicitly ordered DMA read of the used index after both ACKs.
    pub fn observe_used_idx(&mut self, used_idx: u16) -> Result<(), Reject> {
        match self.used_snapshot {
            Some(s) if s != used_idx => {
                self.state = State::Quarantined;
                Err(Reject::Quarantined)
            }
            _ => Ok(()),
        }
    }

    /// Monotone time; a missed deadline quarantines (F12).
    pub fn tick(&mut self, now: u64) -> Result<(), Reject> {
        if self.state == State::Fencing && (self.deadline == 0 || now >= self.deadline) {
            self.state = State::Quarantined;
            return Err(Reject::Quarantined);
        }
        Ok(())
    }

    fn fence_input(&mut self, now: u64) -> Result<(), Reject> {
        if self.state == State::Quarantined {
            return Err(Reject::Quarantined);
        }
        if self.state != State::Fencing {
            return Err(Reject::BadState);
        }
        self.tick(now)
    }

    fn try_ack(&mut self) -> Result<(), Reject> {
        // Fence conjunction (TRANSPORT.md §5): both read-backs required.
        if !(self.status_zero && self.bme_off) {
            return Ok(());
        }
        let Some(epoch) = self.lease.epoch.checked_add(1) else {
            self.state = State::Quarantined;
            return Err(Reject::Quarantined);
        };
        self.slots = [Slot::Free; SLOTS];
        self.lease.epoch = epoch;
        self.used_snapshot = Some(0);
        self.state = State::Offline;
        Ok(())
    }
}

#[cfg(test)]
#[path = "transport_model_tests.rs"]
mod tests;
