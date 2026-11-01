//! Transport encryption for the Linux Secret Service (security §Credentials): the
//! `dh-ietf1024-sha256-aes128-cbc-pkcs7` session the spec defines. It protects secrets only
//! while they cross the session bus; at rest they belong to the user's own store.
//! <https://specifications.freedesktop.org/secret-service-spec/latest/transfer-secrets.html>

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use num_bigint::BigUint;
use zeroize::Zeroizing;

use crate::{CryptoError, random_bytes};

/// The algorithm name passed to `OpenSession`.
pub const ALGORITHM: &str = "dh-ietf1024-sha256-aes128-cbc-pkcs7";

/// RFC 2409 §6.2, the 1024-bit MODP group ("Second Oakley Group"), generator 2.
const PRIME: [u8; 128] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC9, 0x0F, 0xDA, 0xA2, 0x21, 0x68, 0xC2, 0x34,
    0xC4, 0xC6, 0x62, 0x8B, 0x80, 0xDC, 0x1C, 0xD1, 0x29, 0x02, 0x4E, 0x08, 0x8A, 0x67, 0xCC, 0x74,
    0x02, 0x0B, 0xBE, 0xA6, 0x3B, 0x13, 0x9B, 0x22, 0x51, 0x4A, 0x08, 0x79, 0x8E, 0x34, 0x04, 0xDD,
    0xEF, 0x95, 0x19, 0xB3, 0xCD, 0x3A, 0x43, 0x1B, 0x30, 0x2B, 0x0A, 0x6D, 0xF2, 0x5F, 0x14, 0x37,
    0x4F, 0xE1, 0x35, 0x6D, 0x6D, 0x51, 0xC2, 0x45, 0xE4, 0x85, 0xB5, 0x76, 0x62, 0x5E, 0x7E, 0xC6,
    0xF4, 0x4C, 0x42, 0xE9, 0xA6, 0x37, 0xED, 0x6B, 0x0B, 0xFF, 0x5C, 0xB6, 0xF4, 0x06, 0xB7, 0xED,
    0xEE, 0x38, 0x6B, 0xFB, 0x5A, 0x89, 0x9F, 0xA5, 0xAE, 0x9F, 0x24, 0x11, 0x7C, 0x4B, 0x1F, 0xE6,
    0x49, 0x28, 0x66, 0x51, 0xEC, 0xE6, 0x53, 0x81, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];

type Enc = cbc::Encryptor<aes::Aes128>;
type Dec = cbc::Decryptor<aes::Aes128>;

/// Our half of the key exchange, before the server answers.
pub struct DhSession {
    private: Zeroizing<[u8; 128]>,
    public: Vec<u8>,
}

impl DhSession {
    /// # Errors
    /// `Random` if the OS CSPRNG fails.
    pub fn new() -> Result<Self, CryptoError> {
        let private = Zeroizing::new(random_bytes::<128>()?);
        let p = BigUint::from_bytes_be(&PRIME);
        let public = BigUint::from(2u8)
            .modpow(&BigUint::from_bytes_be(&*private), &p)
            .to_bytes_be();
        Ok(Self { private, public })
    }

    /// Sent as the `OpenSession` input.
    #[must_use]
    pub fn public_key(&self) -> &[u8] {
        &self.public
    }

    /// The AES key from the server's public key: HKDF-SHA256 (no salt, no info) over the
    /// shared secret left-padded to 128 bytes.
    ///
    /// # Errors
    /// `Malformed` when the server's key is outside `2..p-1` (a degenerate key would make the
    /// shared secret guessable).
    pub fn finish(self, server_public: &[u8]) -> Result<TransportKey, CryptoError> {
        let p = BigUint::from_bytes_be(&PRIME);
        let y = BigUint::from_bytes_be(server_public);
        if y < BigUint::from(2u8) || y >= &p - 1u8 {
            return Err(CryptoError::Malformed);
        }
        let shared = y.modpow(&BigUint::from_bytes_be(&*self.private), &p);
        let bytes = Zeroizing::new(shared.to_bytes_be());
        let mut ikm = Zeroizing::new([0u8; 128]);
        ikm[128 - bytes.len()..].copy_from_slice(&bytes);
        let mut key = Zeroizing::new([0u8; 16]);
        hkdf::Hkdf::<sha2::Sha256>::new(None, &*ikm)
            .expand(&[], &mut *key)
            .map_err(|_| CryptoError::Kdf)?;
        Ok(TransportKey(key))
    }
}

/// The negotiated AES-128 key for one Secret Service session.
pub struct TransportKey(Zeroizing<[u8; 16]>);

impl TransportKey {
    /// Returns `(iv, ciphertext)`, the Secret struct's `parameters` and `value`.
    ///
    /// # Errors
    /// `Random` if the OS CSPRNG fails.
    pub fn encrypt(&self, plain: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let iv = random_bytes::<16>()?;
        let ct =
            Enc::new(self.0.as_ref().into(), &iv.into()).encrypt_padded_vec_mut::<Pkcs7>(plain);
        Ok((iv.to_vec(), ct))
    }

    /// # Errors
    /// `Malformed` for an IV that isn't 16 bytes, `Decrypt` for bad padding.
    pub fn decrypt(&self, iv: &[u8], ct: &[u8]) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
        let iv: [u8; 16] = iv.try_into().map_err(|_| CryptoError::Malformed)?;
        Dec::new(self.0.as_ref().into(), &iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ct)
            .map(Zeroizing::new)
            .map_err(|_| CryptoError::Decrypt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sides_derive_the_same_key() {
        let (client, server) = (DhSession::new().unwrap(), DhSession::new().unwrap());
        let (cp, sp) = (client.public_key().to_vec(), server.public_key().to_vec());
        let (ck, sk) = (client.finish(&sp).unwrap(), server.finish(&cp).unwrap());
        let (iv, ct) = ck.encrypt(b"hunter2").unwrap();
        assert_ne!(ct, b"hunter2");
        assert_eq!(sk.decrypt(&iv, &ct).unwrap().as_slice(), b"hunter2");
        assert!(sk.decrypt(&iv[..8], &ct).is_err());
    }

    #[test]
    fn degenerate_server_keys_are_refused() {
        let p = BigUint::from_bytes_be(&PRIME);
        for y in [
            BigUint::from(0u8),
            BigUint::from(1u8),
            &p - 1u8,
            p.clone(),
            &p + 1u8,
        ] {
            let s = DhSession::new().unwrap();
            assert_eq!(
                s.finish(&y.to_bytes_be()).err(),
                Some(CryptoError::Malformed),
                "{y}"
            );
        }
    }
}
