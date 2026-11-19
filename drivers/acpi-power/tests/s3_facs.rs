use nieuw_os_acpi_power::{
    fadt_facs_address, parse_s3, parse_s5, AcpiTableSnapshot, Error, S3Plan, TableSnapshot,
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
    bytes[64..68].copy_from_slice(&0x604u32.to_le_bytes());
    bytes[68..72].copy_from_slice(&0x608u32.to_le_bytes());
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
        (32, 0),
        (32, 1),
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
