//! The game library rule shared by the tray and the app: which games, which player streams,
//! in which order, launchable or not, and the recent-play list. Pure data in, pure data out.
//! Spec: docs/specs/ui/status-and-notifications.md (Desktop tray), docs/specs/ui/app-shell.md.

use lemusync_api::{GameInfo, GamesReport, StatusReport, StreamStatus};

/// Most items in the recent-games list (the tray's "Play" submenu, Home's recent list).
pub const RECENT_MAX: usize = 8;

/// One player's synced stream of a game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerStream {
    pub player: String,
    /// The stream's own game id (may differ from the catalog key in ASCII case).
    pub game: String,
    /// When this device last synced a version of the stream (Unix ms), if it has one.
    pub last_version_ms: Option<u64>,
}

/// A game of the catalog, or a synced game the catalog does not know, with its streams.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryEntry {
    pub emulator: String,
    pub key: String,
    pub name: Option<String>,
    /// True when this device has a game file to launch it with.
    pub launchable: bool,
    /// Newest first; ties keep the order of `StatusReport::streams`.
    pub streams: Vec<PlayerStream>,
}

/// One stream in the recent-games list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecentItem {
    pub emulator: String,
    /// The catalog key, or the stream's game id when the catalog does not know the game.
    pub key: String,
    /// The stream's own game id.
    pub game: String,
    /// The catalog name, else the stream's own name; `None` when neither is known.
    pub name: Option<String>,
    pub player: String,
    pub last_version_ms: Option<u64>,
    /// False when this device has no game file (or the catalog does not know the game).
    pub launchable: bool,
}

/// The catalog game a stream belongs to: same emulator, key equal ignoring ASCII case.
fn catalog_game<'a>(games: &'a GamesReport, st: &StreamStatus) -> Option<&'a GameInfo> {
    games
        .games
        .iter()
        .find(|g| g.emulator == st.emulator && g.key.eq_ignore_ascii_case(&st.game))
}

/// Newest first, items without a time last.
fn newest_first(a: Option<u64>, b: Option<u64>) -> std::cmp::Ordering {
    b.cmp(&a)
}

/// The library: one entry per catalog game, plus one per synced game the catalog does not
/// know (not launchable), ordered by its newest stream, then never-synced
/// games by `(emulator, key)`.
#[must_use]
pub fn library(status: &StatusReport, games: &GamesReport) -> Vec<LibraryEntry> {
    let mut entries: Vec<LibraryEntry> = games
        .games
        .iter()
        .map(|g| {
            let mut streams: Vec<PlayerStream> = status
                .streams
                .iter()
                .filter(|st| st.emulator == g.emulator && st.game.eq_ignore_ascii_case(&g.key))
                .map(|st| PlayerStream {
                    player: st.player.clone(),
                    game: st.game.clone(),
                    last_version_ms: st.last_version_ms,
                })
                .collect();
            streams.sort_by(|a, b| newest_first(a.last_version_ms, b.last_version_ms));
            LibraryEntry {
                emulator: g.emulator.clone(),
                key: g.key.clone(),
                name: g.name.clone(),
                launchable: g.path.is_some(),
                streams,
            }
        })
        .collect();
    // Synced streams the catalog does not know (for example a Cemu `.wua` title): one
    // non-launchable entry per `(emulator, game)`, so the library lists what the tray lists.
    let mut groups: Vec<(String, String, Vec<&StreamStatus>)> = Vec::new();
    for st in status
        .streams
        .iter()
        .filter(|st| catalog_game(games, st).is_none())
    {
        let game = st.game.to_ascii_lowercase();
        match groups
            .iter_mut()
            .find(|(e, g, _)| *e == st.emulator && *g == game)
        {
            Some((_, _, members)) => members.push(st),
            None => groups.push((st.emulator.clone(), game, vec![st])),
        }
    }
    entries.extend(groups.into_iter().map(|(emulator, _, members)| {
        let mut streams: Vec<PlayerStream> = members
            .iter()
            .map(|st| PlayerStream {
                player: st.player.clone(),
                game: st.game.clone(),
                last_version_ms: st.last_version_ms,
            })
            .collect();
        streams.sort_by(|a, b| newest_first(a.last_version_ms, b.last_version_ms));
        LibraryEntry {
            emulator,
            key: members[0].game.clone(),
            name: members.iter().find_map(|st| st.name.clone()),
            launchable: false,
            streams,
        }
    }));
    let newest = |e: &LibraryEntry| e.streams.first().and_then(|s| s.last_version_ms);
    entries.sort_by(|a, b| {
        newest_first(newest(a), newest(b))
            .then_with(|| (&a.emulator, &a.key).cmp(&(&b.emulator, &b.key)))
    });
    entries
}

/// The recent-games list: one item per synced stream (also those this device cannot launch,
/// flagged, so the tray can say why), newest first, items without a time last, then by name
/// and player; at most [`RECENT_MAX`], cut after sorting.
#[must_use]
pub fn recent(status: &StatusReport, games: &GamesReport) -> Vec<RecentItem> {
    let mut items: Vec<RecentItem> = status
        .streams
        .iter()
        .map(|st| {
            let known = catalog_game(games, st);
            RecentItem {
                emulator: st.emulator.clone(),
                key: known.map_or_else(|| st.game.clone(), |g| g.key.clone()),
                game: st.game.clone(),
                name: known
                    .and_then(|g| g.name.clone())
                    .or_else(|| st.name.clone()),
                player: st.player.clone(),
                last_version_ms: st.last_version_ms,
                launchable: known.is_some_and(|g| g.path.is_some()),
            }
        })
        .collect();
    let shown = |i: &RecentItem| i.name.clone().unwrap_or_else(|| i.game.clone());
    items.sort_by(|a, b| {
        newest_first(a.last_version_ms, b.last_version_ms)
            .then_with(|| shown(a).cmp(&shown(b)))
            .then_with(|| a.player.cmp(&b.player))
    });
    items.truncate(RECENT_MAX);
    items
}

#[cfg(test)]
mod tests {
    use lemusync_api::{GameInfo, GameSource, GamesReport, StatusReport, StreamStatus};

    use super::{RECENT_MAX, library, recent};

    fn stream(p: &str, g: &str, ms: Option<u64>) -> StreamStatus {
        StreamStatus {
            player: p.into(),
            emulator: "cemu".into(),
            game: g.into(),
            name: None,
            account: None,
            base: None,
            last_version_ms: ms,
        }
    }

    fn game(k: &str, path: Option<&str>) -> GameInfo {
        GameInfo {
            emulator: "cemu".into(),
            key: k.into(),
            name: Some(format!("Game {k}")),
            path: path.map(Into::into),
            source: GameSource::Config,
            aliases: vec![],
        }
    }

    fn status(players: &[&str], streams: Vec<StreamStatus>) -> StatusReport {
        StatusReport {
            api_version: 1,
            device: "pc".into(),
            device_id: "d".into(),
            repo: None,
            backend: None,
            grants: vec![],
            players: players.iter().map(|p| (*p).to_owned()).collect(),
            streams,
            daemon: None,
        }
    }

    fn games(list: Vec<GameInfo>) -> GamesReport {
        GamesReport {
            api_version: 1,
            games: list,
        }
    }

    #[test]
    fn entries_follow_their_newest_stream_and_streams_are_newest_first() {
        let st = status(
            &["Ana", "Bea"],
            vec![
                stream("Ana", "aa", Some(100)),
                stream("Bea", "aa", Some(300)),
                stream("Ana", "bb", Some(200)),
            ],
        );
        let g = games(vec![game("aa", Some("/a")), game("bb", Some("/b"))]);
        let lib = library(&st, &g);
        let keys: Vec<&str> = lib.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, ["aa", "bb"]);
        let players: Vec<&str> = lib[0].streams.iter().map(|s| s.player.as_str()).collect();
        assert_eq!(players, ["Bea", "Ana"]);
    }

    #[test]
    fn a_never_synced_game_sorts_after_synced_ones_by_emulator_and_key() {
        let st = status(&["Ana"], vec![stream("Ana", "zz", Some(5))]);
        let g = games(vec![
            game("mm", Some("/m")),
            game("zz", Some("/z")),
            game("aa", Some("/a")),
        ]);
        let keys: Vec<String> = library(&st, &g).into_iter().map(|e| e.key).collect();
        assert_eq!(keys, ["zz", "aa", "mm"]);
    }

    #[test]
    fn launchable_needs_a_path() {
        let st = status(&["Ana"], vec![]);
        let g = games(vec![game("aa", Some("/a")), game("bb", None)]);
        let lib = library(&st, &g);
        assert!(lib[0].launchable);
        assert!(!lib[1].launchable);
    }

    #[test]
    fn the_key_matches_ignoring_ascii_case() {
        let st = status(&["Ana"], vec![stream("Ana", "AA", Some(1))]);
        let g = games(vec![game("aa", Some("/a"))]);
        let lib = library(&st, &g);
        assert_eq!(lib[0].streams.len(), 1);
        assert_eq!(lib[0].streams[0].game, "AA");
    }

    #[test]
    fn an_uncataloged_stream_makes_a_non_launchable_entry_named_from_the_stream() {
        let mut ana = stream("Ana", "Ghost", Some(1));
        ana.name = Some("Bayonetta 2".into());
        let bea = stream("Bea", "ghost", Some(2));
        let st = status(&["Ana", "Bea"], vec![ana, bea]);
        let lib = library(&st, &games(vec![]));
        assert_eq!(lib.len(), 1);
        assert_eq!(lib[0].key, "Ghost");
        assert_eq!(lib[0].name.as_deref(), Some("Bayonetta 2"));
        assert!(!lib[0].launchable);
        let players: Vec<&str> = lib[0].streams.iter().map(|s| s.player.as_str()).collect();
        assert_eq!(players, ["Bea", "Ana"]);
    }

    #[test]
    fn empty_input_gives_an_empty_library_and_no_recent() {
        let st = status(&[], vec![]);
        let g = games(vec![]);
        assert_eq!(library(&st, &g).len(), 0);
        assert_eq!(recent(&st, &g).len(), 0);
    }

    #[test]
    fn recent_has_one_item_per_stream_ordered_across_games() {
        let st = status(
            &["Ana"],
            vec![
                stream("Ana", "aa", Some(300)),
                stream("Ana", "bb", Some(200)),
                stream("Bea", "aa", Some(100)),
            ],
        );
        let g = games(vec![game("aa", Some("/a")), game("bb", Some("/b"))]);
        let r = recent(&st, &g);
        let got: Vec<(&str, &str, Option<u64>)> = r
            .iter()
            .map(|i| (i.key.as_str(), i.player.as_str(), i.last_version_ms))
            .collect();
        assert_eq!(
            got,
            [
                ("aa", "Ana", Some(300)),
                ("bb", "Ana", Some(200)),
                ("aa", "Bea", Some(100)),
            ]
        );
    }

    #[test]
    fn recent_keeps_unlaunchable_and_uncataloged_streams_flagged() {
        let mut named = stream("Ana", "dd", Some(20));
        named.name = Some("Bayonetta 2".into());
        let st = status(
            &["Ana"],
            vec![
                stream("Ana", "AA", Some(10)),
                stream("Ana", "bb", None),
                named,
            ],
        );
        let g = games(vec![game("aa", Some("/a")), game("bb", None)]);
        let r = recent(&st, &g);
        let got: Vec<(&str, bool, Option<&str>)> = r
            .iter()
            .map(|i| (i.game.as_str(), i.launchable, i.name.as_deref()))
            .collect();
        assert_eq!(
            got,
            [
                ("dd", false, Some("Bayonetta 2")),
                ("AA", true, Some("Game aa")),
                ("bb", false, Some("Game bb")),
            ]
        );
    }

    #[test]
    fn recent_ties_break_by_name_then_player() {
        let st = status(
            &["Ana", "Bea"],
            vec![
                stream("Bea", "bb", None),
                stream("Ana", "bb", None),
                stream("Ana", "aa", None),
            ],
        );
        let g = games(vec![game("aa", Some("/a")), game("bb", Some("/b"))]);
        let got: Vec<(String, String)> = recent(&st, &g)
            .into_iter()
            .map(|i| (i.key, i.player))
            .collect();
        assert_eq!(
            got,
            [
                ("aa".into(), "Ana".into()),
                ("bb".into(), "Ana".into()),
                ("bb".into(), "Bea".into()),
            ]
        );
    }

    #[test]
    fn recent_caps_after_sorting() {
        let first = stream("Ana", "g0", Some(0));
        let st = status(
            &["Ana"],
            (0..20u64)
                .map(|n| StreamStatus {
                    game: format!("g{n}"),
                    last_version_ms: Some(n),
                    ..first.clone()
                })
                .collect(),
        );
        let r = recent(&st, &games(vec![]));
        assert_eq!(r.len(), RECENT_MAX);
        assert_eq!(r[0].game, "g19");
    }
}
