use destination_core::{
    catalog::{
        self,
        v2::{decode, Attributes, CopyV2},
        Catalog, Record, COPY, SUM_AT,
    },
    fnv64, FNV_OFFSET, LABEL, MAX_ENTRIES,
};

const SCOPE: u64 = 4000;

fn record(id: u64, first: u64, sealed_sequence: u64) -> Record {
    let mut label = [0; LABEL];
    label[0] = b'a';
    Record {
        id,
        first,
        length: 512,
        trailer: 432,
        sealed_sequence,
        label_length: 1,
        label,
    }
}

fn copy() -> CopyV2 {
    let mut catalog = Catalog::formatted([1; 16], SCOPE);
    catalog.count = 2;
    catalog.sequence = 8;
    catalog.next_entry = 5;
    catalog.next_free = 1001;
    // Records follow ID order even when a later entry reuses a lower region.
    catalog.records[0] = record(1, 1000, 2);
    catalog.records[1] = record(4, 200, 7);
    let mut attributes = [Attributes::default(); MAX_ENTRIES];
    attributes[0] = Attributes {
        backup_time: 100,
        verified_sequence: 5,
        time_source: 2,
        deleting: true,
        owner: [0xa5; 16],
    };
    CopyV2 {
        catalog,
        attributes,
    }
}

fn edit(mut bytes: [u8; COPY], offset: usize, replacement: &[u8]) -> [u8; COPY] {
    bytes[offset..offset + replacement.len()].copy_from_slice(replacement);
    let checksum = fnv64(FNV_OFFSET, &bytes[..SUM_AT]);
    bytes[SUM_AT..].copy_from_slice(&checksum.to_le_bytes());
    bytes
}

#[test]
fn encodes_canonical_layout_and_preserves_opaque_and_unowned_ids() {
    let copy = copy();
    let bytes = copy.encode(SCOPE).unwrap();
    assert_eq!(&bytes[..10], b"NOSDST\0\x01\x02\0");
    assert_eq!(bytes[1344..1352], 100_i64.to_le_bytes());
    assert_eq!(bytes[1352..1360], 5_u64.to_le_bytes());
    assert_eq!(bytes[1360..1362], [2, 1]);
    assert_eq!(bytes[1728..1744], [0xa5; 16]);
    assert_eq!(bytes[1744..1760], [0; 16]);
    assert_eq!(decode(&bytes, SCOPE), Some(copy));
    assert!(catalog::decode(&bytes, SCOPE).is_none());

    let v1 = Catalog::formatted([1; 16], SCOPE).encode();
    assert!(decode(&v1, SCOPE).is_none());
    let empty = CopyV2 {
        catalog: Catalog::formatted([1; 16], SCOPE),
        attributes: [Attributes::default(); MAX_ENTRIES],
    };
    assert_eq!(decode(&empty.encode(SCOPE).unwrap(), SCOPE), Some(empty));
}

#[test]
fn rejects_noncanonical_bytes_even_with_valid_checksum() {
    let bytes = copy().encode(SCOPE).unwrap();
    for (offset, value) in [
        (10, u8::MAX),   // count outside the fixed table
        (11, 1),         // header padding
        (64 + 41, 1),    // record padding
        (64 + 48, b'/'), // invalid V1 label
        (224, 1),        // unused record
        (1360, 0),       // known time with unknown source
        (1360, 3),       // unknown source value
        (1361, 2),       // unknown state value
        (1362, 1),       // metadata padding
        (1385, 1),       // second deleting record
        (1392, 1),       // unused metadata
        (1760, 1),       // unused owner
        (1984, 1),       // trailing pad
    ] {
        assert!(
            decode(&edit(bytes, offset, &[value]), SCOPE).is_none(),
            "{offset}"
        );
    }
    for (offset, value) in [
        (152, 1000),      // second extent overlaps the first
        (152, SCOPE),     // range ends beyond the scope
        (152, 15),        // reserved sectors
        (152, u64::MAX),  // extent end overflow
        (48, 1000),       // incorrect high-water mark
        (1352, 1),        // verified before sealing
        (1352, 9),        // verified after copy sequence
        (1344, u64::MAX), // negative time
        (144, 1),         // decreasing entry ID
        (176, 2),         // non-increasing sealing sequence
    ] {
        assert!(
            decode(&edit(bytes, offset, &value.to_le_bytes()), SCOPE).is_none(),
            "{offset}"
        );
    }
    assert!(decode(&bytes, SCOPE + 1).is_none());
    let mut corrupt = bytes;
    corrupt[1728] ^= 1;
    assert!(decode(&corrupt, SCOPE).is_none());
}

#[test]
fn encoder_refuses_invalid_or_noncanonical_in_memory_state() {
    let mut input = copy();
    input.attributes[2].owner = [7; 16];
    assert!(input.encode(SCOPE).is_none());
    input = copy();
    input.catalog.records[2] = record(9, 100, 8);
    assert!(input.encode(SCOPE).is_none());
    input = copy();
    input.catalog.count = MAX_ENTRIES + 1;
    assert!(input.encode(SCOPE).is_none());
    input = copy();
    input.attributes[1].deleting = true;
    assert!(input.encode(SCOPE).is_none());
}
