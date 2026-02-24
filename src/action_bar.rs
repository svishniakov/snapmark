use egui::{vec2, Align, Layout, Ui};

use crate::state::EditorState;
use crate::theme::{self, WidthClass};
use crate::ui_controls;

pub struct ActionBarOutput {
    pub undo: bool,
    pub redo: bool,
    pub copy: bool,
    pub save: bool,
}

pub fn should_show_shortcut_label(width_class: WidthClass, available_width: f32) -> bool {
    match width_class {
        WidthClass::Compact => available_width >= 420.0,
        WidthClass::Regular | WidthClass::Wide => true,
    }
}

pub fn show_action_bar(
    ui: &mut Ui,
    state: &EditorState,
    copied_feedback: bool,
    width_class: WidthClass,
) -> ActionBarOutput {
    let theme = theme::premium_dark_theme();
    let action_h = theme.controls.action_height;
    let button_gap = theme.layout.space_3 + 2.0;
    let group_gap = theme.layout.space_4 + 4.0;
    let undo_w = if width_class == WidthClass::Compact {
        88.0
    } else {
        98.0
    };
    let copy_w = if width_class == WidthClass::Compact {
        96.0
    } else {
        108.0
    };
    let save_w = if width_class == WidthClass::Compact {
        92.0
    } else {
        108.0
    };
    let undo_redo_w = undo_w * 2.0 + button_gap;
    let copy_save_w = copy_w + save_w + button_gap;
    let shortcut_visible = should_show_shortcut_label(
        width_class,
        ui.available_width() - undo_redo_w - copy_save_w,
    );

    let mut out = ActionBarOutput {
        undo: false,
        redo: false,
        copy: false,
        save: false,
    };

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(button_gap, 0.0);

        let undo_button = ui.add_enabled_ui(state.can_undo(), |ui| {
            ui_controls::ghost_button(ui, &theme, "↩ Undo", vec2(undo_w, action_h))
        });
        if undo_button.inner.clicked() {
            out.undo = true;
        }

        let redo_button = ui.add_enabled_ui(state.can_redo(), |ui| {
            ui_controls::ghost_button(ui, &theme, "↪ Redo", vec2(undo_w, action_h))
        });
        if redo_button.inner.clicked() {
            out.redo = true;
        }

        ui.add_space(group_gap);

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(theme.layout.space_2);

            if shortcut_visible {
                ui_controls::keycap(ui, &theme, "S");
                ui.add_space(theme.layout.space_2);
                ui_controls::keycap(ui, &theme, "⌘");
                ui.add_space(theme.layout.space_3);
                ui_controls::vertical_divider(ui, &theme, 16.0);
                ui.add_space(theme.layout.space_3);
            }

            let save_button = ui.add_enabled_ui(state.image.is_some(), |ui| {
                ui_controls::ghost_button(ui, &theme, "Save", vec2(save_w, action_h))
            });
            let mut save_response = save_button.inner;
            if !shortcut_visible {
                save_response = save_response.on_hover_text("⌘S");
            }
            if save_response.clicked() {
                out.save = true;
            }

            ui.add_space(button_gap);

            if copied_feedback && width_class != WidthClass::Compact {
                ui_controls::subtle_badge(ui, &theme, "clipboard updated");
                ui.add_space(button_gap);
            }

            let copy_text = if copied_feedback { "Copied" } else { "Copy" };
            let copy_button = ui.add_enabled_ui(state.image.is_some(), |ui| {
                ui_controls::primary_button(ui, &theme, copy_text, vec2(copy_w, action_h))
            });
            if copy_button.inner.clicked() {
                out.copy = true;
            }
        });
    });

    out
}

#[cfg(test)]
mod tests {
    use super::should_show_shortcut_label;
    use crate::theme::WidthClass;

    #[test]
    fn action_bar_compact_hides_shortcut_label_first() {
        assert!(!should_show_shortcut_label(WidthClass::Compact, 320.0));
        assert!(should_show_shortcut_label(WidthClass::Compact, 420.0));
        assert!(should_show_shortcut_label(WidthClass::Regular, 320.0));
    }
}
