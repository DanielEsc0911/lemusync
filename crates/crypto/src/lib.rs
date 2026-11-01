//! Encryption, key derivation and keyed hashing. Pure, no I/O except the OS CSPRNG.
//! Spec: docs/specs/security.md (Cryptography, Keys).

use std::fmt;

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub use zeroize::Zeroizing;

#[cfg(target_os = "linux")]
pub mod secret_service;

pub const OBJECT_CONTEXT: &str = "lemusync v1 object";
pub const BLOB_ID_CONTEXT: &str = "lemusync v1 blob-id";
pub const STREAM_ID_CONTEXT: &str = "lemusync v1 stream-id";
pub const RECOVERY_CONTEXT: &str = "lemusync v1 recovery-kek";

const MAGIC: &[u8; 4] = b"LSE1";
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const RECOVERY_CHARS: usize = 52; // 256 bits in 5-bit groups, last group padded

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("the OS random number generator failed")]
    Random,
    #[error("encryption failed")]
    Encrypt,
    #[error("authentication failed: wrong key or tampered data")]
    Decrypt,
    #[error("not a LemuSync encrypted object")]
    Malformed,
    #[error("key derivation parameters are out of the allowed range")]
    KdfParams,
    #[error("key derivation failed")]
    Kdf,
    #[error("that recovery key isn't valid")]
    RecoveryKey,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Key([u8; 32]);

impl Key {
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// # Errors
    /// `CryptoError::Random` if the OS CSPRNG fails.
    pub fn random() -> Result<Self, CryptoError> {
        random_bytes().map(Self)
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Key(..)")
    }
}

/// # Errors
/// `CryptoError::Random` if the OS CSPRNG fails.
pub fn random_bytes<const N: usize>() -> Result<[u8; N], CryptoError> {
    let mut out = [0u8; N];
    getrandom::fill(&mut out).map_err(|_| CryptoError::Random)?;
    Ok(out)
}

#[must_use]
pub fn derive(master: &Key, context: &str) -> Key {
    Key(blake3::derive_key(context, master.as_bytes()))
}

#[must_use]
pub fn keyed_hash(key: &Key, data: &[u8]) -> [u8; 32] {
    blake3::keyed_hash(key.as_bytes(), data).into()
}

/// Encrypts with a fresh random nonce. Layout: `LSE1` ‖ nonce ‖ ciphertext+tag.
///
/// # Errors
/// `Random` or `Encrypt`.
pub fn seal(key: &Key, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let nonce: [u8; NONCE_LEN] = random_bytes()?;
    let cipher =
        XChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|_| CryptoError::Encrypt)?;
    let ct = cipher
        .encrypt(
            &XNonce::from(nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Encrypt)?;
    let mut out = Vec::with_capacity(MAGIC.len() + NONCE_LEN + ct.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// # Errors
/// `Malformed` for a wrong header or length, `Decrypt` when authentication fails.
pub fn open(key: &Key, aad: &[u8], sealed: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if sealed.len() < MAGIC.len() + NONCE_LEN + TAG_LEN || &sealed[..MAGIC.len()] != MAGIC {
        return Err(CryptoError::Malformed);
    }
    let (nonce, ct) = sealed[MAGIC.len()..].split_at(NONCE_LEN);
    let cipher =
        XChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|_| CryptoError::Decrypt)?;
    cipher
        .decrypt(
            &XNonce::try_from(nonce).map_err(|_| CryptoError::Malformed)?,
            Payload { msg: ct, aad },
        )
        .map_err(|_| CryptoError::Decrypt)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl KdfParams {
    pub const DEFAULT: Self = Self {
        m_kib: 65_536,
        t: 3,
        p: 1,
    };

    /// Bounds from security §Cryptography: a tampered `repo.json` can't weaken or exhaust.
    ///
    /// # Errors
    /// `CryptoError::KdfParams` when out of range.
    pub fn check(&self) -> Result<(), CryptoError> {
        let ok = (19_456..=1_048_576).contains(&self.m_kib)
            && (1..=10).contains(&self.t)
            && (1..=4).contains(&self.p);
        if ok {
            Ok(())
        } else {
            Err(CryptoError::KdfParams)
        }
    }
}

/// Argon2id passphrase → key-encryption key.
///
/// # Errors
/// `KdfParams` for out-of-range parameters, `Kdf` if hashing fails.
pub fn passphrase_kek(
    passphrase: &[u8],
    salt: &[u8; 16],
    params: &KdfParams,
) -> Result<Key, CryptoError> {
    params.check()?;
    let p = argon2::Params::new(params.m_kib, params.t, params.p, Some(32))
        .map_err(|_| CryptoError::KdfParams)?;
    let block_count = p.block_count();
    let argon = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, p);
    let mut out = Zeroizing::new([0u8; 32]);
    // Caller-owned memory, wiped on drop (with argon2's `zeroize` feature, H0 and the final block are wiped too).
    let mut blocks = Zeroizing::new(vec![argon2::Block::default(); block_count]);
    argon
        .hash_password_into_with_memory(passphrase, salt, out.as_mut(), blocks.as_mut_slice())
        .map_err(|_| CryptoError::Kdf)?;
    Ok(Key(*out))
}

/// The three subkeys every repo object uses (security §Keys).
pub struct RepoKeys {
    pub object: Key,
    pub blob_id: Key,
    pub stream_id: Key,
}

impl RepoKeys {
    #[must_use]
    pub fn from_master(master: &Key) -> Self {
        Self {
            object: derive(master, OBJECT_CONTEXT),
            blob_id: derive(master, BLOB_ID_CONTEXT),
            stream_id: derive(master, STREAM_ID_CONTEXT),
        }
    }
}

/// # Errors
/// Same as `seal`.
pub fn wrap(kek: &Key, aad: &[u8], master: &Key) -> Result<Vec<u8>, CryptoError> {
    seal(kek, aad, master.as_bytes())
}

/// # Errors
/// Same as `open`, plus `Malformed` if the payload isn't 32 bytes.
pub fn unwrap_key(kek: &Key, aad: &[u8], wrapped: &[u8]) -> Result<Key, CryptoError> {
    let plain = Zeroizing::new(open(kek, aad, wrapped)?);
    let bytes: Zeroizing<[u8; 32]> = Zeroizing::new(
        plain
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::Malformed)?,
    );
    Ok(Key(*bytes))
}

/// 32 random bytes the user writes down at setup.
pub struct RecoveryKey(Key);

impl RecoveryKey {
    /// # Errors
    /// `CryptoError::Random`.
    pub fn generate() -> Result<Self, CryptoError> {
        Key::random().map(Self)
    }

    #[must_use]
    pub fn kek(&self) -> Key {
        derive(&self.0, RECOVERY_CONTEXT)
    }

    #[must_use]
    pub fn to_display(&self) -> Zeroizing<String> {
        let bits = self.0.as_bytes();
        let mut text = Zeroizing::new(String::with_capacity(RECOVERY_CHARS + RECOVERY_CHARS / 4));
        for i in 0..RECOVERY_CHARS {
            if i > 0 && i % 4 == 0 {
                text.push('-');
            }
            let mut v = 0usize;
            for b in 0..5 {
                let bit = i * 5 + b;
                let set = bit < 256 && bits[bit / 8] & (0x80 >> (bit % 8)) != 0;
                v = (v << 1) | usize::from(set);
            }
            text.push(char::from(CROCKFORD[v]));
        }
        text
    }

    /// Accepts any case, spaces and dashes, and Crockford's `O`→`0`, `I`/`L`→`1`.
    ///
    /// # Errors
    /// `CryptoError::RecoveryKey`.
    pub fn parse(text: &str) -> Result<Self, CryptoError> {
        let digits: Zeroizing<Vec<usize>> = text
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .map(|c| {
                let c = match c.to_ascii_uppercase() {
                    'O' => '0',
                    'I' | 'L' => '1',
                    other => other,
                };
                CROCKFORD
                    .iter()
                    .position(|d| char::from(*d) == c)
                    .ok_or(CryptoError::RecoveryKey)
            })
            .collect::<Result<Vec<usize>, _>>()
            .map(Zeroizing::new)?;
        if digits.len() != RECOVERY_CHARS {
            return Err(CryptoError::RecoveryKey);
        }
        let mut out = Zeroizing::new([0u8; 32]);
        for (i, d) in digits.iter().enumerate() {
            for b in 0..5 {
                let bit = i * 5 + b;
                let set = d & (0x10 >> b) != 0;
                if bit >= 256 {
                    if set {
                        return Err(CryptoError::RecoveryKey);
                    }
                } else if set {
                    out[bit / 8] |= 0x80 >> (bit % 8);
                }
            }
        }
        Ok(Self(Key(*out)))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::*;

    #[test]
    fn seal_open_round_trip_and_tamper_detection() {
        let key = Key::from_bytes([7; 32]);
        let sealed = seal(&key, b"blob\0abc", b"save data").unwrap();
        assert_eq!(&sealed[..4], b"LSE1");
        assert_eq!(open(&key, b"blob\0abc", &sealed).unwrap(), b"save data");
        assert_eq!(
            open(&key, b"blob\0abd", &sealed),
            Err(CryptoError::Decrypt),
            "AAD binds the id"
        );
        assert_eq!(
            open(&Key::from_bytes([8; 32]), b"blob\0abc", &sealed),
            Err(CryptoError::Decrypt)
        );
        let mut flipped = sealed.clone();
        let last = flipped.len() - 1;
        flipped[last] ^= 1;
        assert_eq!(
            open(&key, b"blob\0abc", &flipped),
            Err(CryptoError::Decrypt)
        );
        assert_eq!(
            open(&key, b"blob\0abc", b"LSE1short"),
            Err(CryptoError::Malformed)
        );
        assert_eq!(
            open(&key, b"blob\0abc", &[b'X'; 64]),
            Err(CryptoError::Malformed)
        );
    }

    #[test]
    fn every_seal_uses_a_fresh_nonce() {
        let key = Key::from_bytes([7; 32]);
        assert_ne!(
            seal(&key, b"", b"x").unwrap(),
            seal(&key, b"", b"x").unwrap()
        );
    }

    #[test]
    fn subkeys_differ_and_are_stable() {
        let master = Key::from_bytes([1; 32]);
        let keys = RepoKeys::from_master(&master);
        assert_ne!(keys.object.as_bytes(), keys.blob_id.as_bytes());
        assert_ne!(keys.blob_id.as_bytes(), keys.stream_id.as_bytes());
        assert_eq!(
            RepoKeys::from_master(&master).object.as_bytes(),
            keys.object.as_bytes()
        );
        assert_eq!(
            keyed_hash(&keys.blob_id, b"a"),
            keyed_hash(&keys.blob_id, b"a")
        );
        assert_ne!(
            keyed_hash(&keys.blob_id, b"a"),
            keyed_hash(&keys.stream_id, b"a")
        );
    }

    #[test]
    fn argon2id_matches_rfc_9106() {
        // RFC 9106 section 5.3 (Argon2id): checks the dependency is wired as Argon2id v0x13.
        let params = argon2::ParamsBuilder::new()
            .m_cost(32)
            .t_cost(3)
            .p_cost(4)
            .data(argon2::AssociatedData::new(&[4; 12]).unwrap())
            .output_len(32)
            .build()
            .unwrap();
        let a = argon2::Argon2::new_with_secret(
            &[3; 8],
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            params,
        )
        .unwrap();
        let mut out = [0u8; 32];
        a.hash_password_into(&[1; 32], &[2; 16], &mut out).unwrap();
        let hex = out.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        });
        assert_eq!(
            hex,
            "0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659"
        );
    }

    #[test]
    fn passphrase_kek_is_deterministic_and_bounded() {
        let salt = [9; 16];
        let small = KdfParams {
            m_kib: 19_456,
            t: 1,
            p: 1,
        };
        let a = passphrase_kek(b"correct horse", &salt, &small).unwrap();
        let b = passphrase_kek(b"correct horse", &salt, &small).unwrap();
        assert_eq!(a.as_bytes(), b.as_bytes());
        assert_ne!(
            passphrase_kek(b"wrong", &salt, &small).unwrap().as_bytes(),
            a.as_bytes()
        );
        for bad in [
            KdfParams {
                m_kib: 1024,
                t: 3,
                p: 1,
            },
            KdfParams {
                m_kib: 2_000_000,
                t: 3,
                p: 1,
            },
            KdfParams {
                m_kib: 65_536,
                t: 0,
                p: 1,
            },
            KdfParams {
                m_kib: 65_536,
                t: 11,
                p: 1,
            },
            KdfParams {
                m_kib: 65_536,
                t: 3,
                p: 5,
            },
        ] {
            assert_eq!(
                passphrase_kek(b"x", &salt, &bad).err(),
                Some(CryptoError::KdfParams)
            );
        }
        assert!(KdfParams::DEFAULT.check().is_ok());
    }

    #[test]
    fn wrap_unwrap_with_both_keks() {
        let master = Key::random().unwrap();
        let kek = Key::random().unwrap();
        let wrapped = wrap(&kek, b"wrap\0passphrase", &master).unwrap();
        assert_eq!(
            unwrap_key(&kek, b"wrap\0passphrase", &wrapped)
                .unwrap()
                .as_bytes(),
            master.as_bytes()
        );
        assert_eq!(
            unwrap_key(&kek, b"wrap\0recovery", &wrapped).err(),
            Some(CryptoError::Decrypt)
        );
        let recovery = RecoveryKey::generate().unwrap();
        let wrapped = wrap(&recovery.kek(), b"wrap\0recovery", &master).unwrap();
        let typed =
            RecoveryKey::parse(&recovery.to_display().to_lowercase().replace('-', " ")).unwrap();
        assert_eq!(
            unwrap_key(&typed.kek(), b"wrap\0recovery", &wrapped)
                .unwrap()
                .as_bytes(),
            master.as_bytes()
        );
    }

    #[test]
    fn recovery_key_display_and_parse() {
        let r = RecoveryKey::generate().unwrap();
        let text = r.to_display();
        assert_eq!(text.len(), 13 * 4 + 12);
        assert_eq!(text.split('-').count(), 13);
        assert_eq!(
            RecoveryKey::parse(&text).unwrap().kek().as_bytes(),
            r.kek().as_bytes()
        );
        let ambiguous = text.replace('0', "O").replace('1', "l");
        assert_eq!(
            RecoveryKey::parse(&ambiguous).unwrap().kek().as_bytes(),
            r.kek().as_bytes()
        );
        assert_eq!(
            RecoveryKey::parse("ABCD").err(),
            Some(CryptoError::RecoveryKey)
        );
        assert_eq!(
            RecoveryKey::parse(&text.replacen(|c: char| c.is_ascii_alphanumeric(), "U", 1)).err(),
            Some(CryptoError::RecoveryKey)
        );
    }

    #[test]
    fn recovery_key_rejects_padding_bits_and_accepts_aliases() {
        let r = RecoveryKey::generate().unwrap();
        let text = r.to_display();
        // The last character carries 1 key bit and 4 padding bits: '1' sets a padding bit.
        let mut padded = text.clone();
        padded.pop();
        padded.push('1');
        assert_eq!(
            RecoveryKey::parse(&padded).err(),
            Some(CryptoError::RecoveryKey)
        );
        let canonical = text.replace('0', "O").replace('1', "I").to_lowercase();
        assert_eq!(
            RecoveryKey::parse(&canonical).unwrap().kek().as_bytes(),
            r.kek().as_bytes()
        );
        let mixed = text.replace('0', "o").replace('1', "i");
        assert_eq!(
            RecoveryKey::parse(&mixed).unwrap().kek().as_bytes(),
            r.kek().as_bytes()
        );
    }

    #[test]
    fn minimum_length_object_is_an_authentication_failure() {
        let key = Key::from_bytes([7; 32]);
        let mut exact = b"LSE1".to_vec();
        exact.extend_from_slice(&[0; 40]);
        assert_eq!(exact.len(), 44);
        assert_eq!(open(&key, b"", &exact), Err(CryptoError::Decrypt));
    }

    #[test]
    fn keys_never_print_their_bytes() {
        let k = Key::from_bytes([0xab; 32]);
        assert_eq!(format!("{k:?}"), "Key(..)");
    }
}
