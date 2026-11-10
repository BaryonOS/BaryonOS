//! Opt-in read-only comparison on the actual four-process system manifest.
//! Invoke after State::setup and before timer_init, with the existing IF=0
//! single-CPU serialization. This is neither a GUI test nor a user workload.
use super::{clock, print_number, serial};
use capability_core::Engine;
use core::hint::black_box;

const WARMUP: u64 = 4;
const ROUNDS: usize = 9;
const BATCH: u64 = 16;
type Digest = fn(&Engine) -> u64;

#[derive(Clone, Copy)]
struct Measurement {
    count: u64,
    sum: u64,
    min: u64,
    max: u64,
}
impl Measurement {
    const EMPTY: Self = Self {
        count: 0,
        sum: 0,
        min: u64::MAX,
        max: 0,
    };
    fn record(&mut self, ticks: u64) {
        self.count = self
            .count
            .checked_add(BATCH)
            .unwrap_or_else(|| fail(b"COUNT_OVERFLOW"));
        self.sum = self
            .sum
            .checked_add(ticks)
            .unwrap_or_else(|| fail(b"TSC_SUM_OVERFLOW"));
        self.min = self.min.min(ticks);
        self.max = self.max.max(ticks);
    }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"CAPBENCH:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn require(ok: bool, reason: &[u8]) {
    if !ok {
        fail(reason);
    }
}

// No serial I/O or Stats traversal in this interval. Each call observes an
// opaque function pointer and opaque Engine reference; its result is consumed
// through black_box and compared, including every repetition. RDTSC uses the
// existing parent helper. TCG timestamp units are not physical CPU cycles or
// a host-wallclock/real-time guarantee. Interrupts stay disabled throughout.
#[inline(never)]
fn batch(engine: &Engine, digest: Digest, expected: u64) -> u64 {
    let start = clock();
    let mut difference = 0u64;
    for _ in 0..BATCH {
        let observed = black_box(black_box(digest)(black_box(engine)));
        difference |= observed ^ expected;
    }
    let end = clock();
    require(difference == 0, b"REPEATED_HASH");
    require(end > start, b"TSC_PROGRESS");
    end - start
}

pub(super) fn run(engine: &Engine) -> ! {
    let before = engine.stats();
    require(engine.check_invariants(), b"MANIFEST_INVARIANTS");
    require(
        before.processes == 4
            && before.handles == 25
            && before.nodes == 35
            && before.endpoints == 11
            && before.open_endpoints == 11
            && before.queued_messages == 0
            && before.block_objects == 1
            && before.launch_objects == 1
            && before.process_objects == 0
            && before.entropy_objects == 1
            && before.network_objects == 0
            && before.display_objects == 0
            && before.input_objects == 0
            && before.power_objects == 0,
        b"FOUR_PROCESS_MANIFEST",
    );
    serial::write(b"CAPBENCH:MANIFEST:PASS\n");
    let variants: [Digest; 3] = [
        Engine::diagnostic_reference_digest,
        Engine::diagnostic_direct_digest,
        Engine::diagnostic_zero_digest,
    ];
    let expected = before.state_digest;
    for digest in variants {
        require(
            black_box(black_box(digest)(black_box(engine))) == expected,
            b"PRE_HASH",
        );
        for _ in 0..WARMUP {
            require(
                black_box(black_box(digest)(black_box(engine))) == expected,
                b"WARMUP_HASH",
            );
        }
    }
    let mut measurements = [Measurement::EMPTY; 3];
    let mut round_ticks = [[0u64; 3]; ROUNDS];
    let mut execution_order = [[0u64; 3]; ROUNDS];
    for (round, ticks) in round_ticks.iter_mut().enumerate() {
        // ABC, BCA, CAB, ...; totals count actual invocations per variant.
        for position in 0..3 {
            let variant = (round + position) % 3;
            execution_order[round][position] = variant as u64;
            let elapsed = batch(engine, variants[variant], expected);
            ticks[variant] = elapsed;
            measurements[variant].record(elapsed);
        }
    }
    let after = engine.stats();
    require(before == after, b"STATS_CHANGED");
    require(engine.check_invariants(), b"FINAL_INVARIANTS");
    for digest in variants {
        require(
            black_box(black_box(digest)(black_box(engine))) == expected,
            b"POST_HASH",
        );
    }
    print_number(b"CAPBENCH:DIGEST:", expected);
    print_number(b"CAPBENCH:PROCESSES:", before.processes as u64);
    print_number(b"CAPBENCH:HANDLES:", before.handles as u64);
    print_number(b"CAPBENCH:NODES:", before.nodes as u64);
    print_number(b"CAPBENCH:ENDPOINTS:", before.endpoints as u64);
    print_number(b"CAPBENCH:WARMUP:", WARMUP);
    print_number(b"CAPBENCH:ROUNDS:", ROUNDS as u64);
    print_number(b"CAPBENCH:BATCH:", BATCH);
    for (measurement, labels) in measurements.into_iter().zip([
        [
            b"CAPBENCH:REF_COUNT:".as_slice(),
            b"CAPBENCH:REF_TSC_SUM:",
            b"CAPBENCH:REF_TSC_MIN:",
            b"CAPBENCH:REF_TSC_MAX:",
        ],
        [
            b"CAPBENCH:DIRECT_COUNT:",
            b"CAPBENCH:DIRECT_TSC_SUM:",
            b"CAPBENCH:DIRECT_TSC_MIN:",
            b"CAPBENCH:DIRECT_TSC_MAX:",
        ],
        [
            b"CAPBENCH:ZERO_COUNT:",
            b"CAPBENCH:ZERO_TSC_SUM:",
            b"CAPBENCH:ZERO_TSC_MIN:",
            b"CAPBENCH:ZERO_TSC_MAX:",
        ],
    ]) {
        require(measurement.count == ROUNDS as u64 * BATCH, b"COUNT");
        print_number(labels[0], measurement.count);
        print_number(labels[1], measurement.sum);
        print_number(labels[2], measurement.min);
        print_number(labels[3], measurement.max);
    }
    // Record actual variant positions and fixed REF/DIRECT/ZERO batch times.
    for (order, ticks) in execution_order.into_iter().zip(round_ticks) {
        for variant in order {
            print_number(b"CAPBENCH:ROUND_VARIANT:", variant);
        }
        print_number(b"CAPBENCH:ROUND_REF_TSC:", ticks[0]);
        print_number(b"CAPBENCH:ROUND_DIRECT_TSC:", ticks[1]);
        print_number(b"CAPBENCH:ROUND_ZERO_TSC:", ticks[2]);
    }
    serial::write(b"CAPBENCH:PARITY:PASS\nCAPBENCH:STATE:PASS\nCAPBENCH:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
