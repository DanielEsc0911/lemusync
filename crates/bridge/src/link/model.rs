//! Bridge-owned DTOs: what Dart gets. No wire types, no serde, no secrets, no paths (B2).
//! The tests destructure every struct without `..`, so a new field (a path, a secret)
//! breaks the build and forces a review.

use lemusync_api::{ErrorKind, Session, Status, StatusState, TransferPhase, TransferProgress};
use lemusync_engine::{LibraryEntry, RecentItem};

/// Where the link to the daemon stands (app-shell §Connection).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Online {
        daemon_version: String,
    },
    NotRunning,
    Unsupported,
    /// The socket or its folder is a link or belongs to another user. The path is dropped.
    Unsafe,
    /// The daemon rejected the handshake.
    Refused {
        message: String,
    },
    /// The connection works but a call failed or timed out.
    Failed {
        code: String,
        message: String,
    },
}

/// Same order as [`StatusState`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateDto {
    Synced,
    Scanning,
    Syncing,
    Paused,
    Offline,
    Attention,
    Error,
}

impl From<StatusState> for StateDto {
    fn from(s: StatusState) -> Self {
        match s {
            StatusState::Synced => StateDto::Synced,
            StatusState::Scanning => StateDto::Scanning,
            StatusState::Syncing => StateDto::Syncing,
            StatusState::Paused => StateDto::Paused,
            StatusState::Offline => StateDto::Offline,
            StatusState::Attention => StateDto::Attention,
            StatusState::Error => StateDto::Error,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKindDto {
    Auth,
    Quota,
    Permission,
    Config,
    NotSetUp,
}

impl From<ErrorKind> for ErrorKindDto {
    fn from(k: ErrorKind) -> Self {
        match k {
            ErrorKind::Auth => ErrorKindDto::Auth,
            ErrorKind::Quota => ErrorKindDto::Quota,
            ErrorKind::Permission => ErrorKindDto::Permission,
            ErrorKind::Config => ErrorKindDto::Config,
            ErrorKind::NotSetUp => ErrorKindDto::NotSetUp,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusDto {
    pub state: StateDto,
    pub last_sync_ms: Option<u64>,
    pub uploads: u32,
    pub downloads: u32,
    pub bytes: u64,
    pub paused_until_ms: Option<u64>,
    pub queued: u32,
    pub conflicts: u32,
    pub foreign_leases: u32,
    pub error: Option<ErrorKindDto>,
}

impl From<&Status> for StatusDto {
    fn from(s: &Status) -> Self {
        StatusDto {
            state: s.state.into(),
            last_sync_ms: s.last_sync_ms,
            uploads: s.uploads,
            downloads: s.downloads,
            bytes: s.bytes,
            paused_until_ms: s.paused_until_ms,
            queued: s.queued,
            conflicts: s.conflicts,
            foreign_leases: s.foreign_leases,
            error: s.error.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionDto {
    pub game: String,
    pub name: Option<String>,
    pub emulator: String,
    pub player: String,
    pub device: String,
    pub started_ms: u64,
}

impl From<&Session> for SessionDto {
    fn from(s: &Session) -> Self {
        SessionDto {
            game: s.game.clone(),
            name: s.name.clone(),
            emulator: s.emulator.clone(),
            player: s.player.clone(),
            device: s.device.clone(),
            started_ms: s.started_ms,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerStreamDto {
    pub player: String,
    pub last_version_ms: Option<u64>,
}

/// A game of the library. `launchable` is all Dart learns about the game file (B2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameEntry {
    pub emulator: String,
    pub key: String,
    pub name: Option<String>,
    pub launchable: bool,
    pub streams: Vec<PlayerStreamDto>,
}

impl From<&LibraryEntry> for GameEntry {
    fn from(e: &LibraryEntry) -> Self {
        GameEntry {
            emulator: e.emulator.clone(),
            key: e.key.clone(),
            name: e.name.clone(),
            launchable: e.launchable,
            streams: e
                .streams
                .iter()
                .map(|s| PlayerStreamDto {
                    player: s.player.clone(),
                    last_version_ms: s.last_version_ms,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecentDto {
    pub emulator: String,
    pub key: String,
    pub name: Option<String>,
    pub player: String,
    pub last_version_ms: Option<u64>,
    /// False shows the "no game file" marker, like the tray.
    pub launchable: bool,
}

impl From<&RecentItem> for RecentDto {
    fn from(i: &RecentItem) -> Self {
        RecentDto {
            emulator: i.emulator.clone(),
            key: i.key.clone(),
            name: i.name.clone(),
            player: i.player.clone(),
            last_version_ms: i.last_version_ms,
            launchable: i.launchable,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GamesDto {
    pub games: Vec<GameEntry>,
    pub recent: Vec<RecentDto>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseDto {
    Checking,
    Uploading,
    Downloading,
}

impl From<TransferPhase> for PhaseDto {
    fn from(p: TransferPhase) -> Self {
        match p {
            TransferPhase::Checking => PhaseDto::Checking,
            TransferPhase::Uploading => PhaseDto::Uploading,
            TransferPhase::Downloading => PhaseDto::Downloading,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressDto {
    pub player: String,
    pub emulator: String,
    pub game: String,
    pub phase: PhaseDto,
    pub files_done: u32,
    pub files_total: u32,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

impl From<&TransferProgress> for ProgressDto {
    fn from(p: &TransferProgress) -> Self {
        ProgressDto {
            player: p.player.clone(),
            emulator: p.emulator.clone(),
            game: p.game.clone(),
            phase: p.phase.into(),
            files_done: p.files_done,
            files_total: p.files_total,
            bytes_done: p.bytes_done,
            bytes_total: p.bytes_total,
        }
    }
}

/// Everything one `Status` call tells (games arrive separately).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub connection: ConnectionState,
    pub status: Option<StatusDto>,
    pub session: Option<SessionDto>,
    pub device: Option<String>,
    pub players: Vec<String>,
}

impl Snapshot {
    /// A snapshot of a connection that is not online: no data.
    pub(crate) fn offline(connection: ConnectionState) -> Self {
        Snapshot {
            connection,
            status: None,
            session: None,
            device: None,
            players: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkEvent {
    Connection(ConnectionState),
    Status(StatusDto),
    Session(Option<SessionDto>),
    Progress(ProgressDto),
    Snapshot(Snapshot),
    Games(GamesDto),
}

#[cfg(test)]
mod tests {
    use super::*;

    // No `..` anywhere: a new field in a DTO fails to compile here (B2).
    #[test]
    fn no_dto_can_grow_a_field_unnoticed() {
        let StatusDto {
            state: _,
            last_sync_ms: _,
            uploads: _,
            downloads: _,
            bytes: _,
            paused_until_ms: _,
            queued: _,
            conflicts: _,
            foreign_leases: _,
            error: _,
        } = StatusDto::from(&Status {
            state: StatusState::Synced,
            last_sync_ms: None,
            uploads: 0,
            downloads: 0,
            bytes: 0,
            paused_until_ms: None,
            queued: 0,
            conflicts: 0,
            foreign_leases: 0,
            error: None,
        });
        let SessionDto {
            game: _,
            name: _,
            emulator: _,
            player: _,
            device: _,
            started_ms: _,
        } = SessionDto::from(&Session {
            game: "g".into(),
            name: None,
            emulator: "e".into(),
            player: "p".into(),
            device: "d".into(),
            started_ms: 1,
        });
    }

    #[test]
    fn no_game_dto_can_grow_a_field_unnoticed() {
        let GameEntry {
            emulator: _,
            key: _,
            name: _,
            launchable: _,
            streams,
        } = GameEntry::from(&LibraryEntry {
            emulator: "e".into(),
            key: "k".into(),
            name: None,
            launchable: true,
            streams: vec![lemusync_engine::PlayerStream {
                player: "p".into(),
                game: "k".into(),
                last_version_ms: Some(1),
            }],
        });
        let PlayerStreamDto {
            player: _,
            last_version_ms: _,
        } = &streams[0];
        let RecentDto {
            emulator: _,
            key: _,
            name: _,
            player: _,
            last_version_ms: _,
            launchable: _,
        } = RecentDto::from(&RecentItem {
            emulator: "e".into(),
            key: "k".into(),
            game: "k".into(),
            name: None,
            player: "p".into(),
            last_version_ms: None,
            launchable: false,
        });
        let GamesDto {
            games: _,
            recent: _,
        } = GamesDto {
            games: vec![],
            recent: vec![],
        };
        let ProgressDto {
            player: _,
            emulator: _,
            game: _,
            phase: _,
            files_done: _,
            files_total: _,
            bytes_done: _,
            bytes_total: _,
        } = ProgressDto::from(&TransferProgress {
            player: "p".into(),
            emulator: "e".into(),
            game: "g".into(),
            phase: TransferPhase::Uploading,
            files_done: 1,
            files_total: 2,
            bytes_done: 3,
            bytes_total: 4,
        });
        let Snapshot {
            connection: _,
            status: _,
            session: _,
            device: _,
            players: _,
        } = Snapshot::offline(ConnectionState::NotRunning);
    }
}
