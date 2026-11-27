use ed25519_dalek::{Signature, VerifyingKey};
use rustls::{
    crypto::WebPkiSupportedAlgorithms,
    pki_types::{alg_id, AlgorithmIdentifier, InvalidSignature, SignatureVerificationAlgorithm},
    SignatureScheme,
};

#[derive(Debug)]
struct DalekEd25519;

impl SignatureVerificationAlgorithm for DalekEd25519 {
    fn verify_signature(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), InvalidSignature> {
        let key_bytes = public_key.try_into().map_err(|_| InvalidSignature)?;
        let key = VerifyingKey::from_bytes(key_bytes).map_err(|_| InvalidSignature)?;
        let signature = Signature::from_slice(signature).map_err(|_| InvalidSignature)?;
        key.verify_strict(message, &signature)
            .map_err(|_| InvalidSignature)
    }

    fn public_key_alg_id(&self) -> AlgorithmIdentifier {
        alg_id::ED25519
    }

    fn signature_alg_id(&self) -> AlgorithmIdentifier {
        alg_id::ED25519
    }
}

const ALGORITHMS: &[&dyn SignatureVerificationAlgorithm] = &[&DalekEd25519];

pub(super) const VERIFY: WebPkiSupportedAlgorithms = WebPkiSupportedAlgorithms {
    all: ALGORITHMS,
    mapping: &[(SignatureScheme::ED25519, ALGORITHMS)],
};
