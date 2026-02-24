use egui::epaint::Shadow;
use egui::{
    vec2, Color32, Context, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Visuals,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidthClass {
    Compact,
    Regular,
    Wide,
}

#[derive(Clone, Debug)]
pub struct AppTheme {
    pub surfaces: SurfaceTokens,
    pub text: TextTokens,
    pub controls: ControlTokens,
    pub layout: LayoutTokens,
    pub breakpoints: Breakpoints,
    pub shadows: ShadowTokens,
    pub motion: MotionTokens,
}

#[derive(Clone, Debug)]
pub struct SurfaceTokens {
    pub app_bg: Color32,
    pub panel_bg: Color32,
    pub panel_bg_alt: Color32,
    pub card_bg: Color32,
    pub card_bg_alt: Color32,
    pub canvas_bg: Color32,
    pub stroke_soft: Color32,
    pub stroke_strong: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
}

#[derive(Clone, Debug)]
pub struct TextTokens {
    pub primary: Color32,
    pub secondary: Color32,
    pub muted: Color32,
    pub accent: Color32,
}

#[derive(Clone, Debug)]
pub struct ControlTokens {
    pub card_rounding: f32,
    pub panel_rounding: f32,
    pub chip_rounding: f32,
    pub button_rounding: f32,
    pub toolbar_icon_size: f32,
    pub action_height: f32,
    pub global_spacing_scale: f32,
    pub hover_intensity: f32,
    pub pressed_intensity: f32,
}

#[derive(Clone, Debug)]
pub struct LayoutTokens {
    pub space_1: f32,
    pub space_2: f32,
    pub space_3: f32,
    pub space_4: f32,
    pub panel_padding_x: f32,
    pub panel_padding_y: f32,
    pub control_gap: f32,
    pub group_gap: f32,
    pub toolbar_height: f32,
    pub action_bar_height: f32,
    pub chip_h: f32,
    pub chip_w_tool: f32,
    pub chip_w_segment: f32,
}

#[derive(Clone, Debug)]
pub struct Breakpoints {
    pub compact_max: f32,
    pub regular_max: f32,
}

#[derive(Clone, Debug)]
pub struct ShadowTokens {
    pub ambient: Color32,
    pub elevation: Color32,
    pub focus_ring: Color32,
}

#[derive(Clone, Debug)]
pub struct MotionTokens {
    pub fast_ms: u32,
    pub normal_ms: u32,
    pub slow_ms: u32,
}

impl AppTheme {
    pub fn width_class(&self, width: f32) -> WidthClass {
        width_class(width, &self.breakpoints)
    }
}

pub fn width_class(width: f32, breakpoints: &Breakpoints) -> WidthClass {
    if width <= breakpoints.compact_max {
        WidthClass::Compact
    } else if width <= breakpoints.regular_max {
        WidthClass::Regular
    } else {
        WidthClass::Wide
    }
}

pub fn premium_dark_theme() -> AppTheme {
    AppTheme {
        surfaces: SurfaceTokens {
            app_bg: Color32::from_rgb(0x17, 0x18, 0x1C),
            panel_bg: Color32::from_rgb(0x1C, 0x1D, 0x22),
            panel_bg_alt: Color32::from_rgb(0x1C, 0x1D, 0x22),
            card_bg: Color32::from_rgb(0x20, 0x22, 0x2A),
            card_bg_alt: Color32::from_rgb(0x1F, 0x21, 0x29),
            canvas_bg: Color32::from_rgb(0x12, 0x14, 0x1A),
            stroke_soft: Color32::from_rgba_unmultiplied(255, 255, 255, 26),
            stroke_strong: Color32::from_rgba_unmultiplied(255, 255, 255, 48),
            accent: Color32::from_rgb(0x4D, 0x8D, 0xFF),
            accent_soft: Color32::from_rgba_unmultiplied(77, 141, 255, 80),
        },
        text: TextTokens {
            primary: Color32::from_rgb(0xF5, 0xF8, 0xFF),
            secondary: Color32::from_rgb(0xB5, 0xC0, 0xD6),
            muted: Color32::from_rgb(0x86, 0x92, 0xAA),
            accent: Color32::from_rgb(0x8F, 0xBB, 0xFF),
        },
        controls: ControlTokens {
            card_rounding: 12.0,
            panel_rounding: 10.0,
            chip_rounding: 8.0,
            button_rounding: 8.0,
            toolbar_icon_size: 18.0,
            action_height: 28.0,
            global_spacing_scale: 1.0,
            hover_intensity: 1.0,
            pressed_intensity: 1.0,
        },
        layout: LayoutTokens {
            space_1: 4.0,
            space_2: 8.0,
            space_3: 12.0,
            space_4: 16.0,
            panel_padding_x: 12.0,
            panel_padding_y: 8.0,
            control_gap: 8.0,
            group_gap: 12.0,
            toolbar_height: 44.0,
            action_bar_height: 48.0,
            chip_h: 28.0,
            chip_w_tool: 40.0,
            chip_w_segment: 42.0,
        },
        breakpoints: Breakpoints {
            compact_max: 760.0,
            regular_max: 1024.0,
        },
        shadows: ShadowTokens {
            ambient: Color32::from_rgba_unmultiplied(0, 0, 0, 56),
            elevation: Color32::from_rgba_unmultiplied(0, 0, 0, 110),
            focus_ring: Color32::from_rgba_unmultiplied(104, 158, 255, 210),
        },
        motion: MotionTokens {
            fast_ms: 120,
            normal_ms: 180,
            slow_ms: 280,
        },
    }
}

pub fn apply_theme(ctx: &Context, theme: &AppTheme) {
    let mut style: Style = (*ctx.style()).clone();

    style.spacing.item_spacing = vec2(
        theme.layout.control_gap * theme.controls.global_spacing_scale,
        theme.layout.space_2,
    );
    style.spacing.button_padding = vec2(theme.layout.space_3, theme.layout.space_2);
    style.spacing.menu_margin = egui::Margin::symmetric(theme.layout.space_2, theme.layout.space_2);
    style.spacing.window_margin =
        egui::Margin::symmetric(theme.layout.space_3, theme.layout.space_3);
    style.animation_time = theme.motion.normal_ms as f32 / 1000.0;

    style.visuals = Visuals::dark();
    style.visuals.override_text_color = Some(theme.text.primary);
    style.visuals.panel_fill = theme.surfaces.panel_bg;
    style.visuals.window_fill = theme.surfaces.panel_bg_alt;
    style.visuals.faint_bg_color = theme.surfaces.panel_bg;
    style.visuals.extreme_bg_color = theme.surfaces.app_bg;
    style.visuals.code_bg_color = theme.surfaces.card_bg_alt;
    style.visuals.window_rounding = Rounding::same(theme.controls.panel_rounding);
    style.visuals.widgets.noninteractive.bg_fill = theme.surfaces.panel_bg;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text.secondary);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.surfaces.stroke_soft);

    style.visuals.widgets.inactive.bg_fill = theme.surfaces.card_bg_alt;
    style.visuals.widgets.inactive.weak_bg_fill = theme.surfaces.card_bg_alt;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme.surfaces.stroke_soft);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text.secondary);

    style.visuals.widgets.hovered.bg_fill = theme.surfaces.card_bg;
    style.visuals.widgets.hovered.weak_bg_fill = theme.surfaces.card_bg;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, theme.surfaces.stroke_strong);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text.primary);

    style.visuals.widgets.active.bg_fill = theme.surfaces.accent_soft;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, theme.surfaces.accent);
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme.text.primary);

    style.visuals.widgets.open.bg_fill = theme.surfaces.card_bg;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, theme.surfaces.stroke_strong);
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, theme.text.primary);

    style.visuals.selection.bg_fill = theme.surfaces.accent_soft;
    style.visuals.selection.stroke = Stroke::new(1.0, theme.surfaces.accent);
    style.visuals.hyperlink_color = theme.text.accent;
    style.visuals.popup_shadow = Shadow {
        offset: vec2(0.0, 10.0),
        blur: 22.0,
        spread: 0.0,
        color: theme.shadows.ambient,
    };
    style.visuals.window_shadow = Shadow {
        offset: vec2(0.0, 14.0),
        blur: 28.0,
        spread: 0.0,
        color: theme.shadows.elevation,
    };

    style.visuals.widgets.noninteractive.rounding = Rounding::same(theme.controls.button_rounding);
    style.visuals.widgets.inactive.rounding = Rounding::same(theme.controls.button_rounding);
    style.visuals.widgets.hovered.rounding = Rounding::same(theme.controls.button_rounding);
    style.visuals.widgets.active.rounding = Rounding::same(theme.controls.button_rounding);
    style.visuals.widgets.open.rounding = Rounding::same(theme.controls.button_rounding);

    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(34.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Name("Title".into()),
        FontId::new(24.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(16.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(13.0, FontFamily::Proportional),
    );

    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::{width_class, Breakpoints, WidthClass};

    #[test]
    fn width_class_boundaries_are_stable() {
        let breakpoints = Breakpoints {
            compact_max: 760.0,
            regular_max: 1024.0,
        };

        assert_eq!(width_class(640.0, &breakpoints), WidthClass::Compact);
        assert_eq!(width_class(760.0, &breakpoints), WidthClass::Compact);
        assert_eq!(width_class(761.0, &breakpoints), WidthClass::Regular);
        assert_eq!(width_class(1024.0, &breakpoints), WidthClass::Regular);
        assert_eq!(width_class(1025.0, &breakpoints), WidthClass::Wide);
    }
}
