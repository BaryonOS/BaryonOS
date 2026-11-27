//! ADR-N03-PROVIDER-20261125 B1 provider. The caller owns its authorized
//! RANDOM lease and resolves its signing identity through the TLS profile.

mod cipher_suite;
mod key_exchange;
mod signature;

use alloc::{sync::Arc, vec};
use nieuw_os_keyring_random_client::Transport;
use rustls::{
    crypto::{CryptoProvider, KeyProvider},
    pki_types::PrivateKeyDer,
    sign::SigningKey,
    Error,
};
use zeroize::Zeroize;

use crate::random::{KeyServiceRandom, MonotonicClock};

/// A TLS 1.3 provider bound to one Key-Service RANDOM lease.
pub struct Provider {
    group: key_exchange::HybridGroup,
}

impl Provider {
    pub fn new<T: Transport + Send + 'static, C: MonotonicClock + 'static>(
        random: &'static KeyServiceRandom<T, C>,
    ) -> Self {
        Self {
            group: key_exchange::HybridGroup(random),
        }
    }

    /// The provider and Rustls reference the same lease for independent draws.
    /// Use an explicitly resolved signing identity with the profile builders.
    pub fn crypto_provider(&'static self) -> CryptoProvider {
        CryptoProvider {
            cipher_suites: vec![cipher_suite::TLS13_AES_256_GCM_SHA384],
            kx_groups: vec![&self.group],
            signature_verification_algorithms: signature::VERIFY,
            secure_random: self.group.0,
            key_provider: &NoKeyImport,
        }
    }
}

#[derive(Debug)]
struct NoKeyImport;

impl KeyProvider for NoKeyImport {
    fn load_private_key(
        &self,
        mut key_der: PrivateKeyDer<'static>,
    ) -> Result<Arc<dyn SigningKey>, Error> {
        key_der.zeroize();
        Err(Error::General(
            "B1 requires a resolved signing identity".into(),
        ))
    }
}
