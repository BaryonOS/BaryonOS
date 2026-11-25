//! REL-05 step S3: host tests for the policy-session, NV and PCR codecs.
//! Golden bytes follow TPM 2.0 Part 3 command layouts; they prove encoding
//! and decoding only. Acceptance by swtpm is step S5.
use nieuw_os_tpm_crb::codec::{
    self, Auth, Error, NvDefinition, NvIndex, Password, PolicySession, Request, Response,
    MAX_COMMAND,
};

// The real S2 constants, compiled from update-core's source: the cross-check
// needs no dependency edge or lockfile change.
#[allow(dead_code)]
#[path = "../../../crates/update-core/src/trial_tpm.rs"]
mod trial_tpm;

fn none() -> Password<'static> {
    Password::new(&[]).unwrap()
}
fn session() -> PolicySession {
    PolicySession::new(0x0300_0000).unwrap()
}
fn index(v: u32) -> NvIndex {
    NvIndex::new(v).unwrap()
}
fn enc(request: Request<'_>) -> Vec<u8> {
    let mut out = [0u8; MAX_COMMAND];
    let c = codec::encode(request, &mut out).unwrap();
    let bytes = c.bytes().to_vec();
    assert!(bytes.len() <= MAX_COMMAND);
    assert_eq!(
        u32::from_be_bytes(bytes[2..6].try_into().unwrap()) as usize,
        bytes.len()
    );
    bytes
}
fn enc_err(request: Request<'_>) -> Error {
    let mut out = [0xa5u8; 256];
    match codec::encode(request, &mut out) {
        Ok(_) => panic!("encode accepted"),
        Err(e) => {
            assert!(out.iter().all(|&b| b == 0xa5), "output touched on error");
            e
        }
    }
}
fn head(tag: u16, cc: u32, body: &[u8]) -> Vec<u8> {
    let mut v = tag.to_be_bytes().to_vec();
    v.extend_from_slice(&((10 + body.len()) as u32).to_be_bytes());
    v.extend_from_slice(&cc.to_be_bytes());
    v.extend_from_slice(body);
    v
}
fn pw_area(pw: &[u8]) -> Vec<u8> {
    let mut v = ((9 + pw.len()) as u32).to_be_bytes().to_vec();
    v.extend_from_slice(&0x4000_0009u32.to_be_bytes());
    v.extend_from_slice(&[0, 0, 0]);
    v.extend_from_slice(&(pw.len() as u16).to_be_bytes());
    v.extend_from_slice(pw);
    v
}
fn rsp(tag: u16, body: &[u8]) -> Vec<u8> {
    head(tag, 0, body)
}
/// Session response: parameterSize, parameters, then the trailer.
fn rsp_s(params: &[u8], trailer: &[u8]) -> Vec<u8> {
    let mut b = (params.len() as u32).to_be_bytes().to_vec();
    b.extend_from_slice(params);
    b.extend_from_slice(trailer);
    rsp(codec::SESSIONS, &b)
}
const PW_TAIL: [u8; 5] = [0, 0, 1, 0, 0];
fn policy_tail() -> Vec<u8> {
    let mut t = vec![0, 32];
    t.extend_from_slice(&[0x5a; 32]);
    t.extend_from_slice(&[0, 0, 0]);
    t
}
fn decode(request: Request<'_>, response: &[u8]) -> Result<Response, Error> {
    let mut out = [0u8; MAX_COMMAND];
    codec::encode(request, &mut out).unwrap().decode(response)
}
/// Every strict prefix fails, both as-is (size field disagrees) and with the
/// size field rewritten to the prefix length; no single-bit flip panics.
fn truncations_fail(request: impl Fn() -> Request<'static>, good: &[u8]) {
    assert!(decode(request(), good).is_ok());
    for len in 0..good.len() {
        assert!(decode(request(), &good[..len]).is_err(), "prefix {len}");
        if len >= 6 {
            let mut p = good[..len].to_vec();
            p[2..6].copy_from_slice(&(len as u32).to_be_bytes());
            assert!(decode(request(), &p).is_err(), "resized prefix {len}");
        }
    }
    for i in 0..good.len() {
        for bit in 0..8 {
            let mut m = good.to_vec();
            m[i] ^= 1 << bit;
            let _ = decode(request(), &m);
        }
    }
}

#[test]
fn start_policy_session_golden_bytes_and_bounds() {
    let nonce = [7u8; 32];
    let mut body = Vec::new();
    body.extend_from_slice(&0x4000_0007u32.to_be_bytes()); // tpmKey NULL: unsalted
    body.extend_from_slice(&0x4000_0007u32.to_be_bytes()); // bind NULL: unbound
    body.extend_from_slice(&[0, 32]);
    body.extend_from_slice(&nonce);
    // empty encryptedSalt, TPM_SE_POLICY, symmetric TPM_ALG_NULL, authHash SHA-256
    body.extend_from_slice(&[0, 0, 0x01, 0x00, 0x10, 0x00, 0x0b]);
    let c = enc(Request::StartPolicySession {
        nonce_caller: &nonce,
    });
    assert_eq!(c, head(codec::NO_SESSIONS, 0x176, &body));
    assert_eq!(c.len(), 59);
    assert_eq!(
        enc(Request::StartPolicySession {
            nonce_caller: &nonce[..16]
        })
        .len(),
        43
    );
    let long = [1u8; 33];
    for n in [0, 15, 33] {
        assert_eq!(
            enc_err(Request::StartPolicySession {
                nonce_caller: &long[..n]
            }),
            Error::Input
        );
    }
}

#[test]
fn start_policy_session_response_decodes_handle_and_nonce() {
    let req = || Request::StartPolicySession {
        nonce_caller: &[9u8; 16],
    };
    let mut body = 0x0300_0001u32.to_be_bytes().to_vec();
    body.extend_from_slice(&[0, 32]);
    body.extend_from_slice(&[0x33; 32]);
    let good = rsp(codec::NO_SESSIONS, &body);
    assert_eq!(
        decode(req(), &good),
        Ok(Response::Session {
            handle: PolicySession::new(0x0300_0001).unwrap(),
            nonce_tpm: [0x33; 32]
        })
    );
    truncations_fail(req, &good);
    // An HMAC session (0x02) or a transient handle is not a policy session.
    for h in [0x0200_0000u32, 0x8000_0000] {
        let mut b = body.clone();
        b[..4].copy_from_slice(&h.to_be_bytes());
        assert_eq!(
            decode(req(), &rsp(codec::NO_SESSIONS, &b)),
            Err(Error::Protocol)
        );
    }
    let mut short = 0x0300_0001u32.to_be_bytes().to_vec();
    short.extend_from_slice(&[0, 16]);
    short.extend_from_slice(&[0x33; 16]);
    assert_eq!(
        decode(req(), &rsp(codec::NO_SESSIONS, &short)),
        Err(Error::Length)
    );
    assert_eq!(
        decode(req(), &rsp(codec::SESSIONS, &body)),
        Err(Error::Protocol)
    );
    assert!(PolicySession::new(0x0400_0000).is_err());
}

#[test]
fn policy_pcr_and_command_code_golden_bytes() {
    let mut body = 0x0300_0000u32.to_be_bytes().to_vec();
    body.extend_from_slice(&[0, 0]); // empty pcrDigest: TPM uses current value
    body.extend_from_slice(&trial_tpm::PCR_SELECTION_PHASE);
    let c = enc(Request::PolicyPcr {
        session: session(),
        pcr: 13,
    });
    assert_eq!(
        c,
        head(codec::NO_SESSIONS, trial_tpm::TPM_CC_POLICY_PCR, &body)
    );
    assert_eq!(c.len(), 26);
    let mut body = 0x0300_0000u32.to_be_bytes().to_vec();
    body.extend_from_slice(&trial_tpm::TPM_CC_NV_WRITE.to_be_bytes());
    let c = enc(Request::PolicyCommandCode {
        session: session(),
        code: trial_tpm::TPM_CC_NV_WRITE,
    });
    assert_eq!(
        c,
        head(
            codec::NO_SESSIONS,
            trial_tpm::TPM_CC_POLICY_COMMAND_CODE,
            &body
        )
    );
    assert_eq!(c.len(), 18);
    // Selection bit layout across the three select bytes.
    for (pcr, byte, bit) in [
        (0u8, 0usize, 0x01u8),
        (7, 0, 0x80),
        (8, 1, 0x01),
        (23, 2, 0x80),
    ] {
        let c = enc(Request::PolicyPcr {
            session: session(),
            pcr,
        });
        assert_eq!(&c[16..23], &[0, 0, 0, 1, 0, 0x0b, 3]);
        let mut sel = [0u8; 3];
        sel[byte] = bit;
        assert_eq!(&c[23..26], &sel);
    }
    assert_eq!(
        enc_err(Request::PolicyPcr {
            session: session(),
            pcr: 24
        }),
        Error::Input
    );
    let ok = rsp(codec::NO_SESSIONS, &[]);
    assert_eq!(
        decode(
            Request::PolicyPcr {
                session: session(),
                pcr: 13
            },
            &ok
        ),
        Ok(Response::Empty)
    );
    let extra = rsp(codec::NO_SESSIONS, &[0]);
    assert_eq!(
        decode(
            Request::PolicyCommandCode {
                session: session(),
                code: 1
            },
            &extra
        ),
        Err(Error::Length)
    );
    // A TPM error stays visible and is never normalized to success.
    let fail = head(codec::NO_SESSIONS, 0x99d, &[]);
    assert_eq!(
        decode(
            Request::PolicyPcr {
                session: session(),
                pcr: 13
            },
            &fail
        ),
        Err(Error::Tpm(0x99d))
    );
}

fn define(spec: &trial_tpm::NvSpec, owner: &[u8]) -> Vec<u8> {
    let policy = spec.auth_policy;
    enc(Request::NvDefineSpace {
        owner_auth: Password::new(owner).unwrap(),
        public: NvDefinition {
            index: index(spec.handle),
            attributes: spec.attributes,
            auth_policy: policy.as_ref(),
            size: spec.size,
        },
    })
}

#[test]
fn nv_define_space_public_area_equals_update_core_nvspec() {
    let mut specs = vec![
        trial_tpm::NV_TRIAL,
        trial_tpm::NV_CONFIRM,
        trial_tpm::NV_FLOOR,
        trial_tpm::NV_GEN,
    ];
    for d in 0..trial_tpm::ANCHOR_DOMAINS {
        let (counter, slot) = trial_tpm::anchor_domain(d).unwrap();
        specs.extend([counter, slot]);
    }
    for spec in &specs {
        for owner in [&[][..], &[0x11; 32][..]] {
            let c = define(spec, owner);
            let (public, len) = spec.encode_public();
            assert_eq!(&c[10..14], &0x4000_0001u32.to_be_bytes());
            assert_eq!(&c[14..27 + owner.len()], &pw_area(owner)[..]);
            let at = 27 + owner.len();
            assert_eq!(&c[at..at + 2], &[0, 0], "empty index authValue");
            assert_eq!(&c[at + 2..at + 4], &(len as u16).to_be_bytes());
            assert_eq!(&c[at + 4..], &public[..len], "{:#x}", spec.handle);
            // Size proof: 45 + |ownerAuth| + |authPolicy| <= 109.
            let expected = 45 + owner.len() + spec.auth_policy.map_or(0, |_| 32);
            assert_eq!(c.len(), expected);
            assert!(c.len() <= 109);
        }
    }
}

#[test]
fn nv_define_space_rejects_bad_type_or_size_before_writing() {
    let def = |attributes, size| NvDefinition {
        index: index(0x0100_B300),
        attributes,
        auth_policy: None,
        size,
    };
    for (attributes, size) in [
        (trial_tpm::ATTR_UPDATE_ORDINARY, 0),
        (trial_tpm::ATTR_UPDATE_ORDINARY, 65),
        (trial_tpm::ATTR_UPDATE_COUNTER, 64),
        (trial_tpm::ATTR_UPDATE_COUNTER, 7),
        (2 << 4, 8), // bit field
        (4 << 4, 8), // extend
        (8 << 4, 8), // pin fail
    ] {
        assert_eq!(
            enc_err(Request::NvDefineSpace {
                owner_auth: none(),
                public: def(attributes, size)
            }),
            Error::Input
        );
    }
    let req = || Request::NvDefineSpace {
        owner_auth: none(),
        public: NvDefinition {
            index: NvIndex::new(0x0100_B303).unwrap(),
            attributes: trial_tpm::ATTR_UPDATE_COUNTER,
            auth_policy: Some(&trial_tpm::POLICY_LOADER_INCREMENT),
            size: 8,
        },
    };
    let good = rsp_s(&[], &PW_TAIL);
    assert_eq!(decode(req(), &good), Ok(Response::Empty));
    let mut out = [0u8; MAX_COMMAND];
    assert!(codec::encode(req(), &mut out).unwrap().is_nv_mutation());
    truncations_fail(req, &good);
}

#[test]
fn nv_write_policy_and_password_sizes_are_bounded() {
    let data = [0xabu8; 64];
    let c = enc(Request::NvWrite {
        index: index(0x0100_B301),
        auth: Auth::Policy(session()),
        data: &data,
        offset: 0,
    });
    let mut body = Vec::new();
    body.extend_from_slice(&0x0100_B301u32.to_be_bytes());
    body.extend_from_slice(&0x0100_B301u32.to_be_bytes());
    // Policy session: empty nonce, continueSession clear, empty HMAC.
    body.extend_from_slice(&[0, 0, 0, 9, 0x03, 0, 0, 0, 0, 0, 0, 0, 0]);
    body.extend_from_slice(&[0, 64]);
    body.extend_from_slice(&data);
    body.extend_from_slice(&[0, 0]);
    assert_eq!(c, head(codec::SESSIONS, trial_tpm::TPM_CC_NV_WRITE, &body));
    assert_eq!(c.len(), 99);
    // Anchor slot (AUTHWRITE, empty authValue): 35 + 64 fits.
    let c = enc(Request::NvWrite {
        index: index(0x0100_B311),
        auth: Auth::Password(none()),
        data: &data,
        offset: 0,
    });
    assert_eq!(c.len(), 99);
    assert_eq!(&c[18..31], &pw_area(&[])[..]);
    // 32-byte password + 64 data = 131 > 128: refused, nothing written.
    let pw = [1u8; 32];
    assert_eq!(
        enc_err(Request::NvWrite {
            index: index(0x0100_B311),
            auth: Auth::Password(Password::new(&pw).unwrap()),
            data: &data,
            offset: 0,
        }),
        Error::Length
    );
    let d = [0u8; 65];
    for n in [0usize, 65] {
        assert_eq!(
            enc_err(Request::NvWrite {
                index: index(0x0100_B300),
                auth: Auth::Policy(session()),
                data: &d[..n],
                offset: 0,
            }),
            Error::Input
        );
    }
}

#[test]
fn nv_write_response_trailer_must_match_the_session_kind() {
    let policy = || Request::NvWrite {
        index: NvIndex::new(0x0100_B300).unwrap(),
        auth: Auth::Policy(PolicySession::new(0x0300_0000).unwrap()),
        data: &[1, 2, 3],
        offset: 0,
    };
    let password = || Request::NvWrite {
        index: NvIndex::new(0x0100_B311).unwrap(),
        auth: Auth::Password(none()),
        data: &[1, 2, 3],
        offset: 0,
    };
    let good_policy = rsp_s(&[], &policy_tail());
    let good_password = rsp_s(&[], &PW_TAIL);
    assert_eq!(decode(policy(), &good_policy), Ok(Response::Empty));
    assert_eq!(decode(password(), &good_password), Ok(Response::Empty));
    assert!(decode(policy(), &good_password).is_err());
    assert!(decode(password(), &good_policy).is_err());
    truncations_fail(policy, &good_policy);
    truncations_fail(password, &good_password);
    // continueSession echoed set, non-empty HMAC, or a short nonceTPM.
    let mut t = policy_tail();
    t[34] = 1;
    assert_eq!(decode(policy(), &rsp_s(&[], &t)), Err(Error::Protocol));
    let mut t = policy_tail();
    t[36] = 1;
    assert_eq!(decode(policy(), &rsp_s(&[], &t)), Err(Error::Protocol));
    let mut t = vec![0, 16];
    t.extend_from_slice(&[0; 16 + 3]);
    assert!(decode(policy(), &rsp_s(&[], &t)).is_err());
    assert_eq!(
        decode(policy(), &rsp_s(&[0], &policy_tail())),
        Err(Error::Length)
    );
    let mut out = [0u8; MAX_COMMAND];
    assert!(codec::encode(policy(), &mut out).unwrap().is_nv_mutation());
}

#[test]
fn nv_read_ordinary_returns_exact_bytes() {
    let req = || Request::NvRead {
        index: NvIndex::new(0x0100_B302).unwrap(),
        auth: none(),
        size: 64,
        offset: 0,
    };
    let c = enc(req());
    let mut body = Vec::new();
    body.extend_from_slice(&0x0100_B302u32.to_be_bytes());
    body.extend_from_slice(&0x0100_B302u32.to_be_bytes());
    body.extend_from_slice(&pw_area(&[]));
    body.extend_from_slice(&[0, 64, 0, 0]);
    assert_eq!(c, head(codec::SESSIONS, 0x14e, &body));
    assert_eq!(c.len(), 35);
    let data: Vec<u8> = (0..64).collect();
    let mut params = vec![0, 64];
    params.extend_from_slice(&data);
    let good = rsp_s(&params, &PW_TAIL);
    let Ok(Response::Data { len, bytes }) = decode(req(), &good) else {
        panic!("decode");
    };
    assert_eq!((len, &bytes[..]), (64, &data[..]));
    truncations_fail(req, &good);
    let mut wrong = vec![0, 63];
    wrong.extend_from_slice(&data[..63]);
    assert_eq!(decode(req(), &rsp_s(&wrong, &PW_TAIL)), Err(Error::Length));
    for size in [0u16, 65] {
        assert_eq!(
            enc_err(Request::NvRead {
                index: index(0x0100_B302),
                auth: none(),
                size,
                offset: 0
            }),
            Error::Input
        );
    }
}

#[test]
fn pcr_read_decodes_one_sha256_digest_and_refuses_empty_bank() {
    let req = || Request::PcrRead { pcr: 13 };
    let c = enc(req());
    assert_eq!(
        c,
        head(codec::NO_SESSIONS, 0x17e, &trial_tpm::PCR_SELECTION_PHASE)
    );
    assert_eq!(c.len(), 20);
    let mut body = 77u32.to_be_bytes().to_vec();
    body.extend_from_slice(&trial_tpm::PCR_SELECTION_PHASE);
    body.extend_from_slice(&[0, 0, 0, 1, 0, 32]);
    body.extend_from_slice(&trial_tpm::P_OPEN);
    let good = rsp(codec::NO_SESSIONS, &body);
    assert_eq!(
        decode(req(), &good),
        Ok(Response::Pcr {
            update_counter: 77,
            digest: trial_tpm::P_OPEN
        })
    );
    truncations_fail(req, &good);
    // Unallocated SHA-256 bank: empty selection and no digests.
    let mut empty = 77u32.to_be_bytes().to_vec();
    empty.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);
    assert!(decode(req(), &rsp(codec::NO_SESSIONS, &empty)).is_err());
    // Same length, but another PCR or another bank selected.
    for (i, v) in [(12usize, 0x40u8), (9, 0x04)] {
        let mut b = body.clone();
        b[i] = v;
        assert_eq!(
            decode(req(), &rsp(codec::NO_SESSIONS, &b)),
            Err(Error::Protocol)
        );
    }
    assert_eq!(enc_err(Request::PcrRead { pcr: 24 }), Error::Input);
}

#[test]
fn pcr_extend_golden_bytes_and_size() {
    let req = || Request::PcrExtend {
        pcr: 13,
        auth: none(),
        digest: &trial_tpm::DIGEST_OPEN,
    };
    let mut body = 13u32.to_be_bytes().to_vec();
    body.extend_from_slice(&pw_area(&[]));
    body.extend_from_slice(&[0, 0, 0, 1, 0, 0x0b]);
    body.extend_from_slice(&trial_tpm::DIGEST_OPEN);
    let c = enc(req());
    assert_eq!(c, head(codec::SESSIONS, 0x182, &body));
    assert_eq!(c.len(), 65);
    assert_eq!(
        enc_err(Request::PcrExtend {
            pcr: 24,
            auth: none(),
            digest: &[0; 32]
        }),
        Error::Input
    );
    let good = rsp_s(&[], &PW_TAIL);
    assert_eq!(decode(req(), &good), Ok(Response::Empty));
    truncations_fail(req, &good);
}

fn public_body(handle: u32, attributes: u32, policy: &[u8], size: u16) -> Vec<u8> {
    let mut nv = handle.to_be_bytes().to_vec();
    nv.extend_from_slice(&[0, 0x0b]);
    nv.extend_from_slice(&attributes.to_be_bytes());
    nv.extend_from_slice(&(policy.len() as u16).to_be_bytes());
    nv.extend_from_slice(policy);
    nv.extend_from_slice(&size.to_be_bytes());
    let mut body = (nv.len() as u16).to_be_bytes().to_vec();
    body.extend_from_slice(&nv);
    body.extend_from_slice(&[0, 34, 0, 0x0b]);
    body.extend_from_slice(&[0xee; 32]);
    rsp(codec::NO_SESSIONS, &body)
}
fn public_rsp(spec: &trial_tpm::NvSpec, attributes: u32) -> Vec<u8> {
    let policy = spec.auth_policy.map_or(vec![], |p| p.to_vec());
    public_body(spec.handle, attributes, &policy, spec.size)
}

#[test]
fn nv_read_public_area_decodes_type_attributes_size_and_policy() {
    let (anchor_counter, anchor_slot) = trial_tpm::anchor_domain(0).unwrap();
    let mut name = [0xeeu8; 34];
    name[..2].copy_from_slice(&[0, 0x0b]);
    for spec in [
        trial_tpm::NV_TRIAL,
        trial_tpm::NV_GEN,
        anchor_counter,
        anchor_slot,
    ] {
        let req = || Request::NvReadPublicArea {
            index: NvIndex::new(spec.handle).unwrap(),
        };
        assert_eq!(enc(req()).len(), 14);
        let written = spec.attributes | trial_tpm::TPMA_NV_WRITTEN;
        let good = public_rsp(&spec, written);
        let Ok(Response::NvPublicArea(area)) = decode(req(), &good) else {
            panic!("decode {:#x}", spec.handle);
        };
        assert_eq!(area.index.raw(), spec.handle);
        assert_eq!(area.attributes, written);
        assert_eq!(area.size, spec.size);
        assert_eq!(area.name, name);
        let counter = spec.attributes & trial_tpm::TPM_NT_COUNTER != 0;
        assert_eq!(area.nv_type(), u32::from(counter));
        // The loader's fail-closed comparison accepts exactly the decoded fields.
        assert!(spec.matches(0x000b, area.attributes, area.auth_policy(), area.size));
        truncations_fail(req, &good);
        // A substituted policy decodes, but no longer matches.
        let Ok(Response::NvPublicArea(area)) = decode(
            req(),
            &public_body(spec.handle, written, &[0; 32], spec.size),
        ) else {
            panic!("decode");
        };
        assert!(!spec.matches(0x000b, area.attributes, area.auth_policy(), area.size));
    }
}

#[test]
fn nv_read_public_area_refuses_foreign_index_alg_and_odd_policy() {
    let spec = trial_tpm::NV_TRIAL;
    let req = || Request::NvReadPublicArea {
        index: NvIndex::new(0x0100_B300).unwrap(),
    };
    let good = public_rsp(&spec, spec.attributes);
    let mut foreign = good.clone();
    foreign[15] ^= 1; // nvIndex low byte
    assert_eq!(decode(req(), &foreign), Err(Error::Protocol));
    let mut alg = good.clone();
    alg[17] = 0x04; // nameAlg SHA-1
    assert_eq!(decode(req(), &alg), Err(Error::Protocol));
    let mut name_alg = good.clone();
    let n = name_alg.len();
    name_alg[n - 33] = 0x04; // name's hash algorithm
    assert_eq!(decode(req(), &name_alg), Err(Error::Protocol));
    // A 16-byte authPolicy with consistent sizes is not a SHA-256 digest.
    let odd = public_body(spec.handle, spec.attributes, &[0; 16], 64);
    assert_eq!(decode(req(), &odd), Err(Error::Length));
}

#[test]
fn every_new_command_fits_the_128_byte_bound_at_its_maximum() {
    let pw = [0u8; 32];
    let pw = Password::new(&pw).unwrap();
    let data = [0u8; 64];
    let policy = [0u8; 32];
    let max = [
        (
            Request::StartPolicySession {
                nonce_caller: &[0; 32],
            },
            59,
        ),
        (
            Request::PolicyPcr {
                session: session(),
                pcr: 23,
            },
            26,
        ),
        (
            Request::PolicyCommandCode {
                session: session(),
                code: u32::MAX,
            },
            18,
        ),
        (
            Request::NvDefineSpace {
                owner_auth: pw,
                public: NvDefinition {
                    index: index(0x01ff_ffff),
                    attributes: trial_tpm::ATTR_UPDATE_ORDINARY,
                    auth_policy: Some(&policy),
                    size: 64,
                },
            },
            109,
        ),
        (
            Request::NvWrite {
                index: index(0x0100_0000),
                auth: Auth::Policy(session()),
                data: &data,
                offset: u16::MAX,
            },
            99,
        ),
        (
            Request::NvRead {
                index: index(0x0100_0000),
                auth: pw,
                size: 64,
                offset: 0,
            },
            67,
        ),
        (Request::PcrRead { pcr: 23 }, 20),
        (
            Request::PcrExtend {
                pcr: 23,
                auth: pw,
                digest: &[0; 32],
            },
            97,
        ),
        (
            Request::NvReadPublicArea {
                index: index(0x0100_0000),
            },
            14,
        ),
    ];
    for (request, len) in max {
        assert_eq!(enc(request).len(), len);
        assert!(len <= MAX_COMMAND);
    }
    // An output buffer one byte short is refused without any write.
    let mut short = [0xa5u8; 13];
    assert_eq!(
        codec::encode(
            Request::NvReadPublicArea {
                index: index(0x0100_0000)
            },
            &mut short
        )
        .err(),
        Some(Error::Length)
    );
    assert!(short.iter().all(|&b| b == 0xa5));
}
