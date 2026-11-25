//! Bounded, oracle-checked enumeration of the virtio PCI capability walk
//! (`parse_capabilities_for`). Host-only B05 evidence for the virtio-dma
//! boundary; see docs/security-review/B05-T6-VIRTIO-CAPABILITY-INVARIANTS.md.
//! Every accepted snapshot is re-checked by an independent chain walker and
//! BAR decoder below; a passing run is not a proof outside these bounds.

use std::panic::{catch_unwind, AssertUnwindSafe};
use virtio_core::{
    parse_capabilities_for, Bar, Error, Region, Regions, DEVICE_ID_BLOCK, DEVICE_ID_ENTROPY,
    PHYSICAL_LIMIT,
};

/// virtio-snd PCI ID, 0x1040 + 25 (drivers/virtio-snd/src/lib.rs:16). That
/// crate is not a dependency of virtio-core, so the value is a literal here.
const DEVICE_ID_SND: u16 = 0x1059;
const IDS: [u16; 3] = [DEVICE_ID_BLOCK, DEVICE_ID_ENTROPY, DEVICE_ID_SND];

const BAR_BASE: u64 = 0x8_0000_0000;
const BAR_SIZE: u64 = 0x1000;
/// Chain order: MSI-X, common (1), notify (2), isr (3), device (4), pci-cfg (5).
const CAP_AT: [usize; 6] = [0x40, 0x4c, 0x5c, 0x70, 0x80, 0x90];
/// (offset, length) of kinds 1..=4 inside the region BAR; notify multiplier 4.
const LAYOUT: [(u32, u32); 4] = [(0x000, 0x38), (0x038, 0x10), (0x048, 1), (0x050, 8)];
const MULTIPLIER: u32 = 4;
const MIN_LENGTH: [u32; 4] = [56, 2, 1, 8];
const ALIGNMENT: [u32; 4] = [8, 2, 1, 8];

fn r16(c: &[u8; 256], i: usize) -> u16 {
    u16::from_le_bytes([c[i], c[i + 1]])
}
fn r32(c: &[u8; 256], i: usize) -> u32 {
    u32::from_le_bytes([c[i], c[i + 1], c[i + 2], c[i + 3]])
}
fn set32(c: &mut [u8; 256], i: usize, v: u32) {
    c[i..i + 4].copy_from_slice(&v.to_le_bytes());
}

fn cap_bytes(index: usize, bar: u8) -> Vec<u8> {
    if index == 0 {
        // MSI-X: four vectors, table and PBA in BAR2.
        return vec![0x11, 0, 3, 0, 2, 0, 0, 0, 0x02, 0x08, 0, 0];
    }
    let kind = index as u8;
    let (offset, length, extra) = match index {
        1..=4 => {
            let (o, l) = LAYOUT[index - 1];
            (o, l, MULTIPLIER)
        }
        _ => (0, 0, 0),
    };
    let len = if kind == 2 || kind == 5 { 20 } else { 16 };
    let bar = if kind == 5 { 0 } else { bar };
    let mut b = vec![9, 0, len, kind, bar, 0, 0, 0];
    b.extend_from_slice(&offset.to_le_bytes());
    b.extend_from_slice(&length.to_le_bytes());
    if len == 20 {
        b.extend_from_slice(&extra.to_le_bytes());
    }
    b
}

/// QEMU-like modern virtio-pci snapshot: 64-bit memory BAR0 holds the four
/// regions, BAR2 is a 32-bit memory BAR (MSI-X), BAR3 an I/O BAR. The moved
/// capability is written last so it may overwrite other bytes.
fn build(id: u16, offsets: &[usize; 6], moved: Option<usize>, bar: u8) -> [u8; 256] {
    let mut c = [0u8; 256];
    c[..2].copy_from_slice(&0x1af4u16.to_le_bytes());
    c[2..4].copy_from_slice(&id.to_le_bytes());
    c[6] = 0x10;
    c[8] = 1; // revision ID of a modern device (header byte that is not 0 or 0xff)
    set32(&mut c, 0x10, (BAR_BASE as u32) | 0xc);
    set32(&mut c, 0x14, (BAR_BASE >> 32) as u32);
    set32(&mut c, 0x18, 0xfebd_1000);
    set32(&mut c, 0x1c, 0xc001);
    c[0x2c..0x2e].copy_from_slice(&0x1af4u16.to_le_bytes());
    c[0x2e..0x30].copy_from_slice(&(id - 0x1040).to_le_bytes());
    c[0x34] = offsets[0] as u8;
    c[0x3d] = 1;
    let order = (0..6).filter(|&i| Some(i) != moved).chain(moved);
    for i in order {
        let mut bytes = cap_bytes(i, bar);
        bytes[1] = offsets.get(i + 1).map_or(0, |&n| n as u8);
        let at = offsets[i];
        let end = (at + bytes.len()).min(256);
        c[at..end].copy_from_slice(&bytes[..end - at]);
    }
    c
}
fn bars() -> [Bar; 6] {
    let mut b = [Bar::NONE; 6];
    b[0] = Bar {
        base: BAR_BASE,
        size: BAR_SIZE,
        is_memory: true,
    };
    b[2] = Bar {
        base: 0xfebd_1000,
        size: 0x1000,
        is_memory: true,
    };
    b[3] = Bar {
        base: 0xc000,
        size: 0x20,
        is_memory: false,
    };
    b
}
fn snapshot(id: u16) -> [u8; 256] {
    build(id, &CAP_AT, None, 0)
}

/// Independent BAR decode from config space: `None` for I/O, reserved types,
/// a 64-bit BAR without an upper register and the upper half of a 64-bit BAR.
fn register_base(c: &[u8; 256], index: usize) -> Option<u64> {
    let mut i = 0;
    while i < 6 {
        let low = r32(c, 0x10 + i * 4);
        if i == index {
            return match low & 7 {
                0 => Some(u64::from(low & !15)),
                4 if i < 5 => Some(u64::from(low & !15) | u64::from(r32(c, 0x14 + i * 4)) << 32),
                _ => None,
            };
        }
        i += if low & 7 == 4 { 2 } else { 1 };
    }
    None
}

/// Oracle for an accepted snapshot, derived from the bytes, not from the parser.
fn oracle(c: &[u8; 256], bars: &[Bar; 6], id: u16, r: &Regions, case: &dyn Fn() -> String) {
    assert_eq!(r16(c, 0), 0x1af4, "{}: vendor", case());
    assert_eq!(r16(c, 2), id, "{}: device", case());
    assert_eq!(c[0x0e] & 0x7f, 0, "{}: header type", case());
    assert_ne!(r16(c, 6) & 0x10, 0, "{}: status capability list", case());

    let mut found: [Option<(Region, u32)>; 4] = [None; 4];
    let mut seen = [false; 256];
    let mut at = c[0x34] as usize;
    while at != 0 {
        assert!(
            (0x40..=0xfc).contains(&at) && at.is_multiple_of(4) && !seen[at],
            "{}: accepted chain pointer {at:#x}",
            case()
        );
        seen[at] = true;
        assert!(!matches!(c[at], 0 | 0xff), "{}: cap id at {at:#x}", case());
        if c[at] == 9 {
            let len = c[at + 2] as usize;
            assert!(len >= 4 && at + len <= 256, "{}: vendor len", case());
            let kind = c[at + 3];
            if (1..=4).contains(&kind) {
                let k = kind as usize - 1;
                assert!(len >= if kind == 2 { 20 } else { 16 }, "{}: len", case());
                assert!(found[k].is_none(), "{}: duplicate kind {kind}", case());
                let region = Region {
                    bar: c[at + 4],
                    offset: r32(c, at + 8),
                    length: r32(c, at + 12),
                };
                let multiplier = if kind == 2 { r32(c, at + 16) } else { 0 };
                found[k] = Some((region, multiplier));
            }
        }
        at = c[at + 1] as usize;
    }

    let got = [r.common, r.notify, r.isr, r.device];
    let mut spans = [(0u128, 0u128); 4];
    for (k, region) in got.iter().enumerate() {
        let (expected, multiplier) =
            found[k].unwrap_or_else(|| panic!("{}: kind {} absent", case(), k + 1));
        assert_eq!(*region, expected, "{}: kind {} fields", case(), k + 1);
        if k == 1 {
            assert_eq!(r.notify_multiplier, multiplier, "{}: multiplier", case());
        }
        assert!(region.length >= MIN_LENGTH[k], "{}: min length", case());
        assert_eq!(region.offset % ALIGNMENT[k], 0, "{}: alignment", case());
        let index = region.bar as usize;
        assert!(index < 6, "{}: bar index", case());
        let b = bars[index];
        assert!(
            b.is_memory && b.size.is_power_of_two() && b.base != 0 && b.base.is_multiple_of(b.size),
            "{}: bar {index} shape {b:?}",
            case()
        );
        let bar_end = u128::from(b.base) + u128::from(b.size);
        assert!(bar_end <= u128::from(PHYSICAL_LIMIT), "{}: limit", case());
        assert_eq!(
            register_base(c, index),
            Some(b.base),
            "{}: register",
            case()
        );
        let start = u128::from(b.base) + u128::from(region.offset);
        let end = start + u128::from(region.length);
        assert!(end <= bar_end, "{}: kind {} outside bar", case(), k + 1);
        assert_eq!(
            region.address(bars),
            Ok(start as u64),
            "{}: address",
            case()
        );
        spans[k] = (start, end);
    }
    for i in 0..4 {
        for j in 0..i {
            assert!(
                spans[i].1 <= spans[j].0 || spans[j].1 <= spans[i].0,
                "{}: kinds {} and {} overlap",
                case(),
                j + 1,
                i + 1
            );
        }
    }
    let (start, end) = spans[1];
    for q in [0u16, 1, 2, 3, 4, 0x100, u16::MAX] {
        let at = start + u128::from(q) * u128::from(r.notify_multiplier);
        let expect = (at + 2 <= end && at % 2 == 0).then_some(at as u64);
        assert_eq!(
            r.notify_address(bars, q).ok(),
            expect,
            "{}: notify {q}",
            case()
        );
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Tally {
    ok: usize,
    invalid: usize,
    unsupported: usize,
    range: usize,
    protocol: usize,
}
impl Tally {
    fn total(&self) -> usize {
        self.ok + self.invalid + self.unsupported + self.range + self.protocol
    }
}

fn run(
    t: &mut Tally,
    c: &[u8; 256],
    bars: &[Bar; 6],
    id: u16,
    case: &dyn Fn() -> String,
) -> Result<Regions, Error> {
    let first = catch_unwind(AssertUnwindSafe(|| parse_capabilities_for(c, bars, id)))
        .unwrap_or_else(|_| panic!("{}: parser panicked", case()));
    assert_eq!(
        first,
        parse_capabilities_for(c, bars, id),
        "{}: nondeterministic",
        case()
    );
    match first {
        Ok(r) => {
            oracle(c, bars, id, &r, case);
            t.ok += 1;
        }
        Err(Error::Invalid) => t.invalid += 1,
        Err(Error::Unsupported) => t.unsupported += 1,
        Err(Error::Range) => t.range += 1,
        Err(Error::Protocol) => t.protocol += 1,
        // Only the four variants above belong to the capability walk.
        Err(e) => panic!("{}: unexpected {e:?}", case()),
    }
    first
}

#[test]
fn fixture_is_accepted_with_exact_regions() {
    for id in IDS {
        let r = parse_capabilities_for(&snapshot(id), &bars(), id).unwrap();
        let regions = [r.common, r.notify, r.isr, r.device];
        for (region, (offset, length)) in regions.iter().zip(LAYOUT) {
            assert_eq!(
                *region,
                Region {
                    bar: 0,
                    offset,
                    length
                }
            );
        }
        assert_eq!(r.notify_multiplier, MULTIPLIER);
    }
}

#[test]
fn every_config_byte_takes_every_value() {
    let bars = bars();
    let mut t = Tally::default();
    for id in IDS {
        let base = snapshot(id);
        for index in 0..256 {
            for value in 0..=255u8 {
                let mut c = base;
                c[index] = value;
                let _ = run(&mut t, &c, &bars, id, &|| {
                    format!("id {id:#x} byte {index:#x}={value:#x}")
                });
            }
        }
    }
    println!("byte enumeration: {t:?}");
    assert_eq!(t.total(), 3 * 256 * 256);
    assert_eq!(
        t,
        Tally {
            ok: 147390,
            invalid: 432,
            unsupported: 10650,
            range: 28434,
            protocol: 9702
        }
    );
}

#[test]
fn every_capability_at_every_aligned_offset() {
    let bars = bars();
    let mut t = Tally::default();
    for cap in 0..6 {
        for at in (0..=0xfc).step_by(4) {
            let mut offsets = CAP_AT;
            offsets[cap] = at;
            let c = build(DEVICE_ID_BLOCK, &offsets, Some(cap), 0);
            let _ = run(&mut t, &c, &bars, DEVICE_ID_BLOCK, &|| {
                format!("cap {cap} at {at:#x}")
            });
        }
    }
    println!("relocation: {t:?}");
    assert_eq!(t.total(), 6 * 64);
    assert_eq!(
        t,
        Tally {
            ok: 157,
            invalid: 5,
            unsupported: 50,
            range: 88,
            protocol: 84
        }
    );
}

#[test]
fn bar_grid() {
    // Low register bits: 32-bit, 32-bit prefetchable, 64-bit, 64-bit
    // prefetchable, legacy below-1M, reserved, I/O.
    const TYPES: [u32; 7] = [0x0, 0x8, 0x4, 0xc, 0x2, 0x6, 0x1];
    const SIZES: [u64; 8] = [0, 3, 0x40, 0x80, 0x1000, 1 << 32, 1 << 51, 1 << 52];
    let mut t = Tally::default();
    for index in 0..6usize {
        for kind in TYPES {
            for size in SIZES {
                let bases = [
                    0,
                    0x1000,
                    0x1010,
                    0xfeb0_0000,
                    1 << 32,
                    PHYSICAL_LIMIT.wrapping_sub(size),
                    PHYSICAL_LIMIT.wrapping_sub(size) + 0x1000,
                    PHYSICAL_LIMIT,
                    0u64.wrapping_sub(size),
                ];
                for base in bases {
                    for is_memory in [true, false] {
                        for agree in [true, false] {
                            for wide_before in [false, true] {
                                let mut c = build(DEVICE_ID_BLOCK, &CAP_AT, None, index as u8);
                                c[0x10..0x28].fill(0);
                                let register = if agree { base } else { base ^ 0x10_0000 };
                                set32(&mut c, 0x10 + index * 4, (register as u32 & !15) | kind);
                                if kind & 6 == 4 && index < 5 {
                                    set32(&mut c, 0x14 + index * 4, (register >> 32) as u32);
                                }
                                if wide_before && index > 0 {
                                    set32(&mut c, 0x0c + index * 4, 0x4);
                                }
                                let mut bars = [Bar::NONE; 6];
                                bars[index] = Bar {
                                    base,
                                    size,
                                    is_memory,
                                };
                                let _ = run(&mut t, &c, &bars, DEVICE_ID_BLOCK, &|| {
                                    format!(
                                        "bar {index} type {kind:#x} base {base:#x} size {size:#x} \
                                         memory {is_memory} agree {agree} wide {wide_before}"
                                    )
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    println!("bar grid: {t:?}");
    assert_eq!(t.total(), 6 * 7 * 8 * 9 * 2 * 2 * 2);
    assert_eq!(
        t,
        Tally {
            ok: 188,
            invalid: 21672,
            unsupported: 960,
            range: 1372,
            protocol: 0
        }
    );
}

#[test]
fn capability_offset_length_and_multiplier_edges() {
    const OFFSETS: [u32; 18] = [
        0,
        1,
        2,
        4,
        7,
        8,
        0x38,
        0x48,
        0x50,
        0x58,
        0x60,
        0xff8,
        0xffe,
        0xfff,
        0x1000,
        0x8000_0000,
        u32::MAX - 7,
        u32::MAX,
    ];
    const LENGTHS: [u32; 14] = [
        0,
        1,
        2,
        7,
        8,
        55,
        56,
        57,
        0x10,
        0xfa8,
        0xfff,
        0x1000,
        0x1001,
        u32::MAX,
    ];
    const MULTIPLIERS: [u32; 7] = [0, 1, 2, 3, MULTIPLIER, 0x8000_0000, u32::MAX];
    let bars = bars();
    let mut t = Tally::default();
    for (kind, &at) in CAP_AT.iter().enumerate().take(5).skip(1) {
        let multipliers: &[u32] = if kind == 2 {
            &MULTIPLIERS
        } else {
            &[MULTIPLIER]
        };
        for offset in OFFSETS {
            for length in LENGTHS {
                for &multiplier in multipliers {
                    let mut c = snapshot(DEVICE_ID_BLOCK);
                    set32(&mut c, at + 8, offset);
                    set32(&mut c, at + 12, length);
                    if kind == 2 {
                        set32(&mut c, at + 16, multiplier);
                    }
                    let _ = run(&mut t, &c, &bars, DEVICE_ID_BLOCK, &|| {
                        format!("kind {kind} offset {offset:#x} length {length:#x} mult {multiplier:#x}")
                    });
                }
            }
        }
    }
    println!("offset/length edges: {t:?}");
    assert_eq!(t.total(), 3 * 18 * 14 + 18 * 14 * 7);
    assert_eq!(
        t,
        Tally {
            ok: 213,
            invalid: 0,
            unsupported: 0,
            range: 1440,
            protocol: 867
        }
    );
}

#[test]
fn device_id_must_match_the_requested_profile() {
    let bars = bars();
    let mut t = Tally::default();
    for built in IDS {
        let c = snapshot(built);
        for requested in [IDS[0], IDS[1], IDS[2], 0x1041, 0x1001, 0x1000, 0, 0xffff] {
            let result = run(&mut t, &c, &bars, requested, &|| {
                format!("built {built:#x} requested {requested:#x}")
            });
            assert_eq!(result.is_ok(), built == requested);
            if built != requested {
                assert_eq!(result, Err(Error::Unsupported));
            }
        }
    }
    assert_eq!(
        t,
        Tally {
            ok: 3,
            unsupported: 21,
            ..Tally::default()
        }
    );
}
