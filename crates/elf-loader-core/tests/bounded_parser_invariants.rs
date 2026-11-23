//! B05-T4: bounded, systematically enumerated invariants for `elf_loader_core::parse`.
//!
//! Every accepted plan is checked by an oracle written from the crate's documented
//! rules against the (mutated) input bytes; every rejection is a typed `Error`.
//! Enumerations (all from one 3-segment RX/R/RW fixture of 0x3010 bytes):
//! 1. every byte of the ELF header and program-header table (0..232) replaced by
//!    {0x00, 0x01, 0x7f, 0x80, 0xff, b^0x01};
//! 2. every truncation 0..len;
//! 3. per segment, every pair of the fields offset, address (p_vaddr = p_paddr),
//!    p_vaddr alone, file size, memory size and alignment over a 26-value edge set
//!    (zero, page edges, kernel-window edges, file-length edges, u64::MAX, values
//!    whose page round-up overflows, the x86_64 kernel half), plus the complete
//!    4-way product of offset/address/file size/memory size for the last segment;
//! 4. header e_entry/e_phoff/e_phnum pairs over the same set;
//! 5. a two-segment placement grid at 0x40 granularity for page-overlap.
//!
//! Bounded enumeration in the debug profile (overflow checks on), not a proof.
//! Mutation control: remove the `Error::Overlap` loop or the `flags` W^X match;
//! the oracle must fail.
use boot_protocol::{KERNEL_LEN, KERNEL_START, PAGE_SIZE};
use elf_loader_core::{parse, Error, LoadPlan, MAX_SEGMENTS};
use std::panic::{catch_unwind, AssertUnwindSafe};

const PHOFF: usize = 64;
const PHENT: usize = 56;
const LEN: usize = 0x3010;

fn put16(b: &mut [u8], o: usize, n: u16) {
    b[o..o + 2].copy_from_slice(&n.to_le_bytes());
}
fn put32(b: &mut [u8], o: usize, n: u32) {
    b[o..o + 4].copy_from_slice(&n.to_le_bytes());
}
fn put64(b: &mut [u8], o: usize, n: u64) {
    b[o..o + 8].copy_from_slice(&n.to_le_bytes());
}
fn get16(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes(b[o..o + 2].try_into().unwrap())
}
fn get32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn get64(b: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(b[o..o + 8].try_into().unwrap())
}

/// Segment i: flags [RX, R, RW], offset (i+1)*4K, address KERNEL_START+i*4K,
/// 16 file bytes, 64 memory bytes; entry at the first RX byte.
fn fixture() -> Vec<u8> {
    let mut b = vec![0; LEN];
    b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    put16(&mut b, 16, 2);
    put16(&mut b, 18, 62);
    put32(&mut b, 20, 1);
    put64(&mut b, 24, KERNEL_START);
    put64(&mut b, 32, PHOFF as u64);
    put16(&mut b, 52, 64);
    put16(&mut b, 54, PHENT as u16);
    put16(&mut b, 56, 3);
    for i in 0..3 {
        let p = PHOFF + i * PHENT;
        put32(&mut b, p, 1);
        put32(&mut b, p + 4, [5, 4, 6][i]);
        put64(&mut b, p + 8, (i as u64 + 1) * PAGE_SIZE);
        put64(&mut b, p + 16, KERNEL_START + i as u64 * PAGE_SIZE);
        put64(&mut b, p + 24, KERNEL_START + i as u64 * PAGE_SIZE);
        put64(&mut b, p + 32, 16);
        put64(&mut b, p + 40, 64);
        put64(&mut b, p + 48, PAGE_SIZE);
    }
    b
}

fn edge_values() -> Vec<u64> {
    let (k, l, p, n) = (KERNEL_START, KERNEL_LEN, PAGE_SIZE, LEN as u64);
    vec![
        0,
        1,
        16,
        64,
        p - 1,
        p,
        p + 1,
        2 * p,
        k - 1,
        k,
        k + 0x40,
        k + p,
        k + l - p,
        k + l - 1,
        k + l,
        n - 16,
        n - 1,
        n,
        n + 1,
        u64::MAX,
        u64::MAX - p + 2, // page round-up overflows
        u64::MAX - n + 1,
        1 << 63,
        i64::MAX as u64,
        0xffff_8000_0000_0000, // x86_64 kernel half
        0xffff_ffff_8000_0000,
    ]
}

#[derive(Default, Debug)]
struct Tally {
    ok: usize,
    err: [usize; 8],
}

impl Tally {
    fn total(&self) -> usize {
        self.ok + self.err.iter().sum::<usize>()
    }
}

fn index(e: Error) -> usize {
    match e {
        Error::Size => 0,
        Error::Header => 1,
        Error::Arithmetic => 2,
        Error::ProgramType => 3,
        Error::Segment => 4,
        Error::Permissions => 5,
        Error::Overlap => 6,
        Error::Entry => 7,
    }
}

fn page_end(n: u64) -> u64 {
    n.checked_add(PAGE_SIZE - 1).expect("oracle: page end") & !(PAGE_SIZE - 1)
}

/// The documented contract of an accepted plan, checked against the input bytes.
fn check_plan(file: &[u8], plan: &LoadPlan, label: &str) {
    let segments = plan.segments();
    assert!(
        (1..=MAX_SEGMENTS).contains(&segments.len()),
        "{label}: count"
    );
    let window_end = KERNEL_START + KERNEL_LEN;
    let mut entry_hits = 0;
    for (i, s) in segments.iter().enumerate() {
        let file_end = s.offset.checked_add(s.file_size);
        assert!(
            file_end.is_some_and(|end| end <= file.len()),
            "{label}: file range {s:?}"
        );
        assert!(s.file_size as u64 <= s.memory_size, "{label}: filesz>memsz");
        assert!(s.memory_size > 0, "{label}: empty segment kept");
        // W^X: only R, RX or RW; never W+X and never non-readable.
        assert!(matches!(s.flags, 4..=6), "{label}: flags {}", s.flags);
        let mem_end = s.physical_address.checked_add(s.memory_size);
        assert!(
            s.physical_address >= KERNEL_START && mem_end.is_some_and(|end| end <= window_end),
            "{label}: outside kernel window {s:?}"
        );
        assert_eq!(
            s.offset as u64 % PAGE_SIZE,
            s.physical_address % PAGE_SIZE,
            "{label}: congruence"
        );
        for t in &segments[..i] {
            let a = (
                s.physical_address & !(PAGE_SIZE - 1),
                page_end(mem_end.unwrap()),
            );
            let b = (
                t.physical_address & !(PAGE_SIZE - 1),
                page_end(t.physical_address + t.memory_size),
            );
            assert!(
                a.1 <= b.0 || b.1 <= a.0,
                "{label}: page overlap {s:?} {t:?}"
            );
        }
        if s.flags == 5
            && plan.entry >= s.physical_address
            && plan.entry < s.physical_address + s.file_size as u64
        {
            entry_hits += 1;
        }
    }
    assert_eq!(
        entry_hits, 1,
        "{label}: entry not in exactly one RX file range"
    );
    assert_eq!(
        plan.entry,
        get64(file, 24),
        "{label}: entry differs from e_entry"
    );
    // The plan is exactly the input's PT_LOAD entries with memsz > 0, in order.
    let phoff = get64(file, 32) as usize;
    let expected: Vec<_> = (0..usize::from(get16(file, 56)))
        .map(|i| &file[phoff + i * PHENT..phoff + (i + 1) * PHENT])
        .filter(|h| get32(h, 0) == 1 && get64(h, 40) > 0)
        .map(|h| {
            assert_eq!(get64(h, 16), get64(h, 24), "{label}: vaddr != paddr");
            (
                get64(h, 8),
                get64(h, 24),
                get64(h, 32),
                get64(h, 40),
                get32(h, 4),
            )
        })
        .collect();
    let actual: Vec<_> = segments
        .iter()
        .map(|s| {
            (
                s.offset as u64,
                s.physical_address,
                s.file_size as u64,
                s.memory_size,
                s.flags,
            )
        })
        .collect();
    assert_eq!(
        actual, expected,
        "{label}: plan is not the input's PT_LOADs"
    );
}

fn run(file: &[u8], tally: &mut Tally, label: &dyn Fn() -> String) {
    match catch_unwind(AssertUnwindSafe(|| parse(file))) {
        Err(_) => panic!("{}: parse panicked", label()),
        Ok(Ok(plan)) => {
            check_plan(file, &plan, &label());
            // Deterministic: the same bytes give the same plan.
            assert_eq!(parse(file).unwrap().segments(), plan.segments());
            tally.ok += 1;
        }
        Ok(Err(e)) => tally.err[index(e)] += 1,
    }
}

#[test]
fn fixture_is_accepted() {
    let mut t = Tally::default();
    run(&fixture(), &mut t, &|| "fixture".into());
    assert_eq!(t.ok, 1);
}

#[test]
fn every_header_and_program_header_byte() {
    let base = fixture();
    let mut t = Tally::default();
    let mut file = base.clone();
    for position in 0..PHOFF + 3 * PHENT {
        let mut values = vec![0x00, 0x01, 0x7f, 0x80, 0xff, base[position] ^ 1];
        values.sort_unstable();
        values.dedup();
        for value in values.into_iter().filter(|v| *v != base[position]) {
            file[position] = value;
            run(&file, &mut t, &|| format!("byte {position}={value:#04x}"));
        }
        file[position] = base[position];
    }
    println!("single-byte: {} cases {t:?}", t.total());
    assert_eq!(t.total(), 1_001, "{t:?}");
    // Unchecked ident/padding bytes are accepted; checked fields reject.
    assert!(
        t.ok > 0 && t.err[0] > 0 && t.err[1] > 0 && t.err[5] > 0,
        "{t:?}"
    );
}

#[test]
fn every_truncation_rejects() {
    let base = fixture();
    let mut t = Tally::default();
    for length in 0..base.len() {
        run(&base[..length], &mut t, &|| format!("truncate {length}"));
    }
    println!("truncations: {} cases {t:?}", t.total());
    assert_eq!(t.total(), 12_304, "{t:?}");
    assert_eq!(t.ok, 0, "a truncated fixture was accepted");
}

#[test]
fn segment_field_edge_products() {
    // Field writers; "address" keeps p_vaddr == p_paddr so later checks are reached.
    type Set = fn(&mut [u8], usize, u64);
    let fields: [(&str, Set); 6] = [
        ("offset", |b, p, v| put64(b, p + 8, v)),
        ("address", |b, p, v| {
            put64(b, p + 16, v);
            put64(b, p + 24, v)
        }),
        ("vaddr", |b, p, v| put64(b, p + 16, v)),
        ("filesz", |b, p, v| put64(b, p + 32, v)),
        ("memsz", |b, p, v| put64(b, p + 40, v)),
        ("align", |b, p, v| put64(b, p + 48, v)),
    ];
    let values = edge_values();
    let base = fixture();
    let mut t = Tally::default();
    for segment in 0..3 {
        let p = PHOFF + segment * PHENT;
        for (a, (name_a, set_a)) in fields.iter().enumerate() {
            for (name_b, set_b) in &fields[a..] {
                for &x in &values {
                    for &y in &values {
                        let mut file = base.clone();
                        set_a(&mut file, p, x);
                        set_b(&mut file, p, y);
                        run(&file, &mut t, &|| {
                            format!("seg {segment} {name_a}={x:#x} {name_b}={y:#x}")
                        });
                    }
                }
            }
        }
    }
    // Complete 4-way product for the last segment (RW, ends at file end).
    let p = PHOFF + 2 * PHENT;
    let mut file = base.clone();
    for &offset in &values {
        for &address in &values {
            for &filesz in &values {
                for &memsz in &values {
                    put64(&mut file, p + 8, offset);
                    put64(&mut file, p + 16, address);
                    put64(&mut file, p + 24, address);
                    put64(&mut file, p + 32, filesz);
                    put64(&mut file, p + 40, memsz);
                    run(&file, &mut t, &|| {
                        format!("4-way off={offset:#x} addr={address:#x} filesz={filesz:#x} memsz={memsz:#x}")
                    });
                }
            }
        }
    }
    println!("segment edges: {} cases {t:?}", t.total());
    assert_eq!(t.total(), 499_564, "{t:?}");
    for variant in [
        Error::Arithmetic,
        Error::Segment,
        Error::Overlap,
        Error::Entry,
    ] {
        assert!(
            t.err[index(variant)] > 0,
            "{variant:?} never reached: {t:?}"
        );
    }
    assert!(t.ok > 1, "{t:?}");
}

#[test]
fn header_entry_phoff_phnum_edges() {
    let values = edge_values();
    let counts = [0u64, 1, 2, 3, 4, 63, 64, 65, 0xffff];
    let base = fixture();
    let mut t = Tally::default();
    for &entry in &values {
        for &phoff in &values {
            for &count in &counts {
                let mut file = base.clone();
                put64(&mut file, 24, entry);
                put64(&mut file, 32, phoff);
                put16(&mut file, 56, count as u16);
                run(&file, &mut t, &|| {
                    format!("entry={entry:#x} phoff={phoff:#x} phnum={count}")
                });
            }
        }
    }
    println!("header edges: {} cases {t:?}", t.total());
    assert_eq!(t.total(), 6_084, "{t:?}");
    assert!(
        t.ok > 0 && t.err[0] > 0 && t.err[1] > 0 && t.err[7] > 0,
        "{t:?}"
    );
}

#[test]
fn two_segment_placement_grid() {
    let base = fixture();
    let mut t = Tally::default();
    let p = PHOFF + PHENT; // segment 1 (R) moves around segment 0 (RX) and 2 (RW).
    for step in 0..=(4 * PAGE_SIZE / 0x40) {
        let address = KERNEL_START + step * 0x40;
        for memsz in [
            1,
            16,
            64,
            PAGE_SIZE - 0x40,
            PAGE_SIZE,
            PAGE_SIZE + 1,
            2 * PAGE_SIZE,
        ] {
            for filesz in [0, 16] {
                let mut file = base.clone();
                put64(&mut file, p + 8, 2 * PAGE_SIZE + address % PAGE_SIZE);
                put64(&mut file, p + 16, address);
                put64(&mut file, p + 24, address);
                put64(&mut file, p + 32, filesz);
                put64(&mut file, p + 40, memsz);
                run(&file, &mut t, &|| {
                    format!("seg1 addr={address:#x} memsz={memsz:#x} filesz={filesz}")
                });
            }
        }
    }
    println!("placement grid: {} cases {t:?}", t.total());
    assert_eq!(t.total(), 3_598, "{t:?}");
    assert!(t.ok > 0 && t.err[index(Error::Overlap)] > 0, "{t:?}");
}
