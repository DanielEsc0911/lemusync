//! DTOs for the game flows (3c): history, conflicts, leases and what a restore or resolve
//! moved. Same rules as `model`: no wire types, no serde, no secrets, no paths (B2). The tests
//! destructure every struct without `..`, so a new field forces a review.

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "reached from the FRB surface in 3c Task 4")
)]

use lemusync_api::{ConflictInfo, HeadInfo, HistoryEntry, HistoryReport, LeaseInfo, StreamReport};

/// One version of a stream, newest first in [`HistoryDto::versions`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionDto {
    pub id: String,
    /// Unix milliseconds (the wire has seconds).
    pub created_ms: u64,
    /// The device that wrote it.
    pub device: String,
    pub files: u32,
    pub bytes: u64,
    /// A current head of the stream.
    pub head: bool,
    /// Its session took over another device's stale lease.
    pub took_over: bool,
}

impl From<&HistoryEntry> for VersionDto {
    fn from(e: &HistoryEntry) -> Self {
        VersionDto {
            id: e.id.clone(),
            created_ms: e.created_at.saturating_mul(1000),
            device: e.device.clone(),
            files: e.files,
            bytes: e.bytes,
            head: e.head,
            took_over: e.takeover.is_some(),
        }
    }
}

/// One stream's history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryDto {
    /// The game's name on this device, when known.
    pub name: Option<String>,
    pub versions: Vec<VersionDto>,
    /// Local backups (names, not paths) that `restore` also takes.
    pub backups: Vec<String>,
}

impl From<&HistoryReport> for HistoryDto {
    fn from(r: &HistoryReport) -> Self {
        HistoryDto {
            name: r.name.clone(),
            versions: r.versions.iter().map(VersionDto::from).collect(),
            backups: r.backups.clone(),
        }
    }
}

/// One head of a conflicted stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadDto {
    pub version: String,
    pub device: String,
    /// Unix milliseconds on the writer's clock.
    pub created_ms: i64,
    pub files: u32,
    pub bytes: u64,
    pub changed_files: u32,
}

impl From<&HeadInfo> for HeadDto {
    fn from(h: &HeadInfo) -> Self {
        HeadDto {
            version: h.version.clone(),
            device: h.device_name.clone(),
            created_ms: h.created_at_ms,
            files: h.files,
            bytes: h.bytes,
            changed_files: h.changed_files,
        }
    }
}

/// A stream with more than one head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictDto {
    pub player: String,
    pub emulator: String,
    pub game: String,
    pub name: Option<String>,
    /// Oldest first.
    pub heads: Vec<HeadDto>,
}

impl From<&ConflictInfo> for ConflictDto {
    fn from(c: &ConflictInfo) -> Self {
        ConflictDto {
            player: c.player.clone(),
            emulator: c.emulator.clone(),
            game: c.game.clone(),
            name: c.name.clone(),
            heads: c.heads.iter().map(HeadDto::from).collect(),
        }
    }
}

/// One lease. The holder's device id stays in Rust: the name is what the user knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseDto {
    pub player: String,
    pub emulator: String,
    pub game: String,
    /// The holder's device name.
    pub device: String,
    /// Last renewal, Unix milliseconds on the holder's clock.
    pub renewed_ms: i64,
    /// False once it expired (stale: another device may take it over).
    pub live: bool,
    /// This device holds it.
    pub ours: bool,
}

impl From<&LeaseInfo> for LeaseDto {
    fn from(l: &LeaseInfo) -> Self {
        LeaseDto {
            player: l.player.clone(),
            emulator: l.emulator.clone(),
            game: l.game.clone(),
            device: l.device.clone(),
            renewed_ms: l.renewed_at_ms,
            live: l.live,
            ours: l.ours,
        }
    }
}

/// What a restore or a resolve moved on this device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovedDto {
    /// The version the stream ended on.
    pub version: Option<String>,
    pub files: u32,
    pub bytes: u64,
}

impl From<&StreamReport> for MovedDto {
    fn from(r: &StreamReport) -> Self {
        MovedDto {
            version: r.version.clone(),
            files: r.files,
            bytes: r.bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lemusync_api::StreamAction;

    fn history_report() -> HistoryReport {
        HistoryReport {
            api_version: lemusync_api::API_VERSION,
            name: Some("Game aa".into()),
            versions: vec![
                HistoryEntry {
                    id: "v2".into(),
                    created_at: 1_700_000_100,
                    device: "deck".into(),
                    files: 3,
                    bytes: 4096,
                    parents: vec!["v1".into()],
                    head: true,
                    takeover: Some("dev-9".into()),
                },
                HistoryEntry {
                    id: "v1".into(),
                    created_at: 1_700_000_000,
                    device: "pc".into(),
                    files: 2,
                    bytes: 2048,
                    parents: vec![],
                    head: false,
                    takeover: None,
                },
            ],
            backups: vec!["backup-1".into()],
        }
    }

    #[test]
    fn history_keeps_the_order_and_turns_seconds_into_milliseconds() {
        let HistoryDto {
            name,
            versions,
            backups,
        } = HistoryDto::from(&history_report());
        assert_eq!(name.as_deref(), Some("Game aa"));
        assert_eq!(backups, ["backup-1"]);
        let VersionDto {
            id,
            created_ms,
            device,
            files,
            bytes,
            head,
            took_over,
        } = &versions[0];
        assert_eq!(
            (id.as_str(), *created_ms, device.as_str()),
            ("v2", 1_700_000_100_000, "deck")
        );
        assert_eq!((*files, *bytes, *head, *took_over), (3, 4096, true, true));
        assert_eq!(versions[1].id, "v1");
        assert!(!versions[1].took_over && !versions[1].head);
    }

    #[test]
    fn a_conflict_keeps_every_head_in_order() {
        let head = |v: &str, ms: i64| HeadInfo {
            version: v.into(),
            device_name: format!("dev {v}"),
            player: "Ana".into(),
            created_at_ms: ms,
            files: 1,
            bytes: 10,
            changed_files: 1,
        };
        let ConflictDto {
            player,
            emulator,
            game,
            name,
            heads,
        } = ConflictDto::from(&ConflictInfo {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            name: None,
            heads: vec![head("h1", 1), head("h2", 2)],
        });
        assert_eq!(
            (player.as_str(), emulator.as_str(), game.as_str(), name),
            ("Ana", "cemu", "aa", None)
        );
        let HeadDto {
            version,
            device,
            created_ms,
            files,
            bytes,
            changed_files,
        } = &heads[0];
        assert_eq!(
            (version.as_str(), device.as_str(), *created_ms),
            ("h1", "dev h1", 1)
        );
        assert_eq!((*files, *bytes, *changed_files), (1, 10, 1));
        assert_eq!(heads[1].version, "h2");
    }

    #[test]
    fn a_lease_drops_the_holders_device_id() {
        let LeaseDto {
            player,
            emulator,
            game,
            device,
            renewed_ms,
            live,
            ours,
        } = LeaseDto::from(&LeaseInfo {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            device_id: "dev-secret-id".into(),
            device: "pc".into(),
            renewed_at_ms: 5,
            live: false,
            ours: false,
        });
        assert_eq!(
            (player.as_str(), emulator.as_str(), game.as_str()),
            ("Ana", "cemu", "aa")
        );
        assert_eq!(
            (device.as_str(), renewed_ms, live, ours),
            ("pc", 5, false, false)
        );
    }

    #[test]
    fn moved_keeps_the_counts_and_the_version() {
        let MovedDto {
            version,
            files,
            bytes,
        } = MovedDto::from(&StreamReport {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            account: Some("8000000a".into()),
            action: StreamAction::FastForward,
            version: Some("v1".into()),
            heads: vec!["v1".into()],
            done: true,
            files: 2,
            bytes: 2048,
        });
        assert_eq!((version.as_deref(), files, bytes), (Some("v1"), 2, 2048));
    }
}
