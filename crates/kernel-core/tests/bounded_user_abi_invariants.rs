//! B05-T5: bounded, systematically enumerated invariants for the ring-3 user ABI
//! policy in `kernel_core::user` (usercopy planning, user ELF parsing and the
//! saved-frame return checks).
//!
//! Every oracle is written from the rules in `crates/kernel-core/src/user.rs`
//! and checked against the inputs, not against the function's own output.
//! Bounded enumeration in the debug profile (overflow checks on); not a proof,
//! not formal verification and not a human audit. The kernel page-table lookup
//! closures, the physical copy itself and TLB/SMP races are out of scope.
use kernel_core::user::{
    copy_plan, parse, return_flags, sector_input_plan, valid_return_layout, CopyPlan, Image,
    PageAccess, MAX_SEGMENTS,
};
use kernel_core::{Error, PAGE, USER_QUOTA};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

// ---------------------------------------------------------------- usercopy

const WINDOW_START: u64 = 0x4000_0000;
const WINDOW_END: u64 = 0x8000_0000;
const PID: u64 = 7;

/// Synthetic page-table models. `*Second` breaks only pages after the first
/// page of the requested range, so a check on the first page alone is not enough.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Model {
    Good,
    MissingAll,
    MissingSecond,
    ForeignAll,
    ForeignSecond,
    ReadOnlyAll,
    ReadOnlySecond,
    NonUserAll,
    NonUserSecond,
    UnalignedAll,
    UnalignedSecond,
}
const MODELS: [Model; 11] = [
    Model::Good,
    Model::MissingAll,
    Model::MissingSecond,
    Model::ForeignAll,
    Model::ForeignSecond,
    Model::ReadOnlyAll,
    Model::ReadOnlySecond,
    Model::NonUserAll,
    Model::NonUserSecond,
    Model::UnalignedAll,
    Model::UnalignedSecond,
];

fn page_of(address: u64) -> u64 {
    address & !(PAGE - 1)
}
/// Distinct, aligned frame per virtual page so a swapped chunk is detected.
fn frame_of(address: u64) -> u64 {
    0x0008_0000_0000_0000 | page_of(address)
}
fn second_only(model: Model) -> bool {
    matches!(
        model,
        Model::MissingSecond
            | Model::ForeignSecond
            | Model::ReadOnlySecond
            | Model::NonUserSecond
            | Model::UnalignedSecond
    )
}
fn model_access(model: Model, address: u64, pointer: u64) -> Option<PageAccess> {
    let broken = !second_only(model) || page_of(address) != page_of(pointer);
    let mut access = PageAccess {
        physical: frame_of(address),
        owner: PID,
        user: true,
        writable: true,
    };
    if broken {
        match model {
            Model::Good => {}
            Model::MissingAll | Model::MissingSecond => return None,
            Model::ForeignAll | Model::ForeignSecond => access.owner = PID + 1,
            Model::ReadOnlyAll | Model::ReadOnlySecond => access.writable = false,
            Model::NonUserAll | Model::NonUserSecond => access.user = false,
            Model::UnalignedAll | Model::UnalignedSecond => access.physical += 8,
        }
    }
    Some(access)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Cause {
    Accepted,
    ZeroLength,
    TooLong,
    EndOverflow,
    BelowWindow,
    AboveWindow,
    Missing,
    NotUser,
    Foreign,
    ReadOnlyOutput,
    UnalignedFrame,
}

/// Independent reference: the range predicate, then every touched page in
/// address order. Returns the expected chunks `(physical, offset, length)`.
fn reference(
    pointer: u64,
    length: u64,
    maximum: u64,
    output: bool,
    model: Model,
) -> Result<Vec<(u64, usize, usize)>, Cause> {
    if length == 0 {
        return Err(Cause::ZeroLength);
    }
    if length > maximum {
        return Err(Cause::TooLong);
    }
    let Some(end) = pointer.checked_add(length) else {
        return Err(Cause::EndOverflow);
    };
    if pointer < WINDOW_START {
        return Err(Cause::BelowWindow);
    }
    if end > WINDOW_END {
        return Err(Cause::AboveWindow);
    }
    let mut chunks = Vec::new();
    let mut page = page_of(pointer);
    while page < end {
        let start = pointer.max(page);
        let stop = end.min(page + PAGE);
        let Some(access) = model_access(model, start, pointer) else {
            return Err(Cause::Missing);
        };
        if !access.user {
            return Err(Cause::NotUser);
        }
        if access.owner != PID {
            return Err(Cause::Foreign);
        }
        if output && !access.writable {
            return Err(Cause::ReadOnlyOutput);
        }
        if access.physical % PAGE != 0 {
            return Err(Cause::UnalignedFrame);
        }
        chunks.push((
            access.physical,
            (start - page) as usize,
            (stop - start) as usize,
        ));
        page += PAGE;
    }
    Ok(chunks)
}

/// Runs one plan, logs every lookup address and checks it against `reference`.
fn check_plan(
    label: &str,
    pointer: u64,
    length: u64,
    maximum: u64,
    output: bool,
    model: Model,
    plan: impl FnOnce(&mut dyn FnMut(u64) -> Option<PageAccess>) -> Result<CopyPlan, Error>,
) -> Cause {
    let mut calls = Vec::new();
    let result = catch_unwind(AssertUnwindSafe(|| {
        plan(&mut |va| {
            calls.push(va);
            model_access(model, va, pointer)
        })
    }))
    .unwrap_or_else(|_| panic!("{label}: panic"));
    let expected = reference(pointer, length, maximum, output, model);
    let cause = match &expected {
        Ok(_) => Cause::Accepted,
        Err(c) => *c,
    };
    match (&result, &expected) {
        (Ok(plan), Ok(chunks)) => {
            assert!(plan.count == 1 || plan.count == 2, "{label}: count");
            assert_eq!(plan.count, chunks.len(), "{label}: chunk count");
            let mut cursor = pointer;
            for (i, &(physical, offset, len)) in chunks.iter().enumerate() {
                let c = plan.chunks[i];
                assert_eq!(
                    (c.physical, c.offset, c.length),
                    (physical, offset, len),
                    "{label}: chunk {i}"
                );
                assert!(
                    c.length > 0 && c.offset + c.length <= PAGE as usize,
                    "{label}: page bound"
                );
                assert_eq!(c.physical, frame_of(cursor), "{label}: frame");
                assert_eq!(c.offset as u64, cursor & (PAGE - 1), "{label}: offset");
                cursor += c.length as u64;
            }
            assert_eq!(cursor, pointer + length, "{label}: total length");
            for c in &plan.chunks[plan.count..] {
                assert_eq!(
                    (c.physical, c.offset, c.length),
                    (0, 0, 0),
                    "{label}: stale chunk"
                );
            }
            assert_eq!(calls.len(), plan.count, "{label}: one lookup per page");
        }
        (
            Err(Error::Range),
            Err(
                Cause::ZeroLength
                | Cause::TooLong
                | Cause::EndOverflow
                | Cause::BelowWindow
                | Cause::AboveWindow,
            ),
        ) => assert!(calls.is_empty(), "{label}: lookup before the range check"),
        (Err(Error::Permission), Err(_)) if cause >= Cause::Missing => {
            // Rejected at the first broken page: page 1 for `*All`, page 2 for `*Second`.
            let failing_page = if second_only(model) { 2 } else { 1 };
            assert_eq!(calls.len(), failing_page, "{label}: lookups {calls:?}");
        }
        _ => panic!("{label}: got {result:?}, expected {expected:?}"),
    }
    // Lookups only ever touch addresses inside the requested range, in order.
    for (i, &va) in calls.iter().enumerate() {
        assert!(
            va >= pointer && va - pointer < length,
            "{label}: lookup {va:#x} out of range"
        );
        if i > 0 {
            assert_eq!(
                page_of(va),
                page_of(calls[i - 1]) + PAGE,
                "{label}: lookup order"
            );
        }
    }
    cause
}

fn pointer_edges(length: u64) -> Vec<u64> {
    let mut out = vec![
        0,
        1,
        0x3fff_ffff,
        WINDOW_START,
        WINDOW_START + 1,
        0x4001_2fff,
        0x4001_3000,
        0x4001_3001,
        0x4001_3fff,
        0x4001_3ec0,
        0x4001_3f00,
        0x4001_3f01,
        0x5fff_ffff,
        0x6000_0000,
        0x7fff_f000,
        0x7fff_fec0,
        0x7fff_ffff,
        WINDOW_END,
        1 << 63,
        0x0001_0000_4000_0000,
        0xffff_8000_0000_0000,
        u64::MAX - 255,
        u64::MAX,
    ];
    for delta in [0u64, 1] {
        out.push(WINDOW_END.wrapping_sub(length).wrapping_sub(delta));
        out.push(WINDOW_END.wrapping_sub(length).wrapping_add(delta));
        out.push(u64::MAX.wrapping_sub(length).wrapping_add(delta + 1));
        out.push(
            page_of(0x4001_3000)
                .wrapping_add(PAGE)
                .wrapping_sub(length)
                .wrapping_add(delta),
        );
    }
    out.sort_unstable();
    out.dedup();
    out
}

#[test]
fn copy_plan_pointer_length_and_lookup_model_product() {
    let lengths = [0u64, 1, 2, 255, 256, 319, 320, 321, 511, 512, 513, u64::MAX];
    let mut tally: BTreeMap<Cause, usize> = BTreeMap::new();
    let mut cases = 0usize;
    for length in lengths {
        // For accepted lengths, every byte offset in an interior page and in the
        // top page of the window; rejected lengths use the pointer edges only.
        let mut pointers = pointer_edges(length);
        if (1..=320).contains(&length) {
            pointers.extend((0..PAGE).map(|o| 0x4001_3000 + o));
            pointers.extend((0..PAGE).map(|o| 0x7fff_f000 + o));
        }
        pointers.sort_unstable();
        pointers.dedup();
        for &pointer in &pointers {
            for output in [false, true] {
                for model in MODELS {
                    let label = format!("copy {pointer:#x}+{length} out={output} {model:?}");
                    let cause = check_plan(&label, pointer, length, 320, output, model, |l| {
                        copy_plan(pointer, length, PID, output, l)
                    });
                    *tally.entry(cause).or_default() += 1;
                    cases += 1;
                }
            }
        }
    }
    println!("copy_plan cases={cases} {tally:?}");
    assert_eq!(cases, CASES_COPY);
    for cause in [
        Cause::Accepted,
        Cause::ZeroLength,
        Cause::TooLong,
        Cause::EndOverflow,
        Cause::BelowWindow,
        Cause::AboveWindow,
        Cause::Missing,
        Cause::NotUser,
        Cause::Foreign,
        Cause::ReadOnlyOutput,
        Cause::UnalignedFrame,
    ] {
        assert!(
            tally.get(&cause).copied().unwrap_or(0) > 0,
            "copy_plan never reached {cause:?}"
        );
    }
}
const CASES_COPY: usize = 1_087_174;

#[test]
fn sector_input_plan_is_exactly_512_input_bytes() {
    let mut tally: BTreeMap<Cause, usize> = BTreeMap::new();
    let mut cases = 0usize;
    let mut pointers = pointer_edges(512);
    pointers.extend((0..PAGE).map(|o| 0x4001_3000 + o));
    pointers.extend((0..PAGE).map(|o| 0x7fff_f000 + o));
    pointers.sort_unstable();
    pointers.dedup();
    for &pointer in &pointers {
        for model in MODELS {
            let label = format!("sector {pointer:#x} {model:?}");
            let cause = check_plan(&label, pointer, 512, 512, false, model, |l| {
                sector_input_plan(pointer, PID, l)
            });
            *tally.entry(cause).or_default() += 1;
            cases += 1;
        }
    }
    println!("sector_input_plan cases={cases} {tally:?}");
    assert_eq!(cases, CASES_SECTOR);
    // Input never requires a writable page; the other reject classes all occur.
    assert_eq!(tally.get(&Cause::ReadOnlyOutput), None);
    for cause in [
        Cause::Accepted,
        Cause::EndOverflow,
        Cause::BelowWindow,
        Cause::AboveWindow,
        Cause::Missing,
        Cause::NotUser,
        Cause::Foreign,
        Cause::UnalignedFrame,
    ] {
        assert!(
            tally.get(&cause).copied().unwrap_or(0) > 0,
            "sector never reached {cause:?}"
        );
    }
}
const CASES_SECTOR: usize = 90_288;

// ---------------------------------------------------------------- user ELF

const PHOFF: usize = 64;
const PHENT: usize = 56;
const LEN: usize = 0x3010;
const IMAGE_START: u64 = 0x4000_0000;
const IMAGE_END: u64 = 0x6000_0000;

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

/// Header 0: PT_GNU_STACK (RW, skipped). Headers 1..=3: PT_LOAD RX, R, RW at
/// IMAGE_START + i*4K from file offset (i+1)*4K, 16 file bytes; the RW segment
/// has 0x2000 memory bytes (bss). Entry is the first RX byte.
fn fixture() -> Vec<u8> {
    let mut b = vec![0; LEN];
    b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    put16(&mut b, 16, 2);
    put16(&mut b, 18, 62);
    put32(&mut b, 20, 1);
    put64(&mut b, 24, IMAGE_START);
    put64(&mut b, 32, PHOFF as u64);
    put16(&mut b, 52, 64);
    put16(&mut b, 54, PHENT as u16);
    put16(&mut b, 56, 4);
    put32(&mut b, PHOFF, 0x6474_e551);
    put32(&mut b, PHOFF + 4, 6);
    for (i, flags) in [5u32, 4, 6].into_iter().enumerate() {
        let at = PHOFF + (i + 1) * PHENT;
        put32(&mut b, at, 1);
        put32(&mut b, at + 4, flags);
        put64(&mut b, at + 8, (i as u64 + 1) * PAGE);
        put64(&mut b, at + 16, IMAGE_START + i as u64 * PAGE);
        put64(&mut b, at + 24, IMAGE_START + i as u64 * PAGE);
        put64(&mut b, at + 32, 16);
        put64(&mut b, at + 40, if flags == 6 { 0x2000 } else { 16 });
        put64(&mut b, at + 48, PAGE);
    }
    b
}

fn rounded(s: &kernel_core::user::Segment) -> (u64, u64) {
    let first = s.va & !(PAGE - 1);
    let last = (s.va + s.memory_size).div_ceil(PAGE) * PAGE;
    (first, last)
}

/// Oracle for an accepted image, checked against the input bytes.
fn check_image(label: &str, b: &[u8], image: &Image) {
    assert_eq!(&b[..7], b"\x7fELF\x02\x01\x01", "{label}: ident");
    assert_eq!(
        (
            get16(b, 16),
            get16(b, 18),
            get32(b, 20),
            get32(b, 48),
            get16(b, 52),
            get16(b, 54)
        ),
        (2, 62, 1, 0, 64, 56),
        "{label}: header"
    );
    assert!(
        (1..=MAX_SEGMENTS).contains(&image.count),
        "{label}: count {}",
        image.count
    );
    assert_eq!(image.entry, get64(b, 24), "{label}: entry");
    // The image equals, in order, the input's PT_LOAD headers; every other header
    // is PT_NULL, PT_NOTE, PT_PHDR or a non-executable PT_GNU_STACK.
    let phoff = get64(b, 32) as usize;
    let phnum = usize::from(get16(b, 56));
    assert!(
        (1..=32).contains(&phnum) && phoff + phnum * PHENT <= b.len(),
        "{label}: phdr table"
    );
    let mut loads = Vec::new();
    for i in 0..phnum {
        let at = phoff + i * PHENT;
        let (kind, flags) = (get32(b, at), get32(b, at + 4));
        match kind {
            1 => {
                assert!(matches!(flags, 4..=6), "{label}: flags {flags}");
                loads.push((
                    get64(b, at + 16),
                    get64(b, at + 8),
                    get64(b, at + 32),
                    get64(b, at + 40),
                    get64(b, at + 48),
                    flags,
                ));
            }
            0 | 4 | 6 => {}
            0x6474_e551 => assert_eq!(flags & 1, 0, "{label}: executable stack"),
            other => panic!("{label}: accepted program header type {other:#x}"),
        }
    }
    assert_eq!(loads.len(), image.count, "{label}: PT_LOAD count");
    let mut pages = 0u64;
    let segments = &image.segments[..image.count];
    for (s, &(va, offset, filesz, memsz, align, flags)) in segments.iter().zip(&loads) {
        assert_eq!(
            (
                s.va,
                s.file_offset,
                s.file_size,
                s.memory_size,
                s.writable,
                s.executable
            ),
            (va, offset, filesz, memsz, flags & 2 != 0, flags & 1 != 0),
            "{label}: segment fields"
        );
        assert!(!(s.writable && s.executable), "{label}: W+X");
        assert!(memsz > 0 && filesz <= memsz, "{label}: sizes");
        assert!(
            offset
                .checked_add(filesz)
                .is_some_and(|e| e <= b.len() as u64),
            "{label}: file range"
        );
        assert!(
            va >= IMAGE_START && va.checked_add(memsz).is_some_and(|e| e <= IMAGE_END),
            "{label}: window"
        );
        assert!(
            align >= PAGE && align.is_power_of_two() && offset % align == va % align,
            "{label}: align"
        );
        let (first, last) = rounded(s);
        pages += (last - first) / PAGE;
    }
    for (i, a) in segments.iter().enumerate() {
        for c in &segments[i + 1..] {
            let (af, al) = rounded(a);
            let (cf, cl) = rounded(c);
            assert!(al <= cf || cl <= af, "{label}: page-rounded overlap");
        }
    }
    assert_eq!(image.pages as u64, pages, "{label}: pages");
    assert!(image.pages <= USER_QUOTA - 32, "{label}: quota");
    assert!(
        segments
            .iter()
            .any(|s| s.executable && image.entry >= s.va && image.entry - s.va < s.file_size),
        "{label}: entry outside executable file bytes"
    );
    for s in &image.segments[image.count..] {
        assert_eq!(
            (s.va, s.memory_size, s.file_size),
            (0, 0, 0),
            "{label}: stale segment"
        );
    }
}

fn run_parse(label: &str, b: &[u8], tally: &mut BTreeMap<String, usize>) {
    let result =
        catch_unwind(AssertUnwindSafe(|| parse(b))).unwrap_or_else(|_| panic!("{label}: panic"));
    let key = match &result {
        Ok(image) => {
            check_image(label, b, image);
            // Deterministic: the same bytes give the same image.
            let again = parse(b).unwrap();
            assert_eq!(
                (again.entry, again.count, again.pages),
                (image.entry, image.count, image.pages)
            );
            assert_eq!(again.segments, image.segments, "{label}: nondeterministic");
            "Ok".to_string()
        }
        Err(e) => format!("{e:?}"),
    };
    *tally.entry(key).or_default() += 1;
}

fn expect_classes(name: &str, tally: &BTreeMap<String, usize>, total: usize, classes: &[&str]) {
    println!("{name} cases={} {tally:?}", tally.values().sum::<usize>());
    assert_eq!(tally.values().sum::<usize>(), total, "{name}: case total");
    for class in classes {
        assert!(
            tally.get(*class).copied().unwrap_or(0) > 0,
            "{name}: never reached {class}"
        );
    }
}

#[test]
fn parse_fixture_is_accepted() {
    let image = parse(&fixture()).unwrap();
    check_image("fixture", &fixture(), &image);
    assert_eq!((image.count, image.pages), (3, 4));
}

#[test]
fn parse_every_header_and_program_header_byte() {
    let base = fixture();
    let mut tally = BTreeMap::new();
    for at in 0..PHOFF + 4 * PHENT {
        let original = base[at];
        let mut values = vec![0x00, 0x01, 0x7f, 0x80, 0xff, original ^ 0x01];
        values.retain(|&v| v != original);
        values.sort_unstable();
        values.dedup();
        for v in values {
            let mut b = base.clone();
            b[at] = v;
            run_parse(&format!("byte {at}={v:#04x}"), &b, &mut tally);
        }
    }
    expect_classes(
        "bytes",
        &tally,
        CASES_BYTES,
        &["Ok", "Permission", "Range", "Overlap"],
    );
}
const CASES_BYTES: usize = 1_235;

#[test]
fn parse_every_truncation_rejects() {
    let base = fixture();
    let mut tally = BTreeMap::new();
    for len in 0..base.len() {
        run_parse(&format!("truncate {len}"), &base[..len], &mut tally);
    }
    assert_eq!(tally.get("Ok"), None, "a truncated image was accepted");
    expect_classes("truncation", &tally, LEN, &["Permission", "Range"]);
}

const EDGES: [u64; 26] = [
    0,
    1,
    16,
    0xfff,
    0x1000,
    0x1001,
    0x2000,
    0x3000,
    LEN as u64 - 16,
    LEN as u64,
    LEN as u64 + 1,
    0x3fff_ffff,
    IMAGE_START,
    IMAGE_START + 0x1000,
    IMAGE_START + 0x2000,
    IMAGE_END - PAGE,
    IMAGE_END - 1,
    IMAGE_END,
    2014 * PAGE,
    2016 * PAGE,
    2017 * PAGE,
    0x0001_0000_4000_0000,
    1 << 63,
    u64::MAX - 4094,
    u64::MAX - 15,
    u64::MAX,
];

#[test]
fn parse_segment_field_edge_pairs() {
    let base = fixture();
    // offset, va, filesz, memsz, align
    let fields = [8usize, 16, 32, 40, 48];
    let mut tally = BTreeMap::new();
    for segment in 1..=3 {
        let at = PHOFF + segment * PHENT;
        for (i, &f) in fields.iter().enumerate() {
            for &g in &fields[i + 1..] {
                for &x in &EDGES {
                    for &y in &EDGES {
                        let mut b = base.clone();
                        put64(&mut b, at + f, x);
                        put64(&mut b, at + g, y);
                        run_parse(
                            &format!("seg {segment} +{f}={x:#x} +{g}={y:#x}"),
                            &b,
                            &mut tally,
                        );
                    }
                }
            }
        }
    }
    expect_classes(
        "segment edges",
        &tally,
        3 * 10 * EDGES.len() * EDGES.len(),
        &["Ok", "Permission", "Range", "Overlap", "Quota"],
    );
}

#[test]
fn parse_header_entry_phoff_phnum_edges() {
    let base = fixture();
    let entries = [
        0,
        IMAGE_START - 1,
        IMAGE_START,
        IMAGE_START + 15,
        IMAGE_START + 16,
        IMAGE_START + PAGE,
        IMAGE_START + 2 * PAGE,
        u64::MAX,
    ];
    let phoffs = [
        0u64,
        1,
        63,
        64,
        65,
        120,
        LEN as u64 - 4 * 56,
        LEN as u64 - 4 * 56 + 1,
        LEN as u64,
        1 << 63,
        u64::MAX,
    ];
    let phnums = [0u16, 1, 2, 3, 4, 5, 31, 32, 33, 0xffff];
    let mut tally = BTreeMap::new();
    for entry in entries {
        for phoff in phoffs {
            for phnum in phnums {
                let mut b = base.clone();
                put64(&mut b, 24, entry);
                put64(&mut b, 32, phoff);
                put16(&mut b, 56, phnum);
                run_parse(
                    &format!("entry {entry:#x} phoff {phoff} phnum {phnum}"),
                    &b,
                    &mut tally,
                );
            }
        }
    }
    expect_classes(
        "header",
        &tally,
        entries.len() * phoffs.len() * phnums.len(),
        &["Ok", "Permission", "Range"],
    );
}

/// n one-page PT_LOAD segments (first RX, rest R) behind n program headers.
/// Total page count exactly around the quota edge `USER_QUOTA - 32`. The RX
/// and R segments keep one page each; the RW segment grows to `total - 2`
/// pages, so the limit is crossed only by the running sum over all three
/// `PT_LOAD` headers. The edge itself must be accepted and one page more must
/// be rejected with `Quota` (not `Range` or `Overlap`).
#[test]
fn parse_quota_boundary_is_exact() {
    let limit = USER_QUOTA - 32;
    let mut tally = BTreeMap::new();
    for total in limit - 2..=limit + 2 {
        let mut b = fixture();
        let rw = PHOFF + 3 * PHENT;
        put64(&mut b, rw + 40, (total as u64 - 2) * PAGE);
        let label = format!("{total} pages");
        run_parse(&label, &b, &mut tally);
        match parse(&b) {
            Ok(image) => {
                assert!(total <= limit, "{label}: accepted above the quota");
                assert_eq!(image.pages, total, "{label}: page count");
            }
            Err(e) => {
                assert!(total > limit, "{label}: rejected at or below the quota");
                assert_eq!(format!("{e:?}"), "Quota", "{label}: reject class");
            }
        }
    }
    expect_classes("quota boundary", &tally, 5, &["Ok", "Quota"]);
}

/// The RW segment is moved so that its end lies exactly at, one byte before
/// and one byte past `IMAGE_END`. Ending at the window end is legal; one byte
/// past it is rejected with `Range`.
#[test]
fn parse_window_end_is_exact() {
    let mut tally = BTreeMap::new();
    for (delta, accept) in [(-1i64, true), (0, true), (1, false)] {
        let mut b = fixture();
        let rw = PHOFF + 3 * PHENT;
        let va = IMAGE_END - PAGE;
        let memsz = (PAGE as i64 + delta) as u64;
        put64(&mut b, rw + 16, va);
        put64(&mut b, rw + 24, va);
        put64(&mut b, rw + 40, memsz);
        let label = format!("end {:#x}", va + memsz);
        run_parse(&label, &b, &mut tally);
        match parse(&b) {
            Ok(_) => assert!(accept, "{label}: accepted past the window"),
            Err(e) => {
                assert!(!accept, "{label}: rejected inside the window");
                assert_eq!(format!("{e:?}"), "Range", "{label}: reject class");
            }
        }
    }
    expect_classes("window end", &tally, 3, &["Ok", "Range"]);
}

fn n_segments(n: usize) -> Vec<u8> {
    let mut b = vec![0; (n + 1) * PAGE as usize];
    b[..64].copy_from_slice(&fixture()[..64]);
    put16(&mut b, 56, n as u16);
    for i in 0..n {
        let at = PHOFF + i * PHENT;
        put32(&mut b, at, 1);
        put32(&mut b, at + 4, if i == 0 { 5 } else { 4 });
        put64(&mut b, at + 8, (i as u64 + 1) * PAGE);
        put64(&mut b, at + 16, IMAGE_START + i as u64 * PAGE);
        put64(&mut b, at + 32, 16);
        put64(&mut b, at + 40, 16);
        put64(&mut b, at + 48, PAGE);
    }
    b
}

#[test]
fn parse_segment_count_edges() {
    let mut tally = BTreeMap::new();
    for n in 1..=12 {
        let b = n_segments(n);
        let label = format!("{n} segments");
        run_parse(&label, &b, &mut tally);
        assert_eq!(parse(&b).is_ok(), n <= MAX_SEGMENTS, "{label}");
    }
    expect_classes("segment count", &tally, 12, &["Ok", "Range"]);
}

// ---------------------------------------------------------- ring-3 return

#[test]
fn return_flags_keep_only_arithmetic_flags_and_force_if() {
    // Bits copied from the user frame: CF, PF, AF, ZF, SF, OF. Forced set:
    // reserved bit 1 and IF. Every other bit (TF, DF, IOPL, NT, RF, VM, AC,
    // VIF, VIP, ID and all reserved high bits) is cleared.
    const COPIED: [u32; 6] = [0, 2, 4, 6, 7, 11];
    const FORCED: [u32; 2] = [1, 9];
    let mut inputs: Vec<u64> = (0..1u64 << 16).collect();
    for bit in 16..64 {
        inputs.push(1 << bit);
        inputs.push(!(1u64 << bit));
    }
    inputs.push(u64::MAX);
    for flags in &inputs {
        let out = return_flags(*flags);
        for bit in 0..64 {
            let expected = if FORCED.contains(&bit) {
                1
            } else if COPIED.contains(&bit) {
                (flags >> bit) & 1
            } else {
                0
            };
            assert_eq!((out >> bit) & 1, expected, "flags {flags:#x} bit {bit}");
        }
    }
    assert_eq!(inputs.len(), (1 << 16) + 2 * 48 + 1);
}

#[test]
fn valid_return_layout_selector_rip_rsp_product() {
    let selectors = [
        0u64,
        0x08,
        0x10,
        0x13,
        0x18,
        0x1b,
        0x20,
        0x23,
        0x2b,
        0x1_0023,
        0x1_001b,
        u64::MAX,
    ];
    let rips = [
        0,
        0x3fff_ffff,
        IMAGE_START,
        IMAGE_START + 1,
        IMAGE_END - 1,
        IMAGE_END,
        0x7fff_ffff,
        0x0001_0000_4000_0000,
        0xffff_8000_4000_0000,
        u64::MAX,
    ];
    let stacks = [
        0x7ffe_f000u64..0x7fff_f000,
        0x7ff7_f000..0x7fff_f000,
        0x7fff_f000..0x7fff_f000,
    ];
    let mut cases = 0usize;
    let mut accepted = 0usize;
    for stack in &stacks {
        let rsps = [
            0,
            stack.start.wrapping_sub(1),
            stack.start,
            stack.start + 8,
            stack.end - 8,
            stack.end - 1,
            stack.end,
            u64::MAX,
        ];
        for &cs in &selectors {
            for &ss in &selectors {
                for &rip in &rips {
                    for &rsp in &rsps {
                        let ok = valid_return_layout(rip, rsp, cs, ss, stack.clone());
                        let expected = cs == 0x23
                            && ss == 0x1b
                            && (IMAGE_START..IMAGE_END).contains(&rip)
                            && rsp >= stack.start
                            && rsp < stack.end;
                        assert_eq!(
                            ok, expected,
                            "rip {rip:#x} rsp {rsp:#x} cs {cs:#x} ss {ss:#x} {stack:?}"
                        );
                        if ok {
                            // Only ring-3 selectors resume.
                            assert_eq!((cs & 3, ss & 3), (3, 3));
                            accepted += 1;
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    println!("valid_return_layout cases={cases} accepted={accepted}");
    assert_eq!(
        cases,
        3 * 8 * selectors.len() * selectors.len() * rips.len()
    );
    assert_eq!(accepted, CASES_RETURN_ACCEPTED);
}
const CASES_RETURN_ACCEPTED: usize = 24;
