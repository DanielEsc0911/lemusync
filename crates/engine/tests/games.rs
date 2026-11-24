//! Game names from the catalog in `status`, `history` and `conflicts` (emulator-profiles
//! §Game discovery, D34). Synthetic data only.
#![allow(clippy::unwrap_used)]

mod common;

use common::{Pc, join, map};
use lemusync_engine::{EmulatorOverride, Locations, MapChoice};

const GAME: &str = "101c9400";

/// An unpacked game in `<root>/games/G` and `[emulators.cemu] game_dirs` pointing at it.
fn install_game(pc: Pc) -> Pc {
    let dir = pc.root.join("games");
    let g = dir.join("G");
    for sub in ["code", "content", "meta"] {
        std::fs::create_dir_all(g.join(sub)).unwrap();
    }
    std::fs::write(
        g.join("meta/meta.xml"),
        "<menu><title_id>00050000101C9400</title_id><longname_en>Synthetic Game</longname_en></menu>",
    )
    .unwrap();
    let path = Locations::under(&pc.root).config_dir.join("config.toml");
    let mut c = pc.engine.local_config().clone();
    c.emulators.insert(
        "cemu".into(),
        EmulatorOverride {
            game_dirs: vec![dir],
            ..EmulatorOverride::default()
        },
    );
    c.save(&path).unwrap();
    pc.reopen()
}

#[tokio::test]
async fn a_detected_game_s_name_shows_in_status_and_history_only_where_installed() {
    let root = tempfile::tempdir().unwrap();
    let remote = root.path().join("remote");
    let mut a = Pc::new(&root.path().join("a"), false, b"save");
    a.engine.backend_add_fs(&remote).unwrap();
    a.engine.init("pass phrase").await.unwrap();
    a.engine.players_add("P").await.unwrap();
    map(&mut a, "P", "80000001", MapChoice::Ask).await;
    a.sync().await;
    let mut b = Pc::new(&root.path().join("b"), false, b"save");
    join(&mut b, &remote).await;

    let mut a = install_game(a);
    let status = a.engine.status().await.unwrap();
    let st = status.streams.iter().find(|s| s.game == GAME).unwrap();
    assert_eq!(st.name.as_deref(), Some("Synthetic Game"));
    // Recency is the base version's embedded time.
    let base: lemusync_core::VersionId = st.base.as_deref().unwrap().parse().unwrap();
    assert_eq!(st.last_version_ms, Some(base.unix_ms()));
    assert!(base.unix_ms() > 0);
    let history = a.engine.history("P", "cemu", GAME).await.unwrap();
    assert_eq!(history.name.as_deref(), Some("Synthetic Game"));

    // Device b doesn't have the game: names aren't synced (D34).
    let status = b.engine.status().await.unwrap();
    let st = status.streams.iter().find(|s| s.game == GAME).unwrap();
    assert_eq!(st.name, None);
    assert_eq!(st.last_version_ms, None);
    let json = serde_json::to_value(st).unwrap();
    assert!(json.get("name").is_none(), "{json}");
}
