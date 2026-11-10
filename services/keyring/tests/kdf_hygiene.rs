//! Published vectors and byte compatibility of the zeroize-only vendor patch.
//! Tests use deterministic synthetic data; no native/physical entropy claim.
use chacha20::ChaCha20Rng;
use chacha20_legacy::{
    cipher::{KeyIvInit, StreamCipher},
    ChaCha20,
};
use hex_literal::hex;
use hkdf_protected::{
    hmac::{self, KeyInit, Mac},
    Hkdf,
};
use keyring_core::{random, Drbg, EntropySource, Service, Status};
use rand_core_rng::{Rng, SeedableRng};
use sha2_kdf::Sha256;
use zeroize::ZeroizeOnDrop;

#[test]
fn actual_selected_hash_hmac_hkdf_drop_chain_is_required() {
    fn check<T: ZeroizeOnDrop>() {}
    check::<Sha256>();
    check::<hmac::block_api::HmacCore<Sha256>>();
    check::<hmac::block_api::HmacResetCore<Sha256>>();
    check::<hmac::digest::block_api::Buffer<hmac::block_api::HmacCore<Sha256>>>();
    check::<hmac::Hmac<Sha256>>();
    check::<hmac::HmacReset<Sha256>>();
    check::<hmac::SimpleHmac<Sha256>>();
    check::<hmac::SimpleHmacReset<Sha256>>();
    check::<Hkdf<Sha256>>();
    check::<hkdf_protected::HkdfExtract<Sha256>>();
    check::<hkdf_protected::SimpleHkdf<Sha256>>();
    check::<hkdf_protected::SimpleHkdfExtract<Sha256>>();
}

#[test]
fn protected_hkdf_matches_published_rfc5869_cases_one_and_three() {
    let ikm = [0x0b; 22];
    for (salt,info,expected) in [
        (Some(&hex!("000102030405060708090a0b0c")[..]),&hex!("f0f1f2f3f4f5f6f7f8f9")[..],hex!("3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865")),
        (None,&[][..],hex!("8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8")),
    ] {
        let hk=Hkdf::<Sha256>::new(salt,&ikm);let mut out=[0;42];hk.expand(info,&mut out).unwrap();assert_eq!(out,expected);
    }
}

#[test]
fn patched_hkdf_matches_legacy_at_hash_block_and_maximum_output_boundaries() {
    for salt_len in [0, 1, 32, 80] {
        for ikm_len in [0, 1, 32, 128] {
            for info_len in [0, 1, 48] {
                let salt = vec![0x61; salt_len];
                let ikm = vec![0x32; ikm_len];
                let info = vec![0x73; info_len];
                let old = hkdf::Hkdf::<sha2::Sha256>::new(Some(&salt), &ikm);
                let new = Hkdf::<Sha256>::new(Some(&salt), &ikm);
                for count in [0, 1, 31, 32, 33, 64, 96, 8160] {
                    let mut a = vec![0xa5; count];
                    let mut b = a.clone();
                    old.expand(&info, &mut a).unwrap();
                    new.expand(&info, &mut b).unwrap();
                    assert_eq!(a, b);
                }
                let mut failed = [0xa5; 8161];
                assert!(new.expand(&info, &mut failed).is_err());
                assert_eq!(failed, [0xa5; 8161]);
            }
        }
    }
    assert!(Hkdf::<Sha256>::from_prk(&[0; 31]).is_err());
}

#[test]
fn hmac_rfc4231_and_all_reset_variants_preserve_short_and_long_key_results() {
    for (key, message, expected) in [
        (
            vec![0x0b; 20],
            &b"Hi There"[..],
            hex!("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"),
        ),
        (
            vec![0xaa; 131],
            &b"Test Using Larger Than Block-Size Key - Hash Key First"[..],
            hex!("60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"),
        ),
    ] {
        let mut mac = hmac::Hmac::<Sha256>::new_from_slice(&key).unwrap();
        mac.update(message);
        assert_eq!(&mac.finalize().into_bytes()[..], &expected);
        let mut simple = hmac::SimpleHmac::<Sha256>::new_from_slice(&key).unwrap();
        simple.update(message);
        assert_eq!(&simple.finalize().into_bytes()[..], &expected);
        let mut reset = hmac::HmacReset::<Sha256>::new_from_slice(&key).unwrap();
        let mut simple_reset = hmac::SimpleHmacReset::<Sha256>::new_from_slice(&key).unwrap();
        for _ in 0..3 {
            reset.update(message);
            simple_reset.update(message);
            assert_eq!(&reset.finalize_reset().into_bytes()[..], &expected);
            assert_eq!(&simple_reset.finalize_reset().into_bytes()[..], &expected);
        }
    }
}

fn old_kdf(salt: &[u8], ikm: &[u8], info: &[u8]) -> [u8; 32] {
    let mut key = [0; 32];
    hkdf::Hkdf::<sha2::Sha256>::new(Some(salt), ikm)
        .expand(info, &mut key)
        .unwrap();
    key
}
fn old_drbg_fill(key: &mut [u8; 32], out: &mut [u8]) {
    let mut cipher = ChaCha20::new((&*key).into(), (&[0; 12]).into());
    let mut next = [0; 32];
    cipher.apply_keystream(&mut next);
    out.fill(0);
    cipher.apply_keystream(out);
    *key = next;
}
#[test]
fn existing_drbg_entropy_and_reseed_keep_byte_exact_legacy_outputs() {
    let raw = [0x71; 1024];
    let mut expected_key = old_kdf(b"nieuw-os keyring seed v1", &raw, b"drbg");
    let mut actual = Drbg::from_entropy(&raw);
    for count in [0, 1, 24, 32, 64, 128] {
        let mut expected = vec![0; count];
        let mut output = vec![0; count];
        old_drbg_fill(&mut expected_key, &mut expected);
        actual.fill(&mut output);
        assert_eq!(output, expected);
    }
    let fresh = [0x42; 256];
    expected_key = old_kdf(&expected_key, &fresh, b"drbg");
    actual.reseed(&fresh);
    let mut expected = [0; 128];
    let mut output = [0; 128];
    old_drbg_fill(&mut expected_key, &mut expected);
    actual.fill(&mut output);
    assert_eq!(output, expected);
}

#[derive(Clone)]
struct Source(u64);
impl EntropySource for Source {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        for b in out {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            *b = (self.0 >> 24) as u8;
        }
        Ok(())
    }
}
fn expected_random(source: &mut Source, domain: u8, epoch: u64) -> [u8; 128] {
    let mut raw = [0; 256];
    source.read(&mut raw).unwrap();
    let mut info = b"nieuw-os random domain v1".to_vec();
    info.push(domain);
    info.extend_from_slice(&epoch.to_le_bytes());
    let seed = old_kdf(b"nieuw-os random entropy v1", &raw, &info);
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut out = [0; 128];
    rng.fill_bytes(&mut out);
    out
}
#[test]
fn actual_random_seed_reseed_and_principal_labels_match_legacy_hkdf_bytes() {
    let mut source = Source(0x31415926);
    let mut service = Service::new();
    service.start(&mut source).unwrap();
    let mut oracle = source.clone();
    let mut out = [0; random::MAX_BYTES];
    let first = expected_random(&mut oracle, 0, 1);
    service
        .random_at(0, 1, 128, 0, &mut source, &mut out)
        .unwrap();
    assert_eq!(&out[..128], &first);
    for id in 2..=64 {
        service
            .random_at(0, id, 1, 0, &mut source, &mut out)
            .unwrap();
    }
    let second = expected_random(&mut oracle, 0, 2);
    service
        .random_at(0, 65, 128, 100, &mut source, &mut out)
        .unwrap();
    assert_eq!(&out[..128], &second);
    let other = expected_random(&mut oracle, 1, 1);
    service
        .random_at(1, 1, 128, 100, &mut source, &mut out)
        .unwrap();
    assert_eq!(&out[..128], &other);
}
