#![no_std]

use core::num::NonZeroU64;

const IDENTITY_SLOTS: usize = 8;

macro_rules! identity_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(NonZeroU64);

        impl $name {
            pub const fn new(value: u64) -> Option<Self> {
                match NonZeroU64::new(value) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            pub const fn get(self) -> u64 {
                self.0.get()
            }
        }
    };
}

identity_type!(JobIdentity);
identity_type!(OwnerIdentity);
identity_type!(CapabilityScopeId);

/// Identifies one owner lifetime within the issuing `BackgroundJobs` instance.
/// Registrations and handles are supplied by a trusted runtime, not authenticated
/// by this core. Handles from different instances must not be interchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnerHandle {
    identity: OwnerIdentity,
    generation: NonZeroU64,
}

impl OwnerHandle {
    pub const fn identity(self) -> OwnerIdentity {
        self.identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityScope {
    pub id: CapabilityScopeId,
    pub generation: NonZeroU64,
}

impl CapabilityScope {
    pub const fn new(id: u64, generation: u64) -> Option<Self> {
        let Some(id) = CapabilityScopeId::new(id) else {
            return None;
        };
        let Some(generation) = NonZeroU64::new(generation) else {
            return None;
        };
        Some(Self { id, generation })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Purpose {
    Download,
    Synchronization,
    Indexing,
    LocalAi,
    UpdateStaging,
    ImportExport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceBudget {
    pub cpu_ticks: u64,
    pub memory_bytes: u64,
    pub io_bytes: u64,
}

impl ResourceBudget {
    pub const fn new(cpu_ticks: u64, memory_bytes: u64, io_bytes: u64) -> Option<Self> {
        if cpu_ticks == 0 || memory_bytes == 0 || io_bytes == 0 {
            None
        } else {
            Some(Self {
                cpu_ticks,
                memory_bytes,
                io_bytes,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Progress {
    pub completed_units: u64,
    pub total_units: u64,
}

impl Progress {
    pub const fn new(completed_units: u64, total_units: u64) -> Option<Self> {
        if total_units == 0 || completed_units > total_units {
            None
        } else {
            Some(Self {
                completed_units,
                total_units,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lifecycle {
    Queued,
    Running,
    Suspended,
    Completed,
    Failed,
    Cancelled,
}

impl Lifecycle {
    const fn terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Job {
    pub identity: JobIdentity,
    pub owner: OwnerHandle,
    pub purpose: Purpose,
    pub capability_scope: CapabilityScope,
    pub budget: ResourceBudget,
    pub progress: Progress,
    pub lifecycle: Lifecycle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidCapacity,
    InvalidBudget,
    InvalidProgress,
    DuplicateJob,
    Full,
    UnknownJob,
    OwnerLimit,
    OwnerAlreadyRegistered,
    OwnerGenerationExhausted,
    OwnerNotRegistered,
    OwnerExited,
    ScopeLimit,
    ScopeNotRegistered,
    ScopeAlreadyRegistered,
    ScopeStillActive,
    StaleScopeGeneration,
    ScopeRevoked,
    InvalidTransition,
    ProgressRegressed,
}

#[derive(Clone, Copy)]
struct OwnerState {
    handle: Option<OwnerHandle>,
    exited: bool,
}

impl OwnerState {
    const EMPTY: Self = Self {
        handle: None,
        exited: false,
    };
}

#[derive(Clone, Copy)]
struct ScopeState {
    id: Option<CapabilityScopeId>,
    generation: u64,
    active: bool,
}

impl ScopeState {
    const EMPTY: Self = Self {
        id: None,
        generation: 0,
        active: false,
    };
}

pub struct BackgroundJobs<const CAPACITY: usize> {
    jobs: [Option<Job>; CAPACITY],
    owners: [OwnerState; IDENTITY_SLOTS],
    owner_generation: u64,
    scopes: [ScopeState; IDENTITY_SLOTS],
}

impl<const CAPACITY: usize> BackgroundJobs<CAPACITY> {
    pub fn new() -> Result<Self, Error> {
        if CAPACITY == 0 {
            return Err(Error::InvalidCapacity);
        }
        Ok(Self {
            jobs: [None; CAPACITY],
            owners: [OwnerState::EMPTY; IDENTITY_SLOTS],
            owner_generation: 0,
            scopes: [ScopeState::EMPTY; IDENTITY_SLOTS],
        })
    }

    /// Issues a new lifetime handle for a trusted runtime's owner identity.
    /// A live identity can be registered only once; after exit it can be reused.
    pub fn register_owner(&mut self, identity: OwnerIdentity) -> Result<OwnerHandle, Error> {
        if self
            .owners
            .iter()
            .any(|owner| !owner.exited && owner.handle.is_some_and(|h| h.identity == identity))
        {
            return Err(Error::OwnerAlreadyRegistered);
        }
        let index = self
            .owners
            .iter()
            .position(|owner| owner.handle.is_none() || owner.exited)
            .ok_or(Error::OwnerLimit)?;
        let generation = self
            .owner_generation
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or(Error::OwnerGenerationExhausted)?;
        let handle = OwnerHandle {
            identity,
            generation,
        };
        self.owners[index] = OwnerState {
            handle: Some(handle),
            exited: false,
        };
        self.owner_generation = generation.get();
        Ok(handle)
    }

    pub fn register_scope(&mut self, scope: CapabilityScope) -> Result<(), Error> {
        if let Some(state) = self
            .scopes
            .iter_mut()
            .find(|state| state.id == Some(scope.id))
        {
            if state.active {
                return if state.generation == scope.generation.get() {
                    Err(Error::ScopeAlreadyRegistered)
                } else {
                    Err(Error::ScopeStillActive)
                };
            }
            if scope.generation.get() <= state.generation {
                return Err(Error::StaleScopeGeneration);
            }
            state.generation = scope.generation.get();
            state.active = true;
            return Ok(());
        }
        let state = self
            .scopes
            .iter_mut()
            .find(|state| state.id.is_none())
            .ok_or(Error::ScopeLimit)?;
        *state = ScopeState {
            id: Some(scope.id),
            generation: scope.generation.get(),
            active: true,
        };
        Ok(())
    }

    pub fn create(
        &mut self,
        identity: JobIdentity,
        owner: OwnerHandle,
        purpose: Purpose,
        scope: CapabilityScope,
        budget: ResourceBudget,
        total_units: u64,
    ) -> Result<Job, Error> {
        if budget.cpu_ticks == 0 || budget.memory_bytes == 0 || budget.io_bytes == 0 {
            return Err(Error::InvalidBudget);
        }
        if total_units == 0 {
            return Err(Error::InvalidProgress);
        }
        if self
            .jobs
            .iter()
            .flatten()
            .any(|job| job.identity == identity)
        {
            return Err(Error::DuplicateJob);
        }
        let owner_state = self
            .owners
            .iter()
            .find(|state| state.handle == Some(owner))
            .ok_or(Error::OwnerNotRegistered)?;
        if owner_state.exited {
            return Err(Error::OwnerExited);
        }
        self.require_active_scope(scope)?;
        let slot = self
            .jobs
            .iter()
            .position(Option::is_none)
            .ok_or(Error::Full)?;
        let job = Job {
            identity,
            owner,
            purpose,
            capability_scope: scope,
            budget,
            progress: Progress {
                completed_units: 0,
                total_units,
            },
            lifecycle: Lifecycle::Queued,
        };
        self.jobs[slot] = Some(job);
        Ok(job)
    }

    pub fn start(&mut self, identity: JobIdentity) -> Result<Job, Error> {
        self.transition(identity, Lifecycle::Queued, Lifecycle::Running)
    }

    pub fn suspend(&mut self, identity: JobIdentity) -> Result<Job, Error> {
        self.transition(identity, Lifecycle::Running, Lifecycle::Suspended)
    }

    pub fn resume(&mut self, identity: JobIdentity) -> Result<Job, Error> {
        self.transition(identity, Lifecycle::Suspended, Lifecycle::Running)
    }

    pub fn complete(&mut self, identity: JobIdentity) -> Result<Job, Error> {
        let job = self.get(identity)?;
        if job.lifecycle != Lifecycle::Running
            || job.progress.completed_units != job.progress.total_units
        {
            return Err(Error::InvalidTransition);
        }
        self.set_lifecycle(identity, Lifecycle::Completed)
    }

    pub fn fail(&mut self, identity: JobIdentity) -> Result<Job, Error> {
        let job = self.get(identity)?;
        if job.lifecycle.terminal() {
            return Err(Error::InvalidTransition);
        }
        self.set_lifecycle(identity, Lifecycle::Failed)
    }

    pub fn cancel(&mut self, identity: JobIdentity) -> Result<Job, Error> {
        let job = self.get(identity)?;
        if job.lifecycle.terminal() {
            return Err(Error::InvalidTransition);
        }
        self.set_lifecycle(identity, Lifecycle::Cancelled)
    }

    pub fn update_progress(
        &mut self,
        identity: JobIdentity,
        completed_units: u64,
    ) -> Result<Job, Error> {
        let index = self.job_index(identity)?;
        let mut job = self.jobs[index].ok_or(Error::UnknownJob)?;
        if job.lifecycle != Lifecycle::Running
            || completed_units < job.progress.completed_units
            || completed_units > job.progress.total_units
        {
            return Err(if completed_units < job.progress.completed_units {
                Error::ProgressRegressed
            } else {
                Error::InvalidTransition
            });
        }
        job.progress.completed_units = completed_units;
        self.jobs[index] = Some(job);
        Ok(job)
    }

    pub fn revoke_scope(&mut self, scope: CapabilityScope) -> Result<usize, Error> {
        let state = self
            .scopes
            .iter_mut()
            .find(|state| state.id == Some(scope.id))
            .ok_or(Error::ScopeNotRegistered)?;
        if !state.active || state.generation != scope.generation.get() {
            return Err(Error::ScopeRevoked);
        }
        state.active = false;
        let mut cancelled = 0;
        for job in self.jobs.iter_mut().flatten() {
            if job.capability_scope == scope && !job.lifecycle.terminal() {
                job.lifecycle = Lifecycle::Cancelled;
                cancelled += 1;
            }
        }
        Ok(cancelled)
    }

    pub fn owner_exit(&mut self, owner: OwnerHandle) -> Result<usize, Error> {
        let state = self
            .owners
            .iter_mut()
            .find(|state| state.handle == Some(owner))
            .ok_or(Error::OwnerNotRegistered)?;
        state.exited = true;
        let mut cancelled = 0;
        for job in self.jobs.iter_mut().flatten() {
            if job.owner == owner && !job.lifecycle.terminal() {
                job.lifecycle = Lifecycle::Cancelled;
                cancelled += 1;
            }
        }
        Ok(cancelled)
    }

    pub fn get(&self, identity: JobIdentity) -> Result<Job, Error> {
        self.jobs
            .iter()
            .flatten()
            .find(|job| job.identity == identity)
            .copied()
            .ok_or(Error::UnknownJob)
    }

    fn require_active_scope(&self, scope: CapabilityScope) -> Result<(), Error> {
        let state = self
            .scopes
            .iter()
            .find(|state| state.id == Some(scope.id))
            .ok_or(Error::ScopeNotRegistered)?;
        if !state.active || state.generation != scope.generation.get() {
            return Err(Error::ScopeRevoked);
        }
        Ok(())
    }

    fn transition(
        &mut self,
        identity: JobIdentity,
        from: Lifecycle,
        to: Lifecycle,
    ) -> Result<Job, Error> {
        if self.get(identity)?.lifecycle != from {
            return Err(Error::InvalidTransition);
        }
        self.set_lifecycle(identity, to)
    }

    fn set_lifecycle(&mut self, identity: JobIdentity, lifecycle: Lifecycle) -> Result<Job, Error> {
        let index = self.job_index(identity)?;
        let mut job = self.jobs[index].ok_or(Error::UnknownJob)?;
        job.lifecycle = lifecycle;
        self.jobs[index] = Some(job);
        Ok(job)
    }

    fn job_index(&self, identity: JobIdentity) -> Result<usize, Error> {
        self.jobs
            .iter()
            .position(|job| job.is_some_and(|job| job.identity == identity))
            .ok_or(Error::UnknownJob)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner(value: u64) -> OwnerIdentity {
        OwnerIdentity::new(value).unwrap()
    }

    fn job(value: u64) -> JobIdentity {
        JobIdentity::new(value).unwrap()
    }

    fn scope(generation: u64) -> CapabilityScope {
        CapabilityScope::new(50, generation).unwrap()
    }

    fn budget() -> ResourceBudget {
        ResourceBudget::new(100, 4096, 8192).unwrap()
    }

    fn setup<const N: usize>() -> (BackgroundJobs<N>, OwnerHandle, CapabilityScope) {
        let mut jobs = BackgroundJobs::new().unwrap();
        let owner = jobs.register_owner(owner(10)).unwrap();
        let scope = scope(1);
        jobs.register_scope(scope).unwrap();
        (jobs, owner, scope)
    }

    #[test]
    fn rejects_empty_bounds_and_invalid_budget_or_progress() {
        assert_eq!(
            BackgroundJobs::<0>::new().err(),
            Some(Error::InvalidCapacity)
        );
        assert!(ResourceBudget::new(0, 1, 1).is_none());
        assert!(ResourceBudget::new(1, 0, 1).is_none());
        assert!(ResourceBudget::new(1, 1, 0).is_none());
        assert!(Progress::new(0, 0).is_none());
        assert!(Progress::new(2, 1).is_none());
        assert!(CapabilityScope::new(0, 1).is_none());
        assert!(CapabilityScope::new(1, 0).is_none());
    }

    #[test]
    fn lifecycle_preserves_the_explicit_scope_and_bounds_progress() {
        let (mut jobs, owner, scope) = setup::<2>();
        let created = jobs
            .create(job(1), owner, Purpose::Synchronization, scope, budget(), 3)
            .unwrap();
        assert_eq!(created.lifecycle, Lifecycle::Queued);
        assert_eq!(jobs.suspend(job(1)), Err(Error::InvalidTransition));
        assert_eq!(jobs.complete(job(1)), Err(Error::InvalidTransition));
        assert_eq!(jobs.start(job(1)).unwrap().capability_scope, scope);
        assert_eq!(
            jobs.update_progress(job(1), 2)
                .unwrap()
                .progress
                .completed_units,
            2
        );
        assert_eq!(
            jobs.update_progress(job(1), 1),
            Err(Error::ProgressRegressed)
        );
        assert_eq!(jobs.complete(job(1)), Err(Error::InvalidTransition));
        assert_eq!(jobs.suspend(job(1)).unwrap().capability_scope, scope);
        assert_eq!(jobs.resume(job(1)).unwrap().capability_scope, scope);
        jobs.update_progress(job(1), 3).unwrap();
        let completed = jobs.complete(job(1)).unwrap();
        assert_eq!(completed.lifecycle, Lifecycle::Completed);
        assert_eq!(completed.capability_scope, scope);
    }

    #[test]
    fn revoke_cancels_jobs_and_stale_scope_cannot_be_reused() {
        let (mut jobs, owner, old_scope) = setup::<3>();
        jobs.create(job(1), owner, Purpose::Download, old_scope, budget(), 10)
            .unwrap();
        jobs.start(job(1)).unwrap();
        assert_eq!(jobs.revoke_scope(old_scope), Ok(1));
        assert_eq!(jobs.get(job(1)).unwrap().lifecycle, Lifecycle::Cancelled);
        assert_eq!(
            jobs.create(job(2), owner, Purpose::Download, old_scope, budget(), 10),
            Err(Error::ScopeRevoked)
        );
        assert_eq!(
            jobs.register_scope(old_scope),
            Err(Error::StaleScopeGeneration)
        );
        let renewed = scope(2);
        jobs.register_scope(renewed).unwrap();
        jobs.create(job(3), owner, Purpose::Download, renewed, budget(), 10)
            .unwrap();
        assert_eq!(
            jobs.get(job(3)).unwrap().capability_scope.generation,
            renewed.generation
        );
    }

    #[test]
    fn owner_exit_cancels_jobs_and_does_not_transfer_authority() {
        let (mut jobs, owner, scope) = setup::<3>();
        jobs.create(job(1), owner, Purpose::Indexing, scope, budget(), 4)
            .unwrap();
        jobs.start(job(1)).unwrap();
        assert_eq!(jobs.owner_exit(owner), Ok(1));
        let cancelled = jobs.get(job(1)).unwrap();
        assert_eq!(cancelled.lifecycle, Lifecycle::Cancelled);
        assert_eq!(cancelled.capability_scope, scope);
        assert_eq!(
            jobs.create(job(2), owner, Purpose::Indexing, scope, budget(), 4),
            Err(Error::OwnerExited)
        );
        assert_eq!(jobs.start(job(1)), Err(Error::InvalidTransition));
    }

    #[test]
    fn owner_slots_reuse_lifetimes_without_retargeting_jobs() {
        let (mut jobs, old, scope) = setup::<2>();
        assert_eq!(
            jobs.register_owner(old.identity()),
            Err(Error::OwnerAlreadyRegistered)
        );
        jobs.create(job(1), old, Purpose::Indexing, scope, budget(), 4)
            .unwrap();
        jobs.start(job(1)).unwrap();
        // Keep the other seven slots occupied while the first slot is recycled.
        for id in 1..IDENTITY_SLOTS as u64 {
            jobs.register_owner(owner(id)).unwrap();
        }
        let generation = jobs.owner_generation;
        assert_eq!(jobs.register_owner(owner(100)), Err(Error::OwnerLimit));
        assert_eq!(jobs.owner_generation, generation);
        assert_eq!(jobs.owner_exit(old), Ok(1));
        assert_eq!(jobs.owner_exit(old), Ok(0));

        let mut current = jobs.register_owner(old.identity()).unwrap();
        assert_ne!(current, old);
        assert_eq!(current.identity(), old.identity());
        assert_eq!(jobs.get(job(1)).unwrap().owner, old);
        assert_eq!(jobs.get(job(1)).unwrap().lifecycle, Lifecycle::Cancelled);
        assert_eq!(jobs.owner_exit(old), Err(Error::OwnerNotRegistered));
        assert_eq!(
            jobs.create(job(2), old, Purpose::Indexing, scope, budget(), 4),
            Err(Error::OwnerNotRegistered)
        );
        // More than eight lifetimes, in the same slot and with the same ID.
        for _ in 0..IDENTITY_SLOTS + 1 {
            assert_eq!(jobs.owner_exit(current), Ok(0));
            let next = jobs.register_owner(old.identity()).unwrap();
            assert_ne!(next, current);
            assert_eq!(jobs.owner_exit(current), Err(Error::OwnerNotRegistered));
            current = next;
        }
        jobs.create(job(2), current, Purpose::Indexing, scope, budget(), 4)
            .unwrap();
        assert_eq!(jobs.owner_exit(old), Err(Error::OwnerNotRegistered));
        assert_eq!(jobs.get(job(2)).unwrap().lifecycle, Lifecycle::Queued);
        assert_eq!(jobs.owner_exit(current), Ok(1));
        assert_eq!(jobs.get(job(1)).unwrap().owner, old);
        assert_eq!(jobs.get(job(2)).unwrap().owner, current);
    }

    #[test]
    fn exhausted_owner_generation_does_not_mutate_registration_state() {
        let (mut jobs, old, scope) = setup::<1>();
        jobs.owner_exit(old).unwrap();
        jobs.owner_generation = u64::MAX - 1;
        let last = jobs.register_owner(old.identity()).unwrap();
        assert_eq!(jobs.owner_generation, u64::MAX);
        jobs.owner_exit(last).unwrap();
        assert_eq!(
            jobs.register_owner(old.identity()),
            Err(Error::OwnerGenerationExhausted)
        );
        assert_eq!(jobs.owner_generation, u64::MAX);
        assert_eq!(jobs.owner_exit(last), Ok(0));
        assert_eq!(
            jobs.create(job(1), last, Purpose::Indexing, scope, budget(), 4),
            Err(Error::OwnerExited)
        );
    }

    #[test]
    fn capacity_and_terminal_transitions_fail_closed() {
        let (mut jobs, owner, scope) = setup::<1>();
        jobs.create(job(1), owner, Purpose::LocalAi, scope, budget(), 1)
            .unwrap();
        assert_eq!(
            jobs.create(job(2), owner, Purpose::LocalAi, scope, budget(), 1),
            Err(Error::Full)
        );
        assert_eq!(jobs.cancel(job(1)).unwrap().lifecycle, Lifecycle::Cancelled);
        assert_eq!(jobs.cancel(job(1)), Err(Error::InvalidTransition));
        assert_eq!(
            jobs.create(job(1), owner, Purpose::LocalAi, scope, budget(), 1),
            Err(Error::DuplicateJob)
        );
    }

    #[test]
    fn scope_generations_cannot_advance_while_authority_is_active() {
        let (mut jobs, _owner, old_scope) = setup::<1>();
        assert_eq!(jobs.register_scope(scope(2)), Err(Error::ScopeStillActive));
        jobs.revoke_scope(old_scope).unwrap();
        assert_eq!(
            jobs.register_scope(old_scope),
            Err(Error::StaleScopeGeneration)
        );
        jobs.register_scope(scope(2)).unwrap();
        assert_eq!(
            jobs.register_scope(scope(2)),
            Err(Error::ScopeAlreadyRegistered)
        );
    }
}
