//! ADR-028 peer profile over a caller-supplied rustls `CryptoProvider`.
//!
//! Profile: TLS 1.3 only, X25519MLKEM768 only, TLS13_AES_256_GCM_SHA384 only,
//! one fixed ALPN, mutual exact end-entity pinning, no resumption, tickets,
//! 0-RTT or half-RTT data, no classical fallback. This module selects no
//! provider and implements no cryptography: key exchange, AEAD and handshake
//! signature verification all come from the supplied provider. A provider that
//! lacks the profile group or suite is refused, never silently degraded.
//!
//! The pin is the peer identity: server name, trust roots and certificate
//! validity periods are not consulted (no trusted certificate-time authority
//! exists). The pin authenticates nothing without the TLS 1.3 CertificateVerify,
//! which is delegated to the provider's signature verification.
//!
//! A consumer must call [`accept`] after the handshake and before its first
//! application read or write; see the findings documented there.

use alloc::sync::Arc;
use alloc::vec::Vec;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::{ResolvesClientCert, Resumption};
use rustls::crypto::{verify_tls13_signature, CryptoProvider, WebPkiSupportedAlgorithms};
use rustls::pki_types::{CertificateDer, ServerName, SubjectPublicKeyInfoDer, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::server::{NoServerSessionStorage, ParsedCertificate, ResolvesServerCert};
use rustls::time_provider::TimeProvider;
use rustls::{
    CertificateError, CipherSuite, ClientConfig, CommonState, DigitallySignedStruct,
    DistinguishedName, Error, NamedGroup, PeerIncompatible, PeerMisbehaved, ProtocolVersion,
    ServerConfig, SignatureScheme, SupportedProtocolVersion,
};

static TLS13_ONLY: &[&SupportedProtocolVersion] = &[&rustls::version::TLS13];

/// Why a profile configuration or pin could not be built.
#[derive(Debug, PartialEq)]
pub enum ProfileError {
    /// The supplied provider offers no X25519MLKEM768 key exchange.
    MissingKxGroup,
    /// The supplied provider offers no TLS 1.3 TLS13_AES_256_GCM_SHA384 suite.
    MissingCipherSuite,
    /// The fixed ALPN identifier is empty or longer than 255 bytes.
    InvalidAlpn,
    /// The pinned SPKI is not the pinned certificate's own SPKI.
    InconsistentPin,
    /// No signature scheme allowed, or one the provider cannot verify.
    UnsupportedScheme,
    /// The pinned certificate could not be parsed, or rustls refused the config.
    Rustls(Error),
}

impl From<Error> for ProfileError {
    fn from(error: Error) -> Self {
        Self::Rustls(error)
    }
}

/// Restricts `provider` to the profile group and suite, or refuses it.
pub fn restrict(mut provider: CryptoProvider) -> Result<Arc<CryptoProvider>, ProfileError> {
    provider
        .kx_groups
        .retain(|group| group.name() == NamedGroup::X25519MLKEM768);
    if provider.kx_groups.is_empty() {
        return Err(ProfileError::MissingKxGroup);
    }
    provider.cipher_suites.retain(|suite| {
        suite.suite() == CipherSuite::TLS13_AES_256_GCM_SHA384
            && suite.version().version == ProtocolVersion::TLSv1_3
    });
    if provider.cipher_suites.is_empty() {
        return Err(ProfileError::MissingCipherSuite);
    }
    Ok(Arc::new(provider))
}

fn alpn_list(alpn: &[u8]) -> Result<Vec<Vec<u8>>, ProfileError> {
    if alpn.is_empty() || alpn.len() > 255 {
        return Err(ProfileError::InvalidAlpn);
    }
    Ok(alloc::vec![alpn.to_vec()])
}

/// Client side: pins `server`, always presents `identity`.
pub fn client_config(
    provider: CryptoProvider,
    time: Arc<dyn TimeProvider>,
    alpn: &[u8],
    server: Arc<PinnedPeer>,
    identity: Arc<dyn ResolvesClientCert>,
) -> Result<ClientConfig, ProfileError> {
    let alpn = alpn_list(alpn)?;
    let mut config = ClientConfig::builder_with_details(restrict(provider)?, time)
        .with_protocol_versions(TLS13_ONLY)?
        .dangerous()
        .with_custom_certificate_verifier(server)
        .with_client_cert_resolver(identity);
    config.alpn_protocols = alpn;
    config.resumption = Resumption::disabled();
    config.enable_early_data = false;
    Ok(config)
}

/// Server side: requires and pins `client`, presents `identity`.
pub fn server_config(
    provider: CryptoProvider,
    time: Arc<dyn TimeProvider>,
    alpn: &[u8],
    client: Arc<PinnedPeer>,
    identity: Arc<dyn ResolvesServerCert>,
) -> Result<ServerConfig, ProfileError> {
    let alpn = alpn_list(alpn)?;
    let mut config = ServerConfig::builder_with_details(restrict(provider)?, time)
        .with_protocol_versions(TLS13_ONLY)?
        .with_client_cert_verifier(client)
        .with_cert_resolver(identity);
    config.alpn_protocols = alpn;
    config.session_storage = Arc::new(NoServerSessionStorage {});
    config.send_tls13_tickets = 0;
    config.max_early_data_size = 0;
    config.send_half_rtt_data = false;
    Ok(config)
}

/// Exact end-entity pin (DER and SPKI), no intermediates, TLS 1.3 signatures
/// only, with an explicit set of allowed signature schemes.
#[derive(Debug)]
pub struct PinnedPeer {
    cert: CertificateDer<'static>,
    spki: SubjectPublicKeyInfoDer<'static>,
    algorithms: WebPkiSupportedAlgorithms,
    schemes: Vec<SignatureScheme>,
}

impl PinnedPeer {
    /// `algorithms` normally is the provider's `signature_verification_algorithms`.
    pub fn new(
        cert: CertificateDer<'static>,
        spki: SubjectPublicKeyInfoDer<'static>,
        algorithms: WebPkiSupportedAlgorithms,
        schemes: Vec<SignatureScheme>,
    ) -> Result<Self, ProfileError> {
        let supported = algorithms.supported_schemes();
        if schemes.is_empty() || !schemes.iter().all(|s| supported.contains(s)) {
            return Err(ProfileError::UnsupportedScheme);
        }
        if ParsedCertificate::try_from(&cert)?.subject_public_key_info() != spki {
            return Err(ProfileError::InconsistentPin);
        }
        Ok(Self {
            cert,
            spki,
            algorithms,
            schemes,
        })
    }

    /// Exact match of the presented end entity and an empty intermediate list.
    pub fn check(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
    ) -> Result<(), Error> {
        let parsed = ParsedCertificate::try_from(end_entity)
            .map_err(|_| Error::InvalidCertificate(CertificateError::BadEncoding))?;
        if intermediates.is_empty()
            && end_entity.as_ref() == self.cert.as_ref()
            && parsed.subject_public_key_info() == self.spki
        {
            Ok(())
        } else {
            Err(Error::InvalidCertificate(
                CertificateError::ApplicationVerificationFailure,
            ))
        }
    }

    fn tls13(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        if !self.schemes.contains(&dss.scheme) {
            return Err(PeerMisbehaved::SignedHandshakeWithUnadvertisedSigScheme.into());
        }
        // The pin authenticates nothing without proof of possession of the key.
        self.check(cert, &[])?;
        verify_tls13_signature(message, cert, dss, &self.algorithms)
    }
}

fn tls12_refused() -> Result<HandshakeSignatureValid, Error> {
    Err(Error::PeerIncompatible(
        PeerIncompatible::Tls12NotOfferedOrEnabled,
    ))
}

impl ServerCertVerifier for PinnedPeer {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        self.check(end_entity, intermediates)
            .map(|()| ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        tls12_refused()
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.tls13(message, cert, dss)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.schemes.clone()
    }
}

impl ClientCertVerifier for PinnedPeer {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }
    fn client_auth_mandatory(&self) -> bool {
        true
    }
    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        _: UnixTime,
    ) -> Result<ClientCertVerified, Error> {
        self.check(end_entity, intermediates)
            .map(|()| ClientCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        tls12_refused()
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.tls13(message, cert, dss)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.schemes.clone()
    }
}

/// First acceptance check that failed, in checking order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Handshaking,
    Version,
    Group,
    Suite,
    Alpn,
    Peer,
}

/// Profile acceptance gate over a completed connection's negotiated state.
///
/// Call it after the handshake and before the first application read or
/// write, and treat any `Err` as fatal. Two findings from the N03-T2 host
/// loopback make this mandatory rather than redundant with the configs:
///
/// 1. In TLS 1.3 the client completes, and its own gate passes, before the
///    server rejects a wrong or missing client certificate (or its
///    CertificateVerify). Only the server's later fatal alert ends the
///    exchange, so a client consumer must treat any received alert as revoking
///    the connection, including after this gate passed.
/// 2. rustls completes a handshake with a server that selects no ALPN; only
///    this gate refuses it. Client data written before the gate already reached
///    that server, so nothing may be written or read before the gate passes.
pub fn accept(state: &CommonState, alpn: &[u8], peer: &PinnedPeer) -> Result<(), Refusal> {
    if state.is_handshaking() {
        return Err(Refusal::Handshaking);
    }
    if state.protocol_version() != Some(ProtocolVersion::TLSv1_3) {
        return Err(Refusal::Version);
    }
    if state.negotiated_key_exchange_group().map(|g| g.name()) != Some(NamedGroup::X25519MLKEM768) {
        return Err(Refusal::Group);
    }
    if state.negotiated_cipher_suite().map(|s| s.suite())
        != Some(CipherSuite::TLS13_AES_256_GCM_SHA384)
    {
        return Err(Refusal::Suite);
    }
    if alpn.is_empty() || state.alpn_protocol() != Some(alpn) {
        return Err(Refusal::Alpn);
    }
    match state.peer_certificates() {
        Some([cert]) if peer.check(cert, &[]).is_ok() => Ok(()),
        _ => Err(Refusal::Peer),
    }
}
