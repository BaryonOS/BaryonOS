//! Allocator-free request codec for the proposed userspace quantum service v1.
//!
//! The exact 256-byte payload carries no caller identity or authority. The
//! future service must bind the authenticated IPC envelope to each operation.
//! This crate neither executes jobs nor implements cancellation or replies.
#![no_std]
#![forbid(unsafe_code)]

use core::num::NonZeroU64;

pub const PAYLOAD_BYTES: usize = 256;
pub const MAX_GATES: usize = 64;
pub const MAX_QUBITS: u8 = 6;
pub const MAX_SHOTS: u32 = 4096;
pub const MAX_WORK_BUDGET: u64 = 1_048_576;
pub const RESULT_PAGES: u8 = 4;
const MAGIC: &[u8; 4] = b"QJ01";
const VERSION: u16 = 1;
const GATES_OFFSET: usize = 64;

/// Local validation errors. These are not a reply or syscall ABI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Length { actual: usize },
    Magic,
    Version { observed: u16 },
    Operation { observed: u8 },
    ZeroJobId,
    ZeroRequestId,
    ReservedBytes,
    UnexpectedFields,
    Qubits,
    Shots,
    WorkBudget,
    GateCount,
    ResultPage,
    UnusedGateBytes,
    GateOperation { index: u8, observed: u8 },
    GateTarget { index: u8 },
    GateControl { index: u8 },
    EqualControlAndTarget { index: u8 },
    SingleQubitControl { index: u8 },
}

/// Correlation IDs only. Possession of these IDs grants no job authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Identifiers {
    job_id: NonZeroU64,
    request_id: NonZeroU64,
}

impl Identifiers {
    pub fn new(job_id: u64, request_id: u64) -> Result<Self, Error> {
        Ok(Self {
            job_id: NonZeroU64::new(job_id).ok_or(Error::ZeroJobId)?,
            request_id: NonZeroU64::new(request_id).ok_or(Error::ZeroRequestId)?,
        })
    }

    pub const fn job_id(self) -> u64 {
        self.job_id.get()
    }

    pub const fn request_id(self) -> u64 {
        self.request_id.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gate {
    H { target: u8 },
    X { target: u8 },
    Z { target: u8 },
    S { target: u8 },
    Cx { target: u8, control: u8 },
}

impl Gate {
    fn fields(self) -> (u8, u8, u8) {
        match self {
            Self::H { target } => (1, target, 0),
            Self::X { target } => (2, target, 0),
            Self::Z { target } => (3, target, 0),
            Self::S { target } => (4, target, 0),
            Self::Cx { target, control } => (5, target, control),
        }
    }

    fn validate(self, qubits: u8, index: u8) -> Result<(), Error> {
        let (_, target, control) = self.fields();
        // Validate the wire-sized indices before any backend conversion.
        if target >= qubits {
            return Err(Error::GateTarget { index });
        }
        if matches!(self, Self::Cx { .. }) {
            if control >= qubits {
                return Err(Error::GateControl { index });
            }
            if control == target {
                return Err(Error::EqualControlAndTarget { index });
            }
        }
        Ok(())
    }
}

fn validate_limits(qubits: u8, shots: u32, budget: u64) -> Result<(), Error> {
    if !(1..=MAX_QUBITS).contains(&qubits) {
        return Err(Error::Qubits);
    }
    if !(1..=MAX_SHOTS).contains(&shots) {
        return Err(Error::Shots);
    }
    if !(1..=MAX_WORK_BUDGET).contains(&budget) {
        return Err(Error::WorkBudget);
    }
    Ok(())
}

/// Validated bounded request, independent of the in-memory/wire enum layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Submit {
    identifiers: Identifiers,
    qubits: u8,
    seed: u64,
    shots: u32,
    work_budget: u64,
    gate_count: u8,
    gates: [Gate; MAX_GATES],
}

impl Submit {
    /// Validate before copying. The work budget is a request, not enforcement.
    pub fn new(
        identifiers: Identifiers,
        qubits: u8,
        seed: u64,
        shots: u32,
        work_budget: u64,
        gates: &[Gate],
    ) -> Result<Self, Error> {
        validate_limits(qubits, shots, work_budget)?;
        if gates.len() > MAX_GATES {
            return Err(Error::GateCount);
        }
        for (index, gate) in gates.iter().enumerate() {
            // The gate-count check bounds index to 0..63 before narrowing.
            gate.validate(qubits, index as u8)?;
        }
        let mut fixed = [Gate::H { target: 0 }; MAX_GATES];
        fixed[..gates.len()].copy_from_slice(gates);
        Ok(Self {
            identifiers,
            qubits,
            seed,
            shots,
            work_budget,
            gate_count: gates.len() as u8,
            gates: fixed,
        })
    }

    pub const fn identifiers(&self) -> Identifiers {
        self.identifiers
    }

    pub const fn qubits(&self) -> u8 {
        self.qubits
    }

    pub const fn seed(&self) -> u64 {
        self.seed
    }

    pub const fn shots(&self) -> u32 {
        self.shots
    }

    pub const fn work_budget(&self) -> u64 {
        self.work_budget
    }

    pub fn gates(&self) -> &[Gate] {
        &self.gates[..usize::from(self.gate_count)]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResultPage {
    identifiers: Identifiers,
    page: u8,
}

impl ResultPage {
    pub fn new(identifiers: Identifiers, page: u8) -> Result<Self, Error> {
        if page >= RESULT_PAGES {
            return Err(Error::ResultPage);
        }
        Ok(Self { identifiers, page })
    }

    pub const fn identifiers(self) -> Identifiers {
        self.identifiers
    }

    pub const fn page(self) -> u8 {
        self.page
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "A bounded request owns all 64 gates without allocation or borrowed untrusted input"
)]
pub enum Request {
    Submit(Submit),
    Cancel(Identifiers),
    Status(Identifiers),
    ResultPage(ResultPage),
}

impl Request {
    pub const fn identifiers(&self) -> Identifiers {
        match self {
            Self::Submit(submit) => submit.identifiers,
            Self::Cancel(ids) | Self::Status(ids) => *ids,
            Self::ResultPage(page) => page.identifiers,
        }
    }
}

/// Encode a validated request canonically, including every reserved zero byte.
pub fn encode(request: &Request) -> [u8; PAYLOAD_BYTES] {
    let mut bytes = [0; PAYLOAD_BYTES];
    bytes[..4].copy_from_slice(MAGIC);
    bytes[4..6].copy_from_slice(&VERSION.to_le_bytes());
    let ids = request.identifiers();
    bytes[8..16].copy_from_slice(&ids.job_id().to_le_bytes());
    bytes[16..24].copy_from_slice(&ids.request_id().to_le_bytes());
    match request {
        Request::Submit(submit) => {
            bytes[6] = 1;
            bytes[7] = submit.qubits;
            bytes[24..32].copy_from_slice(&submit.seed.to_le_bytes());
            bytes[32..36].copy_from_slice(&submit.shots.to_le_bytes());
            bytes[36..38].copy_from_slice(&u16::from(submit.gate_count).to_le_bytes());
            bytes[40..48].copy_from_slice(&submit.work_budget.to_le_bytes());
            for (index, gate) in submit.gates().iter().enumerate() {
                let (op, target, control) = gate.fields();
                let at = GATES_OFFSET + index * 3;
                bytes[at..at + 3].copy_from_slice(&[op, target, control]);
            }
        }
        Request::Cancel(_) => bytes[6] = 2,
        Request::Status(_) => bytes[6] = 3,
        Request::ResultPage(page) => {
            bytes[6] = 4;
            bytes[48] = page.page;
        }
    }
    bytes
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes([
        bytes[at],
        bytes[at + 1],
        bytes[at + 2],
        bytes[at + 3],
        bytes[at + 4],
        bytes[at + 5],
        bytes[at + 6],
        bytes[at + 7],
    ])
}

/// Decode exactly one payload. Reject truncation, extension and ignored bytes.
pub fn decode(bytes: &[u8]) -> Result<Request, Error> {
    if bytes.len() != PAYLOAD_BYTES {
        return Err(Error::Length {
            actual: bytes.len(),
        });
    }
    if &bytes[..4] != MAGIC {
        return Err(Error::Magic);
    }
    let version = u16_at(bytes, 4);
    if version != VERSION {
        return Err(Error::Version { observed: version });
    }
    let operation = bytes[6];
    if !(1..=4).contains(&operation) {
        return Err(Error::Operation {
            observed: operation,
        });
    }
    let ids = Identifiers::new(u64_at(bytes, 8), u64_at(bytes, 16))?;
    if bytes[38..40]
        .iter()
        .chain(&bytes[49..64])
        .any(|byte| *byte != 0)
    {
        return Err(Error::ReservedBytes);
    }
    if operation != 1 {
        if bytes[7] != 0
            || bytes[24..48]
                .iter()
                .chain(&bytes[64..])
                .any(|byte| *byte != 0)
        {
            return Err(Error::UnexpectedFields);
        }
        if operation == 4 {
            return ResultPage::new(ids, bytes[48]).map(Request::ResultPage);
        }
        if bytes[48] != 0 {
            return Err(Error::UnexpectedFields);
        }
        return Ok(if operation == 2 {
            Request::Cancel(ids)
        } else {
            Request::Status(ids)
        });
    }

    let qubits = bytes[7];
    let shots = u32_at(bytes, 32);
    let budget = u64_at(bytes, 40);
    validate_limits(qubits, shots, budget)?;
    if bytes[48] != 0 {
        return Err(Error::UnexpectedFields);
    }
    let count = u16_at(bytes, 36);
    if count > MAX_GATES as u16 {
        return Err(Error::GateCount);
    }
    let count = usize::from(count);
    if bytes[GATES_OFFSET + count * 3..]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err(Error::UnusedGateBytes);
    }
    let mut gates = [Gate::H { target: 0 }; MAX_GATES];
    for (index, gate) in gates[..count].iter_mut().enumerate() {
        let at = GATES_OFFSET + index * 3;
        let (op, target, control) = (bytes[at], bytes[at + 1], bytes[at + 2]);
        let index = index as u8;
        if (1..=4).contains(&op) && control != 0 {
            return Err(Error::SingleQubitControl { index });
        }
        *gate = match op {
            1 => Gate::H { target },
            2 => Gate::X { target },
            3 => Gate::Z { target },
            4 => Gate::S { target },
            5 => Gate::Cx { target, control },
            _ => {
                return Err(Error::GateOperation {
                    index,
                    observed: op,
                })
            }
        };
        gate.validate(qubits, index)?;
    }
    Submit::new(
        ids,
        qubits,
        u64_at(bytes, 24),
        shots,
        budget,
        &gates[..count],
    )
    .map(Request::Submit)
}
