use aes_gcm::{AeadInOut, Aes256Gcm, KeyInit};
use alloc::boxed::Box;
use hmac::{Hmac, Mac};
use rustls::{
    crypto::{
        cipher::{
            make_tls13_aad, AeadKey, InboundOpaqueMessage, InboundPlainMessage, Iv,
            MessageDecrypter, MessageEncrypter, Nonce, OutboundOpaqueMessage, OutboundPlainMessage,
            PrefixedPayload, Tls13AeadAlgorithm, UnsupportedOperationError,
        },
        hash, hmac as rustls_hmac,
        tls13::HkdfUsingHmac,
    },
    CipherSuite, CipherSuiteCommon, ConnectionTrafficSecrets, ContentType, Error, ProtocolVersion,
    SupportedCipherSuite, Tls13CipherSuite,
};
use sha2::{Digest, Sha384};
use zeroize::Zeroize;

const HASH_LEN: usize = 48;
const AES_KEY_LEN: usize = 32;
const TAG_LEN: usize = 16;
const TLS_PLAINTEXT_LIMIT: usize = 1 << 14;
const TLS_CIPHERTEXT_LIMIT: usize = TLS_PLAINTEXT_LIMIT + 256;

struct Sha384Hash;
struct Transcript(Sha384);

impl hash::Hash for Sha384Hash {
    fn start(&self) -> Box<dyn hash::Context> {
        Box::new(Transcript(Sha384::new()))
    }

    fn hash(&self, data: &[u8]) -> hash::Output {
        hash::Output::new(&Sha384::digest(data))
    }

    fn output_len(&self) -> usize {
        HASH_LEN
    }

    fn algorithm(&self) -> hash::HashAlgorithm {
        hash::HashAlgorithm::SHA384
    }
}

impl hash::Context for Transcript {
    fn fork_finish(&self) -> hash::Output {
        hash::Output::new(&self.0.clone().finalize())
    }

    fn fork(&self) -> Box<dyn hash::Context> {
        Box::new(Self(self.0.clone()))
    }

    fn finish(self: Box<Self>) -> hash::Output {
        hash::Output::new(&self.0.finalize())
    }

    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }
}

struct HmacSha384;
struct MacKey(Hmac<Sha384>);

impl rustls_hmac::Hmac for HmacSha384 {
    fn with_key(&self, key: &[u8]) -> Box<dyn rustls_hmac::Key> {
        Box::new(MacKey(
            <Hmac<Sha384> as hmac::KeyInit>::new_from_slice(key)
                .expect("HMAC supports arbitrary key lengths"),
        ))
    }

    fn hash_output_len(&self) -> usize {
        HASH_LEN
    }
}

impl rustls_hmac::Key for MacKey {
    fn sign_concat(&self, first: &[u8], middle: &[&[u8]], last: &[u8]) -> rustls_hmac::Tag {
        let mut mac = self.0.clone();
        mac.update(first);
        for bytes in middle {
            mac.update(bytes);
        }
        mac.update(last);
        let mut output = mac.finalize().into_bytes();
        let tag = rustls_hmac::Tag::new(&output);
        output.as_mut_slice().zeroize();
        tag
    }

    fn tag_len(&self) -> usize {
        HASH_LEN
    }
}

struct AesRecords;
struct RecordsCipher {
    key: Aes256Gcm,
    iv: Iv,
}

impl RecordsCipher {
    fn new(key: AeadKey, iv: Iv) -> Self {
        let key =
            Aes256Gcm::new_from_slice(key.as_ref()).expect("rustls derives the suite key length");
        Self { key, iv }
    }
}

impl Tls13AeadAlgorithm for AesRecords {
    fn encrypter(&self, key: AeadKey, iv: Iv) -> Box<dyn MessageEncrypter> {
        Box::new(RecordsCipher::new(key, iv))
    }

    fn decrypter(&self, key: AeadKey, iv: Iv) -> Box<dyn MessageDecrypter> {
        Box::new(RecordsCipher::new(key, iv))
    }

    fn key_len(&self) -> usize {
        AES_KEY_LEN
    }

    fn extract_keys(
        &self,
        _: AeadKey,
        _: Iv,
    ) -> Result<ConnectionTrafficSecrets, UnsupportedOperationError> {
        Err(UnsupportedOperationError)
    }
}

impl MessageEncrypter for RecordsCipher {
    fn encrypt(
        &mut self,
        message: OutboundPlainMessage<'_>,
        sequence: u64,
    ) -> Result<OutboundOpaqueMessage, Error> {
        if message.payload.len() > TLS_PLAINTEXT_LIMIT {
            return Err(Error::EncryptError);
        }
        let length = self.encrypted_payload_len(message.payload.len());
        let mut payload = PrefixedPayload::with_capacity(length);
        payload.extend_from_chunks(&message.payload);
        payload.extend_from_slice(&[message.typ.into()]);
        let nonce = Nonce::new(&self.iv, sequence).0.into();
        let tag = self
            .key
            .encrypt_inout_detached(&nonce, &make_tls13_aad(length), payload.as_mut().into())
            .map_err(|_| Error::EncryptError)?;
        payload.extend_from_slice(&tag);
        Ok(OutboundOpaqueMessage::new(
            ContentType::ApplicationData,
            ProtocolVersion::TLSv1_2,
            payload,
        ))
    }

    fn encrypted_payload_len(&self, plaintext_length: usize) -> usize {
        plaintext_length.saturating_add(1 + TAG_LEN)
    }
}

impl MessageDecrypter for RecordsCipher {
    fn decrypt<'a>(
        &mut self,
        mut message: InboundOpaqueMessage<'a>,
        sequence: u64,
    ) -> Result<InboundPlainMessage<'a>, Error> {
        let length = message.payload.len();
        if !(TAG_LEN + 1..=TLS_CIPHERTEXT_LIMIT).contains(&length) {
            return Err(Error::DecryptError);
        }
        let clear_length = length - TAG_LEN;
        let (ciphertext, tag) = message.payload.split_at_mut(clear_length);
        let tag =
            aes_gcm::aead::Tag::<Aes256Gcm>::try_from(&*tag).map_err(|_| Error::DecryptError)?;
        let nonce = Nonce::new(&self.iv, sequence).0.into();
        self.key
            .decrypt_inout_detached(&nonce, &make_tls13_aad(length), ciphertext.into(), &tag)
            .map_err(|_| Error::DecryptError)?;
        message.payload.truncate(clear_length);
        message.into_tls13_unpadded_message()
    }
}

pub(super) static TLS13_AES_256_GCM_SHA384: SupportedCipherSuite =
    SupportedCipherSuite::Tls13(&Tls13CipherSuite {
        common: CipherSuiteCommon {
            suite: CipherSuite::TLS13_AES_256_GCM_SHA384,
            hash_provider: &Sha384Hash,
            confidentiality_limit: 1 << 24,
        },
        hkdf_provider: &HkdfUsingHmac(&HmacSha384),
        aead_alg: &AesRecords,
        quic: None,
    });

#[cfg(test)]
mod tests {
    use super::TLS13_AES_256_GCM_SHA384;
    use rustls::{
        crypto::cipher::{InboundOpaqueMessage, OutboundChunks, OutboundPlainMessage},
        ContentType, Error, ProtocolVersion, SupportedCipherSuite,
    };

    #[test]
    fn corrupted_and_wrong_sequence_records_release_no_plaintext() {
        let SupportedCipherSuite::Tls13(suite) = TLS13_AES_256_GCM_SHA384 else {
            panic!("TLS 1.3 suite expected");
        };
        let mut enc = suite
            .aead_alg
            .encrypter([0x41; 32].into(), [0x29; 12].into());
        let mut dec = suite
            .aead_alg
            .decrypter([0x41; 32].into(), [0x29; 12].into());
        let outbound = enc
            .encrypt(
                OutboundPlainMessage {
                    typ: ContentType::ApplicationData,
                    version: ProtocolVersion::TLSv1_3,
                    payload: OutboundChunks::Single(b"test-only record"),
                },
                7,
            )
            .unwrap();

        let mut valid = outbound.payload.as_ref().to_vec();
        let prefix = &valid[..valid.len() - 1];
        let mut truncated = prefix.to_vec();
        let mut corrupted = valid.clone();
        *corrupted.last_mut().unwrap() ^= 1;
        for (wire, sequence) in [(&mut truncated, 7), (&mut corrupted, 7), (&mut valid, 8)] {
            let msg = InboundOpaqueMessage::new(outbound.typ, outbound.version, wire);
            assert!(matches!(
                dec.decrypt(msg, sequence),
                Err(Error::DecryptError)
            ));
        }

        let mut valid = outbound.payload.as_ref().to_vec();
        let msg = InboundOpaqueMessage::new(outbound.typ, outbound.version, &mut valid);
        assert_eq!(dec.decrypt(msg, 7).unwrap().payload, b"test-only record");
    }
}
