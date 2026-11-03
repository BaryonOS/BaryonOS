//! Bounded processor inventory and monotonic AP bring-up state.

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
}
