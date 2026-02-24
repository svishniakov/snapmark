use arboard::Clipboard;
use eframe::Frame;

pub struct StatusBarHandle;

pub fn setup_status_bar(
    _on_open_editor: impl Fn() + Send + Sync + 'static,
    _on_hide_from_dock: impl Fn() + Send + Sync + 'static,
) -> Option<StatusBarHandle> {
    None
}

pub fn clear_clipboard() {
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text(String::new());
    }
}

pub fn clipboard_change_count() -> Option<i64> {
    None
}

pub fn active_screen_scale_factor() -> Option<f32> {
    Some(1.0)
}

pub fn elevate_window(_frame: &Frame) {}

pub fn show_alert(_title: &str, message: &str) {
    eprintln!("{message}");
}

pub fn show_saved_notification(_path: &str) {}

pub fn supports_vibrancy() -> bool {
    false
}

pub fn install_vibrancy() -> Result<()> {
    Ok(())
}

pub fn update_vibrancy() -> Result<()> {
    Ok(())
}

pub fn remove_vibrancy() {}

pub fn poll_native_color_panel_color() -> Option<[u8; 4]> {
    None
}

pub fn close_native_color_panel() {}

pub fn set_dock_icon_visible(_visible: bool) {}
