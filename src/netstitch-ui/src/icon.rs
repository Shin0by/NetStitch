use dioxus::desktop::tao::window::Icon as WindowIcon;
use dioxus::desktop::trayicon::Icon as TrayIcon;
use image::ImageFormat;

pub fn window_icon() -> Option<WindowIcon> {
    let (rgba, width, height) = decode_app_icon()?;
    WindowIcon::from_rgba(rgba, width, height).ok()
}

pub fn tray_icon() -> Option<TrayIcon> {
    let (rgba, width, height) = decode_app_icon()?;
    TrayIcon::from_rgba(rgba, width, height).ok()
}

#[cfg(target_os = "windows")]
fn decode_app_icon() -> Option<(Vec<u8>, u32, u32)> {
    let bytes = include_bytes!("../assets/favicon.ico");
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Ico).ok()?;
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    Some((rgba.into_raw(), width, height))
}

#[cfg(not(target_os = "windows"))]
fn decode_app_icon() -> Option<(Vec<u8>, u32, u32)> {
    let bytes = include_bytes!("../assets/shin0by.png");
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Png).ok()?;
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    Some((rgba.into_raw(), width, height))
}
