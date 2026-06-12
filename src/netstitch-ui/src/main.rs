#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod app_meta;
mod app_watcher;
mod cloud_sync;
mod i18n;
mod icon;
mod mock;
mod models;
mod secure_store;
mod theme;
mod tray;
mod ui_entities;
mod watcher_api;

pub(crate) const WINDOW_INITIAL_WIDTH: f64 = 1350.0;
pub(crate) const WINDOW_MIN_WIDTH: f64 = 1350.0;
pub(crate) const WINDOW_MIN_HEIGHT: f64 = 867.0;
pub(crate) const WINDOW_INITIAL_HEIGHT: f64 = WINDOW_MIN_HEIGHT;

fn main() {
    let Some(_single_instance_guard) = acquire_single_instance_guard() else {
        activate_existing_instance();
        return;
    };

    let _embedded_watcher =
        should_start_embedded_watcher().then(|| netstitch_watcher::spawn_embedded_watcher(None));

    use dioxus::desktop::{
        Config, WindowBuilder, WindowCloseBehaviour,
        tao::dpi::{PhysicalPosition, PhysicalSize},
    };

    let remembered_window_placement = app_watcher::read_persisted_window_placement();

    let mut window_builder = WindowBuilder::new()
        .with_title(app_meta::app_window_title())
        .with_inner_size(PhysicalSize::new(
            remembered_window_placement
                .as_ref()
                .map(|placement| placement.width.max(WINDOW_MIN_WIDTH))
                .unwrap_or(WINDOW_INITIAL_WIDTH) as u32,
            remembered_window_placement
                .as_ref()
                .map(|placement| placement.height.max(WINDOW_MIN_HEIGHT))
                .unwrap_or(WINDOW_INITIAL_HEIGHT) as u32,
        ))
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(
            WINDOW_MIN_WIDTH,
            WINDOW_MIN_HEIGHT,
        ));
    if let Some(placement) = remembered_window_placement.as_ref() {
        window_builder = window_builder
            .with_position(PhysicalPosition::new(placement.x, placement.y))
            .with_visible(true);
    }
    if let Some(window_icon) = icon::window_icon() {
        window_builder = window_builder.with_window_icon(Some(window_icon));
    }

    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(window_builder)
                .with_menu(None::<dioxus::desktop::muda::Menu>)
                .with_background_color(theme::initial_window_background_rgba())
                .with_close_behaviour(WindowCloseBehaviour::WindowCloses)
                .with_exits_when_last_window_closes(true)
                .with_tray_icon_show_window_on_click(false),
        )
        .launch(app::App);
}

fn should_start_embedded_watcher() -> bool {
    if std::env::var("NETSTITCH__WATCHER_ADDR")
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
    {
        return false;
    }
    !std::env::var("NETSTITCH__UI_USE_MOCK")
        .ok()
        .is_some_and(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

struct SingleInstanceGuard {
    #[cfg(target_os = "windows")]
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(target_os = "windows")]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if self.handle.is_null() {
            return;
        }
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.handle);
        }
    }
}

#[cfg(target_os = "windows")]
fn acquire_single_instance_guard() -> Option<SingleInstanceGuard> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    let account = windows_account_key();
    let name = format!("Local\\NetStitch-{}", account);
    let wide_name = std::ffi::OsStr::new(&name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let handle = unsafe { CreateMutexW(std::ptr::null(), 1, wide_name.as_ptr()) };
    if handle.is_null() {
        return Some(SingleInstanceGuard { handle });
    }
    let already_exists = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    if already_exists {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(handle);
        }
        None
    } else {
        Some(SingleInstanceGuard { handle })
    }
}

#[cfg(target_os = "windows")]
fn windows_account_key() -> String {
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string());
    let domain = std::env::var("USERDOMAIN").unwrap_or_default();
    format!("{domain}-{user}")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(target_os = "windows")]
fn activate_existing_instance() {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SW_SHOWNORMAL, SetForegroundWindow, ShowWindow,
    };

    let title = std::ffi::OsStr::new(&app_meta::app_window_title())
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    unsafe {
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_SHOWNORMAL);
            SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn acquire_single_instance_guard() -> Option<SingleInstanceGuard> {
    Some(SingleInstanceGuard {})
}

#[cfg(not(target_os = "windows"))]
fn activate_existing_instance() {}

#[cfg(test)]
mod tests {
    use super::{
        WINDOW_INITIAL_HEIGHT, WINDOW_INITIAL_WIDTH, WINDOW_MIN_HEIGHT, WINDOW_MIN_WIDTH, app_meta,
    };

    #[test]
    fn window_minimum_size_keeps_extra_monitoring_row_visible() {
        assert_eq!(WINDOW_MIN_WIDTH, 1350.0);
        assert_eq!(WINDOW_MIN_HEIGHT, 867.0);
        assert_eq!(WINDOW_MIN_WIDTH, 900.0 * 1.5);
        assert!(WINDOW_MIN_HEIGHT > 560.0 * 1.5);
    }

    #[test]
    fn initial_window_size_is_not_smaller_than_minimum() {
        assert!(WINDOW_INITIAL_WIDTH >= WINDOW_MIN_WIDTH);
        assert!(WINDOW_INITIAL_HEIGHT >= WINDOW_MIN_HEIGHT);
        assert_eq!(WINDOW_INITIAL_HEIGHT, WINDOW_MIN_HEIGHT);
    }

    #[test]
    fn native_window_title_includes_version() {
        let title = app_meta::app_window_title();
        assert!(title.starts_with("NetStitch ver. "));
        assert!(title.contains(&netstitch_shared::runtime_build_version()));
    }

    #[test]
    fn startup_window_is_visible_even_when_last_session_was_hidden() {
        let source = include_str!("main.rs");
        assert!(
            source.contains(
                ".with_position(PhysicalPosition::new(placement.x, placement.y))\n            .with_visible(true)"
            ),
            "cold start must ignore persisted hidden state and show the main window"
        );
    }

    #[test]
    fn second_instance_activates_existing_window_without_message_box() {
        let source = include_str!("main.rs");
        let message_box_api = ["Message", "BoxW"].concat();
        assert!(source.contains("activate_existing_instance();"));
        assert!(source.contains("FindWindowW"));
        assert!(source.contains("ShowWindow(hwnd, SW_SHOWNORMAL);"));
        assert!(source.contains("SetForegroundWindow(hwnd);"));
        assert!(
            !source.contains(&message_box_api),
            "second desktop launch should silently activate the existing NetStitch window"
        );
    }
}
