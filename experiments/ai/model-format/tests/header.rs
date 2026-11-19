use artifact_store_core::{
    ArtifactDescriptor, ArtifactRef, ArtifactStore, Error as StoreError, StoreCapacity,
    READ_CHUNK_MAX,
};
use nieuw_os_model_format::{
    parse, Dtype, Error, Tensor, MAX_HEADER_BYTES, MAX_METADATA_ENTRIES, MAX_RANK, MAX_TENSORS,
};
use nieuw_os_model_store_contract::{
    open, Cancellation, Error as ReadError, ReadBudget, VerifiedArtifact,
};
use sha2::{Digest, Sha256};
use std::{cell::Cell, rc::Rc};

struct Cancel(Rc<Cell<bool>>);
impl Cancellation for Cancel {
    fn is_cancelled(&self) -> bool {
        self.0.get()
    }
}

struct Store {
    bytes: Vec<u8>,
    descriptor: ArtifactDescriptor,
    max_read: usize,
    error: Option<StoreError>,
    count: Option<usize>,
    reads: Vec<(u64, usize)>,
    cancel_on_read: Option<Rc<Cell<bool>>>,
}

impl ArtifactStore for Store {
    type Handle = ArtifactDescriptor;
    fn open(
        &mut self,
        _: &ArtifactDescriptor,
    ) -> Result<(Self::Handle, ArtifactDescriptor), StoreError> {
        Ok((self.descriptor, self.descriptor))
    }
    fn read_at(
        &mut self,
        handle: &Self::Handle,
        offset: u64,
        output: &mut [u8],
    ) -> Result<usize, StoreError> {
        assert!(!output.is_empty() && output.len() <= READ_CHUNK_MAX);
        self.reads.push((offset, output.len()));
        if handle.reference != self.descriptor.reference {
            return Err(StoreError::Stale);
        }
        if let Some(error) = self.error {
            return Err(error);
        }
        if let Some(count) = self.count {
            return Ok(count);
        }
        let start = usize::try_from(offset).unwrap();
        let amount = output
            .len()
            .min(self.max_read)
            .min(self.bytes.len() - start);
        output[..amount].copy_from_slice(&self.bytes[start..start + amount]);
        if offset >= 8 {
            if let Some(cancel) = &self.cancel_on_read {
                cancel.set(true);
            }
        }
        Ok(amount)
    }
    fn capacity(&mut self) -> Result<StoreCapacity, StoreError> {
        Err(StoreError::Io)
    }
}

fn verified(bytes: Vec<u8>) -> (Store, VerifiedArtifact<ArtifactDescriptor>, Cancel) {
    let descriptor = ArtifactDescriptor {
        reference: ArtifactRef {
            artifact_id: [7; 32],
            immutable_generation: 4,
        },
        length: bytes.len() as u64,
        sha256: Sha256::digest(&bytes).into(),
    };
    let mut store = Store {
        bytes,
        descriptor,
        max_read: READ_CHUNK_MAX,
        error: None,
        count: None,
        reads: Vec::new(),
        cancel_on_read: None,
    };
    let cancel = Cancel(Rc::new(Cell::new(false)));
    let mut verifier = open(&mut store, descriptor).unwrap();
    let mut budget = ReadBudget::new(u64::MAX);
    while !verifier
        .verify_next(&mut store, &mut budget, &cancel)
        .unwrap()
    {}
    let artifact = verifier.finish().unwrap();
    store.reads.clear();
    (store, artifact, cancel)
}

fn file(header: &[u8], data_len: usize) -> Vec<u8> {
    let mut bytes = (header.len() as u64).to_le_bytes().to_vec();
    bytes.extend_from_slice(header);
    bytes.resize(bytes.len() + data_len, 0);
    bytes
}

fn check_bytes(bytes: Vec<u8>) -> Result<(), Error> {
    let (mut store, mut artifact, cancel) = verified(bytes);
    let mut header = vec![0; MAX_HEADER_BYTES];
    let mut table = vec![Tensor::EMPTY; MAX_TENSORS + 1];
    parse(
        &mut artifact,
        &mut store,
        &mut ReadBudget::new(u64::MAX),
        &cancel,
        &mut header,
        &mut table,
    )
    .map(|_| ())
}

fn check(header: &str, data_len: usize) -> Result<(), Error> {
    check_bytes(file(header.as_bytes(), data_len))
}

fn tensor(name: &str, dtype: &str, shape: &str, offsets: &str) -> String {
    format!(r#""{name}":{{"dtype":"{dtype}","shape":{shape},"data_offsets":{offsets}}}"#)
}

#[test]
fn qwen_named_tensors_read_in_short_chunks_and_ignore_object_order() {
    let header = r#"{"model.layers.0.self_attn.q_proj.weight":{"data_offsets":[8,16],"shape":[2,2],"dtype":"BF16"},"__metadata__":{"format":"pt","note":"小模型"},"model.embed_tokens.weight":{"shape":[2,2],"dtype":"F16","data_offsets":[0,8]}}   "#;
    let (mut store, mut artifact, cancel) = verified(file(header.as_bytes(), 16));
    store.max_read = 3;
    let mut scratch = [0; 512];
    let mut table = [Tensor::EMPTY; 2];
    let descriptor = artifact.descriptor();
    let model = parse(
        &mut artifact,
        &mut store,
        &mut ReadBudget::new(u64::MAX),
        &cancel,
        &mut scratch,
        &mut table,
    )
    .unwrap();
    assert_eq!(model.descriptor(), descriptor);
    assert_eq!(model.data_start(), 8 + header.len() as u64);
    assert_eq!(model.tensors().len(), 2);
    assert_eq!(model.tensors()[0].name(), "model.embed_tokens.weight");
    assert_eq!(model.tensors()[0].dtype(), Dtype::F16);
    assert_eq!(model.tensors()[0].shape(), &[2, 2]);
    assert_eq!(model.tensors()[1].data_offsets(), (8, 16));
    assert_eq!(model.tensors()[1].dtype(), Dtype::BF16);
    assert!(store.reads.len() > 3);
    assert!(store
        .reads
        .iter()
        .all(|(offset, requested)| offset + *requested as u64 <= model.data_start()));
    assert_eq!(store.reads[0].0, 0);
    assert_eq!(store.reads[1].0, 3);
}

#[test]
fn accepts_scalars_zero_dimensions_and_empty_data() {
    assert_eq!(
        check(
            &format!("{{{}}}", tensor("scalar", "F32", "[]", "[0,4]")),
            4
        ),
        Ok(())
    );
    assert_eq!(
        check(
            &format!(
                "{{{}}}",
                tensor("empty", "F64", "[18446744073709551615,2,0]", "[0,0]")
            ),
            0
        ),
        Ok(())
    );
    assert_eq!(check("{}", 0), Ok(()));
    assert_eq!(check(r#"{"__metadata__":{}}"#, 0), Ok(()));
    let tensors = [
        tensor("late", "U8", "[0]", "[1,1]"),
        tensor("one", "U8", "[]", "[0,1]"),
        tensor("early", "U8", "[0]", "[0,0]"),
    ];
    assert_eq!(check(&format!("{{{}}}", tensors.join(",")), 1), Ok(()));
}

#[test]
fn accepts_exact_dtype_allowlist_widths_and_rank_boundary() {
    for (dtype, width) in [
        ("BOOL", 1),
        ("U8", 1),
        ("I8", 1),
        ("U16", 2),
        ("I16", 2),
        ("U32", 4),
        ("I32", 4),
        ("U64", 8),
        ("I64", 8),
        ("F16", 2),
        ("BF16", 2),
        ("F32", 4),
        ("F64", 8),
    ] {
        assert_eq!(
            check(
                &format!("{{{}}}", tensor("t", dtype, "[]", &format!("[0,{width}]"))),
                width
            ),
            Ok(()),
            "{dtype}"
        );
    }
    let shape = format!("[{}]", ["1"; MAX_RANK].join(","));
    assert_eq!(
        check(&format!("{{{}}}", tensor("t", "U8", &shape, "[0,1]")), 1),
        Ok(())
    );
}

#[test]
fn rejects_header_length_bounds_prefix_overflow_and_small_scratch() {
    for bytes in [
        vec![1],
        vec![0; 7],
        vec![0; 8],
        [9u64.to_le_bytes().as_slice(), b"{}"].concat(),
    ] {
        assert_eq!(check_bytes(bytes), Err(Error::InvalidHeaderLength));
    }
    assert_eq!(
        check_bytes(u64::MAX.to_le_bytes().to_vec()),
        Err(Error::Overflow)
    );
    let mut large = vec![b' '; MAX_HEADER_BYTES + 1];
    large[..2].copy_from_slice(b"{}");
    assert_eq!(check_bytes(file(&large, 0)), Err(Error::HeaderTooLarge));
    let (mut store, mut artifact, cancel) = verified(file(b"{}", 0));
    assert_eq!(
        parse(
            &mut artifact,
            &mut store,
            &mut ReadBudget::new(32),
            &cancel,
            &mut [0; 1],
            &mut []
        )
        .unwrap_err(),
        Error::BufferTooSmall
    );
}

#[test]
fn rejects_invalid_utf8_json_escapes_and_nonspace_padding() {
    assert_eq!(
        check_bytes(file(b"{\"\xff\":{}}", 0)),
        Err(Error::InvalidUtf8)
    );
    for header in [
        " {}",
        "{}\n",
        "{}\t",
        "{}{}",
        "{",
        "{,}",
        "{\"__metadata__\":{},}",
        "[]",
        "{\"__metadata__\":{\"x\":0}}",
        "{\"__metadata__\":{\"x\":\"\u{0}\"}}",
    ] {
        assert_eq!(check(header, 0), Err(Error::InvalidJson), "{header:?}");
    }
    for header in [
        r#"{"\u0061":{}}"#,
        r#"{"__metadata__":{"x":"a\nb"}}"#,
        r#"{"__metadata__":{"x":"\""}}"#,
    ] {
        assert_eq!(check(header, 0), Err(Error::UnsupportedEscape));
    }
}

#[test]
fn rejects_duplicate_keys_at_every_object_level() {
    let t = tensor("t", "U8", "[0]", "[0,0]");
    assert_eq!(check(&format!("{{{t},{t}}}"), 0), Err(Error::DuplicateKey));
    assert_eq!(
        check(r#"{"__metadata__":{},"__metadata__":{}}"#, 0),
        Err(Error::DuplicateKey)
    );
    assert_eq!(
        check(r#"{"__metadata__":{"x":"a","x":"b"}}"#, 0),
        Err(Error::DuplicateKey)
    );
    for field in [
        r#""dtype":"U8""#,
        r#""shape":[0]"#,
        r#""data_offsets":[0,0]"#,
    ] {
        let header =
            format!(r#"{{"t":{{"dtype":"U8","shape":[0],"data_offsets":[0,0],{field}}}}}"#);
        assert_eq!(check(&header, 0), Err(Error::DuplicateKey));
    }
    // Semantic duplicates cannot bypass detection through an escaped spelling.
    assert_eq!(
        check(r#"{"__metadata__":{"x":"a","\u0078":"b"}}"#, 0),
        Err(Error::UnsupportedEscape)
    );
}

#[test]
fn rejects_unknown_missing_and_wrongly_typed_tensor_fields() {
    assert_eq!(
        check(
            r#"{"t":{"dtype":"U8","shape":[0],"data_offsets":[0,0],"extra":0}}"#,
            0
        ),
        Err(Error::UnknownField)
    );
    for header in [
        r#"{"t":{"dtype":"U8","shape":[0]}}"#,
        r#"{"t":{}}"#,
        r#"{"t":{"dtype":false}}"#,
        r#"{"t":{"dtype":"U8","shape":[0],"data_offsets":[0,0,0]}}"#,
    ] {
        assert_eq!(check(header, 0), Err(Error::InvalidJson));
    }
    for dtype in ["F8_E4M3", "F4", "f32", "", "CUSTOM"] {
        assert_eq!(
            check(&format!("{{{}}}", tensor("t", dtype, "[0]", "[0,0]")), 0),
            Err(Error::UnsupportedDtype)
        );
    }
}

#[test]
fn rejects_integer_syntax_shape_and_offset_overflow() {
    for number in ["-1", "+1", "01", "1.0", "1e0", "null", "true"] {
        assert_eq!(
            check(
                &format!("{{{}}}", tensor("t", "U8", &format!("[{number}]"), "[0,1]")),
                1
            ),
            Err(Error::InvalidJson),
            "{number}"
        );
    }
    for (dtype, shape, offsets) in [
        ("U8", "[18446744073709551616]", "[0,0]"),
        ("U8", "[18446744073709551615,2]", "[0,0]"),
        ("F64", "[18446744073709551615]", "[0,0]"),
        ("U8", "[0]", "[0,18446744073709551616]"),
    ] {
        assert_eq!(
            check(&format!("{{{}}}", tensor("t", dtype, shape, offsets)), 0),
            Err(Error::Overflow)
        );
    }
    assert_eq!(
        check(
            &format!(
                "{{{}}}",
                tensor(
                    "t",
                    "U8",
                    "[0]",
                    "[18446744073709551615,18446744073709551615]"
                )
            ),
            0
        ),
        Err(Error::InvalidOffsets)
    );
}

#[test]
fn rejects_overlap_gaps_bounds_reversed_offsets_and_size_mismatch() {
    let overlap = [
        tensor("a", "U8", "[2]", "[0,2]"),
        tensor("b", "U8", "[2]", "[1,3]"),
    ]
    .join(",");
    assert_eq!(check(&format!("{{{overlap}}}"), 3), Err(Error::Overlap));
    let gap = [
        tensor("b", "U8", "[1]", "[2,3]"),
        tensor("a", "U8", "[1]", "[0,1]"),
    ]
    .join(",");
    assert_eq!(check(&format!("{{{gap}}}"), 3), Err(Error::UncoveredData));
    assert_eq!(check("{}", 1), Err(Error::UncoveredData));
    for (shape, offsets, data_len, error) in [
        ("[1]", "[0,1]", 2, Error::UncoveredData),
        ("[1]", "[0,1]", 0, Error::InvalidOffsets),
        ("[0]", "[2,1]", 2, Error::InvalidOffsets),
        ("[2]", "[0,1]", 1, Error::SizeMismatch),
    ] {
        assert_eq!(
            check(
                &format!("{{{}}}", tensor("t", "U8", shape, offsets)),
                data_len
            ),
            Err(error)
        );
    }
}

#[test]
fn bounds_rank_tensor_count_metadata_count_and_nesting() {
    let shape = format!("[{}]", ["1"; MAX_RANK + 1].join(","));
    assert_eq!(
        check(&format!("{{{}}}", tensor("t", "U8", &shape, "[0,1]")), 1),
        Err(Error::RankLimit)
    );
    let entries: Vec<_> = (0..=MAX_TENSORS)
        .map(|i| tensor(&format!("t{i}"), "U8", "[0]", "[0,0]"))
        .collect();
    assert_eq!(
        check(&format!("{{{}}}", entries.join(",")), 0),
        Err(Error::TensorLimit)
    );
    let entries: Vec<_> = (0..=MAX_METADATA_ENTRIES)
        .map(|i| format!(r#""k{i}":"v""#))
        .collect();
    assert_eq!(
        check(
            &format!(r#"{{"__metadata__":{{{}}}}}"#, entries.join(",")),
            0
        ),
        Err(Error::MetadataLimit)
    );
    for header in [
        r#"{"t":{"dtype":"U8","shape":[[[[1]]]],"data_offsets":[0,1]}}"#,
        r#"{"__metadata__":{"x":{"y":{"z":{}}}}}"#,
    ] {
        assert_eq!(check(header, 1), Err(Error::DepthLimit));
    }
    let (mut store, mut artifact, cancel) = verified(file(
        format!("{{{}}}", tensor("t", "U8", "[0]", "[0,0]")).as_bytes(),
        0,
    ));
    assert_eq!(
        parse(
            &mut artifact,
            &mut store,
            &mut ReadBudget::new(u64::MAX),
            &cancel,
            &mut [0; 128],
            &mut []
        )
        .unwrap_err(),
        Error::TensorLimit
    );
}

#[test]
fn rejects_all_truncated_header_prefixes_without_panicking() {
    let header = format!("{{{}}}", tensor("t", "U8", "[0]", "[0,0]"));
    for cut in 0..header.len() {
        assert!(
            check_bytes(file(&header.as_bytes()[..cut], 0)).is_err(),
            "cut {cut}"
        );
    }
}

#[test]
fn propagates_store_failures_and_preserves_generation_binding() {
    for (injected, expected) in [
        (StoreError::Unauthorized, ReadError::Unauthorized),
        (StoreError::Io, ReadError::StoreUnavailable),
        (StoreError::Corrupt, ReadError::Corrupt),
    ] {
        let (mut store, mut artifact, cancel) = verified(file(b"{}", 0));
        store.error = Some(injected);
        assert_eq!(
            parse(
                &mut artifact,
                &mut store,
                &mut ReadBudget::new(32),
                &cancel,
                &mut [0; 16],
                &mut []
            )
            .unwrap_err(),
            Error::Store(expected)
        );
    }
    let (mut store, mut artifact, cancel) = verified(file(b"{}", 0));
    store.descriptor.reference.immutable_generation += 1;
    assert_eq!(
        parse(
            &mut artifact,
            &mut store,
            &mut ReadBudget::new(32),
            &cancel,
            &mut [0; 16],
            &mut []
        )
        .unwrap_err(),
        Error::Store(ReadError::Stale)
    );
    for (count, error) in [(0, ReadError::Truncated), (9, ReadError::Protocol)] {
        let (mut store, mut artifact, cancel) = verified(file(b"{}", 0));
        store.count = Some(count);
        assert_eq!(
            parse(
                &mut artifact,
                &mut store,
                &mut ReadBudget::new(32),
                &cancel,
                &mut [0; 16],
                &mut []
            )
            .unwrap_err(),
            Error::Store(error)
        );
    }
}

#[test]
fn preserves_budget_charging_retry_and_cancellation_failures() {
    let (mut store, mut artifact, cancel) = verified(file(b"{}", 0));
    let mut budget = ReadBudget::new(7);
    assert_eq!(
        parse(
            &mut artifact,
            &mut store,
            &mut budget,
            &cancel,
            &mut [0; 16],
            &mut []
        )
        .unwrap_err(),
        Error::Store(ReadError::BudgetExceeded)
    );
    assert!(store.reads.is_empty());
    assert_eq!(budget.remaining_bytes(), 7);
    let mut budget = ReadBudget::new(8);
    assert_eq!(
        parse(
            &mut artifact,
            &mut store,
            &mut budget,
            &cancel,
            &mut [0; 16],
            &mut []
        )
        .unwrap_err(),
        Error::Store(ReadError::BudgetExceeded)
    );
    assert_eq!(budget.remaining_bytes(), 0);
    assert_eq!(store.reads, [(0, 8)]);
    store.max_read = 1;
    let mut budget = ReadBudget::new(39); // Prefix charges 8+7+...+1; header 2+1.
    assert!(parse(
        &mut artifact,
        &mut store,
        &mut budget,
        &cancel,
        &mut [0; 16],
        &mut []
    )
    .is_ok());
    assert_eq!(budget.remaining_bytes(), 0);
    for during_read in [false, true] {
        let (mut store, mut artifact, cancel) = verified(file(b"{}", 0));
        if during_read {
            store.cancel_on_read = Some(Rc::clone(&cancel.0));
        } else {
            cancel.0.set(true);
        }
        assert_eq!(
            parse(
                &mut artifact,
                &mut store,
                &mut ReadBudget::new(32),
                &cancel,
                &mut [0; 16],
                &mut []
            )
            .unwrap_err(),
            Error::Store(ReadError::Cancelled)
        );
        cancel.0.set(false);
        assert_eq!(
            parse(
                &mut artifact,
                &mut store,
                &mut ReadBudget::new(32),
                &cancel,
                &mut [0; 16],
                &mut []
            )
            .unwrap_err(),
            Error::Store(ReadError::Aborted)
        );
    }
}

#[test]
fn reads_maximum_header_in_bounded_chunks_without_reading_data() {
    let mut header = vec![b' '; MAX_HEADER_BYTES];
    header[..2].copy_from_slice(b"{}");
    let (mut store, mut artifact, cancel) = verified(file(&header, 0));
    let mut scratch = vec![0; MAX_HEADER_BYTES];
    let model = parse(
        &mut artifact,
        &mut store,
        &mut ReadBudget::new((8 + MAX_HEADER_BYTES) as u64),
        &cancel,
        &mut scratch,
        &mut [],
    )
    .unwrap();
    assert_eq!(model.data_start(), (8 + MAX_HEADER_BYTES) as u64);
    assert_eq!(store.reads.len(), 1 + MAX_HEADER_BYTES / READ_CHUNK_MAX);
    assert_eq!(
        store.reads.last(),
        Some(&(
            (8 + MAX_HEADER_BYTES - READ_CHUNK_MAX) as u64,
            READ_CHUNK_MAX
        ))
    );
}
