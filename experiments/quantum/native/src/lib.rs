//! Bounded adapter for the unchanged qsim-core on Nieuw OS.
//!
//! This module owns no syscall, device, clock, network, or storage authority.
//! The freestanding probe supplies the allocator and cooperative control.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
pub use qsim_core::{Abort, Control, ExecuteError, Gate, ValidationError};

pub mod heap_budget;

pub const MAX_QUBITS: u64 = 6;
pub const MAX_GATES: usize = 64;
pub const MAX_SHOTS: u64 = 4096;
pub const HISTOGRAM_SIZE: usize = 1 << MAX_QUBITS;
pub const HEAP_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    QubitLimit,
    GateLimit,
    ShotLimit,
    Invalid(ValidationError),
    Execution(ExecuteError),
}

/// Fixed-size result: only the first `bins` entries describe the circuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Result {
    pub counts: [u32; HISTOGRAM_SIZE],
    pub bins: usize,
    pub shots_completed: u32,
    pub buffer_bytes: usize,
}

/// Restrict resources before allocation, then execute the existing core.
/// Cancellation/deadline errors carry no partial histogram.
pub fn execute(
    qubits: u64,
    gates: &[Gate],
    shots: u64,
    seed: u64,
    control: &mut impl Control,
) -> core::result::Result<Result, Error> {
    if !(1..=MAX_QUBITS).contains(&qubits) {
        return Err(Error::QubitLimit);
    }
    if gates.len() > MAX_GATES {
        return Err(Error::GateLimit);
    }
    if !(1..=MAX_SHOTS).contains(&shots) {
        return Err(Error::ShotLimit);
    }
    // Validate gate indices before allocating a copy of untrusted inputs.
    for gate in gates {
        if gate.target() >= qubits {
            return Err(Error::Invalid(ValidationError {
                field: "target",
                resource_limit: false,
            }));
        }
        if let Gate::Cx { control, target } = *gate {
            if control >= qubits || control == target {
                return Err(Error::Invalid(ValidationError {
                    field: "control",
                    resource_limit: false,
                }));
            }
        }
    }
    control
        .checkpoint(0)
        .map_err(|error| Error::Execution(ExecuteError::Aborted(error)))?;
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(gates.len())
        .map_err(|_| Error::Execution(ExecuteError::ResourceExhausted))?;
    owned.extend_from_slice(gates);
    let job = qsim_core::ValidatedJob::new(qubits, owned, shots, seed).map_err(Error::Invalid)?;
    let result = qsim_core::execute(&job, control).map_err(Error::Execution)?;
    // This bound is implied by the checked qubit range; retain it at the copy
    // boundary so a future core change cannot silently enlarge this adapter.
    if result.counts.len() > HISTOGRAM_SIZE || result.buffer_bytes > HEAP_BYTES {
        return Err(Error::Execution(ExecuteError::ResourceExhausted));
    }
    let mut fixed = Result {
        counts: [0; HISTOGRAM_SIZE],
        bins: result.counts.len(),
        shots_completed: result.shots_completed,
        buffer_bytes: result.buffer_bytes,
    };
    fixed.counts[..fixed.bins].copy_from_slice(&result.counts);
    Ok(fixed)
}
