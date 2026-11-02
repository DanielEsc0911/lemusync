//! Status → tray view, localised. Pure. Texts: docs/specs/ui/status-and-notifications.md
//! §Status model.
//!
//! `StatusInput` / `SessionInput` are local on purpose: Task 8 maps `lemusync_api::Status`
//! and `Session` onto them, so this module doesn't depend on the wire types.

use fluent_bundle::FluentValue;

use crate::i18n::{I18n, s};
use crate::platform::{RecentItem, TrayIcon, TrayMenuText, TrayView};

/// Why sync stopped with an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Auth,
    Quota,
    Permission,
    Config,
    NotSetUp,
}

/// The one state shown (priority already applied: error > attention > syncing > offline >
/// paused > scanning > synced).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewState {
    Synced,
    Scanning,
    Syncing,
    Paused,
    Offline,
    Attention,
    Error(ErrorKind),
}

/// What the tray needs from the status model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusInput {
    pub state: ViewState,
    pub uploads: u32,
    pub downloads: u32,
    /// Changes waiting while offline.
    pub queued: u32,
    pub conflicts: u32,
    pub foreign_leases: u32,
    /// Sync is paused (also true when another state wins the icon).
    pub paused: bool,
    /// Newest first.
    pub recent: Vec<RecentItem>,
}

impl StatusInput {
    #[must_use]
    pub fn new(state: ViewState) -> Self {
        Self {
            state,
            uploads: 0,
            downloads: 0,
            queued: 0,
            conflicts: 0,
            foreign_leases: 0,
            paused: state == ViewState::Paused,
            recent: Vec::new(),
        }
    }
}

/// The play session overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInput {
    pub game: String,
    pub player: String,
}

fn n(v: u32) -> FluentValue<'static> {
    FluentValue::from(v)
}

/// The status text for one state.
#[must_use]
pub fn status_text(st: &StatusInput, t: &I18n) -> String {
    match st.state {
        ViewState::Synced => t.t("status-synced", &[]),
        ViewState::Scanning => t.t("status-scanning", &[]),
        ViewState::Syncing => match (st.uploads, st.downloads) {
            (up, 0) => t.t("status-uploading", &[("count", n(up))]),
            (0, down) => t.t("status-downloading", &[("count", n(down))]),
            (up, down) => t.t("status-syncing", &[("count", n(up.saturating_add(down)))]),
        },
        ViewState::Paused => t.t("status-paused", &[]),
        ViewState::Offline => t.t("status-offline", &[("count", n(st.queued))]),
        ViewState::Attention if st.conflicts > 0 => {
            t.t("status-conflicts", &[("count", n(st.conflicts))])
        }
        ViewState::Attention => t.t("status-foreign-leases", &[("count", n(st.foreign_leases))]),
        ViewState::Error(kind) => t.t(
            match kind {
                ErrorKind::Auth => "status-error-auth",
                ErrorKind::Quota => "status-error-quota",
                ErrorKind::Permission => "status-error-permission",
                ErrorKind::Config => "status-error-config",
                ErrorKind::NotSetUp => "status-error-not-set-up",
            },
            &[],
        ),
    }
}

/// The tray's view of the status. Tooltip = status text, with the session overlay first.
#[must_use]
pub fn tray_view(status: &StatusInput, session: Option<&SessionInput>, t: &I18n) -> TrayView {
    let icon = match status.state {
        ViewState::Synced => TrayIcon::Synced,
        ViewState::Scanning | ViewState::Syncing => TrayIcon::Syncing,
        ViewState::Paused => TrayIcon::Paused,
        ViewState::Offline => TrayIcon::Offline,
        ViewState::Attention => TrayIcon::Attention,
        ViewState::Error(_) => TrayIcon::Error,
    };
    let status_line = status_text(status, t);
    let tooltip = match session {
        Some(sn) => {
            let playing = t.t(
                "status-playing",
                &[
                    ("game", s(sn.game.clone())),
                    ("player", s(sn.player.clone())),
                ],
            );
            format!("{playing}\n{status_line}")
        }
        None => status_line.clone(),
    };
    TrayView {
        icon,
        tooltip,
        status_line,
        paused: status.paused,
        recent: status.recent.clone(),
        menu: TrayMenuText {
            sync_now: t.t("tray-sync-now", &[]),
            pause: t.t("tray-pause", &[]),
            pause_1h: t.t("tray-pause-1h", &[]),
            pause_until_resume: t.t("tray-pause-until-resume", &[]),
            resume: t.t("tray-resume", &[]),
            recent: t.t("tray-recent", &[]),
            quit: t.t("tray-quit", &[]),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(st: &StatusInput, lang: &str) -> String {
        tray_view(st, None, &I18n::new(lang)).status_line
    }

    #[test]
    fn every_state_in_en_and_es() {
        let syncing = StatusInput {
            uploads: 3,
            ..StatusInput::new(ViewState::Syncing)
        };
        let offline = StatusInput {
            queued: 2,
            ..StatusInput::new(ViewState::Offline)
        };
        let attention = StatusInput {
            conflicts: 1,
            ..StatusInput::new(ViewState::Attention)
        };
        let cases = [
            (
                StatusInput::new(ViewState::Synced),
                TrayIcon::Synced,
                "All saves synced",
                "Todas las partidas están sincronizadas",
            ),
            (
                StatusInput::new(ViewState::Scanning),
                TrayIcon::Syncing,
                "Checking for changes",
                "Buscando cambios",
            ),
            (
                syncing,
                TrayIcon::Syncing,
                "Uploading 3 changes",
                "Subiendo 3 cambios",
            ),
            (
                StatusInput::new(ViewState::Paused),
                TrayIcon::Paused,
                "Sync paused",
                "Sincronización en pausa",
            ),
            (
                offline,
                TrayIcon::Offline,
                "Offline · 2 changes waiting",
                "Sin conexión · 2 cambios en espera",
            ),
            (
                attention,
                TrayIcon::Attention,
                "1 conflict needs you",
                "1 conflicto necesita tu decisión",
            ),
            (
                StatusInput::new(ViewState::Error(ErrorKind::Auth)),
                TrayIcon::Error,
                "Storage sign-in expired",
                "La sesión del almacenamiento expiró",
            ),
        ];
        for (st, icon, en, es) in cases {
            let view = tray_view(&st, None, &I18n::new("en"));
            assert_eq!(view.icon, icon);
            assert_eq!(view.status_line, en);
            assert_eq!(view.tooltip, en);
            assert_eq!(line(&st, "es"), es);
        }
    }

    #[test]
    fn plurals_and_other_texts() {
        let st = StatusInput {
            uploads: 1,
            ..StatusInput::new(ViewState::Syncing)
        };
        assert_eq!(line(&st, "en"), "Uploading 1 change");
        assert_eq!(line(&st, "es"), "Subiendo 1 cambio");
        let st = StatusInput {
            downloads: 2,
            ..StatusInput::new(ViewState::Syncing)
        };
        assert_eq!(line(&st, "en"), "Downloading 2 changes");
        let st = StatusInput {
            uploads: 1,
            downloads: 2,
            ..StatusInput::new(ViewState::Syncing)
        };
        assert_eq!(line(&st, "es"), "Sincronizando 3 cambios");
        let st = StatusInput {
            foreign_leases: 1,
            ..StatusInput::new(ViewState::Attention)
        };
        assert_eq!(line(&st, "en"), "1 game is in use on another device");
        let st = StatusInput::new(ViewState::Error(ErrorKind::NotSetUp));
        assert_eq!(
            line(&st, "en"),
            "Not set up yet: run lemusync init or lemusync join"
        );
        assert_eq!(
            line(&st, "es"),
            "Aún no está configurado: ejecuta lemusync init o lemusync join"
        );
        for kind in [ErrorKind::Quota, ErrorKind::Permission, ErrorKind::Config] {
            let st = StatusInput::new(ViewState::Error(kind));
            for lang in ["en", "es"] {
                assert!(!line(&st, lang).starts_with("status-"), "{kind:?} {lang}");
            }
        }
    }

    #[test]
    fn session_overlay_and_menu() {
        let st = StatusInput {
            recent: vec![RecentItem {
                id: "cemu/101c9400".into(),
                label: "BotW".into(),
                launchable: true,
            }],
            ..StatusInput::new(ViewState::Paused)
        };
        let sn = SessionInput {
            game: "BotW".into(),
            player: "Daniel".into(),
        };
        let es = tray_view(&st, Some(&sn), &I18n::new("es"));
        assert_eq!(es.tooltip, "Jugando BotW · Daniel\nSincronización en pausa");
        assert!(es.paused);
        assert_eq!(es.recent, st.recent);
        assert_eq!(es.menu.sync_now, "Sincronizar ahora");
        let en = tray_view(&st, Some(&sn), &I18n::new("en"));
        assert_eq!(en.tooltip, "Playing BotW · Daniel\nSync paused");
        for label in [
            &en.menu.sync_now,
            &en.menu.pause,
            &en.menu.pause_1h,
            &en.menu.pause_until_resume,
            &en.menu.resume,
            &en.menu.recent,
            &en.menu.quit,
        ] {
            assert!(!label.starts_with("tray-"), "{label}");
        }
    }
}
