//! Host-only negative controls over the actual observer module. This shim
//! replaces hardware/memory ownership operations and proves no native MMU fact.
use std::{
    cell::RefCell,
    collections::BTreeMap,
    panic::{catch_unwind, AssertUnwindSafe},
};

const DATA: u64 = 0x4001_0000;
const HEAP: u64 = 0x6000_0000;
const MAX: usize = 16;
thread_local! {
    static MEMORY: RefCell<BTreeMap<u64,u64>> = RefCell::new(BTreeMap::new());
    static LOG: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Ready,
    Running,
    Dying,
    Dead,
}
struct AddressSpace {
    root: u64,
    owned: usize,
    writable: bool,
}
impl AddressSpace {
    fn owned_table_count(&self) -> usize {
        self.owned
    }
    fn permissions(&self, _: u64) -> Option<(u64, bool, bool, bool)> {
        Some((self.root, self.writable, false, true))
    }
}
struct Process {
    pid: u64,
    pattern: u64,
    status: Status,
    paging: Option<AddressSpace>,
    count: usize,
    kernel_frames: [u64; 4],
    selected: u64,
    report_base: u64,
}
impl Process {
    fn physical(&self, address: u64) -> u64 {
        if (DATA..DATA + 112).contains(&address) {
            self.report_base + address - DATA
        } else {
            self.report_base * 32 + address - HEAP
        }
    }
}
struct Event {
    pid: u64,
    vector: u64,
    code: u64,
}
struct State {
    amount: usize,
    fault: bool,
    exited: bool,
    unexpected: bool,
    processes: [Process; 2],
    free_baseline: usize,
    free: usize,
    ticks: u64,
    timer_interrupts: u64,
    syscall_calls: u64,
    kernel: AddressSpace,
    current: usize,
    last_event: Event,
    select_calls: usize,
    audit_calls: std::cell::Cell<usize>,
    cleanup_calls: usize,
    leak: bool,
}
impl State {
    fn free_count(&self) -> usize {
        self.free
    }
    fn select(&mut self) -> *const u64 {
        self.select_calls += 1;
        0x8000 as *const u64
    }
    fn audit(&self) {
        self.audit_calls.set(self.audit_calls.get() + 1);
    }
    fn cleanup(&mut self, slot: usize) {
        let p = &mut self.processes[slot];
        assert_eq!(p.status, Status::Dying);
        self.free += p.count + p.kernel_frames.len() + p.paging.take().unwrap().owned;
        if self.leak {
            self.free -= 1;
        }
        p.count = 0;
        p.status = Status::Dead;
        self.cleanup_calls += 1;
    }
}
fn read_word(address: u64) -> u64 {
    MEMORY.with(|m| *m.borrow().get(&address).expect("owned report word"))
}
fn print_number(label: &[u8], value: u64) {
    serial::write(label);
    serial::write(value.to_string().as_bytes());
    serial::write(b"\n");
}
mod arch {
    pub fn activate(_: u64) {}
    pub fn service_stack() -> u64 {
        0x9000
    }
    pub fn process_stack(stack: u64) {
        assert_eq!(stack, service_stack());
    }
}
#[derive(Debug)]
struct Exit(u8);
mod serial {
    pub fn write(bytes: &[u8]) {
        super::LOG.with(|log| log.borrow_mut().extend_from_slice(bytes));
    }
    pub fn exit(code: u8) -> ! {
        std::panic::panic_any(super::Exit(code));
    }
}
#[path = "../../../kernel/src/m2/m3/qsim.rs"]
mod qsim;

fn report(state: &State, slot: usize, words: [u64; 14]) {
    MEMORY.with(|memory| {
        for (i, word) in words.into_iter().enumerate() {
            memory
                .borrow_mut()
                .insert(state.processes[slot].report_base + i as u64 * 8, word);
        }
    });
}
fn complete(state: &State, slot: usize) -> [u64; 14] {
    [
        0,
        253,
        state.processes[slot].pattern,
        0,
        1,
        127,
        2082,
        2014,
        4096,
        7216,
        448,
        4096,
        65536,
        1,
    ]
}
fn fresh() -> State {
    MEMORY.with(|m| m.borrow_mut().clear());
    LOG.with(|l| l.borrow_mut().clear());
    let s = State {
        amount: 2,
        fault: false,
        exited: false,
        unexpected: false,
        processes: std::array::from_fn(|slot| Process {
            pid: slot as u64 + 1,
            pattern: 0x4141000000000000 | (slot as u64 + 1),
            status: Status::Ready,
            paging: Some(AddressSpace {
                root: 0x1000 * (slot as u64 + 1),
                owned: 3,
                writable: true,
            }),
            count: 38,
            kernel_frames: [0; 4],
            selected: 3,
            report_base: 0x10000 * (slot as u64 + 1),
        }),
        free_baseline: 0,
        free: 1000,
        ticks: 1,
        timer_interrupts: 1,
        syscall_calls: 506,
        kernel: AddressSpace {
            root: 0x9000,
            owned: 0,
            writable: true,
        },
        current: 0,
        last_event: Event {
            pid: 0,
            vector: 0,
            code: 0,
        },
        select_calls: 0,
        audit_calls: std::cell::Cell::new(0),
        cleanup_calls: 0,
        leak: false,
    };
    for slot in 0..2 {
        let mut words = [0; 14];
        words[2] = s.processes[slot].pattern;
        report(&s, slot, words);
    }
    s
}
fn prepared() -> State {
    let mut s = fresh();
    qsim::manifest(&mut s);
    for slot in 0..2 {
        report(&s, slot, complete(&s, slot));
    }
    s
}
fn expect_exit(action: impl FnOnce(), code: u8) {
    let panic = catch_unwind(AssertUnwindSafe(action)).unwrap_err();
    assert_eq!(panic.downcast_ref::<Exit>().expect("observer exit").0, code);
}
fn no_pass() {
    LOG.with(|l| assert!(!String::from_utf8_lossy(&l.borrow()).contains("QSIM:PASS\n")));
}

#[test]
fn successful_pair_checks_reports_audits_and_frees_both_processes() {
    let mut s = prepared();
    expect_exit(
        || {
            qsim::poll(&mut s);
        },
        0x10,
    );
    assert_eq!(s.audit_calls.get(), 1);
    assert_eq!(s.cleanup_calls, 2);
    assert_eq!(s.free_count(), s.free_baseline);
    assert_eq!(s.current, MAX);
    assert!(s.processes.iter().all(|p| p.status == Status::Dead));
    LOG.with(|l| {
        assert!(String::from_utf8_lossy(&l.borrow())
            .ends_with("QSIM:CLEANUP:PASS\nQSIM:PASS\nM1:PASS\n"))
    });
}
#[test]
fn pending_peer_and_absent_timer_never_accept() {
    let mut s = fresh();
    qsim::manifest(&mut s);
    report(&s, 0, complete(&s, 0));
    assert!(qsim::poll(&mut s).is_some());
    assert_eq!(s.cleanup_calls, 0);
    no_pass();
    report(&s, 1, complete(&s, 1));
    s.timer_interrupts = 0;
    assert!(qsim::poll(&mut s).is_some());
    assert_eq!(s.select_calls, 2);
    no_pass();
}
#[test]
fn every_report_boundary_rejects_a_false_success_for_either_process() {
    for slot in 0..2 {
        for word in 0..14 {
            let mut s = prepared();
            let mut words = complete(&s, slot);
            words[word] = if word == 1 { 0 } else { words[word] ^ 1 };
            // status=0 is a pending report, not a malformed completed report.
            if word == 4 {
                words[word] = 0x101;
            }
            report(&s, slot, words);
            expect_exit(
                || {
                    qsim::poll(&mut s);
                },
                0x11,
            );
            assert_eq!(s.cleanup_calls, 0);
            no_pass();
        }
    }
}
#[test]
fn faults_exit_unexpected_and_deadlines_fail_closed() {
    for case in 0..6 {
        let mut s = prepared();
        match case {
            0 => s.fault = true,
            1 => s.exited = true,
            2 => s.unexpected = true,
            3 => s.ticks = 2000,
            4 => s.processes[1].status = Status::Dead,
            _ => s.processes[0].selected = 0,
        };
        expect_exit(
            || {
                qsim::poll(&mut s);
            },
            0x11,
        );
        no_pass();
    }
}
#[test]
fn initial_stale_results_shared_roots_and_invalid_counts_are_rejected() {
    for case in 0..6 {
        let mut s = fresh();
        match case {
            0 => report(&s, 1, complete(&s, 1)),
            1 => {
                s.processes[1].paging.as_mut().unwrap().root =
                    s.processes[0].paging.as_ref().unwrap().root
            }
            2 => s.amount = 0,
            3 => s.amount = usize::MAX,
            4 => s.processes[0].paging.as_mut().unwrap().writable = false,
            _ => s.processes[0].status = Status::Running,
        };
        expect_exit(|| qsim::manifest(&mut s), 0x11);
        no_pass();
    }
}
#[test]
fn accounting_drift_and_cleanup_leak_cannot_report_success() {
    let mut s = prepared();
    s.free += 1;
    expect_exit(
        || {
            qsim::poll(&mut s);
        },
        0x11,
    );
    assert_eq!(s.cleanup_calls, 0);
    no_pass();
    let mut s = prepared();
    s.leak = true;
    expect_exit(
        || {
            qsim::poll(&mut s);
        },
        0x11,
    );
    assert_eq!(s.cleanup_calls, 2);
    no_pass();
}
