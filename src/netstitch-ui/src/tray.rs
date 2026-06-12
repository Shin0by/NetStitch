use dioxus::desktop::trayicon::{
    self,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
};

use crate::{app_meta, icon};

pub const HIDE_WHEN_MINIMIZED_ID: &str = "netstitch-tray-hide-when-minimized";
pub const SCANNING_ID: &str = "netstitch-tray-scanning";
pub const EXIT_ID: &str = "netstitch-tray-exit";
pub const WINDOW_VISIBILITY_ID: &str = "netstitch-tray-window-visibility";
pub const WEB_SERVER_ID: &str = "netstitch-tray-web-server";
pub const REMEMBER_WINDOW_PLACEMENT_ID: &str = "netstitch-tray-remember-window-placement";

#[cfg(test)]
pub const TRAY_PRIMARY_ITEM_ORDER: [&str; 6] = [
    WINDOW_VISIBILITY_ID,
    HIDE_WHEN_MINIMIZED_ID,
    WEB_SERVER_ID,
    REMEMBER_WINDOW_PLACEMENT_ID,
    SCANNING_ID,
    EXIT_ID,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayMenuAction {
    ToggleWindowVisibility,
    ToggleHideWhenMinimized,
    ToggleWebServer,
    ToggleRememberWindowPlacement,
    ToggleScanning,
    Exit,
    Ignore,
}

pub fn tray_menu_action_from_id(id: &str) -> TrayMenuAction {
    match id {
        WINDOW_VISIBILITY_ID => TrayMenuAction::ToggleWindowVisibility,
        HIDE_WHEN_MINIMIZED_ID => TrayMenuAction::ToggleHideWhenMinimized,
        WEB_SERVER_ID => TrayMenuAction::ToggleWebServer,
        REMEMBER_WINDOW_PLACEMENT_ID => TrayMenuAction::ToggleRememberWindowPlacement,
        SCANNING_ID => TrayMenuAction::ToggleScanning,
        EXIT_ID => TrayMenuAction::Exit,
        _ => TrayMenuAction::Ignore,
    }
}

pub fn window_visibility_label(window_visible: bool) -> &'static str {
    if window_visible {
        "Скрыть окно"
    } else {
        "Показать окно"
    }
}

pub fn scanning_label(monitoring: bool) -> &'static str {
    if monitoring {
        "Остановить мониторинг"
    } else {
        "Начать мониторинг"
    }
}

#[derive(Clone)]
pub struct TrayController {
    _tray: trayicon::TrayIcon,
    window_visibility: MenuItem,
    hide_when_minimized: CheckMenuItem,
    web_server: CheckMenuItem,
    remember_window_placement: CheckMenuItem,
    scanning: MenuItem,
    exit: MenuItem,
}

impl TrayController {
    pub fn build() -> Self {
        let window_visibility = MenuItem::with_id(
            WINDOW_VISIBILITY_ID,
            window_visibility_label(true),
            true,
            None,
        );
        let hide_when_minimized = CheckMenuItem::with_id(
            HIDE_WHEN_MINIMIZED_ID,
            "Скрывать свернутое",
            true,
            true,
            None,
        );
        let web_server = CheckMenuItem::with_id(WEB_SERVER_ID, "Web сервер", true, false, None);
        let remember_window_placement = CheckMenuItem::with_id(
            REMEMBER_WINDOW_PLACEMENT_ID,
            "Запоминать положение",
            true,
            false,
            None,
        );
        let scanning = MenuItem::with_id(SCANNING_ID, "Начать мониторинг", true, None);
        let exit = MenuItem::with_id(EXIT_ID, "Выход", true, None);
        let separator = PredefinedMenuItem::separator();

        let menu = Menu::new();
        menu.append_items(&[
            &window_visibility,
            &hide_when_minimized,
            &web_server,
            &remember_window_placement,
            &scanning,
            &separator,
            &exit,
        ])
        .expect("tray menu should build");

        let tray_icon = icon::tray_icon().unwrap_or_else(build_fallback_tray_icon);
        let tray = trayicon::init_tray_icon(menu, Some(tray_icon));
        let _ = tray.set_tooltip(Some(app_meta::APP_TITLE));

        Self {
            _tray: tray,
            window_visibility,
            hide_when_minimized,
            web_server,
            remember_window_placement,
            scanning,
            exit,
        }
    }

    pub fn sync(
        &self,
        monitoring: bool,
        hide_when_minimized: bool,
        web_server_enabled: bool,
        remember_window_placement: bool,
        window_visible: bool,
    ) {
        self.window_visibility
            .set_text(window_visibility_label(window_visible));
        self.hide_when_minimized.set_checked(hide_when_minimized);
        self.web_server.set_checked(web_server_enabled);
        self.remember_window_placement
            .set_checked(remember_window_placement);
        self.scanning.set_text(scanning_label(monitoring));
    }

    pub fn action_from_menu_id(&self, id: &trayicon::menu::MenuId) -> TrayMenuAction {
        let _ = (
            self.window_visibility.id(),
            self.hide_when_minimized.id(),
            self.web_server.id(),
            self.remember_window_placement.id(),
            self.scanning.id(),
            self.exit.id(),
        );
        tray_menu_action_from_id(id.as_ref())
    }
}

fn build_fallback_tray_icon() -> trayicon::Icon {
    let size = 16;
    let mut rgba = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let border = x < 2 || x >= size - 2 || y < 2 || y >= size - 2;
            let diagonal = x == y || x + y == size - 1;
            let pixel = if border {
                [47, 79, 110, 255]
            } else if diagonal {
                [77, 212, 166, 255]
            } else {
                [16, 33, 50, 255]
            };
            rgba[idx..idx + 4].copy_from_slice(&pixel);
        }
    }

    trayicon::Icon::from_rgba(rgba, size as u32, size as u32).expect("tray icon should build")
}

#[cfg(test)]
mod tests {
    use super::{
        EXIT_ID, HIDE_WHEN_MINIMIZED_ID, REMEMBER_WINDOW_PLACEMENT_ID, SCANNING_ID,
        TRAY_PRIMARY_ITEM_ORDER, TrayMenuAction, WEB_SERVER_ID, WINDOW_VISIBILITY_ID,
        tray_menu_action_from_id,
    };

    #[test]
    fn tray_menu_action_maps_window_visibility_item() {
        assert_eq!(
            tray_menu_action_from_id(WINDOW_VISIBILITY_ID),
            TrayMenuAction::ToggleWindowVisibility
        );
    }

    #[test]
    fn tray_menu_action_maps_hide_when_minimized_item() {
        assert_eq!(
            tray_menu_action_from_id(HIDE_WHEN_MINIMIZED_ID),
            TrayMenuAction::ToggleHideWhenMinimized
        );
    }

    #[test]
    fn tray_menu_action_maps_scanning_item() {
        assert_eq!(
            tray_menu_action_from_id(SCANNING_ID),
            TrayMenuAction::ToggleScanning
        );
    }

    #[test]
    fn tray_menu_action_maps_web_server_item() {
        assert_eq!(
            tray_menu_action_from_id(WEB_SERVER_ID),
            TrayMenuAction::ToggleWebServer
        );
    }

    #[test]
    fn tray_menu_action_maps_remember_window_placement_item() {
        assert_eq!(
            tray_menu_action_from_id(REMEMBER_WINDOW_PLACEMENT_ID),
            TrayMenuAction::ToggleRememberWindowPlacement
        );
    }

    #[test]
    fn tray_menu_action_maps_exit_item() {
        assert_eq!(tray_menu_action_from_id(EXIT_ID), TrayMenuAction::Exit);
    }

    #[test]
    fn tray_menu_action_maps_unknown_item_to_ignore() {
        assert_eq!(
            tray_menu_action_from_id("netstitch-tray-unknown"),
            TrayMenuAction::Ignore
        );
    }

    #[test]
    fn tray_menu_primary_order_keeps_window_visibility_first() {
        assert_eq!(TRAY_PRIMARY_ITEM_ORDER[0], WINDOW_VISIBILITY_ID);
    }
}
