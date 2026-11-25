//! REL-05 step S3: host tests for the policy-session, NV and PCR codecs.
//! Golden bytes follow TPM 2.0 Part 3 command layouts; they prove encoding
//! and decoding only. Acceptance by swtpm is step S5.
use nieuw_os_tpm_crb::codec::{
    self, Auth, Error, Hierarchy, NvDefinition, NvIndex, Password, PolicySession, Request,
    Response, MAX_COMMAND, MAX_NV_DATA,
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
    PolicySession::new(0x0300_0000, 32).unwrap()
}
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
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
    policy_tail_n(32)
}
fn policy_tail_n(n: u8) -> Vec<u8> {
    let mut t = vec![0, n];
    t.extend_from_slice(&vec![0x5a; usize::from(n)]);
    t.extend_from_slice(&[0, 0, 0]);
    t
}
fn decode(request: Request<'_>, response: &[u8]) -> Result<Response, Error> {
    let mut out = [0u8; MAX_COMMAND];
    codec::encode(request, &mut out).unwrap().decode(response)
}
/// Every strict prefix fails, both as-is (size field disagrees) and with the
/// size field rewritten to the prefix length; no single-bit flip panics.
fn truncations_fail<'a>(request: impl Fn() -> Request<'a>, good: &[u8]) {
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
    for n in [16u8, 32] {
        let caller = [9u8; 32];
        let req = || Request::StartPolicySession {
            nonce_caller: &caller[..usize::from(n)],
        };
        let reply = |m: u8| {
            let mut body = 0x0300_0001u32.to_be_bytes().to_vec();
            body.extend_from_slice(&[0, m]);
            body.extend_from_slice(&vec![0x33; usize::from(m)]);
            body
        };
        let body = reply(n);
        let good = rsp(codec::NO_SESSIONS, &body);
        let mut nonce_tpm = [0; 32];
        nonce_tpm[..usize::from(n)].fill(0x33);
        assert_eq!(
            decode(req(), &good),
            Ok(Response::Session {
                handle: PolicySession::new(0x0300_0001, n).unwrap(),
                nonce_tpm
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
        // nonceTPM must have the caller's nonce size (Part 1, 16.6.3.2).
        for m in [16u8, 17, 31, 32] {
            if m != n {
                assert_eq!(
                    decode(req(), &rsp(codec::NO_SESSIONS, &reply(m))),
                    Err(Error::Length)
                );
            }
        }
        assert_eq!(
            decode(req(), &rsp(codec::SESSIONS, &body)),
            Err(Error::Protocol)
        );
    }
    assert!(PolicySession::new(0x0400_0000, 32).is_err());
    for n in [0, 15, 33] {
        assert!(PolicySession::new(0x0300_0000, n).is_err());
    }
}

#[test]
fn swtpm_transcript_policy_session_round_trip() {
    // docs/verification/b03-rel05-probe-20261125/transcript-u1u7.txt, phase A.
    let start = Request::StartPolicySession {
        nonce_caller: b"BaryonTrialNonce",
    };
    assert_eq!(
        enc(Request::StartPolicySession {
            nonce_caller: b"BaryonTrialNonce"
        }),
        hex("80010000002b0000017640000007400000070010426172796f6e547269616c4e6f6e63650000010010000b")
    );
    let Ok(Response::Session { handle, .. }) = decode(
        start,
        &hex("80010000002000000000030000000010573a543f2c4a88a89e76664b99f24a94"),
    ) else {
        panic!("swtpm StartAuthSession reply refused");
    };
    assert_eq!((handle.raw(), handle.nonce_len()), (0x0300_0000, 16));
    let mut data = b"NOSTRL01".to_vec();
    data.resize(64, 0);
    let write = || Request::NvWrite {
        index: index(0x0100_B300),
        auth: Auth::Policy(handle),
        data: &data,
        offset: 0,
    };
    assert_eq!(
        enc(write()),
        hex("800200000063000001370100b3000100b3000000000903000000000000000000404e4f5354524c303100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")
    );
    let w = hex("80020000002300000000000000000010fb4096445e353dfe96427b981a4e9cce000000");
    assert_eq!(decode(write(), &w), Ok(Response::Empty));
    truncations_fail(write, &w);
    let increment = || Request::NvIncrement {
        index: index(0x0100_B303),
        auth: Auth::Policy(handle),
    };
    assert_eq!(
        enc(increment()),
        hex("80020000001f000001340100b3030100b30300000009030000000000000000")
    );
    let i = hex("80020000002300000000000000000010341cb95e66e731483f01d63dc82dd59e000000");
    assert_eq!(decode(increment(), &i), Ok(Response::Empty));
    let mut out = [0u8; MAX_COMMAND];
    assert!(codec::encode(increment(), &mut out)
        .unwrap()
        .is_nv_mutation());
    // A 32-byte session must not accept the 16-byte trailer, nor the reverse.
    let wide = || Request::NvIncrement {
        index: index(0x0100_B303),
        auth: Auth::Policy(session()),
    };
    assert!(decode(wide(), &i).is_err());
    assert!(decode(increment(), &rsp_s(&[], &policy_tail())).is_err());
    assert_eq!(
        decode(increment(), &hex("80010000000a0000099d")),
        Err(Error::Tpm(0x99d))
    );
    let flush = || Request::FlushContext { session: handle };
    assert_eq!(enc(flush()), hex("80010000000e0000016503000000"));
    assert_eq!(
        decode(flush(), &hex("80010000000a00000000")),
        Ok(Response::Empty)
    );
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
        specs.push(trial_tpm::anchor_domain(d).unwrap());
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
        (4 << 4, 8), // extend, not a SHA-256 digest
        (4 << 4, 31),
        (4 << 4, 33),
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
        auth: Auth::Policy(PolicySession::new(0x0300_0000, 32).unwrap()),
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
    let anchor = trial_tpm::anchor_domain(0).unwrap();
    let mut name = [0xeeu8; 34];
    name[..2].copy_from_slice(&[0, 0x0b]);
    for spec in [trial_tpm::NV_TRIAL, trial_tpm::NV_GEN, anchor] {
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
        assert_eq!(area.nv_type(), (spec.attributes >> 4) & 0xf);
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
        (
            Request::NvIncrement {
                index: index(0x0100_0000),
                auth: Auth::Password(pw),
            },
            63,
        ),
        (
            Request::NvIncrement {
                index: index(0x0100_0000),
                auth: Auth::Policy(session()),
            },
            31,
        ),
        (Request::FlushContext { session: session() }, 14),
        (
            Request::NvReadSecret {
                index: index(0x0100_0000),
                session: session(),
            },
            35,
        ),
        (
            Request::NvExtend {
                index: index(0x0100_0000),
                auth: pw,
                data: &[0; 32],
            },
            97,
        ),
        (
            Request::NvUndefineSpace {
                index: index(0x0100_0000),
                owner_auth: pw,
            },
            63,
        ),
        (
            Request::HierarchyChangeAuth {
                hierarchy: Hierarchy::Lockout,
                auth: pw,
                new_auth: pw,
            },
            93,
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

// M1-HAUTH codecs. Golden bytes are the commands swtpm 0.10.1 accepted in
// docs/verification/b03-rel05-hauth-probe-20261125/transcript-hauth.txt;
// the secret there is redacted, so the hierarchy rows use 0x11 x 32.
const SECRET: [u8; 32] = [0x11; 32];

#[test]
fn nv_extend_golden_bytes() {
    let data: [u8; 32] = hex("6e7f88a89613f0b7a52acbb8ac0c8ea8f7662c0908c82f5468754a318f70a703")
        .try_into()
        .unwrap();
    let req = || Request::NvExtend {
        index: index(0x0100_B310),
        auth: none(),
        data: &data,
    };
    assert_eq!(enc(req()), hex("800200000041000001360100b3100100b3100000000940000009000000000000206e7f88a89613f0b7a52acbb8ac0c8ea8f7662c0908c82f5468754a318f70a703"));
    let mut out = [0u8; MAX_COMMAND];
    assert!(codec::encode(req(), &mut out).unwrap().is_nv_mutation());
    let good = rsp_s(&[], &PW_TAIL);
    assert_eq!(decode(req(), &good), Ok(Response::Empty));
    truncations_fail(req, &good);
}

#[test]
fn nv_undefine_space_golden_bytes() {
    let empty = || Request::NvUndefineSpace {
        index: index(0x0100_B310),
        owner_auth: none(),
    };
    assert_eq!(
        enc(empty()),
        hex("80020000001f00000122400000010100b31000000009400000090000000000")
    );
    let with = Request::NvUndefineSpace {
        index: index(0x0100_B310),
        owner_auth: Password::new(&SECRET).unwrap(),
    };
    let mut want = hex("80020000003f00000122400000010100b31000000029400000090000000020");
    want.extend_from_slice(&SECRET);
    assert_eq!(enc(with), want);
    let good = rsp_s(&[], &PW_TAIL);
    assert_eq!(decode(empty(), &good), Ok(Response::Empty));
    truncations_fail(empty, &good);
}

#[test]
fn hierarchy_change_auth_golden_bytes_owner_and_lockout_only() {
    for (h, handle) in [
        (Hierarchy::Owner, "40000001"),
        (Hierarchy::Lockout, "4000000a"),
    ] {
        let req = || Request::HierarchyChangeAuth {
            hierarchy: h,
            auth: none(),
            new_auth: Password::new(&SECRET).unwrap(),
        };
        let mut want = hex(&format!(
            "80020000003d00000129{handle}000000094000000900000000000020"
        ));
        want.extend_from_slice(&SECRET);
        assert_eq!(enc(req()), want);
        let good = rsp_s(&[], &PW_TAIL);
        assert_eq!(decode(req(), &good), Ok(Response::Empty));
        truncations_fail(req, &good);
    }
}

fn secret_read() -> Request<'static> {
    Request::NvReadSecret {
        index: index(trial_tpm::NV_HAUTH.handle),
        session: session(),
    }
}
/// `decode_secret` into a sentinel-filled buffer; on error the buffer must
/// come back unchanged.
fn decode_secret(response: &[u8]) -> Result<[u8; MAX_NV_DATA], Error> {
    let mut b = [0u8; MAX_COMMAND];
    let mut out = [0xa5; MAX_NV_DATA];
    let r = codec::encode(secret_read(), &mut b)
        .unwrap()
        .decode_secret(response, &mut out);
    if r.is_err() {
        assert_eq!(out, [0xa5; MAX_NV_DATA], "out changed on error");
    }
    r.map(|()| out)
}

#[test]
fn nv_read_secret_decodes_only_into_the_caller_buffer() {
    // Byte-identical to the policy NV_Read of HAUTH in the M1-HAUTH probe.
    assert_eq!(
        enc(secret_read()),
        hex("8002000000230000014e0100b3040100b3040000000903000000000000000000400000")
    );
    let mut b = [0u8; MAX_COMMAND];
    assert!(!codec::encode(secret_read(), &mut b)
        .unwrap()
        .is_nv_mutation());
    let data: Vec<u8> = (1..=64).collect();
    let mut params = vec![0, 64];
    params.extend_from_slice(&data);
    let good = rsp_s(&params, &policy_tail());
    assert_eq!(decode_secret(&good).unwrap()[..], data[..]);
    // The Copy/Debug Response path refuses even a valid response.
    assert_eq!(decode(secret_read(), &good), Err(Error::Input));
    // Every strict prefix, as-is and resized, fails without touching `out`.
    for len in 0..good.len() {
        assert!(decode_secret(&good[..len]).is_err(), "prefix {len}");
        if len >= 6 {
            let mut p = good[..len].to_vec();
            p[2..6].copy_from_slice(&(len as u32).to_be_bytes());
            assert!(decode_secret(&p).is_err(), "resized prefix {len}");
        }
    }
    for i in 0..good.len() {
        for bit in 0..8 {
            let mut m = good.clone();
            m[i] ^= 1 << bit;
            let _ = decode_secret(&m);
        }
    }
    // TPM error (policy failure), password trailer, 63 bytes, size field 63,
    // trailing byte: all refused.
    assert_eq!(
        decode_secret(&head(codec::NO_SESSIONS, 0x99d, &[])),
        Err(Error::Tpm(0x99d))
    );
    assert!(decode_secret(&rsp_s(&params, &PW_TAIL)).is_err());
    let mut short = vec![0, 63];
    short.extend_from_slice(&data[..63]);
    assert_eq!(
        decode_secret(&rsp_s(&short, &policy_tail())),
        Err(Error::Length)
    );
    let mut size63 = params.clone();
    size63[1] = 63;
    assert_eq!(
        decode_secret(&rsp_s(&size63, &policy_tail())),
        Err(Error::Length)
    );
    let mut long = params.clone();
    long.push(0);
    assert_eq!(
        decode_secret(&rsp_s(&long, &policy_tail())),
        Err(Error::Length)
    );
    // decode_secret refuses every other command, even with a matching shape.
    let other = || Request::NvRead {
        index: index(trial_tpm::NV_HAUTH.handle),
        auth: none(),
        size: 64,
        offset: 0,
    };
    let mut out = [0xa5; MAX_NV_DATA];
    let pw_good = rsp_s(&params, &PW_TAIL);
    assert_eq!(
        codec::encode(other(), &mut b)
            .unwrap()
            .decode_secret(&pw_good, &mut out),
        Err(Error::Input)
    );
    assert_eq!(out, [0xa5; MAX_NV_DATA]);
}

#[test]
fn nv_write_owner_matches_the_swtpm_transcript_and_is_bounded() {
    // Squat probe N1 (swtpm 0.10.1): owner NV_Write of 64 random bytes to
    // HAUTH with an empty ownerAuth, accepted with TPM_RC_SUCCESS.
    let golden = hex(concat!(
        "80020000006300000137400000010100b3040000000940000009000000000000402f65afcb6ae8c9fa",
        "4199fc281628d2d941827a2c4d7b634c0316457fad6082344017792aaaac7d973fddcc2766c3e8f67c",
        "fe78ec1de0fdac00c482ad206a8f010000"
    ));
    let data = golden[33..97].to_vec();
    let req = || Request::NvWriteOwner {
        index: index(trial_tpm::NV_HAUTH.handle),
        owner_auth: none(),
        data: &data,
    };
    assert_eq!(enc(req()), golden);
    let mut out = [0u8; MAX_COMMAND];
    assert!(codec::encode(req(), &mut out).unwrap().is_nv_mutation());
    let good = rsp_s(&[], &PW_TAIL);
    assert_eq!(decode(req(), &good), Ok(Response::Empty));
    truncations_fail(req, &good);
    // A policy trailer answers a password session: refused.
    assert!(decode(req(), &rsp_s(&[], &policy_tail())).is_err());
    // 29-byte ownerAuth + 64 data = 128 fits; 30 bytes exceeds MAX_COMMAND.
    let pw = [7u8; 30];
    let at = |n: usize| Request::NvWriteOwner {
        index: index(trial_tpm::NV_HAUTH.handle),
        owner_auth: Password::new(&pw[..n]).unwrap(),
        data: &data,
    };
    assert_eq!(enc(at(29)).len(), MAX_COMMAND);
    assert_eq!(enc_err(at(30)), Error::Length);
    let d = [0u8; 65];
    for n in [0usize, 65] {
        assert_eq!(
            enc_err(Request::NvWriteOwner {
                index: index(trial_tpm::NV_HAUTH.handle),
                owner_auth: none(),
                data: &d[..n],
            }),
            Error::Input
        );
    }
}

/// M1-HAUTH BOUND provisioning in loader order at the real sizes: `HAUTH`
/// is written while ownerAuth is still empty (a 32-byte ownerAuth would
/// exceed `MAX_COMMAND`, pinned above), then ownerAuth and lockoutAuth.
#[test]
fn hauth_bound_provisioning_frames_fit() {
    let secret = [0x5au8; MAX_NV_DATA];
    let hauth = index(trial_tpm::NV_HAUTH.handle);
    let lrec = index(trial_tpm::NV_LREC.handle);
    let define = |spec: &'static trial_tpm::NvSpec| Request::NvDefineSpace {
        owner_auth: none(),
        public: NvDefinition {
            index: index(spec.handle),
            attributes: spec.attributes,
            auth_policy: spec.auth_policy.as_ref(),
            size: spec.size,
        },
    };
    let frames = [
        enc(Request::NvUndefineSpace {
            index: hauth,
            owner_auth: none(),
        })
        .len(),
        enc(define(&trial_tpm::NV_HAUTH)).len(),
        enc(Request::NvWriteOwner {
            index: hauth,
            owner_auth: none(),
            data: &secret,
        })
        .len(),
        enc(define(&trial_tpm::NV_LREC)).len(),
        enc(Request::NvWrite {
            index: lrec,
            auth: Auth::Policy(session()),
            data: &secret,
            offset: 0,
        })
        .len(),
        enc(Request::HierarchyChangeAuth {
            hierarchy: Hierarchy::Owner,
            auth: none(),
            new_auth: Password::new(&secret[..32]).unwrap(),
        })
        .len(),
        enc(Request::NvReadSecret {
            index: hauth,
            session: session(),
        })
        .len(),
        enc(Request::HierarchyChangeAuth {
            hierarchy: Hierarchy::Lockout,
            auth: none(),
            new_auth: Password::new(&secret[32..]).unwrap(),
        })
        .len(),
    ];
    assert_eq!(frames, [31, 77, 99, 77, 99, 61, 35, 61]);
}
