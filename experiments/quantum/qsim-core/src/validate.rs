use crate::Gate;
use alloc::vec::Vec;

pub const MAX_QUBITS: u64 = 12;
pub const MAX_GATES: usize = 4096;
pub const MAX_SHOTS: u64 = 65_536;
pub const MAX_WORK: u64 = 20_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationError {
    pub field: &'static str,
    pub resource_limit: bool,
}

impl ValidationError {
    fn invalid(field: &'static str) -> Self {
        Self {
            field,
            resource_limit: false,
        }
    }
    fn resource(field: &'static str) -> Self {
        Self {
            field,
            resource_limit: true,
        }
    }
}

#[derive(Debug)]
pub struct ValidatedJob {
    qubits: u8,
    gates: Vec<Gate>,
    shots: u32,
    seed: u64,
    work_budget: u64,
}

impl ValidatedJob {
    pub fn new(
        qubits: u64,
        gates: Vec<Gate>,
        shots: u64,
        seed: u64,
    ) -> Result<Self, ValidationError> {
        if !(1..=MAX_QUBITS).contains(&qubits) {
            return Err(ValidationError::invalid("qubits"));
        }
        if gates.len() > MAX_GATES || gates.capacity() > MAX_GATES {
            return Err(ValidationError::resource("gates"));
        }
        if !(1..=MAX_SHOTS).contains(&shots) {
            return Err(ValidationError::invalid("shots"));
        }
        for gate in &gates {
            if gate.target() >= qubits {
                return Err(ValidationError::invalid("target"));
            }
            if let Gate::Cx { control, target } = *gate {
                if control >= qubits || control == target {
                    return Err(ValidationError::invalid("control"));
                }
            }
        }
        let states = 1_u64
            .checked_shl(qubits as u32)
            .ok_or(ValidationError::resource("work_budget"))?;
        let gate_work = (gates.len() as u64)
            .checked_add(1)
            .and_then(|n| n.checked_mul(states));
        let sampling_work = qubits.checked_add(1).and_then(|n| shots.checked_mul(n));
        let work_budget = gate_work
            .and_then(|a| sampling_work.and_then(|b| a.checked_add(b)))
            .filter(|&n| n <= MAX_WORK)
            .ok_or(ValidationError::resource("work_budget"))?;
        Ok(Self {
            qubits: qubits as u8,
            gates,
            shots: shots as u32,
            seed,
            work_budget,
        })
    }
    pub fn qubits(&self) -> u8 {
        self.qubits
    }
    pub fn gates(&self) -> &Vec<Gate> {
        &self.gates
    }
    pub fn shots(&self) -> u32 {
        self.shots
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn work_budget(&self) -> u64 {
        self.work_budget
    }
}
