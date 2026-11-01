//! Sync engine: scanning, snapshots, transfers, journal, state.
//! Spec: docs/specs/architecture.md (Engine internals), docs/specs/reliability.md.

mod accounts;
mod config_keys;
mod discover;
mod error;
mod events;
mod failpoints;
pub mod fsx;
mod game_refs;
mod games;
mod grants;
mod launch;
mod lease;
mod local_config;
mod locations;
mod play;
mod process;
mod profiles;
mod progress;
mod query;
mod remote;
mod remote_play;
mod repo;
mod repo_config;
mod resolve;
mod restore;
mod scan;
#[cfg(target_os = "linux")]
mod secret_service;
mod secrets;
mod service;
mod session;
mod setup;
mod snapshot;
mod state;
mod sync;
mod upload;
mod watch;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use lemusync_core::DeviceId;
use lemusync_crypto::KdfParams;

pub use accounts::{MapChoice, MapOutcome};
pub use error::EngineError;
pub use events::EVENT_BUFFER;
pub use failpoints::Failpoints;
pub use game_refs::is_emulator;
pub use games::GameEntry;
pub use lease::{Acquire, LeaseHandle};
pub use lemusync_core::VersionId;
pub use lemusync_store::redact;
pub use local_config::{
    BackendConfig, ConflictPolicy, DeviceSection, EmulatorOverride, LeaseSettings, LocalConfig,
};
pub use locations::Locations;
pub use play::{LeaseChoice, PlayPrompt};
pub use process::{Child, FakeProbe, Launcher, OsLauncher, ProcessProbe, SysinfoProbe};
pub use progress::{ConnectionSink, ProgressSink};
pub use remote::only_player;
pub use remote_play::{PROMPT_TIMEOUT, PlayCall, PlayFrame};
#[cfg(target_os = "linux")]
pub use secret_service::{SECRETS_NAME, SecretServiceStore};
#[cfg(not(target_os = "linux"))]
pub use secrets::KeyringSecrets;
pub use secrets::{DisabledSecrets, MemorySecrets, SecretStore, os_secrets};
pub use service::{Service, ServiceConfig, ServiceHandle};
pub use session::{DEFAULT_POLL, ProcessPoller};
pub use setup::{GdriveLogin, JoinSecret, S3Backend};
pub use state::{JournalEntry, State, StreamRow};
pub use sync::SyncOptions;
pub use watch::{BURST, SaveWatcher, WatchTarget};

pub struct EngineEnv {
    pub locations: Locations,
    pub secrets: Arc<dyn SecretStore>,
    pub processes: Arc<dyn ProcessProbe>,
    /// Argon2id parameters for new repos (tests use the minimum).
    pub kdf: KdfParams,
}

impl EngineEnv {
    /// The real OS: XDG locations, the OS keystore, `sysinfo`, default Argon2id. Opening an
    /// engine never touches the keystore; secrets are read the first time a step needs them.
    ///
    /// # Errors
    /// `Config` when there is no home directory.
    pub fn system() -> Result<Self, EngineError> {
        Ok(Self {
            locations: Locations::from_os()?,
            secrets: os_secrets(),
            processes: Arc::new(SysinfoProbe),
            kdf: KdfParams::DEFAULT,
        })
    }
}

pub struct Engine {
    env: EngineEnv,
    config: LocalConfig,
    state: State,
    failpoints: Failpoints,
    /// The unlocked repo, opened on first use (`ensure_repo`).
    repo: Option<repo::Repo>,
    /// Where `TransferProgress` goes (`set_progress`); nowhere by default.
    progress: Option<ProgressSink>,
    /// Where `ConnectionEvent`s go (`set_connection_sink`); nowhere by default. One watch for
    /// every store this engine opens, so a recovery is heard even on a fresh store.
    connection: Option<lemusync_store::RetryWatch>,
    /// Streams whose stale lease we took over, by previous holder; read by the next upload.
    takeovers: std::sync::Mutex<std::collections::BTreeMap<lemusync_core::StreamId, DeviceId>>,
    /// Test-only shift of the lease clock (`shift_lease_clock_ms`).
    clock_shift_ms: i64,
    /// Renew, lease-poll and quiet-period intervals for `play` (shortened in tests).
    play_timings: play::Timings,
    /// Test-only request faults for every store opened (`set_store_faults`).
    #[cfg(feature = "failpoints")]
    store_faults: Option<lemusync_store::Faults>,
}

impl Engine {
    /// Takes the state lock, then loads or creates `config.toml` (a new device id on first run).
    ///
    /// # Errors
    /// `Locked` if another engine runs; `Config`/`Io` for unreadable files.
    pub fn open(env: EngineEnv) -> Result<Self, EngineError> {
        // The lock comes first: config.toml is only read or written while holding it.
        let state = State::open(&env.locations.data_dir)?;
        std::fs::create_dir_all(&env.locations.config_dir)?;
        let path = env.locations.config_dir.join("config.toml");
        let config = if let Some(c) = LocalConfig::load(&path)? {
            c
        } else {
            let name = sysinfo::System::host_name().unwrap_or_else(|| "device".into());
            let c = LocalConfig::new(DeviceId(new_version_id()?), &name);
            c.save(&path)?;
            c
        };
        let engine = Self {
            env,
            config,
            state,
            failpoints: Failpoints::default(),
            progress: None,
            connection: None,
            repo: None,
            takeovers: std::sync::Mutex::default(),
            clock_shift_ms: 0,
            #[cfg(feature = "failpoints")]
            store_faults: None,
            play_timings: play::Timings::default(),
        };
        // A running emulator only postpones recovery (to `sync`/`restore`); the CLI still starts.
        match engine.recover_journal() {
            Ok(_) | Err(EngineError::EmulatorRunning(_)) => {}
            Err(e) => return Err(e),
        }
        Ok(engine)
    }

    /// Rolls back every restore a crash interrupted (reliability §Restore procedure).
    pub(crate) fn recover_journal(&self) -> Result<Vec<lemusync_api::Warning>, EngineError> {
        restore::recover(&self.state, self.env.processes.as_ref())
    }

    /// Sends live `TransferProgress` for `sync`, `restore` and `accounts recover` to `sink`.
    pub fn set_progress(&mut self, sink: ProgressSink) {
        self.progress = Some(sink);
    }

    /// Sends a `ConnectionEvent` to `sink` whenever a storage request is retried, and when
    /// the connection is back. Drops the open repo so its store picks the sink up.
    pub fn set_connection_sink(&mut self, sink: ConnectionSink) {
        self.connection = Some(lemusync_store::RetryWatch::new(progress::observer(sink)));
        self.repo = None;
    }

    #[must_use]
    pub fn device_id(&self) -> DeviceId {
        self.config.device.id
    }

    #[must_use]
    pub fn local_config(&self) -> &LocalConfig {
        &self.config
    }

    #[cfg(any(test, feature = "failpoints"))]
    pub fn arm_failpoint(&self, name: &'static str) {
        self.failpoints.arm(name);
    }

    /// Test-only: abort the process at failpoint `name` (crash matrix, `tests/kill.rs`).
    #[cfg(any(test, feature = "failpoints"))]
    pub fn arm_kill(&self, name: &'static str) {
        self.failpoints.arm_kill(name);
    }

    /// Test-only: every store this engine opens from now on injects `faults` (network
    /// matrix). Drops the open repo so the next operation reopens it through the faulty store.
    #[cfg(feature = "failpoints")]
    pub fn set_store_faults(&mut self, faults: Option<lemusync_store::Faults>) {
        self.store_faults = faults;
        self.repo = None;
    }

    /// Test-only: the open journal entries (an interrupted operation not yet recovered).
    ///
    /// # Errors
    /// `Io` for database errors.
    #[cfg(feature = "failpoints")]
    pub fn journal(&self) -> Result<Vec<JournalEntry>, EngineError> {
        self.state.journal_all()
    }

    fn save_config(&self) -> Result<(), EngineError> {
        self.config
            .save(&self.env.locations.config_dir.join("config.toml"))
    }
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

pub(crate) fn new_version_id() -> Result<VersionId, EngineError> {
    Ok(VersionId::from_parts(
        now_ms(),
        lemusync_crypto::random_bytes::<10>()?,
    ))
}

/// Local history for one stream: `<data>/history/<stream id>` (reliability §Local history).
pub(crate) fn history_dir(
    data_dir: &std::path::Path,
    stream: &lemusync_core::StreamId,
) -> std::path::PathBuf {
    data_dir.join("history").join(stream.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn test_env(root: &std::path::Path) -> EngineEnv {
        EngineEnv {
            locations: Locations::under(root),
            secrets: Arc::new(MemorySecrets::default()),
            processes: Arc::new(FakeProbe::default()),
            kdf: lemusync_crypto::KdfParams {
                m_kib: 19_456,
                t: 1,
                p: 1,
            },
        }
    }

    /// Fails the test on any keystore call.
    struct Untouchable;
    impl SecretStore for Untouchable {
        fn get(&self, n: &str) -> Result<Option<zeroize::Zeroizing<String>>, EngineError> {
            panic!("keystore read at open: {n}")
        }
        fn set(&self, n: &str, _: &str) -> Result<(), EngineError> {
            panic!("keystore write at open: {n}")
        }
        fn delete(&self, n: &str) -> Result<(), EngineError> {
            panic!("keystore delete at open: {n}")
        }
    }

    #[test]
    fn opening_an_engine_never_touches_the_keystore() {
        let dir = tempfile::tempdir().unwrap();
        for _ in 0..2 {
            let mut env = test_env(dir.path());
            env.secrets = Arc::new(Untouchable);
            let mut e = Engine::open(env).unwrap();
            e.backend_add_fs(&dir.path().join("store")).unwrap();
        }
    }

    #[test]
    fn the_device_id_survives_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let first = Engine::open(test_env(dir.path())).unwrap().device_id();
        let second = Engine::open(test_env(dir.path())).unwrap().device_id();
        assert_eq!(first, second);
        let text =
            std::fs::read_to_string(Locations::under(dir.path()).config_dir.join("config.toml"))
                .unwrap();
        assert!(text.contains("config_version = 1"));
    }

    #[test]
    fn exit_codes_follow_the_cli_spec() {
        assert_eq!(EngineError::Usage("x".into()).exit_code(), 2);
        assert_eq!(EngineError::Config("x".into()).exit_code(), 3);
        let auth = lemusync_store::StoreError::new(lemusync_store::ErrorClass::Auth, "x");
        assert_eq!(EngineError::Store(auth).exit_code(), 6);
        assert_eq!(EngineError::Unlock.exit_code(), 1);
    }

    #[test]
    fn a_second_engine_is_refused_and_keeps_the_first_device_id() {
        let dir = tempfile::tempdir().unwrap();
        let first = Engine::open(test_env(dir.path())).unwrap();
        let path = Locations::under(dir.path()).config_dir.join("config.toml");
        let before = std::fs::read_to_string(&path).unwrap();
        assert!(matches!(
            Engine::open(test_env(dir.path())),
            Err(EngineError::Locked(_))
        ));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
        assert!(before.contains(&first.device_id().to_string()));
    }

    #[test]
    fn a_locked_data_dir_stops_the_first_run_before_config_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let _held = State::open(&Locations::under(dir.path()).data_dir).unwrap();
        assert!(matches!(
            Engine::open(test_env(dir.path())),
            Err(EngineError::Locked(_))
        ));
        assert!(
            !Locations::under(dir.path())
                .config_dir
                .join("config.toml")
                .exists()
        );
    }
}
