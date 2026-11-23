//! Bounded attach bookkeeping, without transport, buffers or DMA ownership.
#![forbid(unsafe_code)]
use crate::boot_session::Lifetime;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Invalid,
    Busy,
    Stale,
    Exhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Detached,
    Enumerating,
    Retry,
    Ready,
    Failed,
}

/// Correlation only, not proof of a hardware completion or a capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ticket {
    port: usize,
    lifetime: Lifetime,
    attempt: u8,
}

impl Ticket {
    /// The owner-assigned identity a fresh Session for this attach must use.
    /// Reading it confers no device, transfer or completion authority.
    pub const fn lifetime(self) -> Lifetime {
        self.lifetime
    }
}

#[derive(Clone, Copy)]
struct Entry {
    generation: u64,
    attempt: u8,
    phase: Phase,
}

/// Exclusive controller owner. Device IDs must be globally unique and not reused
/// across instances while old completions can exist, as required by Lifetime.
/// Attach budget lasts for this entire instance; detach never replenishes it.
pub struct Ports<const N: usize> {
    devices: [u64; N],
    entries: [Entry; N],
    remaining: u32,
    attempts: u8,
}

impl<const N: usize> Ports<N> {
    pub fn new(devices: [u64; N], attach_budget: u32, attempts: u8) -> Result<Self, Error> {
        if attempts == 0
            || devices
                .iter()
                .enumerate()
                .any(|(i, d)| *d == 0 || devices[..i].contains(d))
        {
            return Err(Error::Invalid);
        }
        Ok(Self {
            devices,
            entries: [Entry {
                generation: 0,
                attempt: 0,
                phase: Phase::Detached,
            }; N],
            remaining: attach_budget,
            attempts,
        })
    }

    pub fn phase(&self, port: usize) -> Result<Phase, Error> {
        self.entries
            .get(port)
            .map(|e| e.phase)
            .ok_or(Error::Invalid)
    }

    pub fn attach(&mut self, port: usize) -> Result<Ticket, Error> {
        let entry = self.entries.get_mut(port).ok_or(Error::Invalid)?;
        if entry.phase != Phase::Detached {
            return Err(Error::Busy);
        }
        if self.remaining == 0 {
            return Err(Error::Exhausted);
        }
        let generation = entry.generation.checked_add(1).ok_or(Error::Exhausted)?;
        self.remaining -= 1;
        *entry = Entry {
            generation,
            attempt: 1,
            phase: Phase::Enumerating,
        };
        Ok(self.ticket(port))
    }

    fn ticket(&self, port: usize) -> Ticket {
        Ticket {
            port,
            lifetime: Lifetime {
                device: self.devices[port],
                generation: self.entries[port].generation,
            },
            attempt: self.entries[port].attempt,
        }
    }

    /// Call only for a real, matched enumeration result. Failure enables a
    /// bounded retry; no live transfer may be overlapped by a retry.
    pub fn complete(&mut self, ticket: Ticket, success: bool) -> Result<Option<Lifetime>, Error> {
        if self.phase(ticket.port)? != Phase::Enumerating || self.ticket(ticket.port) != ticket {
            return Err(Error::Stale);
        }
        let entry = &mut self.entries[ticket.port];
        entry.phase = if success {
            Phase::Ready
        } else if entry.attempt < self.attempts {
            Phase::Retry
        } else {
            Phase::Failed
        };
        Ok(success.then_some(ticket.lifetime))
    }

    pub fn retry(&mut self, port: usize) -> Result<Ticket, Error> {
        let entry = self.entries.get_mut(port).ok_or(Error::Invalid)?;
        if entry.phase != Phase::Retry {
            return Err(Error::Busy);
        }
        entry.attempt = entry.attempt.checked_add(1).ok_or(Error::Exhausted)?;
        entry.phase = Phase::Enumerating;
        Ok(self.ticket(port))
    }

    /// Retires bookkeeping immediately. The adapter must independently stop
    /// hardware and quarantine submitted buffers until real quiescence.
    pub fn detach(&mut self, port: usize) -> Result<(), Error> {
        let entry = self.entries.get_mut(port).ok_or(Error::Invalid)?;
        entry.phase = Phase::Detached;
        entry.attempt = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generation_never_wraps_or_reuses_the_final_lifetime() {
        let mut ports = Ports::new([10], 3, 1).unwrap();
        ports.entries[0].generation = u64::MAX - 1;
        let ticket = ports.attach(0).unwrap();
        assert_eq!(
            ports.complete(ticket, true).unwrap().unwrap().generation,
            u64::MAX
        );
        ports.detach(0).unwrap();
        assert_eq!(ports.attach(0), Err(Error::Exhausted));
        assert_eq!(ports.entries[0].generation, u64::MAX);
        assert_eq!(ports.remaining, 2);
        assert_eq!(ports.complete(ticket, true), Err(Error::Stale));
    }
}
