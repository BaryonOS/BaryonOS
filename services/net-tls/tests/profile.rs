//! Library profile refusals, HOST EVIDENCE ONLY. aws-lc-rs is the host test
//! oracle provider (dev-dependency); the library itself selects no provider.
//! TEST-ONLY Ed25519 certificates are generated at runtime and never stored.

use nieuw_os_net_tls::profile::{self, PinnedPeer, ProfileError};
use rcgen::{CertificateParams, KeyPair, PublicKeyData, PKCS_ED25519};
use rustls::client::danger::ServerCertVerifier;
use rustls::crypto::aws_lc_rs::{cipher_suite, default_provider, kx_group};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::server::danger::ClientCertVerifier;
use rustls::sign::{CertifiedKey, SingleCertAndKey};
use rustls::time_provider::DefaultTimeProvider;
use rustls::{CertificateError, DigitallySignedStruct, Error, PeerIncompatible, SignatureScheme};
use std::sync::Arc;

const ALPN: &[u8] = b"baryon-n03-profile-test/1";

struct Identity {
    cert: CertificateDer<'static>,
    spki: Vec<u8>,
    key: Vec<u8>,
}

fn identity() -> Identity {
    let key = KeyPair::generate_for(&PKCS_ED25519).unwrap();
    let cert = CertificateParams::new(vec!["peer.test".into()])
        .unwrap()
        .self_signed(&key)
        .unwrap();
    Identity {
        cert: cert.der().clone(),
        spki: key.subject_public_key_info(),
        key: key.serialize_der(),
    }
}

fn pin(id: &Identity) -> Result<PinnedPeer, ProfileError> {
    PinnedPeer::new(
        id.cert.clone(),
        id.spki.clone().into(),
        default_provider().signature_verification_algorithms,
        vec![SignatureScheme::ED25519],
    )
}

fn resolver(id: &Identity) -> Arc<SingleCertAndKey> {
    let key = default_provider()
        .key_provider
        .load_private_key(PrivatePkcs8KeyDer::from(id.key.clone()).into())
        .unwrap();
    Arc::new(CertifiedKey::new(vec![id.cert.clone()], key).into())
}

fn build(provider: rustls::crypto::CryptoProvider, alpn: &[u8]) -> [Result<(), ProfileError>; 2] {
    let id = identity();
    let peer = Arc::new(pin(&id).unwrap());
    let time = Arc::new(DefaultTimeProvider);
    [
        profile::client_config(
            provider.clone(),
            time.clone(),
            alpn,
            peer.clone(),
            resolver(&id),
        )
        .map(drop),
        profile::server_config(provider, time, alpn, peer, resolver(&id)).map(drop),
    ]
}

fn assert_both(results: [Result<(), ProfileError>; 2], expected: Result<(), ProfileError>) {
    for result in results {
        assert_eq!(result, expected);
    }
}

#[test]
fn full_provider_is_restricted_to_the_profile() {
    assert_both(build(default_provider(), ALPN), Ok(()));
    let restricted = profile::restrict(default_provider()).unwrap();
    let groups: Vec<_> = restricted.kx_groups.iter().map(|g| g.name()).collect();
    let suites: Vec<_> = restricted.cipher_suites.iter().map(|s| s.suite()).collect();
    assert_eq!(groups, [rustls::NamedGroup::X25519MLKEM768]);
    assert_eq!(suites, [rustls::CipherSuite::TLS13_AES_256_GCM_SHA384]);
}

#[test]
fn provider_without_the_hybrid_group_is_refused() {
    let mut provider = default_provider();
    provider.kx_groups = vec![kx_group::X25519, kx_group::SECP256R1];
    assert_both(build(provider, ALPN), Err(ProfileError::MissingKxGroup));
}

#[test]
fn provider_without_the_profile_suite_is_refused() {
    let mut provider = default_provider();
    provider.cipher_suites = vec![
        cipher_suite::TLS13_AES_128_GCM_SHA256,
        cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
        // Same AEAD and hash, but TLS 1.2: not the profile suite.
        cipher_suite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
    ];
    assert_both(build(provider, ALPN), Err(ProfileError::MissingCipherSuite));
}

#[test]
fn empty_or_oversized_alpn_is_refused() {
    assert_both(
        build(default_provider(), b""),
        Err(ProfileError::InvalidAlpn),
    );
    assert_both(
        build(default_provider(), &[b'a'; 256]),
        Err(ProfileError::InvalidAlpn),
    );
}

#[test]
fn inconsistent_pin_and_unsupported_schemes_are_refused() {
    let (a, b) = (identity(), identity());
    let crossed = PinnedPeer::new(
        a.cert.clone(),
        b.spki.clone().into(),
        default_provider().signature_verification_algorithms,
        vec![SignatureScheme::ED25519],
    );
    assert_eq!(crossed.unwrap_err(), ProfileError::InconsistentPin);
    for schemes in [vec![], vec![SignatureScheme::Unknown(0xfefe)]] {
        let refused = PinnedPeer::new(
            a.cert.clone(),
            a.spki.clone().into(),
            default_provider().signature_verification_algorithms,
            schemes,
        );
        assert_eq!(refused.unwrap_err(), ProfileError::UnsupportedScheme);
    }
}

fn verify_both(peer: &PinnedPeer, ee: &CertificateDer<'_>, chain: &[CertificateDer<'_>]) -> bool {
    let name = ServerName::try_from("peer.test").unwrap();
    let now = UnixTime::now();
    let server = peer.verify_server_cert(ee, chain, &name, &[], now);
    let client = peer.verify_client_cert(ee, chain, now);
    let refused = Error::InvalidCertificate(CertificateError::ApplicationVerificationFailure);
    match (server, client) {
        (Ok(_), Ok(_)) => true,
        (Err(s), Err(c)) => {
            assert_eq!((s, c), (refused.clone(), refused));
            false
        }
        _ => panic!("client and server verifiers disagree"),
    }
}

#[test]
fn pin_accepts_only_the_exact_end_entity() {
    let (pinned, other) = (identity(), identity());
    let peer = pin(&pinned).unwrap();
    assert!(verify_both(&peer, &pinned.cert, &[]));
    assert!(!verify_both(&peer, &other.cert, &[]));
    assert_eq!(
        peer.check(&CertificateDer::from(vec![0x30, 0x00]), &[]),
        Err(Error::InvalidCertificate(CertificateError::BadEncoding))
    );
}

#[test]
fn pinned_end_entity_with_an_intermediate_is_refused() {
    let (pinned, extra) = (identity(), identity());
    let peer = pin(&pinned).unwrap();
    assert!(!verify_both(
        &peer,
        &pinned.cert,
        core::slice::from_ref(&extra.cert)
    ));
    assert!(!verify_both(
        &peer,
        &pinned.cert,
        core::slice::from_ref(&pinned.cert)
    ));
}

/// Wire-decoded DigitallySignedStruct (its constructor is private): `scheme`
/// code point (0x0807 Ed25519, 0x0403 ECDSA P-256) and an all-zero signature.
fn dss(scheme: u16) -> DigitallySignedStruct {
    use rustls::internal::msgs::codec::Codec;
    let mut wire = scheme.to_be_bytes().to_vec();
    wire.extend_from_slice(&64u16.to_be_bytes());
    wire.extend_from_slice(&[0; 64]);
    DigitallySignedStruct::read_bytes(&wire).unwrap()
}

#[test]
fn tls12_and_unlisted_signature_schemes_are_refused() {
    let pinned = identity();
    let peer = pin(&pinned).unwrap();
    assert_eq!(
        ServerCertVerifier::supported_verify_schemes(&peer),
        [SignatureScheme::ED25519]
    );
    let tls12 = Error::PeerIncompatible(PeerIncompatible::Tls12NotOfferedOrEnabled);
    let listed = dss(0x0807);
    assert_eq!(
        ServerCertVerifier::verify_tls12_signature(&peer, b"m", &pinned.cert, &listed).unwrap_err(),
        tls12
    );
    assert_eq!(
        ClientCertVerifier::verify_tls12_signature(&peer, b"m", &pinned.cert, &listed).unwrap_err(),
        tls12
    );
    let unlisted = dss(0x0403);
    assert_eq!(
        ServerCertVerifier::verify_tls13_signature(&peer, b"m", &pinned.cert, &unlisted)
            .unwrap_err(),
        rustls::PeerMisbehaved::SignedHandshakeWithUnadvertisedSigScheme.into()
    );
    // A listed scheme reaches the provider: a zero signature is a bad signature.
    assert_eq!(
        ServerCertVerifier::verify_tls13_signature(&peer, b"m", &pinned.cert, &listed).unwrap_err(),
        Error::InvalidCertificate(CertificateError::BadSignature)
    );
}
