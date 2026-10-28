use qsim_job_protocol::{
    decode, encode, Error, Gate, Identifiers, Request, ResultPage, Submit, MAX_GATES, MAX_SHOTS,
    MAX_WORK_BUDGET, PAYLOAD_BYTES,
};

fn ids() -> Identifiers {
    Identifiers::new(0x0102_0304_0506_0708, 0x1112_1314_1516_1718).unwrap()
}

fn submit(gates: &[Gate]) -> Request {
    Request::Submit(Submit::new(ids(), 6, u64::MAX, MAX_SHOTS, MAX_WORK_BUDGET, gates).unwrap())
}

#[test]
fn golden_bytes_fix_offsets_endianness_and_all_gate_codes() {
    let gates = [
        Gate::H { target: 0 },
        Gate::X { target: 1 },
        Gate::Z { target: 2 },
        Gate::S { target: 3 },
        Gate::Cx {
            target: 5,
            control: 4,
        },
    ];
    let request = submit(&gates);
    let mut golden = [0; 256];
    golden[..8].copy_from_slice(&[b'Q', b'J', b'0', b'1', 1, 0, 1, 6]);
    golden[8..24].copy_from_slice(&[
        8, 7, 6, 5, 4, 3, 2, 1, 0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11,
    ]);
    golden[24..32].fill(0xff);
    golden[32..38].copy_from_slice(&[0, 0x10, 0, 0, 5, 0]);
    golden[40..48].copy_from_slice(&[0, 0, 0x10, 0, 0, 0, 0, 0]);
    golden[64..79].copy_from_slice(&[1, 0, 0, 2, 1, 0, 3, 2, 0, 4, 3, 0, 5, 5, 4]);
    assert_eq!(encode(&request), golden);
    assert_eq!(decode(&golden), Ok(request));
    let Request::Submit(value) = request else {
        panic!("submit fixture")
    };
    assert_eq!(value.identifiers(), ids());
    assert_eq!(value.qubits(), 6);
    assert_eq!(value.seed(), u64::MAX);
    assert_eq!(value.shots(), 4096);
    assert_eq!(value.work_budget(), 1_048_576);
    assert_eq!(value.gates(), &gates);
}

#[test]
fn all_control_operations_and_pages_roundtrip_with_zero_unused_fields() {
    let mut controls = [Request::Cancel(ids()); 6];
    controls[1] = Request::Status(ids());
    for page in 0..4 {
        controls[2 + usize::from(page)] =
            Request::ResultPage(ResultPage::new(ids(), page).unwrap());
    }
    for request in controls {
        let encoded = encode(&request);
        assert_eq!(decode(&encoded), Ok(request));
        assert_eq!(encoded[7], 0);
        assert!(encoded[24..48]
            .iter()
            .chain(&encoded[49..])
            .all(|byte| *byte == 0));
        assert_eq!(request.identifiers(), ids());
        if let Request::ResultPage(page) = request {
            assert_eq!(encoded[48], page.page());
            assert_eq!(page.identifiers(), ids());
        } else {
            assert_eq!(encoded[48], 0);
        }
    }
    assert_eq!(encode(&Request::Cancel(ids()))[6], 2);
    assert_eq!(encode(&Request::Status(ids()))[6], 3);
    assert_eq!(encode(&controls[2])[6], 4);
}

#[test]
fn maximum_gate_array_uses_the_last_three_payload_bytes() {
    let gates = core::array::from_fn::<_, MAX_GATES, _>(|i| Gate::Cx {
        target: (i % 6) as u8,
        control: ((i + 1) % 6) as u8,
    });
    let request = submit(&gates);
    let encoded = encode(&request);
    assert_eq!(encoded.len(), PAYLOAD_BYTES);
    assert_eq!(&encoded[36..38], &[64, 0]);
    assert_eq!(&encoded[253..], &[5, 3, 4]);
    assert_eq!(decode(&encoded), Ok(request));
}

#[test]
fn limits_include_minima_maxima_empty_circuit_and_full_seed_range() {
    for job in [1, u64::MAX] {
        for qubits in [1, 6] {
            for shots in [1, MAX_SHOTS] {
                for budget in [1, MAX_WORK_BUDGET] {
                    for seed in [0, u64::MAX] {
                        let request = Request::Submit(
                            Submit::new(
                                Identifiers::new(job, job).unwrap(),
                                qubits,
                                seed,
                                shots,
                                budget,
                                &[],
                            )
                            .unwrap(),
                        );
                        assert_eq!(decode(&encode(&request)), Ok(request));
                    }
                }
            }
        }
    }
}

#[test]
fn typed_constructors_reject_out_of_range_input_before_encoding() {
    assert_eq!(Identifiers::new(0, 1), Err(Error::ZeroJobId));
    assert_eq!(Identifiers::new(1, 0), Err(Error::ZeroRequestId));
    for qubits in [0, 7, u8::MAX] {
        assert_eq!(Submit::new(ids(), qubits, 0, 1, 1, &[]), Err(Error::Qubits));
    }
    for shots in [0, 4097, u32::MAX] {
        assert_eq!(Submit::new(ids(), 6, 0, shots, 1, &[]), Err(Error::Shots));
    }
    for budget in [0, MAX_WORK_BUDGET + 1, u64::MAX] {
        assert_eq!(
            Submit::new(ids(), 6, 0, 1, budget, &[]),
            Err(Error::WorkBudget)
        );
    }
    assert_eq!(
        Submit::new(ids(), 6, 0, 1, 1, &[Gate::H { target: 0 }; 65]),
        Err(Error::GateCount)
    );
    for page in [4, u8::MAX] {
        assert_eq!(ResultPage::new(ids(), page), Err(Error::ResultPage));
    }
    for gate in [
        Gate::H { target: 6 },
        Gate::X { target: u8::MAX },
        Gate::Cx {
            target: 1,
            control: 6,
        },
        Gate::Cx {
            target: 2,
            control: 2,
        },
    ] {
        assert!(Submit::new(ids(), 6, 0, 1, 1, &[gate]).is_err());
    }
}

#[test]
fn every_truncation_and_trailing_bytes_are_rejected() {
    let valid = encode(&submit(&[]));
    for length in 0..PAYLOAD_BYTES {
        assert_eq!(
            decode(&valid[..length]),
            Err(Error::Length { actual: length })
        );
    }
    for extended in [&[0; 257][..], &[0; 512][..]] {
        assert_eq!(
            decode(extended),
            Err(Error::Length {
                actual: extended.len()
            })
        );
    }
}

#[test]
fn corrupted_magic_version_operation_and_zero_ids_are_rejected() {
    let valid = encode(&submit(&[]));
    for at in 0..4 {
        let mut bytes = valid;
        bytes[at] ^= 0x80;
        assert_eq!(decode(&bytes), Err(Error::Magic));
    }
    for version in [0_u16, 2, u16::MAX] {
        let mut bytes = valid;
        bytes[4..6].copy_from_slice(&version.to_le_bytes());
        assert_eq!(decode(&bytes), Err(Error::Version { observed: version }));
    }
    for operation in [0, 5, u8::MAX] {
        let mut bytes = valid;
        bytes[6] = operation;
        assert_eq!(
            decode(&bytes),
            Err(Error::Operation {
                observed: operation
            })
        );
    }
    for (range, error) in [(8..16, Error::ZeroJobId), (16..24, Error::ZeroRequestId)] {
        let mut bytes = valid;
        bytes[range].fill(0);
        assert_eq!(decode(&bytes), Err(error));
    }
}

#[test]
fn every_reserved_header_byte_is_rejected_for_every_operation() {
    for request in [
        submit(&[]),
        Request::Cancel(ids()),
        Request::Status(ids()),
        Request::ResultPage(ResultPage::new(ids(), 3).unwrap()),
    ] {
        for at in (38..40).chain(49..64) {
            let mut bytes = encode(&request);
            bytes[at] = 1;
            assert_eq!(decode(&bytes), Err(Error::ReservedBytes));
        }
    }
}

#[test]
fn wire_limits_are_checked_before_gate_index_conversion() {
    let valid = encode(&submit(&[Gate::H { target: 0 }]));
    for qubits in [0, 7, u8::MAX] {
        let mut bytes = valid;
        bytes[7] = qubits;
        assert_eq!(decode(&bytes), Err(Error::Qubits));
    }
    for count in [65_u16, u16::MAX] {
        let mut bytes = valid;
        bytes[36..38].copy_from_slice(&count.to_le_bytes());
        assert_eq!(decode(&bytes), Err(Error::GateCount));
    }
    for shots in [0_u32, 4097, u32::MAX] {
        let mut bytes = valid;
        bytes[32..36].copy_from_slice(&shots.to_le_bytes());
        assert_eq!(decode(&bytes), Err(Error::Shots));
    }
    for budget in [0_u64, MAX_WORK_BUDGET + 1, u64::MAX] {
        let mut bytes = valid;
        bytes[40..48].copy_from_slice(&budget.to_le_bytes());
        assert_eq!(decode(&bytes), Err(Error::WorkBudget));
    }
    let mut bytes = valid;
    bytes[65] = 255;
    assert_eq!(decode(&bytes), Err(Error::GateTarget { index: 0 }));
    bytes[64..67].copy_from_slice(&[5, 0, 255]);
    assert_eq!(decode(&bytes), Err(Error::GateControl { index: 0 }));
    bytes[64..67].copy_from_slice(&[5, 0, 0]);
    assert_eq!(
        decode(&bytes),
        Err(Error::EqualControlAndTarget { index: 0 })
    );
    bytes = valid;
    bytes[48] = 1;
    assert_eq!(decode(&bytes), Err(Error::UnexpectedFields));
}

#[test]
fn gate_codes_and_single_qubit_control_bytes_are_strict() {
    let valid = encode(&submit(&[Gate::H { target: 0 }]));
    for opcode in [0, 6, u8::MAX] {
        let mut bytes = valid;
        bytes[64] = opcode;
        assert_eq!(
            decode(&bytes),
            Err(Error::GateOperation {
                index: 0,
                observed: opcode
            })
        );
    }
    for opcode in 1..=4 {
        let mut bytes = valid;
        bytes[64] = opcode;
        bytes[66] = 1;
        assert_eq!(decode(&bytes), Err(Error::SingleQubitControl { index: 0 }));
    }
}

#[test]
fn every_unused_gate_byte_must_remain_zero() {
    for gates in [&[][..], &[Gate::H { target: 0 }][..]] {
        let valid = encode(&submit(gates));
        for at in 64 + gates.len() * 3..PAYLOAD_BYTES {
            let mut bytes = valid;
            bytes[at] = 1;
            assert_eq!(decode(&bytes), Err(Error::UnusedGateBytes));
        }
    }
}

#[test]
fn non_submit_operations_reject_every_unrelated_field_byte() {
    for request in [
        Request::Cancel(ids()),
        Request::Status(ids()),
        Request::ResultPage(ResultPage::new(ids(), 3).unwrap()),
    ] {
        for at in core::iter::once(7).chain(24..48).chain(49..256) {
            let mut bytes = encode(&request);
            bytes[at] = 1;
            assert!(decode(&bytes).is_err(), "accepted unrelated offset {at}");
        }
    }
    for request in [Request::Cancel(ids()), Request::Status(ids())] {
        let mut bytes = encode(&request);
        bytes[48] = 1;
        assert_eq!(decode(&bytes), Err(Error::UnexpectedFields));
    }
    for page in [4, u8::MAX] {
        let mut bytes = encode(&Request::ResultPage(ResultPage::new(ids(), 0).unwrap()));
        bytes[48] = page;
        assert_eq!(decode(&bytes), Err(Error::ResultPage));
    }
}

#[test]
fn any_accepted_single_byte_mutation_roundtrips_without_normalization() {
    let full = [Gate::Cx {
        target: 1,
        control: 0,
    }; MAX_GATES];
    for request in [
        submit(&[]),
        submit(&full),
        Request::Cancel(ids()),
        Request::Status(ids()),
        Request::ResultPage(ResultPage::new(ids(), 3).unwrap()),
    ] {
        let original = encode(&request);
        for at in 0..PAYLOAD_BYTES {
            for replacement in [0, 1, 2, 0x7f, 0xff] {
                let mut mutated = original;
                mutated[at] = replacement;
                if let Ok(decoded) = decode(&mutated) {
                    assert_eq!(encode(&decoded), mutated, "normalized byte at {at}");
                }
            }
        }
    }
}
