//! Directed review properties against the immutable I08 library.
//! No simulator, service, allocator, authority or native execution is modeled.
use qsim_job_protocol::{decode, encode, Gate, Identifiers, Request, ResultPage, Submit};

#[test]
fn every_single_byte_value_preserves_canonical_accepted_payloads() {
    let ids = Identifiers::new(1, 1).unwrap();
    let large = Identifiers::new(u64::MAX, u64::MAX).unwrap();
    let full = [Gate::Cx { target: 5, control: 0 }; 64];
    let requests = [
        Request::Submit(Submit::new(ids, 1, 0, 1, 1, &[]).unwrap()),
        Request::Submit(Submit::new(large, 6, u64::MAX, 4096, 1_048_576, &full).unwrap()),
        Request::Submit(Submit::new(ids, 2, 0, 4096, 1, &[Gate::H { target: 0 }]).unwrap()),
        Request::Cancel(ids),
        Request::Status(ids),
        Request::ResultPage(ResultPage::new(ids, 3).unwrap()),
    ];
    let mut accepted = 0;
    let mut rejected = 0;
    for request in requests {
        let original = encode(&request);
        assert_eq!(decode(&original), Ok(request));
        for offset in 0..256 {
            for value in 0..=u8::MAX {
                let mut bytes = original;
                bytes[offset] = value;
                match decode(&bytes) {
                    Ok(decoded) => {
                        assert_eq!(encode(&decoded), bytes, "normalization at byte {offset}, value {value}");
                        accepted += 1;
                    }
                    Err(_) => rejected += 1,
                }
            }
        }
    }
    assert_eq!(accepted + rejected, 393_216);
    assert!(accepted > 0 && rejected > 0);
    println!("G08 single-byte probes: {accepted} accepted canonically; {rejected} rejected");
}

#[test]
fn all_gate_counts_at_every_valid_qubit_count_roundtrip() {
    let ids = Identifiers::new(17, 19).unwrap();
    let mut checked = 0;
    for qubits in 1..=6 {
        let gate = if qubits == 1 { Gate::S { target: 0 } }
            else { Gate::Cx { target: qubits - 1, control: 0 } };
        let gates = [gate; 64];
        for count in 0..=64 {
            let request = Request::Submit(Submit::new(ids, qubits, 0, 1, 1, &gates[..count]).unwrap());
            let bytes = encode(&request);
            assert_eq!(&bytes[36..38], &(count as u16).to_le_bytes());
            assert!(bytes[64 + count * 3..].iter().all(|byte| *byte == 0));
            assert_eq!(decode(&bytes), Ok(request));
            checked += 1;
        }
    }
    assert_eq!(checked, 390);
    println!("G08 valid qubit/gate-count combinations: {checked}");
}
