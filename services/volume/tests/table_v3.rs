//! Table v3 (docs/implementation/S09-L2-PROVENANCE.md sections 2, 2.1 and 3):
//! required tests 8 (v2 compatibility), 9 (the v3-capable decoder part of
//! mixed A/B pairs) and 10 (corrupt provenance refused on mount). Every v3
//! copy is built through the public table API (`encode_sector`,
//! `encode_with`); reserved bytes are tampered and the copy resealed here.
mod common;

use common::{fill, recover, Local, Op, SimDisk};
use volume_core::{
    checksum_update,
    client::Client,
    get16,
    service::Service,
    table::{Collection, Next, Table, Version, VERSION, VERSION_V3},
    wire, Status, FNV_OFFSET, SECTOR, TABLE_SECTORS, VOLUME_SECTORS,
};

const EPOCH: u64 = 7;

fn version(revision: u64, origin: u64) -> Version {
    let mut version = Version {
        revision,
        name_length: 1,
        origin,
        ..Version::EMPTY
    };
    version.name[0] = b'v';
    version
}

fn collection(origin: u64) -> Collection {
    let mut collection = Collection {
        revision: 1,
        request_id: 1,
        present: true,
        name_length: 4,
        origin_revision: origin,
        ..Collection::EMPTY
    };
    collection.name[..4].copy_from_slice(b"Work");
    collection
}

/// Generation 1 with no provenance: domain 0 wrote object 0 into collection 1.
fn plain() -> Table {
    let mut t = Table::EMPTY;
    t.generation = 1;
    t.request_id = 1;
    t.clocks[0] = 1;
    t.collections[0] = collection(0);
    let entry = &mut t.entries[0];
    entry.revision = 1;
    entry.request_id = 1;
    entry.created = 1;
    entry.modified = 1;
    entry.present = true;
    entry.collection = 1;
    entry.current = version(1, 0);
    t
}

/// Generation 1: domain 0 installed object 0 with n = 3 versions, so its
/// revision is generation + 2 (exactly on the rule R bound), in collection 1,
/// which carries a recorded origin.
fn installed() -> Table {
    let mut t = plain();
    t.format = VERSION_V3;
    t.collections[0].origin_revision = 4;
    let entry = &mut t.entries[0];
    entry.revision = 3;
    entry.current = version(3, 10);
    entry.history = [version(2, 9), version(1, 8)];
    entry.history_count = 2;
    entry.installed = 3;
    entry.origin_revision = 10;
    entry.origin_created = 5;
    entry.origin_modified = 9;
    t
}

/// Image with each table in slot `generation % 2`; other sectors zero.
fn image(tables: &[Table]) -> Vec<[u8; SECTOR]> {
    let mut image = vec![[0u8; SECTOR]; VOLUME_SECTORS as usize];
    for t in tables {
        let base = (t.generation % 2) * TABLE_SECTORS;
        for index in 0..TABLE_SECTORS {
            image[(base + index) as usize] = t.encode_sector(index);
        }
    }
    image
}

fn mount(t: Table) -> Result<Table, Status> {
    recover(image(&[t]))
}

/// Recompute the attempt digest (offset 496, over bytes 0..488 of every
/// sector of the copy) and each sector checksum (offset 504, over slot,
/// index and bytes 0..504), written independently of `table.rs`.
fn reseal(image: &mut [[u8; SECTOR]], slot: u64) {
    let base = (slot * TABLE_SECTORS) as usize;
    let copy = &mut image[base..base + TABLE_SECTORS as usize];
    let attempt = copy.iter().fold(FNV_OFFSET, |digest, sector| {
        checksum_update(digest, &sector[..488])
    });
    for (index, sector) in copy.iter_mut().enumerate() {
        sector[496..504].copy_from_slice(&attempt.to_le_bytes());
        let sum = checksum_update(
            checksum_update(FNV_OFFSET, &[slot as u8, index as u8]),
            &sector[..504],
        );
        sector[504..].copy_from_slice(&sum.to_le_bytes());
    }
}

fn tampered(t: Table, sector: u64, offset: usize, value: u8) -> Result<Table, Status> {
    let slot = t.generation % 2;
    let mut image = image(&[t]);
    image[(slot * TABLE_SECTORS + sector) as usize][offset] = value;
    reseal(&mut image, slot);
    recover(image)
}

fn client<'a>(
    service: &'a mut Service,
    disk: &'a mut SimDisk,
    domain: usize,
    first_request: u64,
) -> Client<Local<'a>> {
    Client::new(
        Local {
            service,
            disk,
            domain,
        },
        EPOCH,
        first_request,
    )
}

// ---------------------------------------------------------------- test 8

/// Runs a fixed v2 scenario and records an FNV-64 digest of the whole
/// settled volume image after every commit.
struct Run {
    service: Service,
    disk: SimDisk,
    step: u64,
    digests: Vec<u64>,
}

impl Run {
    fn op<R>(&mut self, domain: usize, f: impl FnOnce(&mut Client<Local<'_>>) -> R) -> R {
        self.step += 1;
        let mut c = client(&mut self.service, &mut self.disk, domain, 1000 * self.step);
        let result = f(&mut c);
        let digest = self
            .disk
            .settled()
            .iter()
            .fold(FNV_OFFSET, |digest, sector| checksum_update(digest, sector));
        self.digests.push(digest);
        result
    }
}

fn sub(index: u8, op: u8, collection: u8, expected: u64) -> wire::SubOp {
    wire::SubOp {
        index,
        op,
        collection,
        tags: 0,
        expected,
        kind: 0,
    }
}

/// Non-install commits in both domains: WRITE (new, replace, re-create after
/// PURGE), COLLECTION create and delete, META, RENAME, DELETE, RESTORE
/// (undelete, then history), PURGE, TRANSACT META/DELETE/PURGE. Not covered:
/// COLLECTION rename, and re-create after DELETE without PURGE.
fn scenario() -> Run {
    let mut run = Run {
        service: Service::new(EPOCH, Table::EMPTY, false),
        disk: SimDisk::new(),
        step: 0,
        digests: Vec::new(),
    };
    let r = run.op(0, |c| c.write(0, 0, b"Alpha", 5000, fill(0, 1)).unwrap().0);
    let r = run.op(0, |c| c.write(0, r, b"Alpha", 100, fill(0, 2)).unwrap().0);
    run.op(0, |c| c.collection(0, 1, 0, b"Work").unwrap());
    let r = run.op(0, |c| c.meta(0, r, 5, 7, 1).unwrap());
    let r = run.op(0, |c| c.rename(0, r, b"Beta").unwrap());
    let one = run.op(0, |c| c.write(1, 0, b"Gamma", 4097, fill(1, 1)).unwrap().0);
    run.op(1, |c| c.write(8, 0, b"Other", 10, fill(8, 1)).unwrap());
    let r = run.op(0, |c| c.delete(0, r).unwrap());
    let r = run.op(0, |c| c.restore(0, r, 1).unwrap());
    let r = run.op(0, |c| c.restore(0, r, 1).unwrap());
    run.op(0, |c| {
        c.transact(
            0,
            &[
                sub(1, wire::SUB_META, 1, one),
                sub(0, wire::SUB_DELETE, 0, r),
            ],
        )
        .unwrap()
    });
    let r = run.service.table().entries[0].revision;
    let r = run.op(0, |c| c.purge(0, r).unwrap());
    run.op(0, |c| c.write(0, r, b"Again", 1, fill(0, 9)).unwrap());
    let created = run.op(0, |c| c.collection(0, 2, 0, b"Tmp").unwrap());
    run.op(0, |c| c.collection(0, 2, created, b"").unwrap());
    run.op(1, |c| c.write(9, 0, b"Second", 3, fill(9, 1)).unwrap());
    run.op(1, |c| {
        c.transact(1, &[sub(1, wire::SUB_DELETE, 0, 1)]).unwrap()
    });
    run.op(1, |c| {
        c.transact(1, &[sub(1, wire::SUB_PURGE, 0, 2)]).unwrap()
    });
    run
}

/// Digests of the same scenario produced by the encoder at commit 837619ef
/// (`services/volume` and `services/catalog` extracted with `git archive`
/// and run unchanged): one per commit, generations 1..=18.
const GOLDEN_837619EF: [u64; 18] = [
    0x6f6746ad37fa6fde,
    0xb6a4984f2a3acc17,
    0x4b1df211f2150e11,
    0xd08283616e5a101a,
    0xdcda3fee501cb464,
    0x10e7dfb6abb021bb,
    0x36f4b553ca6dc016,
    0x1ca8650ffca9fb1c,
    0xfa43d9edbe961ee9,
    0x5c58c5e5c6dddb73,
    0xb28d59552097eb4c,
    0x5c08b46024ef5488,
    0xc04484785fbb315e,
    0xc0841ac625668261,
    0x584a13f274ce2762,
    0x470826706214630d,
    0x07860058980f4d68,
    0x69f6a0e302771068,
];

#[test]
fn non_install_commits_are_byte_identical_to_the_837619ef_encoder() {
    let run = scenario();
    assert_eq!(run.digests, GOLDEN_837619EF);
    // Those v2 images mount under the dual decoder as format 2.
    let settled = run.disk.settled();
    for slot in 0..2 {
        assert_eq!(get16(&settled[(slot * TABLE_SECTORS) as usize], 8), VERSION);
    }
    let table = recover(settled).unwrap();
    assert_eq!(table.format, VERSION);
    assert_eq!(&table, run.service.table());
}

#[test]
fn first_v3_copy_comes_from_the_table_api_and_later_commits_stay_v3() {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    client(&mut service, &mut disk, 0, 100)
        .write(0, 0, b"Plain", 10, fill(0, 1))
        .unwrap();
    let v2 = *service.table();
    assert_eq!(v2.format, VERSION);
    // The upgrade commit as `publish` streams it: a `Next` with format 3
    // installs object 1 (n = 3) at generation 2.
    let mut entry = installed().entries[0];
    entry.request_id = 77;
    entry.created = 2;
    entry.modified = 2;
    entry.collection = 0;
    let mut clocks = v2.clocks;
    clocks[0] += 1;
    let next = Next {
        format: VERSION_V3,
        generation: 2,
        request_id: 77,
        clocks,
        objects: &[(1, entry)],
        collections: &v2.collections,
    };
    let attempt = v2.attempt_digest(&next);
    let mut image = disk.settled();
    for index in 0..TABLE_SECTORS {
        image[index as usize] = v2.encode_with(index, &next, attempt);
    }
    let mut expected = v2;
    expected.format = VERSION_V3;
    expected.generation = 2;
    expected.request_id = 77;
    expected.clocks = clocks;
    expected.entries[1] = entry;
    assert_eq!(recover(image.clone()), Ok(expected));

    // Ordinary commits of a v3-capable service never lower the format: a new
    // object, a WRITE of the installed object (its current version moves to
    // history with its origin), and META.
    let mut disk = SimDisk::from_image(image);
    let mut service = Service::recover(EPOCH, false, &mut disk).unwrap();
    for step in 0..3u64 {
        let mut c = client(&mut service, &mut disk, 0, 200 + 100 * step);
        match step {
            0 => drop(c.write(2, 0, b"New", 5, fill(2, 1)).unwrap()),
            1 => drop(c.write(1, 3, b"Next", 7, fill(1, 4)).unwrap()),
            _ => drop(c.meta(0, 1, 3, 0, 0).unwrap()),
        }
        let settled = disk.settled();
        let g = service.table().generation;
        let header = &settled[((g % 2) * TABLE_SECTORS) as usize];
        assert_eq!(get16(header, 8), VERSION_V3, "generation {g}");
        assert_eq!(recover(settled), Ok(*service.table()), "generation {g}");
    }
    let table = service.table();
    assert_eq!(table.format, VERSION_V3);
    let entry = &table.entries[1];
    assert_eq!(
        (entry.revision, entry.installed, entry.origin_revision),
        (4, 3, 10)
    );
    assert_eq!(
        (
            entry.current.origin,
            entry.history[0].origin,
            entry.history[1].origin
        ),
        (0, 10, 9)
    );
}

#[test]
fn admission_compares_the_format_and_the_provenance_fields() {
    let durable = installed();
    let mut v2_claim = durable;
    v2_claim.format = VERSION;
    let mut object_origin = durable;
    object_origin.entries[0].origin_modified = 8;
    let mut version_origin = durable;
    version_origin.entries[0].history[1].origin = 7;
    let mut collection_origin = durable;
    collection_origin.collections[0].origin_revision = 3;
    for (supplied, outcome) in [
        (durable, Ok(())),
        (v2_claim, Err(Status::Corrupt)),
        (object_origin, Err(Status::Corrupt)),
        (version_origin, Err(Status::Corrupt)),
        (collection_origin, Err(Status::Corrupt)),
    ] {
        let mut disk = SimDisk::from_image(image(&[durable]));
        let mut service = Service::new(EPOCH, supplied, false);
        let result = client(&mut service, &mut disk, 0, 100)
            .write(1, 0, b"Doc", 1, fill(1, 1))
            .map(|_| ());
        assert_eq!(result, outcome);
        if outcome.is_err() {
            assert!(!disk
                .log
                .iter()
                .any(|(operation, _)| matches!(operation, Op::Write | Op::Flush)));
        }
    }
}

// ---------------------------------------------------------------- test 9

/// Two generation-1/2 tables for the mixed-pair cases: `old` at generation 1
/// and `new` (object 1 written by commit 2) at generation 2. `old_v3` and
/// `new_v3` select v3 with a collection origin.
fn pair(old_v3: bool, new_v3: bool) -> (Table, Table) {
    let mut old = plain();
    if old_v3 {
        old.format = VERSION_V3;
        old.collections[0].origin_revision = 4;
    }
    let mut new = plain();
    new.generation = 2;
    new.request_id = 2;
    new.clocks[0] = 2;
    let entry = &mut new.entries[1];
    entry.revision = 1;
    entry.request_id = 2;
    entry.created = 2;
    entry.modified = 2;
    entry.present = true;
    entry.current = version(1, 0);
    if new_v3 {
        new.format = VERSION_V3;
        new.collections[0].origin_revision = 4;
    }
    (old, new)
}

#[test]
fn mixed_pairs_keep_the_format_sticky() {
    // Controls: both copies of one format recover the newer one.
    for v3 in [false, true] {
        let (old, new) = pair(v3, v3);
        assert_eq!(recover(image(&[old, new])), Ok(new));
    }
    // (v2 g, v3 g+1) is the upgrade commit and recovers v3.
    let (old, new) = pair(false, true);
    assert_eq!(recover(image(&[old, new])), Ok(new));
    // (v3 g, v2 g+1) never comes from a writer: refused, not either copy.
    let (old, new) = pair(true, false);
    assert_eq!(recover(image(&[old, new])), Err(Status::Corrupt));
}

#[test]
fn torn_v3_attempt_next_to_v2_recovers_the_v2_generation() {
    // Durable v2 generation 2 in slot 0; slot 1 still holds v2 generation 1
    // when the v3 upgrade attempt at generation 3 starts overwriting it.
    let (old, durable) = pair(false, false);
    let mut attempt = durable;
    attempt.format = VERSION_V3;
    attempt.collections[0].origin_revision = 4;
    attempt.generation = 3;
    attempt.request_id = 3;
    attempt.clocks[0] = 3;
    attempt.entries[0].revision = 2;
    attempt.entries[0].request_id = 3;
    attempt.entries[0].modified = 3;
    attempt.entries[0].current = version(2, 0);
    attempt.entries[0].history[0] = version(1, 0);
    attempt.entries[0].history_count = 1;
    let base = image(&[old, durable]);
    let complete = image(&[attempt]);
    let slot = TABLE_SECTORS as usize;
    for written in 0..=TABLE_SECTORS as usize {
        for torn in [false, true] {
            let mut image = base.clone();
            image[slot..slot + written].copy_from_slice(&complete[slot..slot + written]);
            if torn && written < TABLE_SECTORS as usize {
                image[slot + written][..256].copy_from_slice(&complete[slot + written][..256]);
            }
            let expected = if written == TABLE_SECTORS as usize {
                attempt
            } else {
                durable
            };
            assert_eq!(
                recover(image),
                Ok(expected),
                "{written} sectors, torn {torn}"
            );
        }
    }
}

// --------------------------------------------------------------- test 10

#[test]
fn v3_entry_on_the_rule_r_bound_mounts_and_one_less_excess_fails() {
    let t = installed();
    assert_eq!(mount(t), Ok(t));
    let mut short = t;
    short.entries[0].installed = 2;
    assert_eq!(mount(short), Err(Status::Corrupt));
    // The format marker (still recording table version 2) followed by a v3
    // generation 1: the first commit can be the v3 upgrade.
    let mut disk = SimDisk::from_image(image(&[t]));
    volume_core::table::write_format_marker(&mut disk).unwrap();
    let mut table = Table::EMPTY;
    assert_eq!(
        volume_core::table::recover_with_state(&mut disk, &mut table),
        Ok(true)
    );
    assert_eq!(table, t);
}

#[test]
fn table_validate_bounds_the_revision_by_the_domain_clock_plus_excess() {
    // Generation 2: commit 1 (domain 0) installed object 0, commit 2
    // (domain 1) wrote object 8. Clock 0 is 1, so object 0 may reach
    // 1 + 2 = 3; revision 4 passes the generation bound (2 + 2) in
    // `decode_head` and only the clock bound in `Table::validate` stops it.
    let table = |revision: u64| {
        let mut t = installed();
        t.generation = 2;
        t.request_id = 2;
        t.clocks = [1, 1];
        let entry = &mut t.entries[0];
        entry.revision = revision;
        entry.current = version(revision, 0);
        entry.history = [version(revision - 1, 10), version(revision - 2, 9)];
        let other = &mut t.entries[8];
        other.revision = 1;
        other.request_id = 2;
        other.created = 1;
        other.modified = 1;
        other.present = true;
        other.current = version(1, 0);
        t
    };
    // Generation 2 needs its generation-1 companion.
    assert_eq!(recover(image(&[plain(), table(3)])), Ok(table(3)));
    assert_eq!(recover(image(&[plain(), table(4)])), Err(Status::Corrupt));
}

/// `installed()` at `generation` = clock 0 with object 0 at `revision`.
fn high(generation: u64, revision: u64) -> Table {
    let mut t = installed();
    t.generation = generation;
    t.clocks[0] = generation;
    let entry = &mut t.entries[0];
    entry.revision = revision;
    entry.current = version(revision, 10);
    entry.history = [version(revision - 1, 9), version(revision - 2, 8)];
    t
}

#[test]
fn rule_r_bound_is_computed_without_overflow_near_u64_max() {
    // clock + excess = u64::MAX - 1 + 2 overflows; in true integers it lies
    // above every u64 revision, so revision u64::MAX is legitimate here.
    let top = high(u64::MAX - 1, u64::MAX);
    let below = high(u64::MAX - 2, u64::MAX - 1);
    assert_eq!(recover(image(&[below, top])), Ok(top));
    // One generation lower the bound is u64::MAX - 1 and still applies.
    let over = high(u64::MAX - 3, u64::MAX);
    assert_eq!(
        recover(image(&[high(u64::MAX - 4, u64::MAX - 2), over])),
        Err(Status::Corrupt)
    );
}

#[test]
fn revision_u64_max_on_a_crafted_image_is_full_without_a_write() {
    // Rule R lets revision u64::MAX coexist with generation < u64::MAX.
    let crafted = high(u64::MAX - 2, u64::MAX);
    let mut d = SimDisk::from_image(image(&[high(u64::MAX - 3, u64::MAX - 1), crafted]));
    let mut s = Service::recover(EPOCH, false, &mut d).unwrap();
    let mut c = client(&mut s, &mut d, 0, 100);
    assert_eq!(
        c.write(0, u64::MAX, b"x", 1, |_, b| b.fill(1)),
        Err(Status::Full)
    );
    assert_eq!(c.restore(0, u64::MAX, 1), Err((Status::Full, 0)));
    assert_eq!(*s.table(), crafted);
    // A collection revision may reach u64::MAX when clock = generation.
    let mut t = high(u64::MAX, u64::MAX);
    t.collections[0].revision = u64::MAX;
    let mut d = SimDisk::from_image(image(&[high(u64::MAX - 1, u64::MAX - 1), t]));
    let mut s = Service::recover(EPOCH, false, &mut d).unwrap();
    let mut c = client(&mut s, &mut d, 0, 100);
    assert_eq!(c.collection(0, 1, u64::MAX, b"Neu"), Err((Status::Full, 0)));
    assert_eq!(*s.table(), t);
    assert_eq!(recover(d.settled()), Ok(t));
}

#[test]
fn unknown_table_versions_are_refused_on_a_plain_copy() {
    let mut control = plain();
    assert_eq!(mount(control), Ok(control));
    // Format 3 with all provenance zero is a valid v3 copy (2.2, 3).
    control.format = VERSION_V3;
    assert_eq!(mount(control), Ok(control));
    for format in [0, 1, 4, u16::MAX] {
        let mut t = plain();
        t.format = format;
        assert_eq!(mount(t), Err(Status::Corrupt), "format {format}");
    }
}

/// Object 0 of `installed()` reduced to one installed version (n = 1).
fn one_version(t: &mut Table) {
    let entry = &mut t.entries[0];
    entry.revision = 1;
    entry.current = version(1, 10);
    entry.history = [Version::EMPTY; 2];
    entry.history_count = 0;
    entry.installed = 1;
}

/// Drop every provenance field of object 0, including `installed`.
fn not_installed(t: &mut Table) {
    let entry = &mut t.entries[0];
    entry.installed = 0;
    entry.origin_revision = 0;
    entry.origin_created = 0;
    entry.origin_modified = 0;
    entry.current.origin = 0;
    entry.history.iter_mut().for_each(|v| v.origin = 0);
}

/// Invariants 2-6 of 2.1, each one field away from a mounting control.
#[test]
fn corrupt_provenance_fields_are_refused_on_mount() {
    type Edit = fn(&mut Table);
    let controls: [(&str, Edit); 9] = [
        ("installed n = 3", |_| {}),
        ("v3 copy with all provenance zero", |t| {
            t.entries[0] = plain().entries[0];
            t.collections[0].origin_revision = 0;
        }),
        ("installed n = 1", one_version),
        (
            "re-created: object origins zero, version origins kept",
            |t| {
                let entry = &mut t.entries[0];
                entry.origin_revision = 0;
                entry.origin_created = 0;
                entry.origin_modified = 0;
            },
        ),
        ("installed entry deleted", |t| {
            let entry = &mut t.entries[0];
            entry.present = false;
            entry.collection = 0;
            entry.current = Version::EMPTY;
        }),
        ("installed 2, revision 2, one history version", |t| {
            let entry = &mut t.entries[0];
            entry.installed = 2;
            entry.revision = 2;
            entry.current = version(2, 10);
            entry.history = [version(1, 9), Version::EMPTY];
            entry.history_count = 1;
        }),
        ("v3 copy with only a collection origin", |t| {
            t.entries[0] = plain().entries[0];
        }),
        ("v3 entry installed 1 without origins", |t| {
            t.collections[0].origin_revision = 0;
            let entry = &mut t.entries[0];
            entry.revision = 1;
            entry.current = version(1, 0);
            entry.history = [Version::EMPTY; 2];
            entry.history_count = 0;
            entry.installed = 1;
            entry.origin_revision = 0;
            entry.origin_created = 0;
            entry.origin_modified = 0;
        }),
        ("absent collection without origin", |t| {
            t.collections[1] = Collection {
                present: false,
                name_length: 0,
                name: [0; 32],
                origin_revision: 0,
                ..collection(0)
            };
        }),
    ];
    for (label, edit) in controls {
        let mut t = installed();
        edit(&mut t);
        assert_eq!(mount(t), Ok(t), "control: {label}");
    }
    let cases: [(&str, Edit); 20] = [
        ("1: table version 4", |t| t.format = 4),
        ("2: installed = 4", |t| {
            let entry = &mut t.entries[0];
            entry.installed = 4;
            entry.revision = 4;
            entry.current.revision = 4;
            entry.history[0].revision = 3;
            entry.history[1].revision = 2;
        }),
        ("2: installed = 0 with revision > clock", |t| {
            let entry = &mut t.entries[0];
            entry.installed = 0;
            entry.origin_revision = 0;
            entry.origin_created = 0;
            entry.origin_modified = 0;
            entry.revision = 2;
            entry.current = version(2, 0);
            entry.history = [version(1, 0), Version::EMPTY];
            entry.history_count = 1;
        }),
        ("2: installed >= 1 with revision < installed", |t| {
            let entry = &mut t.entries[0];
            entry.revision = 2;
            entry.current = version(2, 10);
            entry.history = [version(1, 9), Version::EMPTY];
            entry.history_count = 1;
        }),
        ("2: installed = 0 with an object origin", |t| {
            one_version(t);
            t.entries[0].installed = 0;
            t.entries[0].current.origin = 0;
        }),
        ("2: installed = 0 with a version origin", |t| {
            one_version(t);
            not_installed(t);
            t.entries[0].current.origin = 10;
        }),
        ("3: origin_revision without origin_created", |t| {
            t.entries[0].origin_created = 0;
        }),
        ("3: origin_created without origin_revision", |t| {
            t.entries[0].origin_revision = 0;
        }),
        ("3: origin_modified alone", |t| {
            t.entries[0].origin_revision = 0;
            t.entries[0].origin_created = 0;
        }),
        ("3: origin_created > origin_modified", |t| {
            t.entries[0].origin_created = 9;
            t.entries[0].origin_modified = 5;
        }),
        ("4: origin on an unused history slot", |t| {
            let entry = &mut t.entries[0];
            entry.history_count = 1;
            entry.history[1] = Version {
                origin: 8,
                ..Version::EMPTY
            };
        }),
        ("4: origin on the empty current of a deleted entry", |t| {
            let entry = &mut t.entries[0];
            entry.present = false;
            entry.collection = 0;
            entry.current = Version {
                origin: 10,
                ..Version::EMPTY
            };
        }),
        ("4: current and history origins equal", |t| {
            t.entries[0].history[0].origin = 10;
        }),
        ("4: two history origins equal", |t| {
            t.entries[0].history[1].origin = 9;
        }),
        ("4: version origin above origin_revision", |t| {
            t.entries[0].current.origin = 11;
        }),
        ("5: excess one too small", |t| t.entries[0].installed = 2),
        ("6: origin on a deleted collection", |t| {
            t.collections[1] = Collection {
                present: false,
                name_length: 0,
                name: [0; 32],
                ..collection(4)
            };
        }),
        ("6: origin on a never-used collection", |t| {
            t.collections[2].origin_revision = 4;
        }),
        ("1: v2 copy with a collection origin", |t| {
            t.format = VERSION;
            t.entries[0] = plain().entries[0];
        }),
        ("1: v2 copy with an installed entry", |t| {
            t.format = VERSION;
            t.collections[0].origin_revision = 0;
            one_version(t);
            not_installed(t);
            t.entries[0].installed = 1;
        }),
    ];
    for (label, edit) in cases {
        let mut t = installed();
        edit(&mut t);
        assert_eq!(mount(t), Err(Status::Corrupt), "{label}");
    }
}

/// Invariant 1 at byte level: in a version-2 copy every v3 byte is zero, and
/// bytes outside the v3 fields are zero in both versions.
#[test]
fn nonzero_reserved_and_v2_provenance_bytes_are_refused() {
    let (v2, v3) = (plain(), installed());
    // Controls: the reseal is right. A free byte (tags, head offset 32)
    // changed to another valid value mounts with exactly that value.
    for t in [v2, v3] {
        let mut expected = t;
        expected.entries[0].tags = 9;
        assert_eq!(tampered(t, 2, 32, 9), Ok(expected));
    }
    // Version 2: header 56..120, head 280..320, history 432..448.
    for (sector, offset) in [
        (0, 56),
        (0, 119),
        (2, 280),
        (2, 288),
        (2, 312),
        (2, 319),
        (3, 432),
        (3, 447),
    ] {
        assert_eq!(
            tampered(v2, sector, offset, 1),
            Err(Status::Corrupt),
            "v2 sector {sector} byte {offset}"
        );
    }
    // Both versions: header 120..488, head 281..288 and 320..488, history
    // 448..488.
    for t in [v2, v3] {
        for (sector, offset) in [
            (0, 120),
            (0, 487),
            (2, 281),
            (2, 287),
            (2, 320),
            (2, 487),
            (3, 448),
            (3, 487),
        ] {
            assert_eq!(
                tampered(t, sector, offset, 1),
                Err(Status::Corrupt),
                "format {} sector {sector} byte {offset}",
                t.format
            );
        }
    }
}
