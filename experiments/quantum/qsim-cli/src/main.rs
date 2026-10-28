#![forbid(unsafe_code)]

use qsim_core::{execute, Abort, Control, ExecuteError, Gate, JobResult, ValidatedJob, MAX_GATES};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

const MAX_REQUEST: usize = 262_144;
const SCHEMA: &str = "qsim-v0";
const BACKEND: &str = "local-statevector-v0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields, remote = "Self")]
struct Request {
    schema: String,
    job_id: String,
    qubits: u64,
    gates: BoundedGates,
    shots: u64,
    seed: String,
}

impl<'de> Deserialize<'de> for Request {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RequestVisitor;
        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = Request;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a request object")
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                // Use the derived field checks only after requiring an object.
                Request::deserialize(de::value::MapAccessDeserializer::new(map))
            }
        }
        deserializer.deserialize_map(RequestVisitor)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, remote = "Self")]
struct GateRequest {
    op: Operation,
    target: u64,
    #[serde(default, deserialize_with = "present_control")]
    control: Option<u64>,
}

impl<'de> Deserialize<'de> for GateRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct GateVisitor;
        impl<'de> Visitor<'de> for GateVisitor {
            type Value = GateRequest;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a gate object")
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                GateRequest::deserialize(de::value::MapAccessDeserializer::new(map))
            }
        }
        deserializer.deserialize_map(GateVisitor)
    }
}

fn present_control<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u64>, D::Error> {
    u64::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Operation {
    H,
    X,
    Z,
    S,
    Cx,
}

struct BoundedGates(Vec<Gate>);
impl<'de> Deserialize<'de> for BoundedGates {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct GatesVisitor;
        impl<'de> Visitor<'de> for GatesVisitor {
            type Value = BoundedGates;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("at most 4096 typed gates")
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut gates = Vec::new();
                // One bounded, fallible reservation; never grow from an untrusted size hint.
                gates
                    .try_reserve_exact(MAX_GATES)
                    .map_err(|_| de::Error::custom("gate reservation failed"))?;
                while let Some(input) = sequence.next_element::<GateRequest>()? {
                    if gates.len() == MAX_GATES {
                        return Err(de::Error::custom("too many gates"));
                    }
                    let gate = match (input.op, input.control) {
                        (Operation::H, None) => Gate::H {
                            target: input.target,
                        },
                        (Operation::X, None) => Gate::X {
                            target: input.target,
                        },
                        (Operation::Z, None) => Gate::Z {
                            target: input.target,
                        },
                        (Operation::S, None) => Gate::S {
                            target: input.target,
                        },
                        (Operation::Cx, Some(control)) => Gate::Cx {
                            control,
                            target: input.target,
                        },
                        _ => return Err(de::Error::custom("invalid control field")),
                    };
                    gates.push(gate);
                }
                Ok(BoundedGates(gates))
            }
        }
        deserializer.deserialize_seq(GatesVisitor)
    }
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    field: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    schema: &'static str,
    job_id: Option<&'a str>,
    backend: &'static str,
    status: &'static str,
    error: ErrorDetail,
}

#[derive(Serialize)]
struct SuccessResponse<'a> {
    schema: &'static str,
    job_id: &'a str,
    backend: &'static str,
    status: &'static str,
    qubits: u8,
    shots_completed: u32,
    counts: Counts<'a>,
    seed: &'a str,
}

struct Counts<'a> {
    result: &'a JobResult,
    qubits: u8,
}
impl Serialize for Counts<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        for (index, &count) in self.result.counts.iter().enumerate() {
            if count == 0 {
                continue;
            }
            let mut bits = [b'0'; 12];
            let n = usize::from(self.qubits);
            for (position, bit) in bits[..n].iter_mut().enumerate() {
                if index & (1 << (n - position - 1)) != 0 {
                    *bit = b'1';
                }
            }
            // Only ASCII 0/1 was written, and validated qubits is 1..=12.
            let key = std::str::from_utf8(&bits[..n]).map_err(serde::ser::Error::custom)?;
            map.serialize_entry(key, &count)?;
        }
        map.end()
    }
}

struct Deadline {
    until: Instant,
}
impl Control for Deadline {
    fn checkpoint(&mut self, _: u64) -> Result<(), Abort> {
        if Instant::now() >= self.until {
            Err(Abort::DeadlineExceeded)
        } else {
            Ok(())
        }
    }
}

fn write_json(value: &impl Serialize) -> bool {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer(&mut output, value).is_ok()
        && output.write_all(b"\n").is_ok()
        && output.flush().is_ok()
}

fn fail(id: Option<&str>, code: &'static str, field: &'static str, exit: u8) -> u8 {
    let response = ErrorResponse {
        schema: SCHEMA,
        job_id: id,
        backend: BACKEND,
        status: "error",
        error: ErrorDetail { code, field },
    };
    if write_json(&response) {
        exit
    } else {
        5
    }
}

fn run() -> u8 {
    let mut input = Vec::new();
    if input.try_reserve_exact(MAX_REQUEST + 1).is_err() {
        return fail(None, "RESOURCE_EXHAUSTED", "request", 3);
    }
    let mut limited = io::stdin().lock().take((MAX_REQUEST + 1) as u64);
    if limited.read_to_end(&mut input).is_err() {
        return fail(None, "INVALID_REQUEST", "request", 2);
    }
    if input.len() > MAX_REQUEST {
        return fail(None, "RESOURCE_EXHAUSTED", "request", 3);
    }
    // Strong types preserve duplicate-field detection. Never deserialize input via Value.
    let request: Request = match serde_json::from_slice(&input) {
        Ok(request) => request,
        Err(error) => {
            // Only our exact visitor-error prefix selects a resource failure.
            // Upstream unknown-field/variant errors have their own fixed prefix;
            // arbitrary user text is neither emitted nor used as a field name.
            let description = error.to_string();
            if description.starts_with("gate reservation failed at line ")
                || description.starts_with("too many gates at line ")
            {
                return fail(None, "RESOURCE_EXHAUSTED", "gates", 3);
            }
            return fail(None, "INVALID_REQUEST", "request", 2);
        }
    };
    if request.job_id.is_empty()
        || request.job_id.len() > 64
        || !request
            .job_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return fail(None, "INVALID_REQUEST", "job_id", 2);
    }
    let id = Some(request.job_id.as_str());
    if request.schema != SCHEMA {
        return fail(id, "UNSUPPORTED_SCHEMA", "schema", 2);
    }
    if request.seed.len() != 16
        || !request
            .seed
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return fail(id, "INVALID_REQUEST", "seed", 2);
    }
    let seed = match u64::from_str_radix(&request.seed, 16) {
        Ok(seed) => seed,
        Err(_) => return fail(id, "INVALID_REQUEST", "seed", 2),
    };
    let job = match ValidatedJob::new(request.qubits, request.gates.0, request.shots, seed) {
        Ok(job) => job,
        Err(error) => {
            return fail(
                id,
                if error.resource_limit {
                    "RESOURCE_EXHAUSTED"
                } else {
                    "INVALID_REQUEST"
                },
                error.field,
                if error.resource_limit { 3 } else { 2 },
            )
        }
    };
    let mut control = Deadline {
        until: Instant::now() + Duration::from_secs(10),
    };
    let result = match execute(&job, &mut control) {
        Ok(result) => result,
        Err(error) => {
            let (code, field, exit) = match error {
                ExecuteError::ResourceExhausted => ("RESOURCE_EXHAUSTED", "buffers", 3),
                ExecuteError::NumericalError => ("NUMERICAL_ERROR", "state", 5),
                ExecuteError::Aborted(Abort::Cancelled) => ("CANCELLED", "execution", 4),
                ExecuteError::Aborted(Abort::DeadlineExceeded) => {
                    ("DEADLINE_EXCEEDED", "execution", 4)
                }
            };
            return fail(id, code, field, exit);
        }
    };
    let response = SuccessResponse {
        schema: SCHEMA,
        job_id: &request.job_id,
        backend: BACKEND,
        status: "completed",
        qubits: job.qubits(),
        shots_completed: result.shots_completed,
        counts: Counts {
            result: &result,
            qubits: job.qubits(),
        },
        seed: &request.seed,
    };
    if write_json(&response) {
        0
    } else {
        5
    }
}

fn main() -> std::process::ExitCode {
    run().into()
}
