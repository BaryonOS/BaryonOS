use ed25519_dalek::{Signature, VerifyingKey};
use rustls::{
    crypto::WebPkiSupportedAlgorithms,
    pki_types::{alg_id, AlgorithmIdentifier, InvalidSignature, SignatureVerificationAlgorithm},
    SignatureScheme,
};

fn decode_verifying_key(public_key: &[u8]) -> Result<VerifyingKey, InvalidSignature> {
    let key_bytes = public_key.try_into().map_err(|_| InvalidSignature)?;
    let key = VerifyingKey::from_bytes(key_bytes).map_err(|_| InvalidSignature)?;
    if key.to_edwards().compress().to_bytes() != *key_bytes {
        return Err(InvalidSignature);
    }
    Ok(key)
}

#[derive(Debug)]
struct DalekEd25519;

impl SignatureVerificationAlgorithm for DalekEd25519 {
    fn verify_signature(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), InvalidSignature> {
        let key = decode_verifying_key(public_key)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_noncanonical_ed25519_public_key_encoding() {
        // Encoding of y = p + 1. The compatibility parser reduces it to y = 1.
        let mut noncanonical = [0xff; 32];
        noncanonical[0] = 0xee;
        noncanonical[31] = 0x7f;
        assert!(VerifyingKey::from_bytes(&noncanonical).is_ok());
        assert!(decode_verifying_key(&noncanonical).is_err());
    }
}
