//! `doctor`'s desktop checks, run by the CLI in the user's session on both paths (in-process
//! and through the daemon), so `doctor --json` is the same either way: the daemon, the
//! systemd unit, lingering, a tray host and a notification server (Phase 2 Task 9).
//! Informational ones pass whatever they find; only an installed daemon that doesn't
//! answer fails. Names and details are English, like the engine's checks.

use lemusync_api::Check;

use crate::service;

fn check(name: &str, ok: bool, detail: impl Into<String>) -> Check {
    Check {
        name: name.into(),
        ok,
        detail: detail.into(),
        code: None,
    }
}

/// `daemon`: the version that answered, or why none did.
pub async fn checks(daemon: Option<&str>) -> Vec<Check> {
    let unit = service::current();
    let installed = unit.as_ref().is_some_and(|u| u.installed);
    let mut out = vec![match (daemon, installed) {
        (Some(v), _) => check(
            "daemon",
            true,
            format!("running (lemusyncd {v}), socket answers"),
        ),
        (None, true) => check(
            "daemon",
            false,
            "installed but not answering: run `lemusync service start`, then `lemusync logs`",
        ),
        (None, false) => check(
            "daemon",
            true,
            "not running; commands run in this process (`lemusync service install` keeps saves in sync all day)",
        ),
    }];
    out.push(match &unit {
        Some(u) if u.installed => check(
            "service",
            true,
            format!(
                "{} unit, {}, {} restarts",
                if u.enabled { "enabled" } else { "disabled" },
                if u.active { "active" } else { "inactive" },
                u.restarts
            ),
        ),
        Some(_) => check("service", true, "not installed"),
        None => check("service", true, "no user service manager on this OS yet"),
    });
    if cfg!(target_os = "linux") {
        out.push(linger());
        out.extend(bus().await);
    }
    out
}

fn linger() -> Check {
    let user = std::env::var("USER").unwrap_or_default();
    let on = !user.is_empty() && service::linger_dir().join(&user).exists();
    check(
        "linger",
        true,
        if on {
            "on: the daemon keeps running after you log out"
        } else {
            "off: the daemon stops at logout (needed only headless: `loginctl enable-linger`)"
        },
    )
}

#[cfg(target_os = "linux")]
async fn bus() -> Vec<Check> {
    use std::time::Duration;

    if service::under_test() {
        return vec![
            check("tray host", true, "not checked in tests"),
            check("notifications", true, "not checked in tests"),
        ];
    }
    let limit = Duration::from_secs(2);
    let Ok(Ok(conn)) = tokio::time::timeout(limit, zbus::Connection::session()).await else {
        return vec![
            check("tray host", true, "no session bus (headless): no tray"),
            check(
                "notifications",
                true,
                "no session bus (headless): no notifications",
            ),
        ];
    };
    let tray = async {
        let proxy = zbus::fdo::DBusProxy::new(&conn).await.ok()?;
        let name = zbus::names::BusName::try_from("org.kde.StatusNotifierWatcher").ok()?;
        proxy.name_has_owner(name).await.ok()
    };
    let tray = match tokio::time::timeout(limit, tray).await {
        Ok(Some(true)) => check("tray host", true, "StatusNotifierWatcher found"),
        _ => check(
            "tray host",
            true,
            "no StatusNotifierWatcher: no tray icon (on GNOME install the AppIndicator extension); notifications still work",
        ),
    };
    let notes = async {
        let reply = conn
            .call_method(
                Some("org.freedesktop.Notifications"),
                "/org/freedesktop/Notifications",
                Some("org.freedesktop.Notifications"),
                "GetServerInformation",
                &(),
            )
            .await
            .ok()?;
        reply
            .body()
            .deserialize::<(String, String, String, String)>()
            .ok()
    };
    let notes = match tokio::time::timeout(limit, notes).await {
        Ok(Some((name, vendor, version, _))) => check(
            "notifications",
            true,
            format!("{name} {version} ({vendor})"),
        ),
        _ => check(
            "notifications",
            true,
            "no notification server: LemuSync can't tell you about conflicts outside the terminal",
        ),
    };
    vec![tray, notes]
}

#[cfg(not(target_os = "linux"))]
#[allow(clippy::unused_async)] // same signature as on Linux
async fn bus() -> Vec<Check> {
    Vec::new()
}
