//! `StatusNotifierItem` tray via `ksni`. Optional: with no `org.kde.StatusNotifierWatcher`
//! on the bus, `ksni` waits for one to appear (`assume_sni_available`, it follows
//! `NameOwnerChanged`) and registers then; no panic and no error loop. Without a session bus
//! at all the tray stays off.

use ksni::menu::{MenuItem, StandardItem, SubMenu};
use ksni::{ToolTip, TrayMethods};
use tokio::sync::{mpsc, watch};

use crate::platform::{Tray, TrayAction, TrayIcon, TrayView};

struct LinuxTray {
    views: watch::Sender<Option<TrayView>>,
}

impl Tray for LinuxTray {
    fn show(&self, view: &TrayView) {
        self.views.send_replace(Some(view.clone()));
    }
}

struct Item {
    view: Option<TrayView>,
    /// The pixmaps of the current icon, built once per icon change: ksni reads `icon_pixmap`
    /// on every `handle.update`, so rebuilding there would copy ~15 KB each time.
    icons: Vec<ksni::Icon>,
    tx: mpsc::Sender<TrayAction>,
}

fn build_icons(icon: TrayIcon) -> Vec<ksni::Icon> {
    icon.pixmaps()
        .into_iter()
        .map(|(size, data)| ksni::Icon {
            width: size,
            height: size,
            data: data.to_vec(),
        })
        .collect()
}

fn item(label: &str, action: TrayAction, enabled: bool) -> MenuItem<Item> {
    StandardItem {
        label: label.replace('_', "__"),
        enabled,
        activate: Box::new(move |me: &mut Item| {
            // Menu clicks are rare; a full queue means the daemon is stuck, drop it.
            let _ = me.tx.try_send(action.clone());
        }),
        ..Default::default()
    }
    .into()
}

impl ksni::Tray for Item {
    // A click opens the menu: there is no app window to open until Phase 3.
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        "lemusync".into()
    }

    fn title(&self) -> String {
        "LemuSync".into()
    }

    // Empty on purpose: the SNI spec says a host falls back to `IconPixmap` when `IconName` is
    // empty, and a non-empty theme name would win over the brand pixmaps below.
    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        self.icons.clone()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "LemuSync".into(),
            description: self
                .view
                .as_ref()
                .map(|v| v.tooltip.clone())
                .unwrap_or_default(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let Some(v) = &self.view else {
            return Vec::new();
        };
        let m = &v.menu;
        let status = StandardItem {
            label: v.status_line.replace('_', "__"),
            enabled: false,
            ..Default::default()
        };
        let pause = if v.paused {
            item(&m.resume, TrayAction::Resume, true)
        } else {
            SubMenu {
                label: m.pause.clone(),
                submenu: vec![
                    item(&m.pause_1h, TrayAction::Pause1h, true),
                    item(&m.pause_until_resume, TrayAction::PauseUntilResume, true),
                ],
                ..Default::default()
            }
            .into()
        };
        let recent = SubMenu {
            label: m.recent.clone(),
            enabled: !v.recent.is_empty(),
            submenu: v
                .recent
                .iter()
                .map(|r| item(&r.label, TrayAction::Play(r.id.clone()), r.launchable))
                .collect(),
            ..Default::default()
        };
        vec![
            status.into(),
            MenuItem::Separator,
            item(&m.sync_now, TrayAction::SyncNow, true),
            pause,
            recent.into(),
            MenuItem::Separator,
            item(&m.quit, TrayAction::Quit, true),
        ]
    }
}

/// Start the tray. Must be called inside a tokio runtime.
#[must_use]
pub fn start_tray(tx: mpsc::Sender<TrayAction>) -> Box<dyn Tray> {
    let (views, mut rx) = watch::channel(None);
    tokio::spawn(async move {
        let item = Item {
            view: None,
            icons: build_icons(TrayIcon::Synced),
            tx,
        };
        let Ok(handle) = item.assume_sni_available(true).spawn().await else {
            return; // No session bus: tray stays off (notifications and app still work).
        };
        while rx.changed().await.is_ok() {
            let view = rx.borrow_and_update().clone();
            let updated = handle
                .update(|t| {
                    if t.view == view {
                        return; // Nothing changed: no icon rebuild, ksni sees no diff.
                    }
                    let icon = view.as_ref().map_or(TrayIcon::Synced, |v| v.icon);
                    if t.view.as_ref().map_or(TrayIcon::Synced, |v| v.icon) != icon {
                        t.icons = build_icons(icon);
                    }
                    t.view = view;
                })
                .await;
            if updated.is_none() {
                return;
            }
        }
    });
    Box::new(LinuxTray { views })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{RecentItem, TrayMenuText};

    #[test]
    fn an_unlaunchable_recent_game_is_a_disabled_item() {
        let recent = |id: &str, launchable| RecentItem {
            id: id.into(),
            label: id.into(),
            launchable,
        };
        let (tx, _rx) = mpsc::channel(1);
        let tray = Item {
            view: Some(TrayView {
                icon: TrayIcon::Synced,
                tooltip: String::new(),
                status_line: String::new(),
                paused: false,
                recent: vec![recent("a", true), recent("b", false)],
                menu: TrayMenuText {
                    recent: "R".into(),
                    ..Default::default()
                },
            }),
            icons: Vec::new(),
            tx,
        };
        let menu = ksni::Tray::menu(&tray);
        let sub = menu
            .iter()
            .find_map(|m| match m {
                MenuItem::SubMenu(s) if s.label == "R" => Some(s),
                _ => None,
            })
            .unwrap();
        assert!(sub.enabled);
        let enabled: Vec<bool> = sub
            .submenu
            .iter()
            .map(|m| match m {
                MenuItem::Standard(s) => s.enabled,
                _ => panic!("not a standard item"),
            })
            .collect();
        assert_eq!(enabled, [true, false]);
    }
}
