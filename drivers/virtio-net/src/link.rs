//! Pure FEATURE_STATUS tracker; no MMIO, DMA or automatic device recovery.
use crate::{Error, FEATURE_STATUS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub generation: u64,
    pub up: bool,
}

/// One exclusively owned device lifetime. The adapter supplies strictly increasing
/// observation serials, NOT the wrapping u8 device configuration generation.
/// Reset or budget exhaustion is terminal; replacing this tracker requires an
/// externally confirmed device reset/lifetime handoff, not an app request.
pub struct Tracker {
    last: Option<Event>,
    remaining: u32,
    fault: Option<Error>,
}

impl Tracker {
    /// Budget counts actual up/down transitions after the first observation.
    /// Stable observations do not spend it; no implicit time-window replenishment.
    pub fn new(negotiated_features: u64, transition_budget: u32) -> Result<Self, Error> {
        if negotiated_features & FEATURE_STATUS == 0 {
            return Err(Error::Unsupported);
        }
        Ok(Self {
            last: None,
            remaining: transition_budget,
            fault: None,
        })
    }

    /// No usable link state is exposed after reset or exhaustion.
    pub fn state(&self) -> Result<Option<Event>, Error> {
        match self.fault {
            Some(error) => Err(error),
            None => Ok(self.last),
        }
    }

    /// Consume a coherently read virtio-net status word. Returns an event only
    /// for the first sample or a transition; stale samples cannot change state.
    pub fn observe(
        &mut self,
        generation: u64,
        status: u16,
        resetting: bool,
    ) -> Result<Option<Event>, Error> {
        self.state()?;
        if resetting {
            self.fault = Some(Error::Io);
            return Err(Error::Io);
        }
        if generation == 0 || self.last.is_some_and(|last| generation <= last.generation) {
            return Err(Error::StaleEpoch);
        }
        let event = Event {
            generation,
            up: status & 1 != 0,
        };
        let changed = self.last.is_none_or(|last| last.up != event.up);
        if changed && self.last.is_some() {
            if self.remaining == 0 {
                self.fault = Some(Error::Exhausted);
                return Err(Error::Exhausted);
            }
            self.remaining -= 1;
        }
        self.last = Some(event);
        Ok(changed.then_some(event))
    }
}
