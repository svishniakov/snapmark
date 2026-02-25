use egui::{
    vec2, Align, Align2, Color32, ComboBox, FontId, Layout, Pos2, Rect, RichText, Shape, Stroke, Ui,
};

use crate::annotation::{StrokeWidth, TextSize, Tool};
use crate::state::EditorState;
use crate::theme::{self, WidthClass};
use crate::ui_controls;

const PALETTE: [[u8; 4]; 8] = [
    [0xE5, 0x3E, 0x3E, 0xFF],
    [0xDD, 0x6B, 0x20, 0xFF],
    [0xD6, 0x9E, 0x2E, 0xFF],
    [0x38, 0xA1, 0x69, 0xFF],
    [0x31, 0x82, 0xCE, 0xFF],
    [0x80, 0x5A, 0xD5, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0x1A, 0x20, 0x2C, 0xFF],
];

#[derive(Clone, Copy, Debug)]
pub struct ToolbarPlan {
    pub show_tools_inline: bool,
    pub visible_color_count: usize,
    pub show_stroke_inline: bool,
    pub show_text_size_inline: bool,
    pub show_overflow: bool,
}

pub fn plan_toolbar_items(width_class: WidthClass, state: &EditorState) -> ToolbarPlan {
    let needs_text_size =
        state.active_tool == Tool::Text || state.active_tool == Tool::ArrowWithText;
    let visible_color_count = match width_class {
        WidthClass::Compact => 4,
        WidthClass::Regular => 6,
        WidthClass::Wide => PALETTE.len(),
    };
    let show_stroke_inline = width_class != WidthClass::Compact;
    let show_text_size_inline = needs_text_size && width_class != WidthClass::Compact;

    let hidden_for_overflow = visible_color_count < PALETTE.len()
        || !show_stroke_inline
        || (needs_text_size && !show_text_size_inline);

    ToolbarPlan {
        show_tools_inline: true,
        visible_color_count,
        show_stroke_inline,
        show_text_size_inline,
        show_overflow: hidden_for_overflow,
    }
}

pub fn show_toolbar(ui: &mut Ui, state: &mut EditorState, width_class: WidthClass) {
    let theme = theme::premium_dark_theme();
    let plan = plan_toolbar_items(width_class, state);

    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.spacing_mut().interact_size.y = theme.layout.chip_h;
        ui.spacing_mut().button_padding.y = theme.layout.space_1;
        ui.spacing_mut().item_spacing = vec2(theme.layout.control_gap, 0.0);

        if plan.show_tools_inline {
            render_tool_group(ui, state);
        }

        if plan.visible_color_count > 0 {
            group_separator(ui, &theme);
            render_palette_group(ui, state, &theme, plan.visible_color_count);
        }

        if plan.show_stroke_inline {
            group_separator(ui, &theme);
            ui.label(
                RichText::new("Line thickness")
                    .color(theme.text.muted)
                    .size(12.0),
            );
            stroke_button(ui, state, StrokeWidth::Thin, "S");
            stroke_button(ui, state, StrokeWidth::Medium, "M");
            stroke_button(ui, state, StrokeWidth::Thick, "L");
        }

        if plan.show_text_size_inline {
            group_separator(ui, &theme);
            text_size_points_control(ui, state, "toolbar_text_size_inline");
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if plan.show_overflow {
                ui.menu_button("…", |ui| {
                    ui.spacing_mut().item_spacing =
                        vec2(theme.layout.control_gap, theme.layout.space_2);

                    if plan.visible_color_count < PALETTE.len() {
                        ui.label(RichText::new("Colors").color(theme.text.muted).size(12.0));
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing =
                                vec2(theme.layout.control_gap, theme.layout.space_1);
                            for color in PALETTE.iter().skip(plan.visible_color_count) {
                                let color32 = Color32::from_rgba_unmultiplied(
                                    color[0], color[1], color[2], color[3],
                                );
                                let selected = state.active_color == *color;
                                if ui_controls::color_chip(ui, &theme, color32, selected)
                                    .on_hover_text("Choose color")
                                    .clicked()
                                {
                                    state.set_color(*color);
                                    ui.close_menu();
                                }
                            }
                        });
                    }

                    if !plan.show_stroke_inline {
                        ui.separator();
                        ui.label(
                            RichText::new("Line thickness")
                                .color(theme.text.muted)
                                .size(12.0),
                        );
                        ui.horizontal(|ui| {
                            stroke_button(ui, state, StrokeWidth::Thin, "S");
                            stroke_button(ui, state, StrokeWidth::Medium, "M");
                            stroke_button(ui, state, StrokeWidth::Thick, "L");
                        });
                    }

                    let needs_text_size =
                        state.active_tool == Tool::Text || state.active_tool == Tool::ArrowWithText;
                    if needs_text_size && !plan.show_text_size_inline {
                        ui.separator();
                        ui.label(
                            RichText::new("Text size")
                                .color(theme.text.muted)
                                .size(12.0),
                        );
                        ui.horizontal(|ui| {
                            text_size_points_control(ui, state, "toolbar_text_size_overflow");
                        });
                    }
                });
            }
        });
    });
}

fn render_tool_group(ui: &mut Ui, state: &mut EditorState) {
    tool_button(ui, state, Tool::Select, "Select (V / Esc)");
    tool_button(ui, state, Tool::Arrow, "Arrow (A)");
    tool_button(ui, state, Tool::ArrowWithText, "Arrow + Text (Shift+A)");
    tool_button(ui, state, Tool::Text, "Text (T)");
    tool_button(ui, state, Tool::Rectangle, "Rectangle (R)");
    tool_button(ui, state, Tool::Ellipse, "Ellipse (E)");
}

fn render_palette_group(
    ui: &mut Ui,
    state: &mut EditorState,
    theme: &theme::AppTheme,
    count: usize,
) {
    for color in PALETTE.iter().take(count) {
        let color32 = Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
        let selected = state.active_color == *color;
        if ui_controls::color_chip(ui, theme, color32, selected)
            .on_hover_text("Choose color")
            .clicked()
        {
            state.set_color(*color);
        }
    }
}

fn group_separator(ui: &mut Ui, theme: &theme::AppTheme) {
    ui.separator();
    let extra = (theme.layout.group_gap - theme.layout.control_gap).max(0.0);
    if extra > 0.0 {
        ui.add_space(extra);
    }
}

fn tool_button(ui: &mut Ui, state: &mut EditorState, tool: Tool, hint: &str) {
    let theme = theme::premium_dark_theme();
    let selected = state.active_tool == tool;
    let response = ui_controls::tool_chip(ui, &theme, "", selected).on_hover_text(hint);
    draw_tool_icon(ui, response.rect, tool, selected);
    if response.clicked() {
        state.set_tool(tool);
    }
}

fn draw_tool_icon(ui: &Ui, rect: Rect, tool: Tool, selected: bool) {
    let theme = theme::premium_dark_theme();
    let color = if selected {
        theme.text.primary
    } else {
        theme.text.secondary
    };
    let stroke = Stroke::new(1.65, color);
    let painter = ui.painter();
    let icon_rect = rect.shrink2(vec2(8.0, 5.0));

    match tool {
        Tool::Select => {
            // Tabler "cursor" icon geometry adapted for the tool-chip rect.
            let tip = Pos2::new(icon_rect.left() + 2.0, icon_rect.top() + 1.0);
            let base = Pos2::new(icon_rect.left() + 8.6, icon_rect.bottom() - 1.6);
            let inner = Pos2::new(icon_rect.left() + 10.8, icon_rect.center().y + 1.8);
            let wing = Pos2::new(icon_rect.right() - 1.8, icon_rect.center().y - 0.6);

            painter.add(Shape::convex_polygon(
                vec![tip, base, inner, wing],
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 40),
                Stroke::NONE,
            ));
            painter.line_segment([tip, base], stroke);
            painter.line_segment([base, inner], stroke);
            painter.line_segment([inner, wing], stroke);
            painter.line_segment([wing, tip], stroke);
        }
        Tool::Arrow => {
            let y = icon_rect.center().y + 0.5;
            let start = Pos2::new(icon_rect.left() + 2.0, y);
            let tip = Pos2::new(icon_rect.right() - 2.0, y);
            painter.line_segment([start, tip], stroke);
            painter.add(Shape::convex_polygon(
                vec![
                    tip,
                    Pos2::new(tip.x - 6.0, tip.y - 4.5),
                    Pos2::new(tip.x - 6.0, tip.y + 4.5),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Tool::ArrowWithText => {
            let y = icon_rect.center().y + 0.5;
            let start = Pos2::new(icon_rect.left() + 8.0, y);
            let tip = Pos2::new(icon_rect.right() - 2.0, y);
            painter.line_segment([start, tip], stroke);
            painter.add(Shape::convex_polygon(
                vec![
                    tip,
                    Pos2::new(tip.x - 6.0, tip.y - 4.5),
                    Pos2::new(tip.x - 6.0, tip.y + 4.5),
                ],
                color,
                Stroke::NONE,
            ));
            painter.text(
                Pos2::new(icon_rect.left() + 4.0, icon_rect.center().y),
                Align2::CENTER_CENTER,
                "T",
                FontId::proportional(13.0),
                color,
            );
        }
        Tool::Text => {
            painter.text(
                icon_rect.center(),
                Align2::CENTER_CENTER,
                "T",
                FontId::proportional(14.5),
                color,
            );
        }
        Tool::Rectangle => {
            let r = icon_rect.shrink2(vec2(2.0, 3.0));
            painter.rect_stroke(r, 2.5, stroke);
            painter.circle_filled(r.left_top() + vec2(1.0, 1.0), 1.2, color);
        }
        Tool::Ellipse => {
            let radius = icon_rect.width().min(icon_rect.height()) * 0.40;
            painter.circle_stroke(icon_rect.center(), radius, stroke);
            painter.circle_filled(
                Pos2::new(
                    icon_rect.center().x + radius * 0.35,
                    icon_rect.center().y - radius * 0.15,
                ),
                1.2,
                color,
            );
        }
    }
}

fn stroke_button(ui: &mut Ui, state: &mut EditorState, stroke: StrokeWidth, label: &str) {
    let theme = theme::premium_dark_theme();
    let hint = match stroke {
        StrokeWidth::Thin => "Line thickness: Small",
        StrokeWidth::Medium => "Line thickness: Medium",
        StrokeWidth::Thick => "Line thickness: Large",
    };
    if ui_controls::segmented(ui, &theme, label, state.active_stroke == stroke)
        .on_hover_text(hint)
        .clicked()
    {
        state.set_stroke(stroke);
    }
}

fn text_size_points_control(ui: &mut Ui, state: &mut EditorState, id_suffix: &'static str) {
    let theme = theme::premium_dark_theme();
    let mut points = state.active_text_size.as_u8();
    let control_h = theme.layout.chip_h;

    ui.allocate_ui_with_layout(
        vec2(112.0, control_h),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = theme.layout.space_2;
            ui.scope(|ui| {
                ui.spacing_mut().interact_size.y = control_h;
                ui.spacing_mut().button_padding.y = theme.layout.space_1;

                ComboBox::from_id_source(("snapmark_toolbar_font_size", id_suffix))
                    .selected_text(points.to_string())
                    .width(74.0)
                    .show_ui(ui, |ui| {
                        for size in TextSize::MIN..=TextSize::MAX {
                            ui.selectable_value(&mut points, size, size.to_string());
                        }
                    });
            });

            ui.allocate_ui_with_layout(
                vec2(16.0, control_h),
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.label(RichText::new("pt").color(theme.text.muted).size(12.0));
                },
            );
        },
    );

    if points != state.active_text_size.as_u8() {
        state.set_text_size(TextSize::from_points(points));
    }
}

#[cfg(test)]
mod tests {
    use super::plan_toolbar_items;
    use crate::annotation::Tool;
    use crate::state::EditorState;
    use crate::theme::WidthClass;

    #[test]
    fn plan_toolbar_items_compact_keeps_p0_visible() {
        let mut state = EditorState::default();
        state.active_tool = Tool::Select;
        let plan = plan_toolbar_items(WidthClass::Compact, &state);

        assert!(plan.show_tools_inline);
    }

    #[test]
    fn plan_toolbar_items_moves_low_priority_to_overflow() {
        let mut state = EditorState::default();
        state.active_tool = Tool::Text;
        let plan = plan_toolbar_items(WidthClass::Compact, &state);

        assert!(plan.show_overflow);
        assert!(!plan.show_stroke_inline);
        assert!(!plan.show_text_size_inline);
    }
}
