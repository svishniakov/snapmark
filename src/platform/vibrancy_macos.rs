use std::cell::RefCell;

use anyhow::{anyhow, Result};
use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSAutoresizingMaskOptions, NSView, NSVisualEffectBlendingMode,
    NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView, NSWindow,
    NSWindowOrderingMode,
};
use objc2_foundation::MainThreadMarker;

struct VibrancyHandle {
    window_number: isize,
    effect_view: Retained<NSVisualEffectView>,
}

thread_local! {
    static VIBRANCY_HANDLE: RefCell<Option<VibrancyHandle>> = const { RefCell::new(None) };
}

pub fn supports_vibrancy() -> bool {
    MainThreadMarker::new().is_some()
}

pub fn install_vibrancy() -> Result<()> {
    let (window, content) = active_window_and_content()?;
    let window_number = unsafe { window.windowNumber() };

    VIBRANCY_HANDLE.with(|slot| {
        let mut slot = slot.borrow_mut();

        if let Some(handle) = slot.as_mut() {
            if handle.window_number == window_number {
                sync_effect_view(content.as_ref(), handle.effect_view.as_ref());
                return Ok(());
            }
            unsafe {
                handle.effect_view.removeFromSuperview();
            }
            *slot = None;
        }

        let effect = create_effect_view(content.as_ref())?;
        *slot = Some(VibrancyHandle {
            window_number,
            effect_view: effect,
        });

        Ok(())
    })
}

pub fn update_vibrancy() -> Result<()> {
    let (window, content) = match active_window_and_content() {
        Ok(values) => values,
        Err(_) => {
            remove_vibrancy();
            return Ok(());
        }
    };

    let window_number = unsafe { window.windowNumber() };

    VIBRANCY_HANDLE.with(|slot| {
        let mut slot = slot.borrow_mut();

        let Some(handle) = slot.as_mut() else {
            drop(slot);
            return install_vibrancy();
        };

        if handle.window_number != window_number {
            unsafe {
                handle.effect_view.removeFromSuperview();
            }
            *slot = None;
            drop(slot);
            return install_vibrancy();
        }

        sync_effect_view(content.as_ref(), handle.effect_view.as_ref());
        Ok(())
    })
}

pub fn remove_vibrancy() {
    VIBRANCY_HANDLE.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(handle) = slot.as_mut() {
            unsafe {
                handle.effect_view.removeFromSuperview();
            }
        }
        *slot = None;
    });
}

fn create_effect_view(content_view: &NSView) -> Result<Retained<NSVisualEffectView>> {
    let mtm = MainThreadMarker::new().ok_or_else(|| anyhow!("not on main thread"))?;
    let effect = unsafe { NSVisualEffectView::new(mtm) };

    unsafe {
        effect.setFrame(content_view.bounds());
        effect.setMaterial(NSVisualEffectMaterial::UnderWindowBackground);
        effect.setBlendingMode(NSVisualEffectBlendingMode::WithinWindow);
        effect.setState(NSVisualEffectState::Active);
        effect.setAutoresizingMask(
            NSAutoresizingMaskOptions::NSViewWidthSizable
                | NSAutoresizingMaskOptions::NSViewHeightSizable,
        );
    }

    sync_effect_view(content_view, effect.as_ref());
    Ok(effect)
}

fn sync_effect_view(content_view: &NSView, effect_view: &NSVisualEffectView) {
    unsafe {
        effect_view.setFrame(content_view.bounds());

        content_view.addSubview_positioned_relativeTo(
            effect_view,
            NSWindowOrderingMode::NSWindowBelow,
            None,
        );
    }
}

fn active_window_and_content() -> Result<(Retained<NSWindow>, Retained<NSView>)> {
    let mtm = MainThreadMarker::new().ok_or_else(|| anyhow!("not on main thread"))?;
    let app = NSApplication::sharedApplication(mtm);

    let window = app
        .keyWindow()
        .or_else(|| unsafe { app.mainWindow() })
        .ok_or_else(|| anyhow!("no active window"))?;

    let content = window
        .contentView()
        .ok_or_else(|| anyhow!("window has no contentView"))?;

    Ok((window, content))
}
