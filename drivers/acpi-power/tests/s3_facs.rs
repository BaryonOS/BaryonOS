use nieuw_os_acpi_power::{
    fadt_facs_address, parse_s3, parse_s5, AcpiTableSnapshot, Error, ExtendedVectorPolicy,
    Pm1Event, S3Plan, TableSnapshot, WakePlan,
};

const FADT: u64 = 0x1000;
const DSDT: u64 = 0x2000;
const FACS: u64 = 0x3000;

fn finish(mut bytes: Vec<u8>, signature: &[u8; 4]) -> Vec<u8> {
    bytes[..4].copy_from_slice(signature);
    let len = bytes.len() as u32;
    bytes[4..8].copy_from_slice(&len.to_le_bytes());
    bytes[9] = 0;
    bytes[9] = 0u8.wrapping_sub(bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)));
    bytes
}

fn name(name: &[u8; 4], a: u8, b: u8) -> Vec<u8> {
    let mut bytes = vec![0x08];
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(&[0x12, 6, 2, 0x0a, a, 0x0a, b]);
    bytes
}

fn dsdt(aml: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0; 36];
    bytes.extend_from_slice(aml);
    finish(bytes, b"DSDT")
}

fn fadt(legacy: u32, extended: u64) -> Vec<u8> {
    let mut bytes = vec![0; 148];
    bytes[36..40].copy_from_slice(&legacy.to_le_bytes());
    bytes[40..44].copy_from_slice(&(DSDT as u32).to_le_bytes());
    bytes[56..60].copy_from_slice(&0x600u32.to_le_bytes());
    bytes[64..68].copy_from_slice(&0x604u32.to_le_bytes());
    bytes[68..72].copy_from_slice(&0x608u32.to_le_bytes());
    bytes[88] = 4;
    bytes[89] = 2;
    bytes[132..140].copy_from_slice(&extended.to_le_bytes());
    finish(bytes, b"FACP")
}

fn facs() -> Vec<u8> {
    let mut bytes = vec![0; 64];
    bytes[..4].copy_from_slice(b"FACS");
    bytes[4..8].copy_from_slice(&64u32.to_le_bytes());
    bytes[32] = 2;
    bytes
}

fn tables<'a>(fadt: &'a [u8], dsdt: &'a [u8]) -> AcpiTableSnapshot<'a> {
    AcpiTableSnapshot::new(
        TableSnapshot::new(fadt, FADT),
        TableSnapshot::new(dsdt, DSDT),
    )
}

fn parse(fadt: &[u8], dsdt: &[u8], facs: &[u8]) -> Result<S3Plan, Error> {
    parse_s3(tables(fadt, dsdt), TableSnapshot::new(facs, FACS))
}

#[test]
fn selects_s3_and_s5_independently_and_preserves_register_bits() {
    let mut aml = name(b"_S5_", 5, 6);
    aml.extend(name(b"_S3_", 1, 2));
    let d = dsdt(&aml);
    let f = fadt(FACS as u32, FACS);
    let s3 = parse(&f, &d, &facs()).unwrap();
    let s5 = parse_s5(tables(&f, &d)).unwrap();
    assert_eq!(s3.pm1a().sleep_type(), 1);
    assert_eq!(s3.pm1b().unwrap().sleep_type(), 2);
    assert_eq!(s3.pm1a().address(), 0x604);
    assert_eq!(s3.pm1b().unwrap().address(), 0x608);
    assert_eq!(s5.pm1a().sleep_type(), 5);
    assert_eq!(s5.pm1b().unwrap().sleep_type(), 6);
    assert_eq!(s3.pm1a().apply_to(0xffff), 0xe7ff);
}

#[test]
fn traverses_literal_root_scope_and_absolute_name() {
    let mut absolute = name(b"_S3_", 3, 4);
    absolute.insert(1, 0x5c);
    let mut scope = vec![0x10, (absolute.len() + 3) as u8, 0x5c, 0];
    scope.extend(absolute);
    assert_eq!(
        parse(&fadt(FACS as u32, 0), &dsdt(&scope), &facs())
            .unwrap()
            .pm1a()
            .sleep_type(),
        3
    );
}

#[test]
fn rejects_missing_nested_and_duplicate_s3() {
    let n = name(b"_S3_", 1, 2);
    let mut nested = vec![0x10, (n.len() + 5) as u8];
    nested.extend_from_slice(b"DEV0");
    nested.extend_from_slice(&n);
    for aml in [vec![], name(b"_S5_", 1, 2), nested] {
        assert_eq!(
            parse(&fadt(FACS as u32, 0), &dsdt(&aml), &facs()),
            Err(Error::MissingS3Object)
        );
    }
    let mut duplicate = n.clone();
    duplicate.extend(n);
    assert_eq!(
        parse(&fadt(FACS as u32, 0), &dsdt(&duplicate), &facs()),
        Err(Error::DuplicateS3Object)
    );
}

#[test]
fn rejects_malformed_s3_and_keeps_validating_trailing_terms() {
    let f = fadt(FACS as u32, 0);
    let mut n = name(b"_S3_", 1, 2);
    n.pop();
    assert_eq!(parse(&f, &dsdt(&n), &facs()), Err(Error::MalformedAml));
    let mut n = name(b"_S3_", 1, 2);
    n.push(0x08);
    assert_eq!(parse(&f, &dsdt(&n), &facs()), Err(Error::MalformedAml));
    assert_eq!(
        parse(&f, &dsdt(&name(b"_S3_", 8, 2)), &facs()),
        Err(Error::SleepTypeOutOfRange)
    );
    let mut n = name(b"_S3_", 1, 2);
    n[7] = 3;
    assert_eq!(parse(&f, &dsdt(&n), &facs()), Err(Error::InvalidS3Package));
}

#[test]
fn resolves_legacy_extended_and_equal_facs_pointers() {
    let d = dsdt(&name(b"_S3_", 1, 2));
    for (legacy, extended) in [(FACS as u32, 0), (0, FACS), (FACS as u32, FACS)] {
        assert_eq!(fadt_facs_address(&fadt(legacy, extended)), Ok(FACS));
        assert_eq!(
            parse(&fadt(legacy, extended), &d, &facs())
                .unwrap()
                .facs()
                .source_address(),
            FACS
        );
    }
    let mut legacy = fadt(FACS as u32, 0);
    legacy.truncate(116);
    assert!(parse(&finish(legacy, b"FACP"), &d, &facs()).is_ok());
    let address = 0x1_0000_0000;
    assert!(parse_s3(
        tables(&fadt(0, address), &d),
        TableSnapshot::new(&facs(), address)
    )
    .is_ok());
}

#[test]
fn rejects_absent_conflicting_partial_and_wrongly_bound_pointers() {
    let d = dsdt(&name(b"_S3_", 1, 2));
    assert_eq!(
        parse(&fadt(0, 0), &d, &facs()),
        Err(Error::MissingFacsAddress)
    );
    assert_eq!(
        parse(&fadt(FACS as u32, FACS + 64), &d, &facs()),
        Err(Error::ConflictingFacsAddresses)
    );
    assert_eq!(
        parse(&fadt(0, FACS + 64), &d, &facs()),
        Err(Error::FacsAddressMismatch)
    );
    for length in 133..140 {
        let mut partial = fadt(FACS as u32, FACS);
        partial.truncate(length);
        assert_eq!(
            parse(&finish(partial, b"FACP"), &d, &facs()),
            Err(Error::InvalidFadtLength)
        );
    }
}

#[test]
fn rejects_facs_alignment_and_source_extent_overflow() {
    let d = dsdt(&name(b"_S3_", 1, 2));
    for (address, error) in [
        (FACS + 1, Error::InvalidFacsAlignment),
        (u64::MAX - 63, Error::InvalidTableSourceAddress),
    ] {
        assert_eq!(
            parse_s3(
                tables(&fadt(0, address), &d),
                TableSnapshot::new(&facs(), address)
            ),
            Err(error)
        );
    }
    for (f, d) in [(u64::MAX - 31, DSDT), (FADT, u64::MAX - 31)] {
        let mut fb = fadt(FACS as u32, 0);
        fb[140..148].copy_from_slice(&d.to_le_bytes());
        let fb = finish(fb, b"FACP");
        let db = dsdt(&name(b"_S3_", 1, 2));
        assert_eq!(
            parse_s3(
                AcpiTableSnapshot::new(TableSnapshot::new(&fb, f), TableSnapshot::new(&db, d)),
                TableSnapshot::new(&facs(), FACS)
            ),
            Err(Error::InvalidTableSourceAddress)
        );
    }
}

#[test]
fn rejects_facs_signature_length_and_bounds_without_checksum() {
    let f = fadt(FACS as u32, 0);
    let d = dsdt(&name(b"_S3_", 1, 2));
    let mut bytes = facs();
    bytes[8] = 0x55; // Hardware signature is unrestricted; there is no SDT checksum.
    assert!(parse(&f, &d, &bytes).is_ok());
    bytes[0] = b'X';
    assert_eq!(parse(&f, &d, &bytes), Err(Error::InvalidSignature));
    for len in [0, 7, 36, 63] {
        assert_eq!(parse(&f, &d, &facs()[..len]), Err(Error::TableTooShort));
    }
    for declared in [0u32, 63, 65, u32::MAX] {
        let mut bytes = facs();
        bytes[4..8].copy_from_slice(&declared.to_le_bytes());
        assert_eq!(parse(&f, &d, &bytes), Err(Error::TableLengthMismatch));
    }
    let mut bytes = facs();
    bytes.resize(128, 0);
    assert_eq!(parse(&f, &d, &bytes), Err(Error::TableLengthMismatch));
    bytes[4..8].copy_from_slice(&128u32.to_le_bytes());
    assert!(parse(&f, &d, &bytes).is_ok());
    bytes[127] = 1;
    assert_eq!(parse(&f, &d, &bytes), Err(Error::UnsupportedFacsFields));
    bytes.resize(1024 * 1024 + 1, 0);
    assert_eq!(parse(&f, &d, &bytes), Err(Error::TableTooLarge));
}

#[test]
fn validates_versions_flags_reserved_fields_and_vector_precedence() {
    let f = fadt(FACS as u32, 0);
    let d = dsdt(&name(b"_S3_", 1, 2));
    for version in [2, 3] {
        let mut bytes = facs();
        bytes[32] = version;
        assert_eq!(parse(&f, &d, &bytes).unwrap().facs().waking_vector(), 0);
        bytes[12..16].copy_from_slice(&0x8000u32.to_le_bytes());
        assert_eq!(
            parse(&f, &d, &bytes).unwrap().facs().waking_vector(),
            0x8000
        );
        bytes[24..32].copy_from_slice(&0x1_0000_8000u64.to_le_bytes());
        bytes[20] = 3;
        bytes[36] = 1;
        bytes[16] = 3; // Global-lock pending/owned state is observed, never acquired.
        let info = parse(&f, &d, &bytes).unwrap().facs();
        assert_eq!(info.firmware_waking_vector(), 0x8000);
        assert_eq!(info.x_firmware_waking_vector(), 0x1_0000_8000);
        assert_eq!(info.waking_vector(), 0x1_0000_8000);
        assert_eq!(info.flags(), 3);
        assert_eq!(info.ospm_flags(), 1);
    }
    for (offset, value) in [
        (32, 4),
        (32, 255),
        (20, 4),
        (36, 2),
        (36, 1),
        (16, 4),
        (33, 1),
        (35, 1),
        (40, 1),
        (63, 1),
    ] {
        let mut bytes = facs();
        bytes[offset] = value;
        assert_eq!(
            parse(&f, &d, &bytes),
            Err(Error::UnsupportedFacsFields),
            "offset {offset}, value {value}"
        );
    }
}

#[test]
fn s3_still_validates_sdt_binding_and_pm1_descriptions() {
    let d = dsdt(&name(b"_S3_", 1, 2));
    let mut f = fadt(FACS as u32, 0);
    f[40..44].copy_from_slice(&0x4000u32.to_le_bytes());
    assert_eq!(
        parse(&finish(f, b"FACP"), &d, &facs()),
        Err(Error::DsdtAddressMismatch)
    );
    let mut f = fadt(FACS as u32, 0);
    f[89] = 1;
    assert_eq!(
        parse(&finish(f, b"FACP"), &d, &facs()),
        Err(Error::InvalidPm1Length)
    );
    let mut f = fadt(FACS as u32, 0);
    f[10] ^= 1;
    assert_eq!(parse(&f, &d, &facs()), Err(Error::InvalidChecksum));
}

#[test]
fn bounds_only_the_selected_vectors_execution_environment() {
    let f = fadt(FACS as u32, 0);
    let d = dsdt(&name(b"_S3_", 1, 2));
    let mut bytes = facs();
    bytes[12..16].copy_from_slice(&0x0f_ffffu32.to_le_bytes());
    assert!(parse(&f, &d, &bytes).is_ok());
    bytes[12..16].copy_from_slice(&0x10_0000u32.to_le_bytes());
    assert_eq!(parse(&f, &d, &bytes), Err(Error::UnsupportedFacsFields));
    bytes[24..32].copy_from_slice(&u64::from(u32::MAX).to_le_bytes());
    assert!(parse(&f, &d, &bytes).is_ok()); // Ignored legacy value does not constrain X.
    bytes[24..32].copy_from_slice(&0x1_0000_0000u64.to_le_bytes());
    assert_eq!(parse(&f, &d, &bytes), Err(Error::UnsupportedFacsFields));
    bytes[20] = 2;
    bytes[36] = 1;
    assert!(parse(&f, &d, &bytes).is_ok());
}

#[test]
fn facs_discovery_validates_before_returning_any_address() {
    let valid = fadt(FACS as u32, 0);
    for length in 0..36 {
        assert_eq!(
            fadt_facs_address(&valid[..length]),
            Err(Error::TableTooShort)
        );
    }
    assert_eq!(
        fadt_facs_address(&fadt(0, u64::MAX - 63)),
        Err(Error::InvalidTableSourceAddress)
    );
    let mut invalid = valid;
    invalid[36] ^= 1;
    assert_eq!(fadt_facs_address(&invalid), Err(Error::InvalidChecksum));
    assert_eq!(
        fadt_facs_address(&fadt(FACS as u32, FACS + 64)),
        Err(Error::ConflictingFacsAddresses)
    );
    assert_eq!(
        fadt_facs_address(&fadt(0, 0)),
        Err(Error::MissingFacsAddress)
    );
}

/// FACS exactly as observed in guest memory under QEMU 8.2.2 `pc-q35-8.2`
/// with Ubuntu OVMF_CODE_4M.fd on 19 November 2026: an ACPI 1.0b structure
/// (no version field), length 64, every other byte zero.
fn qemu_acpi_1_0b_facs() -> Vec<u8> {
    let mut bytes = vec![0; 64];
    bytes[..4].copy_from_slice(b"FACS");
    bytes[4..8].copy_from_slice(&64u32.to_le_bytes());
    bytes
}

/// FADT revision 3, length 244, with the PM1 fields observed in the same
/// QEMU/OVMF guest: PM1a_EVT_BLK 0x600 (X GAS width 32), PM1_EVT_LEN 4,
/// PM1a_CNT_BLK 0x604 (X GAS width 16), FIRMWARE_CTRL only.
fn qemu_fadt() -> Vec<u8> {
    let mut bytes = vec![0; 244];
    bytes[8] = 3;
    bytes[36..40].copy_from_slice(&(FACS as u32).to_le_bytes());
    bytes[40..44].copy_from_slice(&(DSDT as u32).to_le_bytes());
    bytes[56..60].copy_from_slice(&0x600u32.to_le_bytes());
    bytes[64..68].copy_from_slice(&0x604u32.to_le_bytes());
    bytes[88] = 4;
    bytes[89] = 2;
    bytes[140..148].copy_from_slice(&DSDT.to_le_bytes());
    bytes[148..152].copy_from_slice(&[1, 32, 0, 0]);
    bytes[152..160].copy_from_slice(&0x600u64.to_le_bytes());
    bytes[172..176].copy_from_slice(&[1, 16, 0, 0]);
    bytes[176..184].copy_from_slice(&0x604u64.to_le_bytes());
    finish(bytes, b"FACP")
}

fn qemu_s3_package() -> Vec<u8> {
    // Name(_S3_, Package(4){One, One, Zero, Zero}) as in the QEMU q35 DSDT.
    vec![
        0x08, b'_', b'S', b'3', b'_', 0x12, 0x06, 0x04, 0x01, 0x01, 0x00, 0x00,
    ]
}

#[test]
fn accepts_observed_qemu_acpi_1_0b_facs_and_event_block() {
    let d = dsdt(&qemu_s3_package());
    let plan = parse(&qemu_fadt(), &d, &qemu_acpi_1_0b_facs()).unwrap();
    assert_eq!(plan.pm1a().address(), 0x604);
    assert_eq!(plan.pm1a().sleep_type(), 1);
    assert_eq!(plan.pm1b(), None);
    assert_eq!(plan.pm1a_event().status(), 0x600);
    assert_eq!(plan.pm1a_event().enable(), 0x602);
    assert_eq!(plan.pm1a_event().block_length(), 4);
    assert_eq!(plan.pm1b_event(), None);
    assert_eq!(Pm1Event::WAKE_STATUS, 0x8000);
    let facs = plan.facs();
    assert_eq!(facs.version(), 0);
    assert_eq!(facs.hardware_signature(), 0);
    assert_eq!(facs.waking_vector(), 0);
    assert_eq!(facs.source_address(), FACS);
}

#[test]
fn version_zero_and_one_reserve_the_fields_they_do_not_define() {
    let f = fadt(FACS as u32, 0);
    let d = dsdt(&name(b"_S3_", 1, 2));
    // Version 0 (ACPI 1.0b): S4BIOS_F and a below-1-MiB vector are defined.
    let mut v0 = qemu_acpi_1_0b_facs();
    v0[20] = 1;
    v0[12..16].copy_from_slice(&0x9000u32.to_le_bytes());
    v0[8..12].copy_from_slice(&0x1234_5678u32.to_le_bytes());
    let info = parse(&f, &d, &v0).unwrap().facs();
    assert_eq!(
        (info.version(), info.flags(), info.waking_vector()),
        (0, 1, 0x9000)
    );
    assert_eq!(info.hardware_signature(), 0x1234_5678);
    for (offset, value) in [
        (20, 2),
        (24, 1),
        (31, 1),
        (33, 1),
        (36, 1),
        (40, 1),
        (63, 1),
    ] {
        let mut bad = qemu_acpi_1_0b_facs();
        bad[offset] = value;
        assert_eq!(
            parse(&f, &d, &bad),
            Err(Error::UnsupportedFacsFields),
            "v0 offset {offset}"
        );
    }
    let mut far = qemu_acpi_1_0b_facs();
    far[12..16].copy_from_slice(&0x10_0000u32.to_le_bytes());
    assert_eq!(parse(&f, &d, &far), Err(Error::UnsupportedFacsFields));

    // Version 1 (ACPI 2.0/3.0): a 32-bit extended vector, no OSPM flags.
    let mut v1 = qemu_acpi_1_0b_facs();
    v1[32] = 1;
    v1[24..32].copy_from_slice(&0xffff_f000u64.to_le_bytes());
    let info = parse(&f, &d, &v1).unwrap().facs();
    assert_eq!((info.version(), info.waking_vector()), (1, 0xffff_f000));
    for (offset, value) in [(20, 2), (28, 1), (33, 1), (36, 1), (40, 1), (63, 1)] {
        let mut bad = v1.clone();
        bad[offset] = value;
        assert_eq!(
            parse(&f, &d, &bad),
            Err(Error::UnsupportedFacsFields),
            "v1 offset {offset}"
        );
    }
}

#[test]
fn event_blocks_are_bounded_unaliased_system_io() {
    let d = dsdt(&name(b"_S3_", 1, 2));
    let mutate = |change: &dyn Fn(&mut Vec<u8>)| {
        let mut f = qemu_fadt();
        change(&mut f);
        finish(f, b"FACP")
    };
    let facs = qemu_acpi_1_0b_facs();
    let qemu_d = dsdt(&qemu_s3_package());
    // Missing PM1a event block and invalid lengths.
    let f = mutate(&|f| {
        f[56..60].fill(0);
        f[152..160].fill(0);
    });
    assert_eq!(parse(&f, &qemu_d, &facs), Err(Error::MissingPm1aEventBlock));
    for length in [0u8, 2, 3, 5] {
        let f = mutate(&|f| f[88] = length);
        assert_eq!(
            parse(&f, &qemu_d, &facs),
            Err(Error::InvalidPm1EventLength),
            "length {length}"
        );
    }
    // X GAS must describe the whole block as 16-bit-accessible System I/O.
    for (offset, value) in [(148, 0u8), (149, 16), (150, 1), (151, 3)] {
        let f = mutate(&|f| f[offset] = value);
        assert_eq!(parse(&f, &qemu_d, &facs), Err(Error::InvalidGas));
    }
    // Word access size is accepted; the X address supersedes the legacy one.
    let f = mutate(&|f| {
        f[151] = 2;
        f[152..160].copy_from_slice(&0x640u64.to_le_bytes());
    });
    assert_eq!(
        parse(&f, &qemu_d, &facs).unwrap().pm1a_event().status(),
        0x640
    );
    // Beyond the 16-bit I/O space and aliasing a control register or PM1b.
    let f = mutate(&|f| f[152..160].copy_from_slice(&0xfffeu64.to_le_bytes()));
    assert_eq!(
        parse(&f, &qemu_d, &facs),
        Err(Error::InvalidRegisterAddress)
    );
    let f = mutate(&|f| f[152..160].copy_from_slice(&0x602u64.to_le_bytes()));
    assert_eq!(parse(&f, &qemu_d, &facs), Err(Error::AliasedRegisters));
    let f = mutate(&|f| {
        f[60..64].copy_from_slice(&0x602u32.to_le_bytes());
    });
    assert_eq!(parse(&f, &qemu_d, &facs), Err(Error::AliasedRegisters));
    let f = mutate(&|f| {
        f[60..64].copy_from_slice(&0x610u32.to_le_bytes());
    });
    let plan = parse(&f, &qemu_d, &facs).unwrap();
    assert_eq!(plan.pm1b_event().unwrap().status(), 0x610);
    // A truncated X_PM1a_EVT_BLK GAS is a malformed FADT, not a fallback.
    let mut short = qemu_fadt();
    short.truncate(150);
    let short = finish(short, b"FACP");
    assert_eq!(parse(&short, &d, &facs), Err(Error::InvalidFadtLength));
}

#[test]
fn wake_plans_follow_the_facs_version_and_explicit_policy() {
    use ExtendedVectorPolicy::{Edk2Version0Extended as Quirk, Specification as Spec};
    let f = fadt(FACS as u32, 0);
    let d = dsdt(&name(b"_S3_", 1, 2));
    let info = |version: u8, ospm: u8| {
        let mut bytes = qemu_acpi_1_0b_facs();
        bytes[32] = version;
        if version >= 2 {
            bytes[20] = 2;
            bytes[36] = ospm;
        }
        parse(&f, &d, &bytes).unwrap().facs()
    };
    let fields = |plan: WakePlan| {
        (
            plan.legacy_vector(),
            plan.extended_vector(),
            plan.ospm_flags(),
        )
    };
    // Version 0 under the specification: real-mode vector only.
    let v0 = info(0, 0);
    assert_eq!(
        fields(v0.wake_plan(0x8000, None, Spec).unwrap()),
        (0x8000, None, None)
    );
    assert_eq!(
        v0.wake_plan(0x8000, Some(0x10_0000), Spec),
        Err(Error::ExtendedVectorUndefined)
    );
    assert_eq!(fields(v0.disarmed_plan(Spec)), (0, None, None));
    // Version 0 under the explicit edk2 profile policy: offset 24 is armed
    // and restored to zero, never left holding an entry.
    assert_eq!(
        fields(v0.wake_plan(0x8000, Some(0x10_0010), Quirk).unwrap()),
        (0x8000, Some(0x10_0010), None)
    );
    assert_eq!(
        fields(v0.wake_plan(0x8000, None, Quirk).unwrap()),
        (0x8000, Some(0), None)
    );
    assert_eq!(fields(v0.disarmed_plan(Quirk)), (0, Some(0), None));
    // Versions 1 and 2 define the field: it is cleared or set either way.
    let v1 = info(1, 0);
    assert_eq!(
        fields(v1.wake_plan(0x9ff0, None, Spec).unwrap()),
        (0x9ff0, Some(0), None)
    );
    assert_eq!(
        fields(v1.wake_plan(0x9ff0, Some(0xffff_fff0), Spec).unwrap()),
        (0x9ff0, Some(0xffff_fff0), None)
    );
    let v2 = info(2, 1);
    assert_eq!(
        fields(v2.wake_plan(0x1000, Some(0x20_0000), Spec).unwrap()),
        (0x1000, Some(0x20_0000), Some(0))
    );
    assert_eq!(fields(v2.disarmed_plan(Spec)), (0, Some(0), Some(0)));
    // Entry validation for both vectors.
    for entry in [0u64, 0x8001, 0x8008, 0xa_0000, 0x10_0000, !15u64] {
        assert_eq!(
            v0.wake_plan(entry, None, Spec),
            Err(Error::InvalidWakeEntry),
            "real-mode entry {entry:#x}"
        );
    }
    for entry in [0u64, 0x10_0008, 0x1_0000_0000, !15u64] {
        assert_eq!(
            v1.wake_plan(0x8000, Some(entry), Spec),
            Err(Error::InvalidWakeEntry),
            "protected entry {entry:#x}"
        );
    }
    assert_eq!(WakePlan::LEGACY_VECTOR_OFFSET, 12);
    assert_eq!(WakePlan::EXTENDED_VECTOR_OFFSET, 24);
    assert_eq!(WakePlan::OSPM_FLAGS_OFFSET, 36);
}
