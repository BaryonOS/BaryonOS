//! N03-T2 host loopback handshake: HOST EVIDENCE ONLY, not a native TLS port.
//!
//! Both peers are rustls 0.23.46 with the aws-lc-rs provider as a host-only test
//! oracle (dev-dependency; owner decision 2026-11-23). The project's own
//! `KeyServiceRandom` (over a synthetic Key-Service transport) is the rustls
//! `SecureRandom` of the profile side, and `CertificateClock` is exercised as the
//! rustls `TimeProvider` negative. aws-lc's own KEM/signing randomness does NOT
//! pass through `KeyServiceRandom`; that route stays an open N03 blocker.
//!
//! Profile (ADR-028): TLS 1.3 only, X25519MLKEM768 only, TLS13_AES_256_GCM_SHA384
//! only, fixed ALPN, mutual exact certificate + SPKI pinning, no resumption,
//! tickets, 0-RTT or half-RTT data, no classical fallback. Profile sides are built
//! by the library (`nieuw_os_net_tls::profile`) from the full aws-lc-rs provider;
//! deviating oracle peers use raw rustls configs because the library refuses them.
//!
//! H2 (ADR-N03-PROVIDER-20261125, H3): the `h2_*` tests repeat the positive and
//! classical-only cases over loopback TCP against Node.js's OpenSSL
//! (tests/openssl_peer.mjs).
//!
//! Credentials: TEST-ONLY Ed25519 keys and self-signed certificates generated at
//! runtime by rcgen; nothing is committed, no trust root is created. Certificate
//! validity periods are NOT checked: no trusted certificate-time authority exists,
//! so positive runs use a fixed test `TimeProvider` (harness assumption).

use core::num::NonZeroU64;
use nieuw_os_keyring_random_client::{Session, Transport};
use nieuw_os_net_tls::profile::{self, PinnedPeer, Refusal};
use nieuw_os_net_tls::random::{KeyServiceRandom, MonotonicClock};
use nieuw_os_net_tls::time::CertificateClock;
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use rcgen::{CertificateParams, KeyPair, PublicKeyData, PKCS_ED25519};
use rustls::crypto::aws_lc_rs::{cipher_suite, default_provider, kx_group};
use rustls::crypto::{CryptoProvider, SupportedKxGroup};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::server::NoServerSessionStorage;
use rustls::sign::{CertifiedKey, SingleCertAndKey};
use rustls::time_provider::TimeProvider;
use rustls::{
    AlertDescription, CertificateError, ClientConfig, ClientConnection, Connection, Error,
    PeerIncompatible, ServerConfig, ServerConnection, SignatureScheme, SupportedCipherSuite,
    SupportedProtocolVersion,
};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const ALPN: &[u8] = b"baryon-n03-host-loopback/1";
const SERVER_NAME: &str = "nas.peer.test";
static TLS13_ONLY: &[&SupportedProtocolVersion] = &[&rustls::version::TLS13];
static TLS12_ONLY: &[&SupportedProtocolVersion] = &[&rustls::version::TLS12];

// ---------------------------------------------------------------- RNG fixture

/// Synthetic Key-Service RANDOM peer, same wire shape as tests/random.rs.
struct KeyService {
    sends: Arc<AtomicUsize>,
    deny: bool,
    id: u64,
    length: usize,
}

impl Transport for KeyService {
    fn send(&mut self, _: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let payload = &message.as_bytes()[64..320];
        self.id = u64::from_le_bytes(payload[4..12].try_into().unwrap());
        self.length = usize::from(u16::from_le_bytes(payload[22..24].try_into().unwrap()));
        self.sends.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut payload = [0; 256];
        payload[0] = 0x8a;
        payload[4..12].copy_from_slice(&self.id.to_le_bytes());
        if self.deny {
            payload[12..16].copy_from_slice(&(-1i32).to_le_bytes());
        } else {
            payload[2..4].copy_from_slice(&((16 + self.length) as u16).to_le_bytes());
            payload[16..20].copy_from_slice(b"RND1");
            payload[20..22].copy_from_slice(&1u16.to_le_bytes());
            payload[22..24].copy_from_slice(&(self.length as u16).to_le_bytes());
            payload[32..32 + self.length].fill(0xa5 ^ self.id as u8);
        }
        let mut envelope = *Outgoing::new(0x4b45_5931, &payload).unwrap().as_bytes();
        envelope[..8].copy_from_slice(&7u64.to_le_bytes());
        Received::from_bytes(envelope, 256)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
    fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
        Ok(())
    }
}

struct Ticks(u64);
impl MonotonicClock for Ticks {
    fn ticks(&mut self) -> Result<u64, RuntimeError> {
        self.0 += 1;
        Ok(self.0)
    }
}

/// rustls takes `&'static dyn SecureRandom`; one leaked lease per config.
fn key_service_random(
    deny: bool,
) -> (
    &'static KeyServiceRandom<KeyService, Ticks>,
    Arc<AtomicUsize>,
) {
    let sends = Arc::new(AtomicUsize::new(0));
    let handle = |slot: u64| Handle::from_raw((1u64 << 32) | slot).unwrap();
    let session = Session::new(
        KeyService {
            sends: sends.clone(),
            deny,
            id: 0,
            length: 0,
        },
        SendEndpoint::from_handle(handle(1)),
        ReceiveEndpoint::from_handle(handle(2)),
        7,
        8,
    )
    .unwrap();
    let rng = KeyServiceRandom::new(session, Ticks(0), NonZeroU64::new(1_000).unwrap());
    (Box::leak(Box::new(rng)), sends)
}

// ------------------------------------------------------- profile and pinning

/// Fixed test-only instant: harness assumption, never a trusted calendar.
#[derive(Debug)]
struct FixedTestTime;
impl TimeProvider for FixedTestTime {
    fn current_time(&self) -> Option<UnixTime> {
        Some(UnixTime::since_unix_epoch(core::time::Duration::from_secs(
            1_793_361_600,
        )))
    }
}

struct Identity {
    cert: CertificateDer<'static>,
    spki: Vec<u8>,
    key: Vec<u8>,
    /// Extra chain certificates presented after `cert` (negatives only).
    extra: Vec<CertificateDer<'static>>,
}

impl Identity {
    /// TEST-ONLY ephemeral Ed25519 identity; never persisted.
    fn generate() -> Self {
        let key = KeyPair::generate_for(&PKCS_ED25519).unwrap();
        let cert = CertificateParams::new(vec![SERVER_NAME.into()])
            .unwrap()
            .self_signed(&key)
            .unwrap();
        Self {
            cert: cert.der().clone(),
            spki: key.subject_public_key_info(),
            key: key.serialize_der(),
            extra: Vec::new(),
        }
    }
    /// Presents `pinned`'s exact certificate but holds an unrelated private key.
    fn forged(pinned: &Identity) -> Self {
        Self {
            cert: pinned.cert.clone(),
            spki: pinned.spki.clone(),
            ..Self::generate()
        }
    }
    /// Presents the genuine certificate and key plus an extra chain certificate.
    fn with_intermediate(genuine: &Identity) -> Self {
        Self {
            cert: genuine.cert.clone(),
            spki: genuine.spki.clone(),
            key: genuine.key.clone(),
            extra: vec![Identity::generate().cert],
        }
    }
    /// Resolver input; unlike `with_single_cert` it does not check that the key
    /// matches the certificate, so forged identities reach the peer verifier.
    fn certified(&self) -> Arc<SingleCertAndKey> {
        let key = PrivatePkcs8KeyDer::from(self.key.clone()).into();
        let key = default_provider()
            .key_provider
            .load_private_key(key)
            .unwrap();
        let chain = core::iter::once(self.cert.clone())
            .chain(self.extra.iter().cloned())
            .collect();
        Arc::new(CertifiedKey::new(chain, key).into())
    }
}

/// Library pin of `identity`'s exact certificate and SPKI, Ed25519 only.
fn pin_of(identity: &Identity) -> Arc<PinnedPeer> {
    Arc::new(
        PinnedPeer::new(
            identity.cert.clone(),
            identity.spki.clone().into(),
            default_provider().signature_verification_algorithms,
            vec![SignatureScheme::ED25519],
        )
        .unwrap(),
    )
}

/// One side's crypto and version selection.
struct Side {
    suites: Vec<SupportedCipherSuite>,
    groups: Vec<&'static dyn SupportedKxGroup>,
    versions: &'static [&'static SupportedProtocolVersion],
    random: Option<&'static KeyServiceRandom<KeyService, Ticks>>,
    time: Arc<dyn TimeProvider>,
    alpn: Option<&'static [u8]>,
    /// Built by the library profile; otherwise a raw deviating oracle peer.
    profile: bool,
}

impl Side {
    /// The fixed ADR-028 profile with the project's Key-Service RNG adapter.
    fn profile(random: &'static KeyServiceRandom<KeyService, Ticks>) -> Self {
        Self {
            suites: vec![cipher_suite::TLS13_AES_256_GCM_SHA384],
            groups: vec![kx_group::X25519MLKEM768],
            versions: TLS13_ONLY,
            random: Some(random),
            time: Arc::new(FixedTestTime),
            alpn: Some(ALPN),
            profile: true,
        }
    }
    /// A deviating oracle peer (aws-lc RNG), used only by negative tests.
    fn peer() -> Self {
        Self {
            random: None,
            profile: false,
            ..Self::profile(key_service_random(false).0)
        }
    }
    /// The full aws-lc-rs provider (only the RNG swapped): the library profile
    /// must do the restriction itself.
    fn full_provider(&self) -> CryptoProvider {
        let mut provider = default_provider();
        if let Some(random) = self.random {
            provider.secure_random = random;
        }
        provider
    }
    fn provider(&self) -> Arc<CryptoProvider> {
        let mut provider = self.full_provider();
        provider.cipher_suites = self.suites.clone();
        provider.kx_groups = self.groups.clone();
        Arc::new(provider)
    }
}

fn client_config(
    side: &Side,
    server_pin: Arc<PinnedPeer>,
    identity: Option<&Identity>,
) -> Result<ClientConfig, Error> {
    if side.profile {
        let identity = identity.expect("the profile always presents a client identity");
        return Ok(profile::client_config(
            side.full_provider(),
            side.time.clone(),
            ALPN,
            server_pin,
            identity.certified(),
        )
        .unwrap());
    }
    let builder = ClientConfig::builder_with_details(side.provider(), side.time.clone())
        .with_protocol_versions(side.versions)?
        .dangerous()
        .with_custom_certificate_verifier(server_pin);
    let mut config = match identity {
        Some(id) => builder.with_client_cert_resolver(id.certified()),
        None => builder.with_no_client_auth(),
    };
    config.alpn_protocols = side.alpn.iter().map(|p| p.to_vec()).collect();
    config.resumption = rustls::client::Resumption::disabled();
    config.enable_early_data = false;
    Ok(config)
}

fn server_config(
    side: &Side,
    client_pin: Arc<PinnedPeer>,
    identity: &Identity,
) -> Result<ServerConfig, Error> {
    if side.profile {
        return Ok(profile::server_config(
            side.full_provider(),
            side.time.clone(),
            ALPN,
            client_pin,
            identity.certified(),
        )
        .unwrap());
    }
    let mut config = ServerConfig::builder_with_details(side.provider(), side.time.clone())
        .with_protocol_versions(side.versions)?
        .with_client_cert_verifier(client_pin)
        .with_cert_resolver(identity.certified());
    config.alpn_protocols = side.alpn.iter().map(|p| p.to_vec()).collect();
    config.session_storage = Arc::new(NoServerSessionStorage {});
    config.send_tls13_tickets = 0;
    config.max_early_data_size = 0;
    config.send_half_rtt_data = false;
    Ok(config)
}

// ------------------------------------------------- in-memory transport pump

struct Outcome {
    client: Connection,
    server: Connection,
    client_error: Option<Error>,
    server_error: Option<Error>,
}

/// Move all pending records from `from` to `to`; returns whether bytes moved.
fn transfer(from: &mut Connection, to: &mut Connection, to_error: &mut Option<Error>) -> bool {
    let mut wire = Vec::new();
    while from.wants_write() {
        from.write_tls(&mut wire).unwrap();
    }
    if wire.is_empty() || to_error.is_some() {
        return !wire.is_empty();
    }
    let mut input = wire.as_slice();
    while !input.is_empty() {
        if to.read_tls(&mut input).unwrap() == 0 {
            break;
        }
        if let Err(error) = to.process_new_packets() {
            *to_error = Some(error);
            break;
        }
    }
    true
}

fn pump(outcome: &mut Outcome) {
    for _ in 0..32 {
        let a = transfer(
            &mut outcome.client,
            &mut outcome.server,
            &mut outcome.server_error,
        );
        let b = transfer(
            &mut outcome.server,
            &mut outcome.client,
            &mut outcome.client_error,
        );
        if !a && !b {
            return;
        }
    }
    panic!("handshake did not settle");
}

/// Bytes a client queues before the handshake. rustls holds them until client
/// traffic keys exist; negatives prove the server never releases them.
const QUEUED: &[u8] = b"client data queued before the handshake";

fn handshake(client: ClientConfig, server: ServerConfig) -> Outcome {
    handshake_with(client, server, QUEUED)
}

fn handshake_with(client: ClientConfig, server: ServerConfig, queued: &[u8]) -> Outcome {
    let name = ServerName::try_from(SERVER_NAME).unwrap();
    let mut outcome = Outcome {
        client: ClientConnection::new(Arc::new(client), name)
            .unwrap()
            .into(),
        server: ServerConnection::new(Arc::new(server)).unwrap().into(),
        client_error: None,
        server_error: None,
    };
    outcome.client.writer().write_all(queued).unwrap();
    pump(&mut outcome);
    outcome
}

/// Plaintext bytes this side has released to the application.
fn plaintext(connection: &mut Connection) -> Vec<u8> {
    let mut out = Vec::new();
    let mut chunk = [0u8; 256];
    while let Ok(n) = connection.reader().read(&mut chunk) {
        if n == 0 {
            break;
        }
        out.extend_from_slice(&chunk[..n]);
    }
    out
}

/// Library acceptance gate. Application data may flow only after this passes.
fn accepted(connection: &Connection, peer: &PinnedPeer) -> Result<(), Refusal> {
    profile::accept(connection, ALPN, peer)
}

/// Negative cases: the server gate refuses, the client gate gives `client_gate`,
/// and neither side released application plaintext (including `QUEUED`).
fn assert_refused(
    outcome: &mut Outcome,
    server_pin: &PinnedPeer,
    client_pin: &PinnedPeer,
    client_gate: Result<(), Refusal>,
) {
    assert!(accepted(&outcome.server, client_pin).is_err());
    assert_eq!(accepted(&outcome.client, server_pin), client_gate);
    assert!(plaintext(&mut outcome.client).is_empty());
    assert!(plaintext(&mut outcome.server).is_empty());
}

fn alert(error: &Option<Error>) -> Option<AlertDescription> {
    match error {
        Some(Error::AlertReceived(description)) => Some(*description),
        _ => None,
    }
}

struct Peers {
    server: Identity,
    client: Identity,
}
impl Peers {
    fn new() -> Self {
        Self {
            server: Identity::generate(),
            client: Identity::generate(),
        }
    }
}

// ----------------------------------------------------------------- positive

#[test]
fn profile_handshake_is_mutually_pinned_hybrid_tls13_and_uses_key_service_random() {
    let peers = Peers::new();
    let (client_rng, client_sends) = key_service_random(false);
    let (server_rng, server_sends) = key_service_random(false);
    let (server_pin, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let client = client_config(
        &Side::profile(client_rng),
        server_pin.clone(),
        Some(&peers.client),
    )
    .unwrap();
    let server = server_config(
        &Side::profile(server_rng),
        client_pin.clone(),
        &peers.server,
    )
    .unwrap();
    let mut outcome = handshake_with(client, server, b"");

    assert!(outcome.client_error.is_none(), "{:?}", outcome.client_error);
    assert!(outcome.server_error.is_none(), "{:?}", outcome.server_error);
    assert_eq!(accepted(&outcome.client, &server_pin), Ok(()));
    assert_eq!(accepted(&outcome.server, &client_pin), Ok(()));
    // The gate binds the peer: each side's own pin is not its peer's pin.
    assert_eq!(accepted(&outcome.client, &client_pin), Err(Refusal::Peer));
    assert_eq!(accepted(&outcome.server, &server_pin), Err(Refusal::Peer));
    // The rustls SecureRandom route reached the project's Key-Service adapter.
    assert!(client_sends.load(Ordering::SeqCst) > 0);
    assert!(server_sends.load(Ordering::SeqCst) > 0);

    outcome
        .client
        .writer()
        .write_all(b"client after acceptance")
        .unwrap();
    outcome
        .server
        .writer()
        .write_all(b"server after acceptance")
        .unwrap();
    pump(&mut outcome);
    assert_eq!(plaintext(&mut outcome.server), b"client after acceptance");
    assert_eq!(plaintext(&mut outcome.client), b"server after acceptance");
}

#[test]
fn queued_client_data_is_released_only_after_a_completed_handshake() {
    // Control for the negatives: the same QUEUED bytes do arrive when both
    // sides accept, so an empty server read in a negative is not vacuous.
    let peers = Peers::new();
    let (server_pin, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let client = client_config(
        &Side::profile(key_service_random(false).0),
        server_pin.clone(),
        Some(&peers.client),
    )
    .unwrap();
    let server = server_config(
        &Side::profile(key_service_random(false).0),
        client_pin.clone(),
        &peers.server,
    )
    .unwrap();
    let mut outcome = handshake(client, server);
    assert_eq!(accepted(&outcome.server, &client_pin), Ok(()));
    assert_eq!(plaintext(&mut outcome.server), QUEUED);
}

// ---------------------------------------------------------------- negatives

/// Profile client against a deviating server.
fn against_server(peer: Side) -> (Outcome, Arc<PinnedPeer>, Arc<PinnedPeer>) {
    let peers = Peers::new();
    let (server_pin, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let client = client_config(
        &Side::profile(key_service_random(false).0),
        server_pin.clone(),
        Some(&peers.client),
    )
    .unwrap();
    let server = server_config(&peer, client_pin.clone(), &peers.server).unwrap();
    (handshake(client, server), server_pin, client_pin)
}

/// Deviating client against the profile server.
fn against_client(peer: Side) -> (Outcome, Arc<PinnedPeer>, Arc<PinnedPeer>) {
    let peers = Peers::new();
    let (server_pin, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let client = client_config(&peer, server_pin.clone(), Some(&peers.client)).unwrap();
    let server = server_config(
        &Side::profile(key_service_random(false).0),
        client_pin.clone(),
        &peers.server,
    )
    .unwrap();
    (handshake(client, server), server_pin, client_pin)
}

#[test]
fn classical_only_x25519_server_gets_no_fallback() {
    let (mut o, sp, cp) = against_server(Side {
        groups: vec![kx_group::X25519],
        ..Side::peer()
    });
    assert_eq!(
        o.server_error,
        Some(Error::PeerIncompatible(
            PeerIncompatible::NoKxGroupsInCommon
        ))
    );
    assert_eq!(
        alert(&o.client_error),
        Some(AlertDescription::HandshakeFailure)
    );
    assert_refused(&mut o, &sp, &cp, Err(Refusal::Handshaking));
}

#[test]
fn classical_only_x25519_client_gets_no_fallback() {
    let (mut o, sp, cp) = against_client(Side {
        groups: vec![kx_group::X25519],
        ..Side::peer()
    });
    assert_eq!(
        o.server_error,
        Some(Error::PeerIncompatible(
            PeerIncompatible::NoKxGroupsInCommon
        ))
    );
    assert_eq!(
        alert(&o.client_error),
        Some(AlertDescription::HandshakeFailure)
    );
    assert_refused(&mut o, &sp, &cp, Err(Refusal::Handshaking));
}

#[test]
fn other_cipher_suite_server_is_refused() {
    let (mut o, sp, cp) = against_server(Side {
        suites: vec![cipher_suite::TLS13_AES_128_GCM_SHA256],
        ..Side::peer()
    });
    assert_eq!(
        o.server_error,
        Some(Error::PeerIncompatible(
            PeerIncompatible::NoCipherSuitesInCommon
        ))
    );
    assert_eq!(
        alert(&o.client_error),
        Some(AlertDescription::HandshakeFailure)
    );
    assert_refused(&mut o, &sp, &cp, Err(Refusal::Handshaking));
}

#[test]
fn tls12_only_server_is_refused() {
    let (mut o, sp, cp) = against_server(Side {
        suites: vec![cipher_suite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384],
        groups: vec![kx_group::X25519],
        versions: TLS12_ONLY,
        ..Side::peer()
    });
    assert_eq!(
        o.server_error,
        Some(Error::PeerIncompatible(
            PeerIncompatible::Tls12NotOfferedOrEnabled
        ))
    );
    assert_eq!(
        alert(&o.client_error),
        Some(AlertDescription::ProtocolVersion)
    );
    assert_refused(&mut o, &sp, &cp, Err(Refusal::Handshaking));
}

#[test]
fn tls12_only_client_is_refused() {
    let (mut o, sp, cp) = against_client(Side {
        suites: vec![cipher_suite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384],
        groups: vec![kx_group::X25519],
        versions: TLS12_ONLY,
        ..Side::peer()
    });
    assert_eq!(
        o.server_error,
        Some(Error::PeerIncompatible(
            PeerIncompatible::Tls12NotOfferedOrEnabled
        ))
    );
    assert_eq!(
        alert(&o.client_error),
        Some(AlertDescription::ProtocolVersion)
    );
    assert_refused(&mut o, &sp, &cp, Err(Refusal::Handshaking));
}

#[test]
fn wrong_server_pin_is_refused_before_any_plaintext() {
    let peers = Peers::new();
    let impostor = Identity::generate();
    let (expected_server, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let client = client_config(
        &Side::profile(key_service_random(false).0),
        expected_server.clone(),
        Some(&peers.client),
    )
    .unwrap();
    let server = server_config(&Side::peer(), client_pin.clone(), &impostor).unwrap();
    let mut o = handshake(client, server);
    assert_eq!(
        o.client_error,
        Some(Error::InvalidCertificate(
            CertificateError::ApplicationVerificationFailure
        ))
    );
    assert_eq!(alert(&o.server_error), Some(AlertDescription::AccessDenied));
    assert_refused(
        &mut o,
        &expected_server,
        &client_pin,
        Err(Refusal::Handshaking),
    );
}

#[test]
fn wrong_client_pin_is_refused_by_the_server_before_any_plaintext() {
    let peers = Peers::new();
    let impostor = Identity::generate();
    let (server_pin, expected_client) = (pin_of(&peers.server), pin_of(&peers.client));
    let client = client_config(&Side::peer(), server_pin.clone(), Some(&impostor)).unwrap();
    let server = server_config(
        &Side::profile(key_service_random(false).0),
        expected_client.clone(),
        &peers.server,
    )
    .unwrap();
    let mut o = handshake(client, server);
    assert_eq!(
        o.server_error,
        Some(Error::InvalidCertificate(
            CertificateError::ApplicationVerificationFailure
        ))
    );
    assert_eq!(alert(&o.client_error), Some(AlertDescription::AccessDenied));
    // TLS 1.3: the client completes before the server rejects its certificate,
    // so its own gate passes; only the later alert (asserted above) and the
    // server's refusal to release QUEUED stop the exchange.
    assert_refused(&mut o, &server_pin, &expected_client, Ok(()));
}

#[test]
fn missing_client_certificate_is_refused() {
    let peers = Peers::new();
    let (server_pin, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let client = client_config(&Side::peer(), server_pin.clone(), None).unwrap();
    let server = server_config(
        &Side::profile(key_service_random(false).0),
        client_pin.clone(),
        &peers.server,
    )
    .unwrap();
    let mut o = handshake(client, server);
    assert_eq!(o.server_error, Some(Error::NoCertificatesPresented));
    assert_eq!(
        alert(&o.client_error),
        Some(AlertDescription::CertificateRequired)
    );
    // TLS 1.3: the client completes before the server rejects its certificate,
    // so its own gate passes; only the later alert (asserted above) and the
    // server's refusal to release QUEUED stop the exchange.
    assert_refused(&mut o, &server_pin, &client_pin, Ok(()));
}

#[test]
fn wrong_alpn_client_is_refused() {
    let (mut o, sp, cp) = against_client(Side {
        alpn: Some(b"http/1.1"),
        ..Side::peer()
    });
    assert_eq!(o.server_error, Some(Error::NoApplicationProtocol));
    assert_eq!(
        alert(&o.client_error),
        Some(AlertDescription::NoApplicationProtocol)
    );
    assert_refused(&mut o, &sp, &cp, Err(Refusal::Handshaking));
}

#[test]
fn server_without_alpn_is_refused_by_the_acceptance_gate() {
    let (mut o, sp, cp) = against_server(Side {
        alpn: None,
        ..Side::peer()
    });
    // rustls itself tolerates a server that selects no ALPN; only the gate
    // refuses it. Data queued before the gate already reached that server, so
    // a profile consumer must run the gate before its first write or read.
    assert!(o.client_error.is_none() && o.server_error.is_none());
    assert_eq!(accepted(&o.client, &sp), Err(Refusal::Alpn));
    assert_eq!(accepted(&o.server, &cp), Err(Refusal::Alpn));
    assert!(plaintext(&mut o.client).is_empty());
    assert_eq!(plaintext(&mut o.server), QUEUED);
}

#[test]
fn key_service_rng_denial_fails_closed_without_fallback() {
    let peers = Peers::new();
    let (denying, sends) = key_service_random(true);
    let client = client_config(
        &Side::profile(denying),
        pin_of(&peers.server),
        Some(&peers.client),
    )
    .unwrap();
    let name = ServerName::try_from(SERVER_NAME).unwrap();
    let error = ClientConnection::new(Arc::new(client), name).unwrap_err();
    assert_eq!(error, Error::FailedToGetRandomBytes);
    assert_eq!(sends.load(Ordering::SeqCst), 1);
}

#[test]
fn unavailable_certificate_time_fails_closed() {
    let peers = Peers::new();
    let (server_pin, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let side = Side {
        time: Arc::new(CertificateClock::new(|| Ok(None))),
        ..Side::profile(key_service_random(false).0)
    };
    let client = client_config(&side, server_pin.clone(), Some(&peers.client)).unwrap();
    let server = server_config(&Side::peer(), client_pin.clone(), &peers.server).unwrap();
    let mut o = handshake(client, server);
    assert_eq!(o.client_error, Some(Error::FailedToGetCurrentTime));
    assert_refused(&mut o, &server_pin, &client_pin, Err(Refusal::Handshaking));
}

#[test]
fn server_with_pinned_certificate_but_impostor_key_is_refused() {
    // Proves CertificateVerify is checked: the pin alone authenticates nothing.
    let peers = Peers::new();
    let (server_pin, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let client = client_config(
        &Side::profile(key_service_random(false).0),
        server_pin.clone(),
        Some(&peers.client),
    )
    .unwrap();
    let forged = Identity::forged(&peers.server);
    let server = server_config(&Side::peer(), client_pin.clone(), &forged).unwrap();
    let mut o = handshake(client, server);
    assert_eq!(
        o.client_error,
        Some(Error::InvalidCertificate(CertificateError::BadSignature))
    );
    assert_eq!(alert(&o.server_error), Some(AlertDescription::DecryptError));
    assert_refused(&mut o, &server_pin, &client_pin, Err(Refusal::Handshaking));
}

#[test]
fn client_with_pinned_certificate_but_impostor_key_is_refused() {
    let peers = Peers::new();
    let (server_pin, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let forged = Identity::forged(&peers.client);
    let client = client_config(&Side::peer(), server_pin.clone(), Some(&forged)).unwrap();
    let server = server_config(
        &Side::profile(key_service_random(false).0),
        client_pin.clone(),
        &peers.server,
    )
    .unwrap();
    let mut o = handshake(client, server);
    assert_eq!(
        o.server_error,
        Some(Error::InvalidCertificate(CertificateError::BadSignature))
    );
    assert_eq!(alert(&o.client_error), Some(AlertDescription::DecryptError));
    // TLS 1.3 client completes before the server checks its CertificateVerify.
    assert_refused(&mut o, &server_pin, &client_pin, Ok(()));
}

#[test]
fn pinned_server_certificate_with_an_intermediate_is_refused() {
    let peers = Peers::new();
    let (server_pin, client_pin) = (pin_of(&peers.server), pin_of(&peers.client));
    let client = client_config(
        &Side::profile(key_service_random(false).0),
        server_pin.clone(),
        Some(&peers.client),
    )
    .unwrap();
    let chained = Identity::with_intermediate(&peers.server);
    let server = server_config(&Side::peer(), client_pin.clone(), &chained).unwrap();
    let mut o = handshake(client, server);
    assert_eq!(
        o.client_error,
        Some(Error::InvalidCertificate(
            CertificateError::ApplicationVerificationFailure
        ))
    );
    assert_refused(&mut o, &server_pin, &client_pin, Err(Refusal::Handshaking));
}

// ------------------------------------------- H2: Node.js/OpenSSL test peer
//
// ADR-N03-PROVIDER-20261125 (H3): besides the rustls oracle above, the profile
// sides handshake over real loopback TCP with an independent peer, Node.js's
// bundled OpenSSL (>= 3.5 for X25519MLKEM768), via tests/openssl_peer.mjs.
// The peer validates the profile's certificate against the pinned self-signed
// leaf and compares the exact certificate bytes. `NOS_NODE` selects the node
// binary (default `node`); a missing or too-old peer fails the test.

use rustls::{NamedGroup, StreamOwned};
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, ChildStdout, Command, Stdio};

struct NodePeer {
    child: Child,
    lines: std::io::Lines<BufReader<ChildStdout>>,
}

impl NodePeer {
    fn spawn(role: &str, own: &Identity, peer: &Identity, group: &str, port: Option<u16>) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "n03-openssl-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let file = |name: &str, bytes: &[u8]| {
            let path = dir.join(name);
            std::fs::write(&path, bytes).unwrap();
            path
        };
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/openssl_peer.mjs");
        let mut command = Command::new(std::env::var("NOS_NODE").unwrap_or_else(|_| "node".into()));
        command
            .arg(script)
            .arg(role)
            .arg(file("cert.der", own.cert.as_ref()))
            .arg(file("key.der", &own.key))
            .arg(file("peer.der", peer.cert.as_ref()))
            .args([group, core::str::from_utf8(ALPN).unwrap(), SERVER_NAME]);
        if let Some(port) = port {
            command.arg(port.to_string());
        }
        let mut child = command
            .stdout(Stdio::piped())
            .spawn()
            .expect("N03 H2 needs Node.js with OpenSSL >= 3.5 (set NOS_NODE)");
        let lines = BufReader::new(child.stdout.take().unwrap()).lines();
        Self { child, lines }
    }
    fn event(&mut self) -> serde_like::Event {
        let line = self.lines.next().expect("node peer closed stdout").unwrap();
        serde_like::Event(line)
    }
    fn finish(mut self) -> (Vec<serde_like::Event>, i32) {
        let rest = self
            .lines
            .by_ref()
            .map(|l| serde_like::Event(l.unwrap()))
            .collect();
        (rest, self.child.wait().unwrap().code().unwrap_or(-1))
    }
}

/// A failed assertion must not leave the peer holding the test's output pipes.
impl Drop for NodePeer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Minimal field access on the peer's flat JSON lines (no serde dependency).
mod serde_like {
    #[derive(Debug)]
    pub struct Event(pub String);
    impl Event {
        pub fn field(&self, name: &str) -> Option<&str> {
            let key = format!("\"{name}\":");
            let rest = &self.0[self.0.find(&key)? + key.len()..];
            Some(match rest.strip_prefix('"') {
                Some(text) => &text[..text.find('"')?],
                None => &rest[..rest.find([',', '}'])?],
            })
        }
    }
}

/// A stalled peer fails the test instead of hanging it.
fn bounded(socket: TcpStream) -> TcpStream {
    let limit = Some(std::time::Duration::from_secs(10));
    socket.set_read_timeout(limit).unwrap();
    socket.set_write_timeout(limit).unwrap();
    socket
}

fn assert_modern_openssl(event: &serde_like::Event) {
    let version = event.field("openssl").expect("peer reports its OpenSSL");
    let mut parts = version.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    let (major, minor) = (parts.next().unwrap(), parts.next().unwrap_or(0));
    assert!(
        (major, minor) >= (3, 5),
        "OpenSSL {version} lacks X25519MLKEM768"
    );
}

fn assert_peer_secure(event: &serde_like::Event) {
    assert_eq!(event.field("event"), Some("secure"), "{event:?}");
    assert_modern_openssl(event);
    assert_eq!(event.field("protocol"), Some("TLSv1.3"));
    assert_eq!(event.field("cipher"), Some("TLS_AES_256_GCM_SHA384"));
    assert_eq!(event.field("alpn"), core::str::from_utf8(ALPN).ok());
    assert_eq!(event.field("authorized"), Some("true"));
    assert_eq!(event.field("pinned"), Some("true"));
}

fn assert_profile_hybrid(connection: &rustls::CommonState) {
    assert_eq!(
        connection.negotiated_key_exchange_group().map(|g| g.name()),
        Some(NamedGroup::X25519MLKEM768)
    );
    assert_eq!(
        connection.negotiated_cipher_suite().map(|s| s.suite()),
        Some(rustls::CipherSuite::TLS13_AES_256_GCM_SHA384)
    );
}

#[test]
fn h2_profile_client_with_openssl_server_is_pinned_hybrid_tls13() {
    let peers = Peers::new();
    let (rng, sends) = key_service_random(false);
    let server_pin = pin_of(&peers.server);
    let config =
        client_config(&Side::profile(rng), server_pin.clone(), Some(&peers.client)).unwrap();
    let mut node = NodePeer::spawn(
        "server",
        &peers.server,
        &peers.client,
        "X25519MLKEM768",
        None,
    );
    let ready = node.event();
    assert_modern_openssl(&ready);
    let port: u16 = ready.field("port").unwrap().parse().unwrap();

    let connection =
        ClientConnection::new(Arc::new(config), SERVER_NAME.try_into().unwrap()).unwrap();
    let mut tls = StreamOwned::new(
        connection,
        bounded(TcpStream::connect(("127.0.0.1", port)).unwrap()),
    );
    while tls.conn.is_handshaking() {
        tls.conn.complete_io(&mut tls.sock).unwrap();
    }
    assert_eq!(profile::accept(&tls.conn, ALPN, &server_pin), Ok(()));
    assert_profile_hybrid(&tls.conn);
    assert!(sends.load(Ordering::SeqCst) > 0);
    tls.write_all(b"profile client").unwrap();
    let mut reply = Vec::new();
    let _ = tls.read_to_end(&mut reply);
    assert_eq!(reply, b"openssl:profile client");
    tls.conn.send_close_notify();
    let _ = tls.conn.complete_io(&mut tls.sock);
    drop(tls);

    assert_peer_secure(&node.event());
    let (rest, code) = node.finish();
    assert_eq!(rest[0].field("text"), Some("profile client"));
    assert_eq!(code, 0);
}

#[test]
fn h2_openssl_client_with_profile_server_is_pinned_hybrid_tls13() {
    let peers = Peers::new();
    let (rng, sends) = key_service_random(false);
    let client_pin = pin_of(&peers.client);
    let config = server_config(&Side::profile(rng), client_pin.clone(), &peers.server).unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let node = NodePeer::spawn(
        "client",
        &peers.client,
        &peers.server,
        "X25519MLKEM768",
        Some(port),
    );

    let (socket, _) = listener.accept().unwrap();
    let mut tls = StreamOwned::new(
        ServerConnection::new(Arc::new(config)).unwrap(),
        bounded(socket),
    );
    while tls.conn.is_handshaking() {
        tls.conn.complete_io(&mut tls.sock).unwrap();
    }
    assert_eq!(profile::accept(&tls.conn, ALPN, &client_pin), Ok(()));
    assert_profile_hybrid(&tls.conn);
    assert!(sends.load(Ordering::SeqCst) > 0);
    let mut request = [0; 14];
    tls.read_exact(&mut request).unwrap();
    assert_eq!(&request, b"openssl client");
    tls.write_all(b"profile server").unwrap();
    tls.conn.send_close_notify();
    let _ = tls.conn.complete_io(&mut tls.sock);

    let (events, code) = node.finish();
    assert_peer_secure(&events[0]);
    assert_eq!(events[1].field("text"), Some("profile server"));
    assert_eq!(code, 0);
}

#[test]
fn h2_classical_only_openssl_server_gets_no_fallback() {
    let peers = Peers::new();
    let (rng, _) = key_service_random(false);
    let config = client_config(
        &Side::profile(rng),
        pin_of(&peers.server),
        Some(&peers.client),
    )
    .unwrap();
    let mut node = NodePeer::spawn("server", &peers.server, &peers.client, "X25519", None);
    let port: u16 = node.event().field("port").unwrap().parse().unwrap();

    let connection =
        ClientConnection::new(Arc::new(config), SERVER_NAME.try_into().unwrap()).unwrap();
    let mut tls = StreamOwned::new(
        connection,
        bounded(TcpStream::connect(("127.0.0.1", port)).unwrap()),
    );
    let mut result = Ok((0, 0));
    while tls.conn.is_handshaking() && result.is_ok() {
        result = tls.conn.complete_io(&mut tls.sock);
    }
    assert!(
        result.is_err(),
        "a classical-only peer must not complete a handshake"
    );
    assert!(tls.conn.negotiated_key_exchange_group().is_none());
    let (events, code) = node.finish();
    assert_eq!(events[0].field("event"), Some("error"), "{events:?}");
    assert_ne!(code, 0);
}
