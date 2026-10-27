#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod complex;
pub mod gate;
pub mod sampler;
mod statevector;
mod validate;

use alloc::vec::Vec;
pub use gate::Gate;
pub use validate::{ValidatedJob, ValidationError, MAX_GATES, MAX_QUBITS, MAX_SHOTS, MAX_WORK};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Abort {
    Cancelled,
    DeadlineExceeded,
}

pub trait Control {
    fn checkpoint(&mut self, work_done: u64) -> Result<(), Abort>;
}

pub struct Uninterrupted;
impl Control for Uninterrupted {
    fn checkpoint(&mut self, _: u64) -> Result<(), Abort> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteError {
    ResourceExhausted,
    NumericalError,
    Aborted(Abort),
}

impl From<Abort> for ExecuteError {
    fn from(error: Abort) -> Self {
        Self::Aborted(error)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct JobResult {
    /// Basis-index order; zero counts are omitted only by the host encoder.
    pub counts: Vec<u32>,
    pub shots_completed: u32,
    /// Peak owned buffer capacities in bytes (not allocator overhead/process RAM).
    pub buffer_bytes: usize,
}

/// Execute one validated job. Cancellation returns no partial histogram.
pub fn execute(job: &ValidatedJob, control: &mut impl Control) -> Result<JobResult, ExecuteError> {
    let (state, mut work) = statevector::evolve(job, control)?;
    let cdf = statevector::distribution(&state, control, &mut work)?;
    control.checkpoint(work)?;
    let mut counts = Vec::new();
    counts
        .try_reserve_exact(state.len())
        .map_err(|_| ExecuteError::ResourceExhausted)?;
    for index in 0..state.len() {
        if index % 256 == 0 {
            control.checkpoint(work)?;
        }
        counts.push(0_u32);
        work += 1;
    }
    let mut rng = sampler::SplitMix64::new(job.seed());
    for shot in 0..job.shots() {
        if shot % 64 == 0 {
            control.checkpoint(work)?;
        }
        let sample = rng.unit();
        // CDF is monotone and ends in 1.0; unit() is strictly below 1.0.
        let index = cdf.partition_point(|&probability| probability <= sample);
        let count = counts.get_mut(index).ok_or(ExecuteError::NumericalError)?;
        *count += 1;
        work += u64::from(job.qubits()) + 1;
    }
    control.checkpoint(work)?;
    let buffer_bytes = state.capacity() * core::mem::size_of::<complex::Complex>()
        + cdf.capacity() * core::mem::size_of::<f64>()
        + counts.capacity() * core::mem::size_of::<u32>()
        + job.gates().capacity() * core::mem::size_of::<Gate>();
    Ok(JobResult {
        counts,
        shots_completed: job.shots(),
        buffer_bytes,
    })
}
