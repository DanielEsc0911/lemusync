//! Engine errors. Applications map `code()` to localised text (architecture §Engine internals).

use lemusync_store::{ErrorClass, StoreError};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("{0}")]
    Usage(String),
    #[error("invalid config: {0}")]
    Config(String),
    #[error("storage: {0}")]
    Store(#[from] StoreError),
    #[error("file system: {0}")]
    Io(String),
    #[error("crypto: {0}")]
    Crypto(#[from] lemusync_crypto::CryptoError),
    #[error("wrong passphrase or recovery key")]
    Unlock,
    #[error("another LemuSync process is using {0}")]
    Locked(String),
    #[error("{0} is running; close it first")]
    EmulatorRunning(String),
    #[error("{0} is outside every granted folder")]
    NotGranted(String),
    #[error("not set up yet: {0}")]
    NotSetUp(&'static str),
    #[error("keystore: {0}")]
    Secrets(String),
    /// The keystore isn't there yet (no owner on the bus, still locked): retried with backoff.
    #[error("keystore not available yet: {0}")]
    SecretsUnavailable(String),
    #[error("invalid data in the repo: {0}")]
    Data(String),
    #[error("the first sync on this device needs confirmation")]
    NeedsConfirmation,
    #[error("{0} changed while it was being read; try again when the emulator is idle")]
    Busy(String),
    #[error("{0} needs a decision: run `lemusync conflicts`")]
    NeedsDecision(String),
    #[error("{0} holds the lease")]
    LeaseHeld(String),
    #[error("fault injected at {0}")]
    Injected(&'static str),
    /// The daemon answered with an error code this build doesn't know (or a transport one:
    /// `shutting_down`, `bad_request`, `api_version_mismatch`).
    #[error("the background service failed: {0}")]
    Daemon(String),
}

impl EngineError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Usage(_) => "usage",
            Self::Config(_) => "config",
            Self::Store(e) => match e.class() {
                ErrorClass::Auth => "backend_auth",
                ErrorClass::Transient => "backend_offline",
                ErrorClass::Quota => "backend_quota",
                ErrorClass::NotFound | ErrorClass::AlreadyExists | ErrorClass::Permanent => {
                    "backend"
                }
            },
            Self::Io(_) => "io",
            Self::Crypto(_) => "crypto",
            Self::Unlock => "unlock",
            Self::Locked(_) => "locked",
            Self::EmulatorRunning(_) => "emulator_running",
            Self::NotGranted(_) => "not_granted",
            Self::NotSetUp(_) => "not_set_up",
            Self::Secrets(_) => "secrets",
            Self::SecretsUnavailable(_) => "secrets_unavailable",
            Self::Data(_) => "data",
            Self::NeedsConfirmation => "needs_confirmation",
            Self::Busy(_) => "busy",
            Self::NeedsDecision(_) => "needs_decision",
            Self::LeaseHeld(_) => "lease_held",
            Self::Injected(_) => "injected",
            Self::Daemon(_) => "daemon",
        }
    }

    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) | Self::NotSetUp(_) | Self::NeedsConfirmation => 2,
            Self::Config(_) => 3,
            Self::NeedsDecision(_) => 4,
            Self::LeaseHeld(_) => 5,
            Self::Store(e) if matches!(e.class(), ErrorClass::Auth | ErrorClass::Transient) => 6,
            _ => 1,
        }
    }
}

const CRYPTO: [lemusync_crypto::CryptoError; 7] = [
    lemusync_crypto::CryptoError::Random,
    lemusync_crypto::CryptoError::Encrypt,
    lemusync_crypto::CryptoError::Decrypt,
    lemusync_crypto::CryptoError::Malformed,
    lemusync_crypto::CryptoError::KdfParams,
    lemusync_crypto::CryptoError::Kdf,
    lemusync_crypto::CryptoError::RecoveryKey,
];

impl EngineError {
    /// The error as it crosses IPC: `code`, the (redacted) display text, and the variant's
    /// payload in `detail` so [`EngineError::from_body`] rebuilds the same error on the client
    /// (same exit code, same localised text, D35).
    #[must_use]
    pub fn to_body(&self) -> lemusync_api::ErrorBody {
        let payload = match self {
            Self::Usage(d)
            | Self::Config(d)
            | Self::Io(d)
            | Self::Locked(d)
            | Self::EmulatorRunning(d)
            | Self::NotGranted(d)
            | Self::Secrets(d)
            | Self::SecretsUnavailable(d)
            | Self::Data(d)
            | Self::Busy(d)
            | Self::NeedsDecision(d)
            | Self::LeaseHeld(d)
            | Self::Daemon(d) => Some(d.clone()),
            Self::Store(e) => Some(e.message().to_owned()),
            Self::Crypto(c) => Some(c.to_string()),
            Self::NotSetUp(d) | Self::Injected(d) => Some((*d).to_owned()),
            Self::Unlock | Self::NeedsConfirmation => None,
        };
        lemusync_api::ErrorBody {
            code: self.code().into(),
            message: lemusync_store::redact(&self.to_string()),
            detail: payload.map(|p| lemusync_store::redact(&p)),
        }
    }

    /// Rebuilds the error a daemon sent (see [`EngineError::to_body`]). Unknown codes become
    /// [`EngineError::Daemon`].
    #[must_use]
    pub fn from_body(b: &lemusync_api::ErrorBody) -> Self {
        let p = b.detail.clone().unwrap_or_default();
        let store = |class| Self::Store(StoreError::new(class, &p));
        match b.code.as_str() {
            "usage" => Self::Usage(p),
            "config" => Self::Config(p),
            "backend_auth" => store(ErrorClass::Auth),
            "backend_offline" => store(ErrorClass::Transient),
            "backend_quota" => store(ErrorClass::Quota),
            "backend" => store(ErrorClass::Permanent),
            "io" => Self::Io(p),
            "crypto" => CRYPTO
                .into_iter()
                .find(|c| c.to_string() == p)
                .map_or(Self::Daemon(b.message.clone()), Self::Crypto),
            "unlock" => Self::Unlock,
            "locked" => Self::Locked(p),
            "emulator_running" => Self::EmulatorRunning(p),
            "not_granted" => Self::NotGranted(p),
            // The payload is `&'static str`; a client rebuilds one error per command, so the
            // leak is a few bytes once.
            "not_set_up" => Self::NotSetUp(Box::leak(p.into_boxed_str())),
            "secrets" => Self::Secrets(p),
            "secrets_unavailable" => Self::SecretsUnavailable(p),
            "data" => Self::Data(p),
            "needs_confirmation" => Self::NeedsConfirmation,
            "busy" => Self::Busy(p),
            "needs_decision" => Self::NeedsDecision(p),
            "lease_held" => Self::LeaseHeld(p),
            "injected" => Self::Injected("daemon"),
            "daemon" => Self::Daemon(p),
            _ => Self::Daemon(b.message.clone()),
        }
    }
}

impl From<std::io::Error> for EngineError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<rusqlite::Error> for EngineError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Io(format!("state database: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_cross_ipc_unchanged() {
        let all = [
            EngineError::Usage("player_needed".into()),
            EngineError::Config("bad".into()),
            EngineError::Store(StoreError::new(ErrorClass::Auth, "expired")),
            EngineError::Store(StoreError::new(ErrorClass::Transient, "down")),
            EngineError::Store(StoreError::new(ErrorClass::Quota, "full")),
            EngineError::Io("disk".into()),
            EngineError::Crypto(lemusync_crypto::CryptoError::Decrypt),
            EngineError::Unlock,
            EngineError::Locked("/x".into()),
            EngineError::EmulatorRunning("cemu".into()),
            EngineError::NotGranted("/y".into()),
            EngineError::NotSetUp("repo"),
            EngineError::Secrets("s".into()),
            EngineError::SecretsUnavailable("no_owner".into()),
            EngineError::Data("d".into()),
            EngineError::NeedsConfirmation,
            EngineError::Busy("b".into()),
            EngineError::NeedsDecision("g".into()),
            EngineError::LeaseHeld("Deck".into()),
            EngineError::Daemon("x".into()),
        ];
        for e in all {
            let back = EngineError::from_body(&e.to_body());
            assert_eq!(
                (back.code(), back.exit_code(), back.to_string()),
                (e.code(), e.exit_code(), e.to_string())
            );
        }
        let odd = lemusync_api::ErrorBody {
            code: "shutting_down".into(),
            message: "the daemon is stopping".into(),
            detail: None,
        };
        assert_eq!(EngineError::from_body(&odd).code(), "daemon");
    }

    #[test]
    fn each_storage_class_has_its_code_and_the_exit_codes_stay() {
        for (class, code, exit) in [
            (ErrorClass::Transient, "backend_offline", 6),
            (ErrorClass::Auth, "backend_auth", 6),
            (ErrorClass::Quota, "backend_quota", 1),
            (ErrorClass::NotFound, "backend", 1),
            (ErrorClass::AlreadyExists, "backend", 1),
            (ErrorClass::Permanent, "backend", 1),
        ] {
            let e = EngineError::Store(StoreError::new(class, "x"));
            assert_eq!((e.code(), e.exit_code()), (code, exit), "{class:?}");
        }
    }
}
