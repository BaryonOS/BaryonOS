//! Bounded processor inventory and monotonic AP bring-up state.

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[cfg(test)]
extern crate std;

pub const MAX_CPUS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Count,
    DuplicateId,
    MissingBootstrap,
    UnknownCpu,
    InvalidTransition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unused,
    Bootstrap,
    Discovered,
    Starting,
    Online,
    Failed,
}

const RENDEZVOUS_COUNT_BITS: u32 = 16;
const RENDEZVOUS_COUNT_MASK: u64 = (1 << RENDEZVOUS_COUNT_BITS) - 1;
const RENDEZVOUS_FAILED: u64 = 1 << 63;
const RENDEZVOUS_MAX_GENERATION: u64 = (RENDEZVOUS_FAILED - 1) >> RENDEZVOUS_COUNT_BITS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendezvousError {
    ParticipantCount,
    AlreadyConfigured,
    NotConfigured,
    Failed,
    Timeout,
    TooManyParticipants,
    GenerationOverflow,
}

/// Fixed-participant rendezvous. Each participant must arrive exactly once per generation.
pub struct Rendezvous {
    state: AtomicU64,
    participants: AtomicUsize,
}

impl Rendezvous {
    pub const fn new() -> Self {
        Self {
            state: AtomicU64::new(0),
            participants: AtomicUsize::new(0),
        }
    }

    pub fn configure(&self, participants: usize) -> Result<(), RendezvousError> {
        if !(2..=MAX_CPUS).contains(&participants) {
            return Err(RendezvousError::ParticipantCount);
        }
        self.participants
            .compare_exchange(0, participants, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| RendezvousError::AlreadyConfigured)
    }

    /// Wait for all configured participants, failing closed after a bounded spin count.
    /// Returns the zero-based generation that completed.
    pub fn wait_bounded(&self, max_spins: usize) -> Result<u64, RendezvousError> {
        let participants = self.participants.load(Ordering::Acquire);
        if participants == 0 {
            return Err(RendezvousError::NotConfigured);
        }

        let mut state = self.state.load(Ordering::Acquire);
        if state & RENDEZVOUS_FAILED != 0 {
            return Err(RendezvousError::Failed);
        }
        let generation = state >> RENDEZVOUS_COUNT_BITS;

        loop {
            if state & RENDEZVOUS_FAILED != 0 {
                return Err(RendezvousError::Failed);
            }
            if state >> RENDEZVOUS_COUNT_BITS != generation {
                return Ok(generation);
            }

            let arrived = (state & RENDEZVOUS_COUNT_MASK) as usize;
            if arrived >= participants {
                return self.fail_generation(generation, RendezvousError::TooManyParticipants);
            }
            if arrived + 1 == participants {
                if generation == RENDEZVOUS_MAX_GENERATION {
                    return self.fail_generation(generation, RendezvousError::GenerationOverflow);
                }
                let next = (generation + 1) << RENDEZVOUS_COUNT_BITS;
                match self.state.compare_exchange_weak(
                    state,
                    next,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return Ok(generation),
                    Err(observed) => state = observed,
                }
            } else {
                match self.state.compare_exchange_weak(
                    state,
                    state + 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => break,
                    Err(observed) => state = observed,
                }
            }
        }

        for _ in 0..max_spins {
            state = self.state.load(Ordering::Acquire);
            if state & RENDEZVOUS_FAILED != 0 {
                return Err(RendezvousError::Failed);
            }
            if state >> RENDEZVOUS_COUNT_BITS != generation {
                return Ok(generation);
            }
            core::hint::spin_loop();
        }

        self.fail_generation(generation, RendezvousError::Timeout)
    }

    fn fail_generation(
        &self,
        generation: u64,
        error: RendezvousError,
    ) -> Result<u64, RendezvousError> {
        let mut state = self.state.load(Ordering::Acquire);
        loop {
            if state & RENDEZVOUS_FAILED != 0 {
                return Err(RendezvousError::Failed);
            }
            if state >> RENDEZVOUS_COUNT_BITS != generation {
                return Ok(generation);
            }
            match self.state.compare_exchange_weak(
                state,
                state | RENDEZVOUS_FAILED,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Err(error),
                Err(observed) => state = observed,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cpu {
    apic_id: u32,
    state: State,
}

const EMPTY_CPU: Cpu = Cpu {
    apic_id: 0,
    state: State::Unused,
};

pub struct CpuSet {
    cpus: [Cpu; MAX_CPUS],
    count: usize,
    bootstrap: usize,
}

impl CpuSet {
    pub fn new(apic_ids: &[u32], bootstrap_id: u32) -> Result<Self, Error> {
        if apic_ids.is_empty() || apic_ids.len() > MAX_CPUS {
            return Err(Error::Count);
        }
        for (index, id) in apic_ids.iter().enumerate() {
            if apic_ids[..index].contains(id) {
                return Err(Error::DuplicateId);
            }
        }
        let bootstrap = apic_ids
            .iter()
            .position(|id| *id == bootstrap_id)
            .ok_or(Error::MissingBootstrap)?;
        let mut cpus = [EMPTY_CPU; MAX_CPUS];
        for (index, id) in apic_ids.iter().copied().enumerate() {
            cpus[index] = Cpu {
                apic_id: id,
                state: if index == bootstrap {
                    State::Bootstrap
                } else {
                    State::Discovered
                },
            };
        }
        Ok(Self {
            cpus,
            count: apic_ids.len(),
            bootstrap,
        })
    }

    pub const fn count(&self) -> usize {
        self.count
    }

    pub const fn bootstrap_index(&self) -> usize {
        self.bootstrap
    }

    pub fn apic_id(&self, index: usize) -> Option<u32> {
        self.cpus
            .get(index)
            .filter(|cpu| cpu.state != State::Unused)
            .map(|cpu| cpu.apic_id)
    }

    pub fn state(&self, apic_id: u32) -> Option<State> {
        self.cpus[..self.count]
            .iter()
            .find(|cpu| cpu.apic_id == apic_id)
            .map(|cpu| cpu.state)
    }

    pub fn online_count(&self) -> usize {
        self.cpus[..self.count]
            .iter()
            .filter(|cpu| matches!(cpu.state, State::Bootstrap | State::Online))
            .count()
    }

    pub fn begin_start(&mut self, apic_id: u32) -> Result<(), Error> {
        self.transition(apic_id, State::Discovered, State::Starting)
    }

    pub fn mark_online(&mut self, apic_id: u32) -> Result<(), Error> {
        self.transition(apic_id, State::Starting, State::Online)
    }

    pub fn mark_failed(&mut self, apic_id: u32) -> Result<(), Error> {
        let cpu = self.cpu_mut(apic_id)?;
        if !matches!(cpu.state, State::Discovered | State::Starting) {
            return Err(Error::InvalidTransition);
        }
        cpu.state = State::Failed;
        Ok(())
    }

    fn transition(&mut self, apic_id: u32, from: State, to: State) -> Result<(), Error> {
        let cpu = self.cpu_mut(apic_id)?;
        if cpu.state != from {
            return Err(Error::InvalidTransition);
        }
        cpu.state = to;
        Ok(())
    }

    fn cpu_mut(&mut self, apic_id: u32) -> Result<&mut Cpu, Error> {
        self.cpus[..self.count]
            .iter_mut()
            .find(|cpu| cpu.apic_id == apic_id)
            .ok_or(Error::UnknownCpu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, thread};

    #[test]
    fn inventory_accepts_sparse_ids_and_marks_only_the_bsp_online() {
        let cpus = CpuSet::new(&[2, 0, 7, 11], 7).unwrap();
        assert_eq!(cpus.count(), 4);
        assert_eq!(cpus.bootstrap_index(), 2);
        assert_eq!(cpus.online_count(), 1);
        assert_eq!(cpus.apic_id(3), Some(11));
        assert_eq!(cpus.apic_id(MAX_CPUS), None);
        assert_eq!(cpus.state(7), Some(State::Bootstrap));
        assert_eq!(cpus.state(2), Some(State::Discovered));
    }

    #[test]
    fn inventory_rejects_empty_oversized_duplicate_and_missing_bsp_topologies() {
        assert_eq!(CpuSet::new(&[], 0).err(), Some(Error::Count));
        assert_eq!(CpuSet::new(&[0, 1, 2, 3, 4], 0).err(), Some(Error::Count));
        assert_eq!(CpuSet::new(&[0, 1, 1], 0).err(), Some(Error::DuplicateId));
        assert_eq!(CpuSet::new(&[1, 2], 0).err(), Some(Error::MissingBootstrap));
    }

    #[test]
    fn bringup_transitions_are_ordered_and_failure_is_terminal() {
        let mut cpus = CpuSet::new(&[0, 3, 9], 0).unwrap();
        assert_eq!(cpus.mark_online(3), Err(Error::InvalidTransition));
        cpus.begin_start(3).unwrap();
        assert_eq!(cpus.begin_start(3), Err(Error::InvalidTransition));
        cpus.mark_online(3).unwrap();
        assert_eq!(cpus.mark_failed(3), Err(Error::InvalidTransition));

        cpus.mark_failed(9).unwrap();
        assert_eq!(cpus.begin_start(9), Err(Error::InvalidTransition));
        assert_eq!(cpus.mark_online(99), Err(Error::UnknownCpu));
        assert_eq!(cpus.online_count(), 2);
        assert_eq!(cpus.state(9), Some(State::Failed));
    }

    #[test]
    fn rendezvous_rejects_invalid_or_repeated_configuration() {
        let rendezvous = Rendezvous::new();
        assert_eq!(
            rendezvous.wait_bounded(1),
            Err(RendezvousError::NotConfigured)
        );
        for participants in [0, 1, MAX_CPUS + 1] {
            assert_eq!(
                rendezvous.configure(participants),
                Err(RendezvousError::ParticipantCount)
            );
        }
        rendezvous.configure(2).unwrap();
        assert_eq!(
            rendezvous.configure(2),
            Err(RendezvousError::AlreadyConfigured)
        );
    }

    #[test]
    fn rendezvous_releases_all_participants_for_repeated_generations() {
        let rendezvous = Arc::new(Rendezvous::new());
        rendezvous.configure(MAX_CPUS).unwrap();
        let workers: [thread::JoinHandle<()>; MAX_CPUS] = core::array::from_fn(|_| {
            let rendezvous = Arc::clone(&rendezvous);
            thread::spawn(move || {
                for generation in 0..128 {
                    assert_eq!(rendezvous.wait_bounded(10_000_000), Ok(generation));
                }
            })
        });
        for worker in workers {
            worker.join().unwrap();
        }
    }

    #[test]
    fn rendezvous_timeout_poison_prevents_late_reuse() {
        let rendezvous = Rendezvous::new();
        rendezvous.configure(2).unwrap();
        assert_eq!(rendezvous.wait_bounded(8), Err(RendezvousError::Timeout));
        assert_eq!(rendezvous.wait_bounded(8), Err(RendezvousError::Failed));
    }
}
