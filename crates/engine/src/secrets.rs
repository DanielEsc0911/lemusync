//! Secrets live in the OS keystore only (security §Credentials).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use lemusync_core::DeviceId;
use zeroize::Zeroizing;

use crate::EngineError;

#[cfg(not(target_os = "linux"))]
const SERVICE: &str = "lemusync";

pub trait SecretStore: Send + Sync {
    /// # Errors
    /// `Secrets` when the keystore is unavailable or locked.
    fn get(&self, name: &str) -> Result<Option<Zeroizing<String>>, EngineError>;
    /// # Errors
    /// `Secrets` when the keystore is unavailable or locked.
    fn set(&self, name: &str, value: &str) -> Result<(), EngineError>;
    /// Idempotent.
    ///
    /// # Errors
    /// `Secrets` when the keystore is unavailable or locked.
    fn delete(&self, name: &str) -> Result<(), EngineError>;
}

#[must_use]
pub fn secret_master(repo: &str) -> String {
    format!("repo-{repo}-master-key")
}
#[must_use]
pub fn secret_gdrive_refresh(device: DeviceId) -> String {
    format!("device-{device}-gdrive-refresh-token")
}
#[must_use]
pub fn secret_gdrive_client(device: DeviceId) -> String {
    format!("device-{device}-gdrive-client-secret")
}
#[must_use]
pub fn secret_s3_key(device: DeviceId) -> String {
    format!("device-{device}-s3-secret-access-key")
}

/// The OS keystore: the Secret Service on Linux (whoever provides it, never started by us),
/// Keychain on macOS, Credential Manager on Windows.
#[must_use]
pub fn os_secrets() -> Arc<dyn SecretStore> {
    #[cfg(target_os = "linux")]
    {
        Arc::new(crate::secret_service::SecretServiceStore::session())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Arc::new(KeyringSecrets)
    }
}

/// Keychain and Credential Manager through `keyring`.
#[cfg(not(target_os = "linux"))]
pub struct KeyringSecrets;

#[cfg(not(target_os = "linux"))]
fn keyring_err(e: &keyring::Error) -> EngineError {
    EngineError::Secrets(lemusync_store::redact(&e.to_string()))
}

#[cfg(not(target_os = "linux"))]
impl SecretStore for KeyringSecrets {
    fn get(&self, name: &str) -> Result<Option<Zeroizing<String>>, EngineError> {
        let entry = keyring::Entry::new(SERVICE, name).map_err(|e| keyring_err(&e))?;
        match entry.get_password() {
            Ok(v) => Ok(Some(Zeroizing::new(v))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(keyring_err(&e)),
        }
    }

    fn set(&self, name: &str, value: &str) -> Result<(), EngineError> {
        keyring::Entry::new(SERVICE, name)
            .and_then(|e| e.set_password(value))
            .map_err(|e| keyring_err(&e))
    }

    fn delete(&self, name: &str) -> Result<(), EngineError> {
        let entry = keyring::Entry::new(SERVICE, name).map_err(|e| keyring_err(&e))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(keyring_err(&e)),
        }
    }
}

/// Under `LEMUSYNC_TEST_ROOT` the binaries never reach the user's real keystore: every call
/// fails (security §Credentials).
pub struct DisabledSecrets;

impl SecretStore for DisabledSecrets {
    fn get(&self, _: &str) -> Result<Option<Zeroizing<String>>, EngineError> {
        Err(EngineError::Secrets("off_under_test_root".into()))
    }

    fn set(&self, _: &str, _: &str) -> Result<(), EngineError> {
        Err(EngineError::Secrets("off_under_test_root".into()))
    }

    fn delete(&self, _: &str) -> Result<(), EngineError> {
        Err(EngineError::Secrets("off_under_test_root".into()))
    }
}

/// In-memory keystore for tests.
#[derive(Default)]
pub struct MemorySecrets(Mutex<BTreeMap<String, String>>);

impl SecretStore for MemorySecrets {
    fn get(&self, name: &str) -> Result<Option<Zeroizing<String>>, EngineError> {
        let map = self
            .0
            .lock()
            .map_err(|_| EngineError::Secrets("poisoned".into()))?;
        Ok(map.get(name).map(|v| Zeroizing::new(v.clone())))
    }

    fn set(&self, name: &str, value: &str) -> Result<(), EngineError> {
        let mut map = self
            .0
            .lock()
            .map_err(|_| EngineError::Secrets("poisoned".into()))?;
        map.insert(name.to_owned(), value.to_owned());
        Ok(())
    }

    fn delete(&self, name: &str) -> Result<(), EngineError> {
        let mut map = self
            .0
            .lock()
            .map_err(|_| EngineError::Secrets("poisoned".into()))?;
        map.remove(name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_secrets_round_trip() {
        let s = MemorySecrets::default();
        assert!(s.get("a").unwrap().is_none());
        s.set("a", "b").unwrap();
        assert_eq!(
            s.get("a").unwrap().as_deref().map(String::as_str),
            Some("b")
        );
        s.delete("a").unwrap();
        s.delete("a").unwrap();
        assert!(s.get("a").unwrap().is_none());
    }

    #[test]
    fn disabled_secrets_refuse_everything() {
        let s = DisabledSecrets;
        assert_eq!(s.get("a").unwrap_err().code(), "secrets");
        assert_eq!(s.set("a", "b").unwrap_err().code(), "secrets");
        assert_eq!(s.delete("a").unwrap_err().code(), "secrets");
    }

    // Linux: the Secret Service client is tested only on a throwaway bus
    // (`scripts/isolated-secret-service.sh`, crates/engine/tests/secret_service.rs).
    #[cfg(not(target_os = "linux"))]
    #[test]
    #[ignore = "writes one `lemusync` item to the real Keychain/Credential Manager; run by hand only"]
    fn keyring_round_trip() {
        let s = KeyringSecrets;
        s.set("lemusync-selftest", "x").unwrap();
        assert_eq!(
            s.get("lemusync-selftest")
                .unwrap()
                .as_deref()
                .map(String::as_str),
            Some("x")
        );
        s.delete("lemusync-selftest").unwrap();
    }
}
