//! Capability/usercopy boundary for the fixed, polled PS/2 controller profile.
//! No scan decoding, focus, text or application policy belongs here.
use super::*;
use nieuw_os_display_client::wire::{self, SIZE};
use nieuw_os_ps2::{Controller, Fault, Ports, Progress};

struct Hardware;
impl Ports for Hardware {
    fn status(&mut self) -> u8 {
        let value: u8;
        // SAFETY: CPL0, fixed i8042 status port, serialized IF0 broker call.
        // Retain a compiler memory barrier around prevalidated usercopy.
        unsafe {
            asm!("in al, 0x64",out("al")value,options(nostack,preserves_flags));
        }
        value
    }
    fn read_data(&mut self) -> u8 {
        let value: u8;
        // SAFETY: Same fixed port pair; core checked OBF before consuming FIFO.
        unsafe {
            asm!("in al, 0x60",out("al")value,options(nostack,preserves_flags));
        }
        value
    }
    fn command(&mut self, value: u8) {
        // SAFETY: Fixed control port; core checked IBF and bounds every command.
        unsafe {
            asm!("out 0x64, al",in("al")value,options(nostack,preserves_flags));
        }
    }
    fn data(&mut self, value: u8) {
        // SAFETY: Fixed data port; only the core's closed initialization plan
        // supplies this byte. No user-provided command or data byte reaches it.
        unsafe {
            asm!("out 0x60, al",in("al")value,options(nostack,preserves_flags));
        }
    }
}
struct Runtime {
    prepared: bool,
    owner: Option<(u64, u64)>,
    controller: Controller,
    started: u64,
    ready: u64,
    last_ready: u64,
    samples: u64,
    denied: u64,
    discarded: u64,
}
static mut INPUT: Runtime = Runtime {
    prepared: false,
    owner: None,
    controller: Controller::new(),
    started: 0,
    ready: 0,
    last_ready: 0,
    samples: 0,
    denied: 0,
    discarded: 0,
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: One CPU/IF0, serialized syscall lifecycle. No references escape
    // into scheduling and no caller buffer or pointer is retained here.
    unsafe { &mut *ptr::addr_of_mut!(INPUT) }
}
/// Does not touch ports or fabricate readiness. Reset starts only on authority.
pub fn prepare() {
    runtime().prepared = true;
}
/// Process death/full principal retirement; not delayed single-grant teardown.
pub fn detach(pid: u64) {
    let r = runtime();
    if r.owner.is_some_and(|o| o.0 == pid) {
        r.owner = None;
        r.controller.detach();
    }
}
#[cfg(any(
    feature = "native-display-probe",
    feature = "native-system-materiaal-probe"
))]
pub fn counts() -> [u64; 5] {
    let r = runtime();
    [r.started, r.ready, r.samples, r.denied, r.discarded]
}
fn error(f: Fault) -> i64 {
    match f {
        Fault::Busy => -8,
        Fault::WrongTicket => -1,
        Fault::CounterExhausted => -7,
        Fault::NeedsReset => -14,
        _ => -15,
    }
}
fn call(s: &mut State, slot: usize, number: u64, a: [u64; 6]) -> Result<u64, i64> {
    let pid = s.processes[slot].pid;
    let key = m4::engine().check_input(pid, a[0]).map_err(|e| e.code())?;
    if !runtime().prepared {
        return Err(-14);
    }
    match number {
        0x82 => {
            if a[1..].iter().any(|n| *n != 0) {
                return Err(-1);
            }
            let r = runtime();
            // A new globally sole object proves the old object/grant retired.
            // It may not inherit old pending commands or a previous Ready.
            if r.owner != Some((pid, key)) {
                r.controller.detach();
                r.owner = Some((pid, key));
            }
            let ticket = r.controller.begin_reset(s.ticks).map_err(error)?;
            r.started = r.started.saturating_add(1);
            Ok(ticket)
        }
        0x83 | 0x84 => {
            let (pointer, length) = if number == 0x83 {
                if a[1] == 0 || a[4] != 0 || a[5] != 0 {
                    return Err(-1);
                }
                (a[2], a[3])
            } else {
                if a[3..].iter().any(|n| *n != 0) {
                    return Err(-1);
                }
                (a[1], a[2])
            };
            if length != SIZE as u64 {
                return Err(-1);
            }
            if runtime().owner != Some((pid, key)) {
                return Err(-5);
            }
            // All writable pages/owner/effective rights validated before the
            // first status read, FIFO consumption or reset-progress command.
            let output = m4::plan(s, slot, pointer, SIZE as u64, true).map_err(|_| -3)?;
            let r = runtime();
            let mut hardware = Hardware;
            let bytes = if number == 0x83 {
                let report = match r
                    .controller
                    .poll_reset(a[1], s.ticks, &mut hardware)
                    .map_err(error)?
                {
                    Progress::Pending => return Err(-8),
                    Progress::Ready(report) => report,
                };
                if r.last_ready != report.ticket {
                    r.last_ready = report.ticket;
                    r.ready = r.ready.saturating_add(1);
                    r.discarded = r.discarded.saturating_add(u64::from(report.discarded));
                }
                wire::Reset {
                    ticket: report.ticket,
                    epoch: report.epoch,
                    discarded: report.discarded,
                    operations: report.operations,
                }
                .encode()
                .map_err(|_| -15)?
            } else {
                let sample = r
                    .controller
                    .read_sample(&mut hardware)
                    .map_err(error)?
                    .ok_or(-8)?;
                r.samples = r.samples.saturating_add(1);
                wire::Sample {
                    epoch: sample.epoch,
                    sequence: sample.sequence,
                    status: sample.status,
                    byte: sample.byte,
                }
                .encode()
                .map_err(|_| -15)?
            };
            let mut out = [0; 320];
            out[..SIZE].copy_from_slice(&bytes);
            m4::write_plan(&output, &out);
            Ok(SIZE as u64)
        }
        _ => Err(-2),
    }
}
pub fn syscall(s: &mut State, slot: usize, number: u64, args: [u64; 6]) -> (i64, u64) {
    match call(s, slot, number, args) {
        Ok(value) => (0, value),
        Err(error) => {
            if error != -8 {
                let r = runtime();
                r.denied = r.denied.saturating_add(1);
            }
            (error, 0)
        }
    }
}
