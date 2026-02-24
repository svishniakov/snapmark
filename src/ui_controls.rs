use egui::{vec2, Color32, Frame, Margin, RichText, Rounding, Sense, Stroke, Ui, Vec2};

use crate::theme::AppTheme;

pub fn card_frame(theme: &AppTheme) -> Frame {
    Frame::none()
        .fill(theme.surfaces.card_bg_alt)
        .rounding(Rounding::same(theme.controls.card_rounding))
        .stroke(Stroke::new(1.0, theme.surfaces.stroke_soft))
        .inner_margin(Margin::symmetric(
            theme.layout.space_4,
            theme.layout.space_3,
        ))
}

pub fn toolbar_frame(theme: &AppTheme) -> Frame {
    Frame::none()
        .fill(theme.surfaces.panel_bg)
        .rounding(Rounding::ZERO)
        .inner_margin(Margin::symmetric(
            theme.layout.panel_padding_x,
            theme.layout.panel_padding_y,
        ))
}

pub fn action_bar_frame(theme: &AppTheme) -> Frame {
    let vertical_padding = ((theme.layout.action_bar_height - theme.controls.action_height) * 0.5)
        .round()
        .max(theme.layout.space_1);

    Frame::none()
        .fill(theme.surfaces.panel_bg)
        .rounding(Rounding::ZERO)
        .inner_margin(Margin::symmetric(
            theme.layout.panel_padding_x,
            vertical_padding,
        ))
}

pub fn tool_chip(ui: &mut Ui, theme: &AppTheme, label: &str, selected: bool) -> egui::Response {
    let mut button = egui::Button::new(RichText::new(label).size(theme.controls.toolbar_icon_size))
        .min_size(vec2(theme.layout.chip_w_tool, theme.layout.chip_h))
        .rounding(Rounding::same(theme.controls.chip_rounding));

    if selected {
        button = button
            .fill(theme.surfaces.accent_soft)
            .stroke(Stroke::new(1.0, theme.shadows.focus_ring));
    } else {
        button = button.fill(theme.surfaces.card_bg_alt);
    }

    ui.add(button)
}

pub fn segmented(ui: &mut Ui, theme: &AppTheme, label: &str, selected: bool) -> egui::Response {
    let mut button = egui::Button::new(RichText::new(label).size(14.0))
        .min_size(vec2(theme.layout.chip_w_segment, theme.layout.chip_h))
        .rounding(Rounding::same(theme.controls.button_rounding));

    if selected {
        button = button
            .fill(theme.surfaces.accent_soft)
            .stroke(Stroke::new(1.0, theme.surfaces.accent));
    } else {
        button = button.fill(theme.surfaces.card_bg_alt);
    }

    ui.add(button)
}

pub fn color_chip(ui: &mut Ui, theme: &AppTheme, color: Color32, selected: bool) -> egui::Response {
    let mut button = egui::Button::new("")
        .min_size(vec2(22.0, 22.0))
        .fill(color)
        .rounding(Rounding::same(11.0));

    if selected {
        button = button.stroke(Stroke::new(2.0, theme.shadows.focus_ring));
    } else {
        button = button.stroke(Stroke::new(1.0, theme.surfaces.stroke_soft));
    }

    ui.add(button)
}

pub fn primary_button(
    ui: &mut Ui,
    theme: &AppTheme,
    label: &str,
    min_size: Vec2,
) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).strong().color(theme.text.primary))
            .min_size(min_size)
            .fill(theme.surfaces.accent_soft)
            .stroke(Stroke::new(1.0, theme.surfaces.accent))
            .rounding(Rounding::same(theme.controls.button_rounding)),
    )
}

pub fn ghost_button(ui: &mut Ui, theme: &AppTheme, label: &str, min_size: Vec2) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(theme.text.secondary))
            .min_size(min_size)
            .fill(theme.surfaces.card_bg_alt)
            .stroke(Stroke::new(1.0, theme.surfaces.stroke_soft))
            .rounding(Rounding::same(theme.controls.button_rounding)),
    )
}

pub fn subtle_badge(ui: &mut Ui, theme: &AppTheme, text: &str) {
    let label = RichText::new(text)
        .size(12.0)
        .color(theme.text.accent)
        .strong();
    Frame::none()
        .fill(Color32::from_rgba_unmultiplied(
            theme.surfaces.accent.r(),
            theme.surfaces.accent.g(),
            theme.surfaces.accent.b(),
            34,
        ))
        .rounding(Rounding::same(10.0))
        .stroke(Stroke::new(1.0, theme.surfaces.accent_soft))
        .inner_margin(Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.label(label);
        });
}

pub fn vertical_divider(ui: &mut Ui, theme: &AppTheme, height: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, height), Sense::hover());
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        Stroke::new(1.0, theme.surfaces.stroke_soft),
    );
}

pub fn keycap(ui: &mut Ui, theme: &AppTheme, label: &str) {
    Frame::none()
        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 18))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 40),
        ))
        .rounding(Rounding::same(5.0))
        .inner_margin(Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(label)
                    .size(11.0)
                    .strong()
                    .color(theme.text.secondary),
            );
        });
}
