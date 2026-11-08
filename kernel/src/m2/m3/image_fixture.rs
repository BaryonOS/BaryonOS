//! Opt-in image fixture observer. It never decodes an image, grants authority,
//! allocates/maps memory or changes process-death policy. Snapshot before free;
//! identity is the captured PID and boot slot, never a later slot occupant.

const EXPECTED_PIXELS: u64 = 149_647;
const EXPECTED_HASH: u64 = 548_683_896_226_622_996;
const EXPECTED_BYTES: u64 = 623_808;
const GROWTH_PAGES: u64 = 1024;
const ARENA_BYTES: u64 = GROWTH_PAGES * 4096;
const MAX_TICKS: u64 = 20_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Failure {
    Binding,
    Identity,
    Replay,
    Fault,
    Completion,
    Pixels,
    Negatives,
    Heap,
    Stack,
    Clock,
    Quality,
    Poisoned,
}

#[derive(Clone, Copy, Debug)]
struct Binding {
    pid: u64,
    slot: usize,
    launch_tick: u64,
    initial_mappings: u64,
    elf_bytes: u64,
    elf_fnv: u64,
}

#[derive(Clone, Copy, Debug)]
struct Snapshot {
    done: u64,
    error: u64,
    png: u64,
    jpeg: u64,
    pixels: u64,
    negatives: u64,
    hash: u64,
    bytes: u64,
    growth: u64,
    peak: u64,
    live: u64,
    cancels: u64,
    start: u64,
    end: u64,
    quality_open: u64,
}
impl Snapshot {
    fn from_words(w: [u64; 15]) -> Self {
        Self {
            done: w[0],
            error: w[1],
            png: w[2],
            jpeg: w[3],
            pixels: w[4],
            negatives: w[5],
            hash: w[6],
            bytes: w[7],
            growth: w[8],
            peak: w[9],
            live: w[10],
            cancels: w[11],
            start: w[12],
            end: w[13],
            quality_open: w[14],
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct KernelFacts {
    heap_pages: u64,
    mappings: u64,
    owned_heap_maps: u64,
    stack_pages: u64,
    map_rights_and_owners: bool,
    stack_guards_unmapped: bool,
}

struct Observer {
    binding: Option<Binding>,
    snapshot: Option<(Snapshot, KernelFacts, u64)>,
    failure: Option<Failure>,
    printed: bool,
}
impl Observer {
    const fn new() -> Self {
        Self {
            binding: None,
            snapshot: None,
            failure: None,
            printed: false,
        }
    }
    fn reject<T>(&mut self, error: Failure) -> Result<T, Failure> {
        self.failure = Some(error);
        Err(error)
    }
    fn bind(&mut self, binding: Binding) -> Result<(), Failure> {
        if self.failure.is_some() {
            return Err(Failure::Poisoned);
        }
        if self.binding.is_some() {
            return self.reject(Failure::Replay);
        }
        if binding.pid == 0
            || binding.slot != 4
            || binding.elf_bytes == 0
            || binding.initial_mappings < 32
            || binding.initial_mappings > 2048
        {
            return self.reject(Failure::Binding);
        }
        self.binding = Some(binding);
        Ok(())
    }
    fn matches(&self, pid: u64, slot: usize, live: bool) -> bool {
        live && self.binding.is_some_and(|b| b.pid == pid && b.slot == slot)
    }
    fn observe_exit(
        &mut self,
        pid: u64,
        slot: usize,
        code: u64,
        vector: u64,
        snapshot: Snapshot,
        facts: KernelFacts,
        tick: u64,
    ) -> Result<(), Failure> {
        if self.failure.is_some() {
            return Err(Failure::Poisoned);
        }
        if self.snapshot.is_some() || self.printed {
            return self.reject(Failure::Replay);
        }
        let Some(b) = self.binding else {
            return self.reject(Failure::Binding);
        };
        if b.pid != pid || b.slot != slot {
            return self.reject(Failure::Identity);
        }
        if code != 0 || vector != 128 || snapshot.done != 1 || snapshot.error != 0 {
            return self.reject(Failure::Completion);
        }
        if snapshot.png != 8
            || snapshot.jpeg != 3
            || snapshot.pixels != EXPECTED_PIXELS
            || snapshot.hash != EXPECTED_HASH
            || snapshot.bytes != EXPECTED_BYTES
        {
            return self.reject(Failure::Pixels);
        }
        if snapshot.negatives != 10 || snapshot.cancels != 1 {
            return self.reject(Failure::Negatives);
        }
        if snapshot.growth != GROWTH_PAGES
            || snapshot.peak == 0
            || snapshot.peak > ARENA_BYTES
            || snapshot.live != 0
            || facts.heap_pages != 16 + GROWTH_PAGES
            || facts.owned_heap_maps != facts.heap_pages
            || b.initial_mappings.checked_add(GROWTH_PAGES) != Some(facts.mappings)
            || facts.mappings > 2048
            || !facts.map_rights_and_owners
        {
            return self.reject(Failure::Heap);
        }
        if facts.stack_pages != 16 || !facts.stack_guards_unmapped {
            return self.reject(Failure::Stack);
        }
        if snapshot.start < b.launch_tick
            || snapshot.end < snapshot.start
            || tick < snapshot.end
            || tick < b.launch_tick
            || snapshot.end - snapshot.start >= MAX_TICKS
            || tick - b.launch_tick >= MAX_TICKS
        {
            return self.reject(Failure::Clock);
        }
        if snapshot.quality_open != 0 {
            return self.reject(Failure::Quality);
        }
        self.snapshot = Some((snapshot, facts, tick));
        Ok(())
    }
    fn fault(&mut self) {
        self.failure = Some(Failure::Fault);
    }
    fn finished(&self) -> bool {
        self.failure.is_none() && self.binding.is_some() && self.snapshot.is_some()
    }
    fn take_for_print(
        &mut self,
        now: u64,
    ) -> Result<(Binding, Snapshot, KernelFacts, u64), Failure> {
        if self.failure.is_some() {
            return Err(Failure::Poisoned);
        }
        if self.printed {
            return self.reject(Failure::Replay);
        }
        let (Some(b), Some((s, f, tick))) = (self.binding, self.snapshot) else {
            return self.reject(Failure::Completion);
        };
        if now < tick {
            return self.reject(Failure::Clock);
        }
        self.printed = true;
        Ok((b, s, f, tick))
    }
}

// Parity fingerprint over actual linked bytes, never cryptographic authentication.
fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(not(test))]
mod native {
    use super::super::{
        print_number, read_word, State, Status, DATA, HEAP, PAGE, STACK, STACK_END,
    };
    use super::*;
    use crate::serial;
    use core::ptr;
    use kernel_core::{Kind, Owner};

    const WORDS: [u64; 15] = [
        40, 240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480, 488, 496,
    ];
    static mut OBSERVER: Observer = Observer::new();

    fn observer() -> &'static mut Observer {
        // SAFETY: KERNEL held, IF=0 fixture callbacks. No accessor borrow
        // crosses scheduling, IPC, another accessor or user execution.
        unsafe { &mut *ptr::addr_of_mut!(OBSERVER) }
    }
    fn fail(error: Failure) -> ! {
        let reason: &[u8] = match error {
            Failure::Binding => b"BINDING",
            Failure::Identity => b"IDENTITY",
            Failure::Replay => b"REPLAY",
            Failure::Fault => b"FAULT",
            Failure::Completion => b"COMPLETION",
            Failure::Pixels => b"PIXELS",
            Failure::Negatives => b"NEGATIVES",
            Failure::Heap => b"HEAP",
            Failure::Stack => b"STACK",
            Failure::Clock => b"CLOCK",
            Failure::Quality => b"QUALITY",
            Failure::Poisoned => b"POISONED",
        };
        serial::write(b"SYSTEM:IMAGE_FAIL:");
        serial::write(reason);
        serial::write(b"\n");
        serial::exit(0x11)
    }
    fn require(ok: bool, error: Failure) {
        if !ok {
            fail(error)
        }
    }
    fn startup_pa(s: &State, slot: usize) -> u64 {
        let p = &s.processes[slot];
        let map = p
            .mappings()
            .iter()
            .find(|m| m.va == DATA)
            .unwrap_or_else(|| fail(Failure::Binding));
        require(
            map.writable
                && !map.executable
                && s.frames.owner(map.pa) == Ok(Owner::User(p.pid, Kind::Data)),
            Failure::Binding,
        );
        map.pa
    }
    pub(in super::super) fn bind(s: &State, actual_pid: u64, linked_elf: &[u8]) {
        require(
            linked_elf.len() >= 64 && linked_elf.starts_with(b"\x7fELF"),
            Failure::Binding,
        );
        let slot = s.processes[..s.amount]
            .iter()
            .position(|p| p.pid == actual_pid && p.status != Status::Dead)
            .unwrap_or_else(|| fail(Failure::Binding));
        require(
            slot == 4 && s.processes[slot].heap_pages == 16,
            Failure::Binding,
        );
        let pa = startup_pa(s, slot);
        require(
            read_word(pa + 176) == actual_pid && read_word(pa + 40) == 0,
            Failure::Binding,
        );
        let binding = Binding {
            pid: actual_pid,
            slot,
            launch_tick: s.ticks,
            initial_mappings: s.processes[slot].count as u64,
            elf_bytes: linked_elf.len() as u64,
            elf_fnv: fnv64(linked_elf),
        };
        observer().bind(binding).unwrap_or_else(|e| fail(e));
    }
    pub(in super::super) fn is_probe(s: &State, slot: usize) -> bool {
        slot < s.amount
            && observer().matches(
                s.processes[slot].pid,
                slot,
                s.processes[slot].status != Status::Dead && s.processes[slot].paging.is_some(),
            )
    }
    fn facts(s: &State, slot: usize) -> KernelFacts {
        let p = &s.processes[slot];
        let paging = p.paging.as_ref().unwrap_or_else(|| fail(Failure::Heap));
        let end = HEAP
            .checked_add(
                p.heap_pages
                    .checked_mul(PAGE)
                    .unwrap_or_else(|| fail(Failure::Heap)),
            )
            .unwrap_or_else(|| fail(Failure::Heap));
        let mut heap_maps = 0;
        let mut stack_maps = 0;
        let mut rights = true;
        for m in p.mappings() {
            let kind = if (HEAP..end).contains(&m.va) {
                heap_maps += 1;
                Kind::Data
            } else if (STACK..STACK_END).contains(&m.va) {
                stack_maps += 1;
                Kind::Stack
            } else {
                continue;
            };
            rights &= m.kind == kind
                && m.writable
                && !m.executable
                && s.frames.owner(m.pa) == Ok(Owner::User(p.pid, kind))
                && paging.permissions(m.va) == Some((m.pa, true, false, true));
        }
        KernelFacts {
            heap_pages: p.heap_pages,
            mappings: p.count as u64,
            owned_heap_maps: heap_maps,
            stack_pages: stack_maps,
            map_rights_and_owners: rights,
            stack_guards_unmapped: STACK_END - STACK == 65536
                && paging.leaf(STACK - PAGE) == 0
                && paging.leaf(STACK_END) == 0
                && paging.leaf(end) == 0,
        }
    }
    pub(in super::super) fn observe_exit(s: &State, slot: usize) {
        require(
            is_probe(s, slot) && s.last_event.pid == s.processes[slot].pid,
            Failure::Identity,
        );
        let pa = startup_pa(s, slot);
        let snapshot = Snapshot::from_words(WORDS.map(|offset| read_word(pa + offset)));
        if s.last_event.code != 0 || snapshot.done != 1 || snapshot.error != 0 {
            print_number(b"SYSTEM:IMAGE_ERROR_STEP:", snapshot.error);
        }
        let kernel = facts(s, slot);
        observer()
            .observe_exit(
                s.last_event.pid,
                slot,
                s.last_event.code,
                s.last_event.vector,
                snapshot,
                kernel,
                s.ticks,
            )
            .unwrap_or_else(|e| fail(e));
    }
    pub(in super::super) fn fault(s: &State, slot: usize, context: &[u64; 22]) -> ! {
        require(
            is_probe(s, slot) && s.last_event.pid == s.processes[slot].pid,
            Failure::Identity,
        );
        observer().fault();
        print_number(b"SYSTEM:IMAGE_FAULT_PID:", s.last_event.pid);
        print_number(b"SYSTEM:IMAGE_FAULT_VECTOR:", s.last_event.vector);
        print_number(b"SYSTEM:IMAGE_FAULT_RIP:", context[17]);
        print_number(b"SYSTEM:IMAGE_FAULT_ADDRESS:", s.last_event.cr2);
        fail(Failure::Fault)
    }
    pub(in super::super) fn finished() -> bool {
        observer().finished()
    }
    pub(in super::super) fn check_and_print(s: &State) {
        let (b, v, f, tick) = observer()
            .take_for_print(s.ticks)
            .unwrap_or_else(|e| fail(e));
        for (label, value) in [
            (b"SYSTEM:IMAGE_PID:".as_slice(), b.pid),
            (b"SYSTEM:IMAGE_ELF_BYTES:", b.elf_bytes),
            (b"SYSTEM:IMAGE_ELF_FNV64:", b.elf_fnv),
            (b"SYSTEM:IMAGE_PNG:", v.png),
            (b"SYSTEM:IMAGE_JPEG:", v.jpeg),
            (b"SYSTEM:IMAGE_PIXELS:", v.pixels),
            (b"SYSTEM:IMAGE_NEGATIVES:", v.negatives),
            (b"SYSTEM:IMAGE_PIXEL_HASH:", v.hash),
            (b"SYSTEM:IMAGE_DECODED_BYTES:", v.bytes),
            (b"SYSTEM:IMAGE_HEAP_GROW_PAGES:", v.growth),
            (b"SYSTEM:IMAGE_ARENA_PEAK:", v.peak),
            (b"SYSTEM:IMAGE_LIVE_ALLOCATIONS:", v.live),
            (b"SYSTEM:IMAGE_CANCELS:", v.cancels),
            (b"SYSTEM:IMAGE_START_TICKS:", v.start),
            (b"SYSTEM:IMAGE_END_TICKS:", v.end),
            (b"SYSTEM:IMAGE_EXIT_TICKS:", tick),
            (b"SYSTEM:IMAGE_QUALITY_OPEN:", v.quality_open),
            (b"SYSTEM:IMAGE_STACK_PAGES:", f.stack_pages),
            (b"SYSTEM:IMAGE_STACK_GUARD:", 1),
            (b"SYSTEM:IMAGE_HEAP_KERNEL_PAGES:", f.heap_pages),
        ] {
            print_number(label, value);
        }
        serial::write(b"SYSTEM:IMAGES:PASS\n");
    }
}

#[cfg(not(test))]
pub(super) use native::{bind, check_and_print, fault, finished, is_probe, observe_exit};

#[cfg(test)]
mod tests {
    use super::*;
    fn binding() -> Binding {
        Binding {
            pid: 41,
            slot: 4,
            launch_tick: 10,
            initial_mappings: 128,
            elf_bytes: 400000,
            elf_fnv: 7,
        }
    }
    fn snapshot() -> Snapshot {
        Snapshot::from_words([
            1,
            0,
            8,
            3,
            EXPECTED_PIXELS,
            10,
            EXPECTED_HASH,
            EXPECTED_BYTES,
            GROWTH_PAGES,
            1000000,
            0,
            1,
            11,
            15,
            0,
        ])
    }
    fn facts() -> KernelFacts {
        KernelFacts {
            heap_pages: 1040,
            mappings: 1152,
            owned_heap_maps: 1040,
            stack_pages: 16,
            map_rights_and_owners: true,
            stack_guards_unmapped: true,
        }
    }
    fn bound() -> Observer {
        let mut o = Observer::new();
        o.bind(binding()).unwrap();
        o
    }
    fn exit(o: &mut Observer, s: Snapshot, f: KernelFacts) -> Result<(), Failure> {
        o.observe_exit(41, 4, 0, 128, s, f, 16)
    }
    #[test]
    fn valid_snapshot_survives_slot_reuse_without_reading_new_process() {
        let mut o = bound();
        exit(&mut o, snapshot(), facts()).unwrap();
        assert!(o.finished());
        assert!(!o.matches(42, 4, true));
        assert!(!o.matches(41, 4, false));
        let (b, s, _, tick) = o.take_for_print(20).unwrap();
        assert_eq!(b.pid, 41);
        assert_eq!(s.hash, EXPECTED_HASH);
        assert_eq!(tick, 16);
        assert_eq!((b.elf_bytes, b.elf_fnv), (400000, 7));
    }
    #[test]
    fn wrong_or_stale_identity_never_substitutes_for_boot_probe() {
        for (pid, slot) in [(42, 4), (41, 3), (0, 4)] {
            let mut o = bound();
            assert_eq!(
                o.observe_exit(pid, slot, 0, 128, snapshot(), facts(), 16),
                Err(Failure::Identity)
            );
            assert!(!o.finished());
        }
    }
    #[test]
    fn each_corrupted_app_word_blocks_acceptance() {
        let valid = [
            1,
            0,
            8,
            3,
            EXPECTED_PIXELS,
            10,
            EXPECTED_HASH,
            EXPECTED_BYTES,
            GROWTH_PAGES,
            1000000,
            0,
            1,
            11,
            15,
            0,
        ];
        for index in [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 14] {
            let mut words = valid;
            words[index] ^= 1;
            let mut o = bound();
            assert!(
                exit(&mut o, Snapshot::from_words(words), facts()).is_err(),
                "word {index}"
            );
            assert!(!o.finished());
        }
        for peak in [0, ARENA_BYTES + 1] {
            let mut s = snapshot();
            s.peak = peak;
            assert_eq!(exit(&mut bound(), s, facts()), Err(Failure::Heap));
        }
    }
    #[test]
    fn guest_claim_cannot_replace_real_heap_owners_mappings_or_guard() {
        for f in [
            KernelFacts {
                heap_pages: 1039,
                ..facts()
            },
            KernelFacts {
                mappings: 1151,
                ..facts()
            },
            KernelFacts {
                owned_heap_maps: 1039,
                ..facts()
            },
            KernelFacts {
                map_rights_and_owners: false,
                ..facts()
            },
            KernelFacts {
                stack_pages: 15,
                ..facts()
            },
            KernelFacts {
                stack_guards_unmapped: false,
                ..facts()
            },
        ] {
            let mut o = bound();
            assert!(exit(&mut o, snapshot(), f).is_err());
            assert!(!o.finished());
        }
    }
    #[test]
    fn clocks_must_fit_real_launch_and_exit_not_nominal_host_deadline() {
        for (start, end, tick) in [(9, 15, 16), (15, 14, 16), (11, 17, 16), (11, 20010, 20010)] {
            let mut s = snapshot();
            s.start = start;
            s.end = end;
            let mut o = bound();
            assert_eq!(
                o.observe_exit(41, 4, 0, 128, s, facts(), tick),
                Err(Failure::Clock)
            );
        }
        let mut o = bound();
        exit(&mut o, snapshot(), facts()).unwrap();
        assert_eq!(o.take_for_print(15).unwrap_err(), Failure::Clock);
    }
    #[test]
    fn premature_print_fault_and_duplicate_exit_fail_closed() {
        assert_eq!(bound().take_for_print(20).unwrap_err(), Failure::Completion);
        let mut o = bound();
        o.fault();
        assert!(!o.finished());
        assert_eq!(exit(&mut o, snapshot(), facts()), Err(Failure::Poisoned));
        let mut o = bound();
        exit(&mut o, snapshot(), facts()).unwrap();
        assert_eq!(exit(&mut o, snapshot(), facts()), Err(Failure::Replay));
        assert!(!o.finished());
        let mut o = bound();
        exit(&mut o, snapshot(), facts()).unwrap();
        o.take_for_print(20).unwrap();
        assert_eq!(o.take_for_print(20).unwrap_err(), Failure::Replay);
    }
    #[test]
    fn failed_attempt_cannot_be_repaired_by_later_good_words() {
        let mut o = bound();
        let mut s = snapshot();
        s.hash ^= 1;
        assert_eq!(exit(&mut o, s, facts()), Err(Failure::Pixels));
        assert_eq!(exit(&mut o, snapshot(), facts()), Err(Failure::Poisoned));
        assert!(!o.finished());
    }
    #[test]
    fn nonzero_exit_exception_vector_and_rebinding_are_rejected() {
        for (code, vector) in [(1, 128), (0, 14)] {
            let mut o = bound();
            assert_eq!(
                o.observe_exit(41, 4, code, vector, snapshot(), facts(), 16),
                Err(Failure::Completion)
            );
        }
        let mut o = bound();
        assert_eq!(o.bind(binding()), Err(Failure::Replay));
        assert!(!o.finished());
        let mut o = Observer::new();
        assert_eq!(exit(&mut o, snapshot(), facts()), Err(Failure::Binding));
    }
    #[test]
    fn linked_byte_fingerprint_changes_with_actual_input_and_is_not_a_signature() {
        assert_eq!(fnv64(b"hello"), 0xa430d84680aabd0b);
        assert_ne!(fnv64(b"ELF-A"), fnv64(b"ELF-B"));
    }
}
