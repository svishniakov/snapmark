use std::cell::RefCell;

use objc2_app_kit::NSColorPanel;
use objc2_foundation::MainThreadMarker;

thread_local! {
    static LAST_COLOR: RefCell<Option<[u8; 4]>> = const { RefCell::new(None) };
}

pub fn poll_native_color_panel_color() -> Option<[u8; 4]> {
    let mtm = MainThreadMarker::new()?;
    if !unsafe { NSColorPanel::sharedColorPanelExists(mtm) } {
        return None;
    }

    let panel = unsafe { NSColorPanel::sharedColorPanel(mtm) };
    if !panel.isVisible() {
        return None;
    }

    let color = unsafe { panel.color() };

    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let mut a = 1.0;
    unsafe {
        color.getRed_green_blue_alpha(&mut r, &mut g, &mut b, &mut a);
    }

    let rgba = [to_u8(r), to_u8(g), to_u8(b), to_u8(a)];

    LAST_COLOR.with(|last| {
        let mut last = last.borrow_mut();
        if last.as_ref() == Some(&rgba) {
            None
        } else {
            *last = Some(rgba);
            Some(rgba)
        }
    })
}

pub fn close_native_color_panel() {
    if let Some(mtm) = MainThreadMarker::new() {
        if unsafe { NSColorPanel::sharedColorPanelExists(mtm) } {
            let panel = unsafe { NSColorPanel::sharedColorPanel(mtm) };
            panel.orderOut(None);
        }
    }
}

fn to_u8(component: f64) -> u8 {
    (component.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::to_u8;

    #[test]
    fn to_u8_clamps_correctly() {
        assert_eq!(to_u8(-0.5), 0);
        assert_eq!(to_u8(0.0), 0);
        assert_eq!(to_u8(0.5), 128);
        assert_eq!(to_u8(1.0), 255);
        assert_eq!(to_u8(2.0), 255);
    }
}
