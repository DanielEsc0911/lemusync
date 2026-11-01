//! Engine API: requests, responses, events and the engine trait. Versioned.
//! Spec: docs/specs/architecture.md (Engine API).

use serde::{Deserialize, Serialize};

pub mod secret;
pub mod status;
pub mod wire;

pub use secret::Secret;
pub use status::{ErrorKind, Session, Status, StatusFacts, StatusState};
pub use wire::{
    ConfigOp, Event, GameName, GameRef, Hello, HelloReply, JoinWire, LeaseAnswer, MapChoiceWire,
    MapResult, PlayAnswer, PlayQuestion, RemoteNewer, Request, Response, WithWarnings,
};

/// Version of the report and request shapes in this crate. Bumped on breaking changes.
pub const API_VERSION: u32 = 1;

/// Result of `lemusync sync`: one entry per stream, plus warnings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncReport {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// True when nothing was written.
    pub dry_run: bool,
    /// One entry per stream the run looked at.
    pub streams: Vec<StreamReport>,
    /// Non-fatal findings for the whole run.
    pub warnings: Vec<Warning>,
}

/// Outcome for one player, emulator, game and account stream.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamReport {
    /// Player name as the user sees it.
    pub player: String,
    /// Emulator profile id.
    pub emulator: String,
    /// Game id (emulator-specific title id).
    pub game: String,
    /// Emulator account id, when the stream is tied to one.
    pub account: Option<String>,
    /// What the sync decided to do for this stream.
    pub action: StreamAction,
    /// Version id the stream ended on, when there is one.
    pub version: Option<String>,
    /// Current head version ids of the stream.
    pub heads: Vec<String>,
    /// True when the stream finished its work in this run.
    pub done: bool,
    /// Files this run uploaded or downloaded for the stream.
    #[serde(default)]
    pub files: u32,
    /// Bytes this run uploaded or downloaded for the stream (plaintext sizes).
    #[serde(default)]
    pub bytes: u64,
}

/// Live progress of one stream during a sync or restore: the `TransferProgress` event
/// (architecture §Engine API). Not part of any report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferProgress {
    /// Player name as the user sees it.
    pub player: String,
    /// Emulator profile id.
    pub emulator: String,
    /// Game id (emulator-specific title id).
    pub game: String,
    /// What the stream is doing now.
    pub phase: TransferPhase,
    /// Files finished in this phase.
    pub files_done: u32,
    /// Files this phase will transfer.
    pub files_total: u32,
    /// Bytes finished in this phase (plaintext sizes).
    pub bytes_done: u64,
    /// Bytes this phase will transfer (plaintext sizes).
    pub bytes_total: u64,
}

/// The storage connection while a request is retried: the `ConnectionEvent` event
/// (architecture §Engine API). Every app shows it its own way; none is part of a report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConnectionEvent {
    /// A request failed with a transient error; try `attempt` of `max_attempts` starts after
    /// `delay_ms`.
    Retrying {
        /// The store operation: `get`, `put`, `list`, `stat` or `delete`.
        operation: String,
        attempt: u32,
        max_attempts: u32,
        delay_ms: u64,
    },
    /// A request succeeded after trouble.
    Recovered,
}

/// The step a stream is in while it syncs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferPhase {
    /// Reading the remote and the local save folder.
    Checking,
    /// Sending new blobs.
    Uploading,
    /// Fetching and verifying blobs for a restore.
    Downloading,
}

/// What a sync run decided for one stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamAction {
    /// Already in sync; nothing to do.
    Nothing,
    /// Local saves adopted as the stream's first version.
    Adopt,
    /// Local side was behind; remote version applied.
    FastForward,
    /// Local version uploaded.
    Upload,
    /// Local version uploaded, and the stream now has concurrent heads.
    UploadThenConflict,
    /// Concurrent heads exist and need a decision.
    Conflict,
    /// Local saves are missing for this stream.
    LocalMissing,
    /// Remote side is behind the local one.
    RemoteBehind,
    /// The stream needs an emulator account mapped first.
    NeedsAccount,
    /// The stream was skipped on purpose.
    Skipped,
    /// A conflict was resolved: a resolution version was written and restored.
    Resolved,
}

/// A non-fatal finding, with a stable code and display detail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warning {
    /// Stable machine-readable code.
    pub code: WarningCode,
    /// Display text, never translated.
    pub detail: String,
}

/// Stable codes for `Warning`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningCode {
    /// The emulator is running, so saves may change under the sync.
    EmulatorRunning,
    /// A save folder is not granted to `LemuSync`.
    NotGranted,
    /// A shared folder that should be empty is not.
    SharedFolderNotEmpty,
    /// A save exists for an account that no player uses.
    OrphanedSave,
    /// Config from another device was merged in.
    ConfigMerged,
    /// A player was renamed during the run.
    PlayerRenamed,
    /// The remote side is behind the local one.
    RemoteBehind,
    /// A restore was rolled back after it failed.
    RestoreRolledBack,
    /// A version id could not be parsed.
    InvalidVersion,
    /// `init` created the repo, but a later step failed on this device; `detail` is the error
    /// code. `lemusync join` with the passphrase finishes the setup.
    SetupIncomplete,
    /// A save folder could not be read safely (symlink or I/O error); `detail` is its path.
    UnreadableFolder,
    /// A lease object didn't decrypt or parse, so it was ignored; `detail` is its device id.
    LeaseUnreadable,
    /// A lease renewal failed during play; it is retried and the game keeps running.
    LeaseRenewFailed,
    /// The lease couldn't be deleted after play; it goes stale after its TTL.
    LeaseReleaseFailed,
}

/// Result of `lemusync scan`: what was found on this machine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanReport {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// One entry per emulator install found.
    pub installs: Vec<InstallReport>,
    /// Non-fatal findings for the scan.
    pub warnings: Vec<Warning>,
}

/// One emulator install found by a scan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallReport {
    /// Emulator profile id.
    pub emulator: String,
    /// Base folder of the install.
    pub base: String,
    /// True when the base folder is granted to `LemuSync`.
    pub granted: bool,
    /// Folders that still need a grant.
    pub needs_grant: Vec<String>,
    /// Accounts found under this install.
    pub accounts: Vec<AccountReport>,
    /// Saves found under this install.
    pub saves: Vec<SaveReport>,
}

/// One emulator account found by a scan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountReport {
    /// Account id as the emulator names it.
    pub id: String,
    /// Display name, when the emulator provides one.
    pub name: Option<String>,
    /// Player this account is mapped to, if any.
    pub player: Option<String>,
}

/// Result of `lemusync accounts create`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedAccount {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// Emulator profile id.
    pub emulator: String,
    /// The new account's id as the emulator names it.
    pub account: String,
    /// Display name written into the account (cut to the emulator's limit).
    pub name: String,
    /// Player the account is now mapped to.
    pub player: String,
    /// The sync that restored the player's saves; `None` before this device's first sync.
    pub sync: Option<SyncReport>,
}

/// Save data found for one game and account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveReport {
    /// Game id.
    pub game: String,
    /// Account id the save belongs to.
    pub account: String,
    /// True when no player uses this account.
    pub orphaned: bool,
    /// Number of files in the save.
    pub files: u32,
    /// Total size of the save in bytes.
    pub bytes: u64,
}

/// Result of `lemusync status`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusReport {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// This device's name.
    pub device: String,
    /// This device's id.
    pub device_id: String,
    /// Repo location, when a repo is set up.
    pub repo: Option<String>,
    /// Backend label, when one is configured.
    pub backend: Option<String>,
    /// Granted folders.
    pub grants: Vec<String>,
    /// Player names.
    pub players: Vec<String>,
    /// Status of each stream.
    pub streams: Vec<StreamStatus>,
    /// The daemon's live state, when the report came from a running daemon (architecture
    /// §Process model). Absent in-process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daemon: Option<DaemonStatus>,
}

/// What a running daemon adds to `status`: its status model and the play session overlay.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// The daemon's crate version.
    pub version: String,
    /// The status model, as the tray shows it.
    pub status: Status,
    /// The running play session, if any.
    pub session: Option<Session>,
}

/// Status of one stream as shown by `status`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamStatus {
    /// Player name.
    pub player: String,
    /// Emulator profile id.
    pub emulator: String,
    /// Game id.
    pub game: String,
    /// The game's name on this device, when known (emulator-profiles §Game discovery). Not synced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Account id, when the stream is tied to one.
    pub account: Option<String>,
    /// Base folder of the local saves, when known.
    pub base: Option<String>,
}

/// Where a game in the catalog was found (config-and-cli §Commands, `games`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameSource {
    /// `[emulators.<id>] games.<key>` in config.toml.
    Config,
    /// A game folder listed in the emulator's own config.
    Emulator,
    /// A folder the user listed in `[emulators.<id>] game_dirs`.
    Scan,
}

/// One installed game on this device.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameInfo {
    /// Emulator profile id.
    pub emulator: String,
    /// Game key (Cemu: the title id's low 8 hex digits).
    pub key: String,
    /// Display name read from the game's metadata on this device, when known.
    pub name: Option<String>,
    /// What `play` starts, when known.
    pub path: Option<String>,
    /// Where the path came from.
    pub source: GameSource,
    /// This device's aliases for the game (config-and-cli §Game references).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

/// Result of `lemusync games`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GamesReport {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// Every game found, by emulator then key.
    pub games: Vec<GameInfo>,
}

/// Result of `lemusync history`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryReport {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// The game's name on this device, when known (emulator-profiles §Game discovery). Not synced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Versions of the stream, newest first.
    pub versions: Vec<HistoryEntry>,
    /// Local backups available for restore.
    pub backups: Vec<String>,
}

/// One version in a history listing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Version id.
    pub id: String,
    /// Creation time as Unix seconds.
    pub created_at: u64,
    /// Name of the device that made the version.
    pub device: String,
    /// Number of files in the version.
    pub files: u32,
    /// Total size in bytes.
    pub bytes: u64,
    /// Parent version ids.
    pub parents: Vec<String>,
    /// True when this version is a current head.
    pub head: bool,
    /// Device id whose stale lease this version's session took over.
    #[serde(default)]
    pub takeover: Option<String>,
}

/// One head of a conflicted stream (sync-model §Conflicts).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadInfo {
    /// Version id.
    pub version: String,
    /// Writer's device name (its id when unknown).
    pub device_name: String,
    /// Player name.
    pub player: String,
    /// Creation time, Unix milliseconds on the writer's clock.
    pub created_at_ms: i64,
    /// Files in the version.
    pub files: u32,
    /// Total plaintext size.
    pub bytes: u64,
    /// Files that differ from the version's base.
    pub changed_files: u32,
}

/// A stream with more than one head.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Player name.
    pub player: String,
    /// Emulator profile id.
    pub emulator: String,
    /// Game id.
    pub game: String,
    /// The game's name on this device, when known (emulator-profiles §Game discovery). Not synced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Every head, oldest first.
    pub heads: Vec<HeadInfo>,
}

/// A step of `lemusync play`, for live status (sync-model §Launcher mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayEvent {
    /// Pulling the newest save (step 2).
    Pulling,
    /// Another device holds the lease; polling until it's free.
    WaitingForLease,
    /// The emulator is running (steps 4 and 5).
    Playing,
    /// The emulator exited; waiting the quiet period, then uploading (step 6).
    Uploading,
}

/// How this device played with respect to the stream's lease.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayLease {
    /// We held the lease for the whole session.
    Held,
    /// We took over a stale lease.
    TookOver,
    /// Another device held it and the player chose to play anyway.
    PlayedAnyway,
}

/// Result of `lemusync play`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayReport {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// The pull before launch.
    pub pulled: StreamReport,
    /// The emulator's exit code (-1 when killed by a signal).
    pub exit_code: i32,
    /// The upload after exit; `action` is `nothing` when no file changed.
    pub uploaded: StreamReport,
    /// How the lease was used.
    pub lease: PlayLease,
    /// Lease renewals written while the emulator ran.
    pub renewals: u32,
    /// Non-fatal findings.
    pub warnings: Vec<Warning>,
}

/// One lease, as `lemusync status` shows it (sync-model §Leases).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseInfo {
    /// Player name.
    pub player: String,
    /// Emulator profile id.
    pub emulator: String,
    /// Game id.
    pub game: String,
    /// Holder's device id.
    pub device_id: String,
    /// Holder's device name (its id when unknown).
    pub device: String,
    /// Last renewal, Unix milliseconds on the holder's clock.
    pub renewed_at_ms: i64,
    /// False once the lease has expired.
    pub live: bool,
    /// True for this device's own lease.
    pub ours: bool,
}

/// Result of `lemusync init`. The recovery key is never included.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitReport {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// Repo location that was created.
    pub repo: String,
    /// Name of this device.
    pub device: String,
    /// Non-fatal findings, e.g. `SetupIncomplete`.
    pub warnings: Vec<Warning>,
}

/// Result of `lemusync doctor`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// Results of each check, in run order.
    pub checks: Vec<Check>,
}

/// One named health check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Check {
    /// Check name.
    pub name: String,
    /// True when the check passed.
    pub ok: bool,
    /// Display text with the result detail. For a failed check with a `code`: the engine
    /// payload, or for storage codes the redacted provider text (never shown as the message).
    pub detail: String,
    /// Failed checks: the stable error code (`backend_auth`, `secrets`…; config-and-cli
    /// §Error codes), or `skipped` when an earlier check's failure made this one pointless.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Error output for any command run with `--json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorReport {
    /// Shape version, always `API_VERSION`.
    pub api_version: u32,
    /// The error itself.
    pub error: ErrorBody,
}

/// Machine code and display message of an error.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Stable machine-readable code.
    pub code: String,
    /// Display text, never translated.
    pub message: String,
    /// Raw, redacted provider text behind a storage error (config-and-cli §Error codes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_report_json_is_stable() {
        let r = SyncReport {
            api_version: API_VERSION,
            dry_run: true,
            streams: vec![StreamReport {
                player: "A".into(),
                emulator: "cemu".into(),
                game: "101c9400".into(),
                account: Some("80000001".into()),
                action: StreamAction::UploadThenConflict,
                version: None,
                heads: vec![],
                done: false,
                files: 2,
                bytes: 300,
            }],
            warnings: vec![Warning {
                code: WarningCode::OrphanedSave,
                detail: "80000002".into(),
            }],
        };
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(
            json,
            r#"{"api_version":1,"dry_run":true,"streams":[{"player":"A","emulator":"cemu","game":"101c9400","account":"80000001","action":"upload_then_conflict","version":null,"heads":[],"done":false,"files":2,"bytes":300}],"warnings":[{"code":"orphaned_save","detail":"80000002"}]}"#
        );
        assert_eq!(serde_json::from_str::<SyncReport>(&json).unwrap(), r);
        // Reports from before `files`/`bytes` still parse.
        let old = json.replace(r#","files":2,"bytes":300"#, "");
        let parsed = serde_json::from_str::<SyncReport>(&old).unwrap();
        assert_eq!((parsed.streams[0].files, parsed.streams[0].bytes), (0, 0));
        // StreamAction and WarningCode are Copy: values can be taken out of borrowed reports.
        let a = r.streams[0].action;
        assert_eq!(a, StreamAction::UploadThenConflict);
        let c = r.warnings[0].code;
        assert_eq!(c, WarningCode::OrphanedSave);
    }

    #[test]
    fn init_report_carries_setup_warnings() {
        let r = InitReport {
            api_version: API_VERSION,
            repo: "0123456789abcdef0123456789abcdef".into(),
            device: "pc".into(),
            warnings: vec![Warning {
                code: WarningCode::SetupIncomplete,
                detail: "secrets".into(),
            }],
        };
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(
            json,
            r#"{"api_version":1,"repo":"0123456789abcdef0123456789abcdef","device":"pc","warnings":[{"code":"setup_incomplete","detail":"secrets"}]}"#
        );
        assert_eq!(serde_json::from_str::<InitReport>(&json).unwrap(), r);
    }
}
