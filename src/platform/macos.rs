use std::sync::Arc;

use anyhow::Result;
use eframe::Frame;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{declare_class, msg_send_id, mutability, sel, ClassType, DeclaredClass};
use objc2_app_kit::{
    NSAlert, NSApplication, NSApplicationActivationPolicy, NSImage, NSMenu, NSMenuItem,
    NSPasteboard, NSScreen, NSSquareStatusItemLength, NSStatusBar, NSStatusItem,
};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSSize, NSString};

#[path = "color_panel_macos.rs"]
mod color_panel_macos;
#[path = "vibrancy_macos.rs"]
mod vibrancy_macos;

struct StatusMenuIvars {
    on_open_editor: Arc<dyn Fn() + Send + Sync + 'static>,
    on_hide_from_dock: Arc<dyn Fn() + Send + Sync + 'static>,
}

declare_class!(
    struct StatusMenuTarget;

    unsafe impl ClassType for StatusMenuTarget {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "SnapMarkStatusMenuTarget";
    }

    impl DeclaredClass for StatusMenuTarget {
        type Ivars = StatusMenuIvars;
    }

    unsafe impl NSObjectProtocol for StatusMenuTarget {}

    unsafe impl StatusMenuTarget {
        #[method(openEditor:)]
        fn open_editor(&self, _sender: Option<&AnyObject>) {
            (self.ivars().on_open_editor)();
        }

        #[method(quitApp:)]
        fn quit_app(&self, _sender: Option<&AnyObject>) {
            std::process::exit(0);
        }

        #[method(hideFromDock:)]
        fn hide_from_dock(&self, _sender: Option<&AnyObject>) {
            (self.ivars().on_hide_from_dock)();
        }
    }
);

impl StatusMenuTarget {
    fn new(
        mtm: MainThreadMarker,
        on_open_editor: Arc<dyn Fn() + Send + Sync + 'static>,
        on_hide_from_dock: Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Retained<Self> {
        let this = mtm.alloc();
        let this = this.set_ivars(StatusMenuIvars {
            on_open_editor,
            on_hide_from_dock,
        });
        unsafe { msg_send_id![super(this), init] }
    }
}

pub struct StatusBarHandle {
    _status_bar: Retained<NSStatusBar>,
    _status_item: Retained<NSStatusItem>,
    _menu: Retained<NSMenu>,
    _target: Retained<StatusMenuTarget>,
}

pub fn setup_status_bar(
    on_open_editor: impl Fn() + Send + Sync + 'static,
    on_hide_from_dock: impl Fn() + Send + Sync + 'static,
) -> Option<StatusBarHandle> {
    let mtm = MainThreadMarker::new()?;

    let status_bar = unsafe { NSStatusBar::systemStatusBar() };
    let status_item = unsafe { status_bar.statusItemWithLength(NSSquareStatusItemLength) };

    let menu = NSMenu::new(mtm);
    let open_callback: Arc<dyn Fn() + Send + Sync + 'static> = Arc::new(on_open_editor);
    let hide_callback: Arc<dyn Fn() + Send + Sync + 'static> = Arc::new(on_hide_from_dock);
    let target = StatusMenuTarget::new(mtm, open_callback, hide_callback);
    let target_obj: &AnyObject = target.as_ref();

    let open_title = NSString::from_str("Open Editor");
    let hide_dock_title = NSString::from_str("Hide from Dock");
    let quit_title = NSString::from_str("Quit SnapMark");
    let empty = NSString::from_str("");

    let open_item = unsafe {
        menu.addItemWithTitle_action_keyEquivalent(&open_title, Some(sel!(openEditor:)), &empty)
    };
    unsafe { open_item.setTarget(Some(target_obj)) };

    let hide_dock_item = unsafe {
        menu.addItemWithTitle_action_keyEquivalent(
            &hide_dock_title,
            Some(sel!(hideFromDock:)),
            &empty,
        )
    };
    unsafe { hide_dock_item.setTarget(Some(target_obj)) };

    let separator = NSMenuItem::separatorItem(mtm);
    menu.addItem(&separator);

    let quit_item = unsafe {
        menu.addItemWithTitle_action_keyEquivalent(&quit_title, Some(sel!(quitApp:)), &empty)
    };
    unsafe { quit_item.setTarget(Some(target_obj)) };

    #[allow(deprecated)]
    unsafe {
        if let Some(icon) = load_status_icon() {
            // Keep menu bar icon visually native-sized on all scale factors.
            icon.setSize(NSSize::new(15.0, 15.0));
            icon.setTemplate(true);
            status_item.setImage(Some(icon.as_ref()));
            status_item.setTitle(None);
        } else {
            let fallback = NSString::from_str("SM");
            status_item.setTitle(Some(&fallback));
        }
        status_item.setMenu(Some(&menu));
    }

    Some(StatusBarHandle {
        _status_bar: status_bar,
        _status_item: status_item,
        _menu: menu,
        _target: target,
    })
}

pub fn clear_clipboard() {
    let pasteboard = unsafe { NSPasteboard::generalPasteboard() };
    let _ = unsafe { pasteboard.clearContents() };
}

pub fn clipboard_change_count() -> Option<i64> {
    let pasteboard = unsafe { NSPasteboard::generalPasteboard() };
    Some(unsafe { pasteboard.changeCount() as i64 })
}

pub fn active_screen_scale_factor() -> Option<f32> {
    let mtm = MainThreadMarker::new()?;
    let screen = NSScreen::mainScreen(mtm)?;
    Some(screen.backingScaleFactor() as f32)
}

pub fn elevate_window(_frame: &Frame) {
    // Managed via egui viewport commands in app loop.
}

pub fn show_alert(_title: &str, message: &str) {
    if let Some(mtm) = MainThreadMarker::new() {
        let alert = unsafe { NSAlert::new(mtm) };
        let title = NSString::from_str("SnapMark");
        let body = NSString::from_str(message);
        unsafe {
            alert.setMessageText(&title);
            alert.setInformativeText(&body);
            alert.runModal();
        }
    } else {
        eprintln!("{message}");
    }
}

pub fn show_saved_notification(_path: &str) {}

pub fn supports_vibrancy() -> bool {
    vibrancy_macos::supports_vibrancy()
}

pub fn install_vibrancy() -> Result<()> {
    vibrancy_macos::install_vibrancy()
}

pub fn update_vibrancy() -> Result<()> {
    vibrancy_macos::update_vibrancy()
}

pub fn remove_vibrancy() {
    vibrancy_macos::remove_vibrancy();
}

pub fn poll_native_color_panel_color() -> Option<[u8; 4]> {
    color_panel_macos::poll_native_color_panel_color()
}

pub fn close_native_color_panel() {
    color_panel_macos::close_native_color_panel();
}

pub fn set_dock_icon_visible(visible: bool) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let policy = if visible {
        NSApplicationActivationPolicy::Regular
    } else {
        NSApplicationActivationPolicy::Accessory
    };
    let _ = app.setActivationPolicy(policy);
}

fn load_status_icon() -> Option<Retained<NSImage>> {
    let primary_name = NSString::from_str("status_icon_template");
    if let Some(image) = unsafe { NSImage::imageNamed(&primary_name) } {
        return Some(image);
    }

    let status_icon_path = format!(
        "{}/assets/status_icon_template.png",
        env!("CARGO_MANIFEST_DIR")
    );
    let status_icon_path = NSString::from_str(&status_icon_path);
    if let Some(image) =
        unsafe { NSImage::initWithContentsOfFile(NSImage::alloc(), &status_icon_path) }
    {
        return Some(image);
    }

    let fallback_symbol = NSString::from_str("pencil.and.outline");
    unsafe { NSImage::imageWithSystemSymbolName_accessibilityDescription(&fallback_symbol, None) }
}
