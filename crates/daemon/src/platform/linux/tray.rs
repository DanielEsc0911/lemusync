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
    tx: mpsc::Sender<TrayAction>,
}

fn item(label: &str, action: TrayAction) -> MenuItem<Item> {
    StandardItem {
        label: label.replace('_', "__"),
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

    // Empty: the brand pixmaps below are the icon (a theme name would win over them).
    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        self.view
            .as_ref()
            .map_or(TrayIcon::Synced, |v| v.icon)
            .pixmaps()
            .into_iter()
            .map(|(size, data)| ksni::Icon {
                width: size,
                height: size,
                data: data.to_vec(),
            })
            .collect()
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
            item(&m.resume, TrayAction::Resume)
        } else {
            SubMenu {
                label: m.pause.clone(),
                submenu: vec![
                    item(&m.pause_1h, TrayAction::Pause1h),
                    item(&m.pause_until_resume, TrayAction::PauseUntilResume),
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
                .map(|(game, label)| item(label, TrayAction::Play(game.clone())))
                .collect(),
            ..Default::default()
        };
        vec![
            status.into(),
            MenuItem::Separator,
            item(&m.sync_now, TrayAction::SyncNow),
            pause,
            recent.into(),
            MenuItem::Separator,
            item(&m.quit, TrayAction::Quit),
        ]
    }
}

/// Start the tray. Must be called inside a tokio runtime.
#[must_use]
pub fn start_tray(tx: mpsc::Sender<TrayAction>) -> Box<dyn Tray> {
    let (views, mut rx) = watch::channel(None);
    tokio::spawn(async move {
        let item = Item { view: None, tx };
        let Ok(handle) = item.assume_sni_available(true).spawn().await else {
            return; // No session bus: tray stays off (notifications and app still work).
        };
        while rx.changed().await.is_ok() {
            let view = rx.borrow_and_update().clone();
            if handle.update(|t| t.view = view).await.is_none() {
                return;
            }
        }
    });
    Box::new(LinuxTray { views })
}
