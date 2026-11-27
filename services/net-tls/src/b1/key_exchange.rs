use alloc::{boxed::Box, vec::Vec};
use ml_kem::{
    ml_kem_768::{Ciphertext, DecapsulationKey, EncapsulationKey},
    Decapsulate, KeyExport, Seed, TryKeyInit, B32,
};
use rustls::{
    crypto::{
        ActiveKeyExchange, CompletedKeyExchange, SecureRandom, SharedSecret, SupportedKxGroup,
    },
    ffdhe_groups::FfdheGroup,
    Error, NamedGroup, PeerMisbehaved, ProtocolVersion,
};
use x25519_dalek::{PublicKey, SharedSecret as X25519Secret, StaticSecret};
use zeroize::Zeroizing;

// RFC 10024 puts the ML-KEM component first in both shares and the secret.
const ENCAPSULATION_KEY_LEN: usize = 1184;
const CIPHERTEXT_LEN: usize = 1088;
const X25519_LEN: usize = 32;

#[derive(Debug)]
pub(super) struct HybridGroup(pub(super) &'static dyn SecureRandom);

struct ClientExchange {
    kem: DecapsulationKey,
    curve: StaticSecret,
    share: [u8; ENCAPSULATION_KEY_LEN + X25519_LEN],
}

fn invalid_share() -> Error {
    PeerMisbehaved::InvalidKeyShare.into()
}

fn split_share<const KEM_LEN: usize>(raw: &[u8]) -> Result<(&[u8], PublicKey), Error> {
    let (kem, curve) = raw.split_at_checked(KEM_LEN).ok_or_else(invalid_share)?;
    let point: [u8; X25519_LEN] = curve.try_into().map_err(|_| invalid_share())?;
    Ok((kem, PublicKey::from(point)))
}

fn curve_secret(our_key: &StaticSecret, peer_key: &PublicKey) -> Result<X25519Secret, Error> {
    let secret = our_key.diffie_hellman(peer_key);
    if !secret.was_contributory() {
        return Err(invalid_share());
    }
    Ok(secret)
}

fn combine(kem: &ml_kem::SharedKey, curve: &X25519Secret) -> SharedSecret {
    let mut bytes = Vec::with_capacity(64);
    bytes.extend_from_slice(kem);
    bytes.extend_from_slice(curve.as_bytes());
    SharedSecret::from(bytes)
}

impl HybridGroup {
    fn draw_curve(&self) -> Result<StaticSecret, Error> {
        let mut scalar = Zeroizing::new([0; X25519_LEN]);
        self.0.fill(scalar.as_mut())?;
        Ok(StaticSecret::from(*scalar))
    }
}

impl SupportedKxGroup for HybridGroup {
    fn start(&self) -> Result<Box<dyn ActiveKeyExchange>, Error> {
        let curve = self.draw_curve()?;
        let mut seed = Zeroizing::new(Seed::default());
        self.0.fill(seed.as_mut())?;
        let kem = DecapsulationKey::from_seed(*seed);
        let mut share = [0; ENCAPSULATION_KEY_LEN + X25519_LEN];
        let (kem_share, curve_share) = share.split_at_mut(ENCAPSULATION_KEY_LEN);
        kem_share.copy_from_slice(&kem.encapsulation_key().to_bytes());
        curve_share.copy_from_slice(PublicKey::from(&curve).as_bytes());
        Ok(Box::new(ClientExchange { kem, curve, share }))
    }

    fn start_and_complete(&self, peer_pub_key: &[u8]) -> Result<CompletedKeyExchange, Error> {
        let (encoded_key, peer_curve) = split_share::<ENCAPSULATION_KEY_LEN>(peer_pub_key)?;
        let kem = EncapsulationKey::new_from_slice(encoded_key).map_err(|_| invalid_share())?;
        let curve = self.draw_curve()?;
        let curve_secret = curve_secret(&curve, &peer_curve)?;
        let mut message = Zeroizing::new(B32::default());
        self.0.fill(message.as_mut())?;
        let (ciphertext, kem_secret) = kem.encapsulate_deterministic(&message);
        let kem_secret = Zeroizing::new(kem_secret);
        let mut share = Vec::with_capacity(CIPHERTEXT_LEN + X25519_LEN);
        share.extend_from_slice(&ciphertext);
        share.extend_from_slice(PublicKey::from(&curve).as_bytes());
        Ok(CompletedKeyExchange {
            group: self.name(),
            pub_key: share,
            secret: combine(&kem_secret, &curve_secret),
        })
    }

    fn ffdhe_group(&self) -> Option<FfdheGroup<'static>> {
        None
    }

    fn name(&self) -> NamedGroup {
        NamedGroup::X25519MLKEM768
    }

    fn usable_for_version(&self, version: ProtocolVersion) -> bool {
        version == ProtocolVersion::TLSv1_3
    }
}

impl ActiveKeyExchange for ClientExchange {
    fn complete(self: Box<Self>, peer_pub_key: &[u8]) -> Result<SharedSecret, Error> {
        let (encoded_ciphertext, peer_curve) = split_share::<CIPHERTEXT_LEN>(peer_pub_key)?;
        let curve_secret = curve_secret(&self.curve, &peer_curve)?;
        let ciphertext = Ciphertext::try_from(encoded_ciphertext).map_err(|_| invalid_share())?;
        let kem_secret = Zeroizing::new(self.kem.decapsulate(&ciphertext));
        Ok(combine(&kem_secret, &curve_secret))
    }

    fn pub_key(&self) -> &[u8] {
        &self.share
    }

    fn group(&self) -> NamedGroup {
        NamedGroup::X25519MLKEM768
    }

    fn ffdhe_group(&self) -> Option<FfdheGroup<'static>> {
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };

    use super::HybridGroup;
    use alloc::vec::Vec;
    use rustls::{
        crypto::{aws_lc_rs, GetRandomFailed, SecureRandom, SupportedKxGroup},
        Error, NamedGroup, PeerMisbehaved,
    };

    #[derive(Debug)]
    struct Draws {
        lengths: Mutex<Vec<usize>>,
        fail_at: AtomicUsize,
    }

    impl SecureRandom for Draws {
        fn fill(&self, bytes: &mut [u8]) -> Result<(), GetRandomFailed> {
            let mut lengths = self.lengths.lock().unwrap();
            lengths.push(bytes.len());
            let draw = lengths.len();
            if draw == self.fail_at.load(Ordering::Relaxed) {
                return Err(GetRandomFailed);
            }
            bytes.fill(u8::try_from(draw).unwrap());
            Ok(())
        }
    }

    #[derive(Debug)]
    struct KnownDraws(Mutex<core::array::IntoIter<&'static [u8], 2>>);

    impl SecureRandom for KnownDraws {
        fn fill(&self, output: &mut [u8]) -> Result<(), GetRandomFailed> {
            let bytes = self.0.lock().unwrap().next().ok_or(GetRandomFailed)?;
            if output.len() != bytes.len() {
                return Err(GetRandomFailed);
            }
            output.copy_from_slice(bytes);
            Ok(())
        }
    }

    fn known_group(draws: [&'static [u8]; 2]) -> HybridGroup {
        use alloc::boxed::Box;
        HybridGroup(Box::leak(Box::new(KnownDraws(Mutex::new(
            draws.into_iter(),
        )))))
    }

    #[test]
    fn hybrid_wire_layout_matches_independent_known_answers() {
        // NIST ACVP ML-KEM-768 key generation and encapsulation, plus RFC 7748 §6.1.
        // The published sample key material belongs exclusively to this fixture.
        let fixture = include_bytes!("../../tests/fixtures/x25519mlkem768-kat.bin");
        let (alice, rest) = fixture.split_at(32);
        let (seed, rest) = rest.split_at(64);
        let (client_share, rest) = rest.split_at(1216);
        let (bob, rest) = rest.split_at(32);
        let (message, rest) = rest.split_at(32);
        let (peer_share, rest) = rest.split_at(1216);
        let (server_share, shared_secret) = rest.split_at(1120);
        assert_eq!(shared_secret.len(), 64);

        let client = known_group([alice, seed]).start().unwrap();
        assert_eq!(client.pub_key(), client_share);
        let server = known_group([bob, message])
            .start_and_complete(peer_share)
            .unwrap();
        assert_eq!(server.pub_key, server_share);
        assert_eq!(server.secret.secret_bytes(), shared_secret);
    }

    #[test]
    fn both_roles_interoperate_with_aws_lc_and_refuse_invalid_inputs() {
        static DRAWS: Draws = Draws {
            lengths: Mutex::new(Vec::new()),
            fail_at: AtomicUsize::new(0),
        };
        let group = HybridGroup(&DRAWS);
        let oracle = aws_lc_rs::default_provider()
            .kx_groups
            .into_iter()
            .find(|group| group.name() == NamedGroup::X25519MLKEM768)
            .unwrap();

        let client = group.start().unwrap();
        assert_eq!(client.pub_key().len(), 1216);
        let server = oracle.start_and_complete(client.pub_key()).unwrap();
        assert_eq!(server.pub_key.len(), 1120);
        let result = client.complete(&server.pub_key).unwrap();
        assert_eq!(result.secret_bytes().len(), 64);
        assert_eq!(result.secret_bytes(), server.secret.secret_bytes());
        assert_eq!(*DRAWS.lengths.lock().unwrap(), [32, 64]);

        let client = oracle.start().unwrap();
        let server = group.start_and_complete(client.pub_key()).unwrap();
        assert_eq!(server.pub_key.len(), 1120);
        let result = client.complete(&server.pub_key).unwrap();
        assert_eq!(result.secret_bytes(), server.secret.secret_bytes());
        assert_eq!(*DRAWS.lengths.lock().unwrap(), [32, 64, 32, 32]);

        let oracle_client = oracle.start().unwrap();
        let mut invalid = oracle_client.pub_key().to_vec();
        assert!(matches!(
            group.start_and_complete(&invalid[..1215]),
            Err(Error::PeerMisbehaved(PeerMisbehaved::InvalidKeyShare))
        ));
        invalid[0] = 0xff;
        invalid[1] |= 0x0f;
        assert!(matches!(
            group.start_and_complete(&invalid),
            Err(Error::PeerMisbehaved(PeerMisbehaved::InvalidKeyShare))
        ));
        assert_eq!(*DRAWS.lengths.lock().unwrap(), [32, 64, 32, 32]);

        invalid.copy_from_slice(oracle_client.pub_key());
        invalid[1184..].fill(0);
        assert!(matches!(
            group.start_and_complete(&invalid),
            Err(Error::PeerMisbehaved(PeerMisbehaved::InvalidKeyShare))
        ));
        assert_eq!(*DRAWS.lengths.lock().unwrap(), [32, 64, 32, 32, 32]);

        let client = group.start().unwrap();
        let mut server = oracle.start_and_complete(client.pub_key()).unwrap();
        server.pub_key[1088..].fill(0);
        assert!(matches!(
            client.complete(&server.pub_key),
            Err(Error::PeerMisbehaved(PeerMisbehaved::InvalidKeyShare))
        ));

        let client = group.start().unwrap();
        let server = oracle.start_and_complete(client.pub_key()).unwrap();
        assert!(matches!(
            client.complete(&server.pub_key[..1119]),
            Err(Error::PeerMisbehaved(PeerMisbehaved::InvalidKeyShare))
        ));

        let next_failure = DRAWS.lengths.lock().unwrap().len() + 2;
        DRAWS.fail_at.store(next_failure, Ordering::Relaxed);
        assert!(matches!(group.start(), Err(Error::FailedToGetRandomBytes)));
        assert_eq!(DRAWS.lengths.lock().unwrap().len(), next_failure);
    }
}
