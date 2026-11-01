//! The Linux Secret Service client, on a throwaway bus only (security §Secret Service
//! activation). Run with `scripts/isolated-secret-service.sh <dir>`: every test here is
//! ignored by default and refuses any bus but the script's, before its first D-Bus call.
#![cfg(target_os = "linux")]
#![allow(clippy::unwrap_used)] // test helpers; clippy.toml only exempts `#[test]` fns

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use dbus::Message;
use dbus::arg::Variant;
use dbus::channel::Channel;
use lemusync_engine::{EngineError, SECRETS_NAME, SecretServiceStore, SecretStore};

const NAME: &str = "lemusync-selftest";
const T: Duration = Duration::from_secs(5);

/// The throwaway bus address and its folder. Panics anywhere else.
fn isolated() -> (String, PathBuf) {
    let base = std::env::var("LEMUSYNC_ISOLATED_MARKER")
        .expect("refusing: run through scripts/isolated-secret-service.sh");
    let base = PathBuf::from(base);
    assert!(
        base.join(".lemusync-isolated").is_file(),
        "refusing: no marker"
    );
    let addr = std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_default();
    let real = std::env::var("LEMUSYNC_REAL_BUS").unwrap_or_default();
    let link = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
    assert!(
        std::fs::read_link(&link).ok() == Some(base.join("runtime"))
            && addr.starts_with(&format!("unix:path={link}/"))
            && addr != real,
        "refusing: {addr} is not the throwaway bus"
    );
    (addr, base)
}

fn bus(addr: &str) -> Channel {
    let mut ch = Channel::open_private(addr).unwrap();
    ch.register().unwrap();
    ch
}

fn call(dest: &str, path: &str, iface: &str, member: &str) -> Message {
    let mut m = Message::new_method_call(dest, path, iface, member).unwrap();
    m.set_auto_start(false);
    m
}

fn has_owner(ch: &Channel) -> bool {
    let m = call(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "NameHasOwner",
    )
    .append1(SECRETS_NAME);
    ch.send_with_reply_and_block(m, T).unwrap().read1().unwrap()
}

fn waiting(r: Result<(), EngineError>) -> String {
    match r {
        Err(EngineError::SecretsUnavailable(k)) => k,
        other => panic!("expected a wait, got {other:?}"),
    }
}

fn wait_for(file: &Path) {
    let end = Instant::now() + Duration::from_secs(30);
    while !file.exists() {
        assert!(Instant::now() < end, "{} never appeared", file.display());
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
#[ignore = "throwaway bus only: scripts/isolated-secret-service.sh"]
fn no_owner_means_wait_and_no_activation() {
    let (addr, base) = isolated();
    let activated = base.join("ACTIVATED");
    let ch = bus(&addr);
    assert!(!activated.exists() && !has_owner(&ch));

    // Every call, retried with backoff, waits; nothing is started.
    let s = SecretServiceStore::at(&addr);
    let mut pause = Duration::from_millis(50);
    for _ in 0..4 {
        assert_eq!(waiting(s.get(NAME).map(drop)), "no_owner");
        assert_eq!(waiting(s.set(NAME, "x")), "no_owner");
        assert_eq!(waiting(s.delete(NAME)), "no_owner");
        std::thread::sleep(pause);
        pause *= 2;
    }
    assert!(!activated.exists(), "lemusync activated a Secret Service");
    assert!(!has_owner(&ch));

    // Control: the trap does catch activation. A plain call with auto-start (what `keyring`
    // sent) runs it.
    let mut ping = Message::new_method_call(
        SECRETS_NAME,
        "/org/freedesktop/secrets",
        "org.freedesktop.DBus.Peer",
        "Ping",
    )
    .unwrap();
    ping.set_auto_start(true);
    assert!(ch.send_with_reply_and_block(ping, T).is_err());
    assert!(activated.exists(), "the harness can't see activation");
    std::fs::remove_file(&activated).unwrap();
}

#[test]
#[ignore = "throwaway bus only: scripts/isolated-secret-service.sh"]
fn waits_for_an_owner_then_round_trips() {
    let (addr, base) = isolated();
    let s = SecretServiceStore::at(&addr);

    // The script starts the fixture store a little later: wait for it with backoff.
    let end = Instant::now() + Duration::from_secs(30);
    let (mut pause, mut waits) = (Duration::from_millis(100), 0);
    loop {
        match s.get(NAME) {
            Ok(v) => {
                assert!(v.is_none());
                break;
            }
            Err(EngineError::SecretsUnavailable(_)) => {
                assert!(Instant::now() < end, "no owner after 30 s");
                waits += 1;
                std::thread::sleep(pause);
                pause = (pause * 2).min(Duration::from_secs(2));
            }
            Err(e) => panic!("{e}"),
        }
    }
    assert!(waits > 0, "the owner was there from the start");
    wait_for(&base.join("fixture-ready"));

    // Round trip, overwrite, idempotent delete. Another app's item under the same user
    // name is never seen.
    s.set(NAME, "one").unwrap();
    assert_eq!(s.get(NAME).unwrap().unwrap().as_str(), "one");
    s.set(NAME, "two").unwrap();
    assert_eq!(s.get(NAME).unwrap().unwrap().as_str(), "two");
    s.delete(NAME).unwrap();
    assert!(s.get(NAME).unwrap().is_none());
    s.delete(NAME).unwrap();

    // Another app's item under the same user name (the script's fixture) is still there.
    let ch = bus(&addr);
    let attrs = std::collections::HashMap::from([("service", "other-app"), ("username", NAME)]);
    let (other, _): (Vec<dbus::Path<'static>>, Vec<dbus::Path<'static>>) = ch
        .send_with_reply_and_block(
            call(
                SECRETS_NAME,
                "/org/freedesktop/secrets",
                "org.freedesktop.Secret.Service",
                "SearchItems",
            )
            .append1(attrs),
            T,
        )
        .unwrap()
        .read2()
        .unwrap();
    assert_eq!(other.len(), 1, "another app's item was touched");

    // A locked store is "not available yet" and stays locked: lemusync never unlocks.
    s.set(NAME, "three").unwrap();
    let alias: dbus::Path<'static> = ch
        .send_with_reply_and_block(
            call(
                SECRETS_NAME,
                "/org/freedesktop/secrets",
                "org.freedesktop.Secret.Service",
                "ReadAlias",
            )
            .append1("default"),
            T,
        )
        .unwrap()
        .read1()
        .unwrap();
    ch.send_with_reply_and_block(
        call(
            SECRETS_NAME,
            "/org/freedesktop/secrets",
            "org.freedesktop.Secret.Service",
            "Lock",
        )
        .append1(vec![alias.clone()]),
        T,
    )
    .unwrap();
    assert_eq!(waiting(s.get(NAME).map(drop)), "locked");
    assert_eq!(waiting(s.set(NAME, "four")), "locked");
    assert_eq!(waiting(s.delete(NAME)), "locked");
    let locked: Variant<bool> = ch
        .send_with_reply_and_block(
            call(
                SECRETS_NAME,
                &alias,
                "org.freedesktop.DBus.Properties",
                "Get",
            )
            .append2("org.freedesktop.Secret.Collection", "Locked"),
            T,
        )
        .unwrap()
        .read1()
        .unwrap();
    assert!(locked.0, "lemusync unlocked the collection");
    assert!(!base.join("ACTIVATED").exists());
}
