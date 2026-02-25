use egui::{
    vec2, Align, Align2, Color32, Context, FontId, Id, Key, Layout, Painter, Pos2, Rect, Response,
    ScrollArea, Sense, Shape, Stroke, Ui,
};

use crate::annotation::{Annotation, AnnotationKind, Handle, Point, RectData, TextSize, Tool};
use crate::state::{DragMode, DragState, EditorState, TextEditState, TextEditTarget};
use crate::theme;

pub fn show_canvas(ui: &mut Ui, ctx: &Context, state: &mut EditorState) {
    if state.image.is_none() {
        empty_canvas(ui);
        return;
    }

    let (texture_id, image_size) = {
        let image = state.image.as_mut().expect("image must exist");
        image.ensure_texture(ctx);
        (
            image.texture.as_ref().expect("texture is missing").id(),
            image.size_vec2(),
        )
    };

    let available = ui.available_size();
    if state.fit_zoom_to_view {
        state.set_fit_zoom(image_size, available - vec2(48.0, 48.0));
        state.fit_zoom_to_view = false;
    }

    let scaled = image_size * state.zoom;
    let canvas_size = vec2(
        (scaled.x + 48.0).max(available.x),
        (scaled.y + 48.0).max(available.y),
    );

    ScrollArea::both()
        .id_source("snapmark_canvas_scroll")
        .show(ui, |ui| {
            let (canvas_rect, response) =
                ui.allocate_exact_size(canvas_size, Sense::click_and_drag());

            let origin = Pos2::new(
                canvas_rect.center().x - scaled.x * 0.5,
                canvas_rect.center().y - scaled.y * 0.5,
            );
            let image_rect = Rect::from_min_size(origin, scaled);

            let painter = ui.painter_at(canvas_rect);
            draw_canvas_background(&painter, canvas_rect);
            let image_card = image_rect.expand(14.0);
            painter.rect_filled(
                image_card,
                18.0,
                Color32::from_rgba_unmultiplied(24, 28, 35, 190),
            );
            painter.rect_stroke(
                image_card,
                18.0,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 38)),
            );

            painter.image(
                texture_id,
                image_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );

            draw_annotations(&painter, state, image_rect);
            draw_drag_preview(&painter, state, image_rect);
            draw_selection(&painter, state, image_rect);

            let _ = handle_pointer_interaction(ctx, state, &response, image_rect);
            draw_text_editor(ui, state, image_rect);
        });
}

fn empty_canvas(ui: &mut Ui) {
    let theme = theme::premium_dark_theme();
    let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 16.0, theme.surfaces.canvas_bg);
    painter.rect_stroke(rect, 16.0, Stroke::new(1.0, theme.surfaces.stroke_soft));
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        "Paste an image (Cmd+V)",
        FontId::proportional(19.0),
        theme.text.secondary,
    );
}

fn draw_canvas_background(painter: &Painter, rect: Rect) {
    let theme = theme::premium_dark_theme();
    painter.rect_filled(rect, 16.0, theme.surfaces.canvas_bg);

    let top = Rect::from_min_max(
        rect.min,
        Pos2::new(rect.max.x, rect.min.y + rect.height() * 0.55),
    );
    let bottom = Rect::from_min_max(
        Pos2::new(rect.min.x, rect.min.y + rect.height() * 0.45),
        rect.max,
    );

    painter.rect_filled(
        top,
        16.0,
        Color32::from_rgba_unmultiplied(
            theme.surfaces.card_bg.r(),
            theme.surfaces.card_bg.g(),
            theme.surfaces.card_bg.b(),
            64,
        ),
    );
    painter.rect_filled(
        bottom,
        16.0,
        Color32::from_rgba_unmultiplied(
            theme.surfaces.panel_bg.r(),
            theme.surfaces.panel_bg.g(),
            theme.surfaces.panel_bg.b(),
            96,
        ),
    );
}

fn draw_annotations(painter: &Painter, state: &EditorState, image_rect: Rect) {
    for annotation in &state.annotations {
        draw_annotation(painter, annotation, image_rect, state.zoom, false);
    }
}

fn draw_drag_preview(painter: &Painter, state: &EditorState, image_rect: Rect) {
    let Some(drag) = state.drag_state.as_ref() else {
        return;
    };

    if drag.mode != DragMode::Draw {
        return;
    }

    let preview = match state.active_tool {
        Tool::Arrow | Tool::ArrowWithText => Annotation {
            id: 0,
            kind: AnnotationKind::Arrow {
                from: drag.start,
                to: drag.current,
            },
            color: state.active_color,
            stroke_width: state.active_stroke,
        },
        Tool::Rectangle => Annotation {
            id: 0,
            kind: AnnotationKind::Rectangle {
                rect: RectData {
                    min: drag.start,
                    max: drag.current,
                },
            },
            color: state.active_color,
            stroke_width: state.active_stroke,
        },
        Tool::Ellipse => Annotation {
            id: 0,
            kind: AnnotationKind::Ellipse {
                rect: RectData {
                    min: drag.start,
                    max: drag.current,
                },
            },
            color: state.active_color,
            stroke_width: state.active_stroke,
        },
        _ => return,
    };

    draw_annotation(painter, &preview, image_rect, state.zoom, true);
}

fn draw_selection(painter: &Painter, state: &EditorState, image_rect: Rect) {
    let Some(selected_id) = state.selection else {
        return;
    };
    let Some(annotation) = state.annotations.iter().find(|item| item.id == selected_id) else {
        return;
    };

    let bounds = annotation.bounds();
    let min = image_to_screen(bounds.min, image_rect, state.zoom);
    let max = image_to_screen(bounds.max, image_rect, state.zoom);
    let selection_rect = Rect::from_min_max(min, max);

    painter.rect_stroke(
        selection_rect,
        8.0,
        Stroke::new(1.8, Color32::from_rgb(77, 141, 255)),
    );

    for (_, point) in annotation.handles() {
        let handle_pos = image_to_screen(point.to_pos2(), image_rect, state.zoom);
        painter.rect_filled(
            Rect::from_center_size(handle_pos, vec2(9.0, 9.0)),
            4.0,
            Color32::from_rgb(77, 141, 255),
        );
        painter.rect_stroke(
            Rect::from_center_size(handle_pos, vec2(9.0, 9.0)),
            4.0,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 200)),
        );
    }
}

fn draw_annotation(
    painter: &Painter,
    annotation: &Annotation,
    image_rect: Rect,
    zoom: f32,
    preview: bool,
) {
    let mut color = annotation.color32();
    if preview {
        color = color.linear_multiply(0.7);
    }

    let stroke = Stroke::new((annotation.stroke_width.px() * zoom).max(1.0), color);

    match &annotation.kind {
        AnnotationKind::Arrow { from, to } => {
            draw_arrow(painter, *from, *to, image_rect, zoom, stroke)
        }
        AnnotationKind::ArrowWithText {
            from,
            to,
            text,
            size,
        } => {
            draw_arrow(painter, *from, *to, image_rect, zoom, stroke);
            let anchor = arrow_text_anchor(*from, *to);
            let pos = image_to_screen(anchor.to_pos2(), image_rect, zoom);
            painter.text(
                pos,
                Align2::LEFT_TOP,
                text,
                FontId::proportional(size.points() * zoom.min(1.25)),
                color,
            );
        }
        AnnotationKind::Text { pos, content, size } => {
            let screen = image_to_screen(pos.to_pos2(), image_rect, zoom);
            painter.text(
                screen,
                Align2::LEFT_TOP,
                content,
                FontId::proportional(size.points() * zoom.min(1.25)),
                color,
            );
        }
        AnnotationKind::Rectangle { rect } => {
            let r = rect.normalize().to_rect();
            let min = image_to_screen(r.min, image_rect, zoom);
            let max = image_to_screen(r.max, image_rect, zoom);
            painter.rect_stroke(Rect::from_min_max(min, max), 0.0, stroke);
        }
        AnnotationKind::Ellipse { rect } => {
            let r = rect.normalize().to_rect();
            let min = image_to_screen(r.min, image_rect, zoom);
            let max = image_to_screen(r.max, image_rect, zoom);
            let points = ellipse_polyline(Rect::from_min_max(min, max), 56);
            painter.add(Shape::closed_line(points, stroke));
        }
    }
}

fn draw_arrow(
    painter: &Painter,
    from: Point,
    to: Point,
    image_rect: Rect,
    zoom: f32,
    stroke: Stroke,
) {
    let from_screen = image_to_screen(from.to_pos2(), image_rect, zoom);
    let to_screen = image_to_screen(to.to_pos2(), image_rect, zoom);
    painter.line_segment([from_screen, to_screen], stroke);

    let direction = to_screen - from_screen;
    let len = direction.length().max(1.0);
    let unit = direction / len;
    let head_length = 12.0;
    let head_half_width = 7.0;

    let tip = to_screen;
    let base = tip - unit * head_length;
    let normal = vec2(-unit.y, unit.x);
    let left = base + normal * head_half_width;
    let right = base - normal * head_half_width;

    painter.add(Shape::convex_polygon(
        vec![tip, left, right],
        stroke.color,
        Stroke::NONE,
    ));
}

fn handle_pointer_interaction(
    ctx: &Context,
    state: &mut EditorState,
    response: &Response,
    image_rect: Rect,
) -> bool {
    if !response.hovered() && !response.dragged() && !response.clicked() {
        return false;
    }

    let pointer = ctx.input(|input| input.pointer.clone());
    let Some(pointer_pos) = pointer.interact_pos() else {
        return false;
    };

    if !image_rect.contains(pointer_pos)
        && !matches!(state.active_tool, Tool::Select)
        && state.drag_state.is_none()
    {
        return false;
    }

    let image_pos = screen_to_image(pointer_pos, image_rect, state.zoom);

    if response.double_clicked() {
        handle_double_click(state, image_pos, pointer_pos);
        return true;
    }

    if response.drag_started() {
        begin_drag(ctx, state, image_pos, pointer_pos, image_rect);
    }

    if response.dragged() {
        update_drag(ctx, state, image_pos);
    }

    if response.drag_stopped() {
        finish_drag(ctx, state);
    }

    if response.clicked() && !response.dragged() {
        handle_click(ctx, state, image_pos, pointer_pos, image_rect);
    }

    true
}

fn begin_drag(
    ctx: &Context,
    state: &mut EditorState,
    image_pos: Point,
    _screen_pos: Pos2,
    image_rect: Rect,
) {
    match state.active_tool {
        Tool::Arrow | Tool::ArrowWithText | Tool::Rectangle | Tool::Ellipse => {
            state.drag_state = Some(DragState {
                mode: DragMode::Draw,
                start: image_pos,
                current: image_pos,
                selection_id: None,
                handle: None,
                original: None,
            });
        }
        Tool::Select => {
            if let Some(selected_id) = state.selection {
                if let Some((handle, _)) =
                    detect_handle_hit(state, selected_id, image_pos, image_rect)
                {
                    let original = state
                        .annotations
                        .iter()
                        .find(|a| a.id == selected_id)
                        .cloned();
                    state.drag_state = Some(DragState {
                        mode: DragMode::Resize,
                        start: image_pos,
                        current: image_pos,
                        selection_id: Some(selected_id),
                        handle: Some(handle),
                        original,
                    });
                    return;
                }
            }

            if let Some(hit_id) = pick_annotation(state, image_pos) {
                state.selection = Some(hit_id);
                let original = state.annotations.iter().find(|a| a.id == hit_id).cloned();
                state.drag_state = Some(DragState {
                    mode: DragMode::Move,
                    start: image_pos,
                    current: image_pos,
                    selection_id: Some(hit_id),
                    handle: None,
                    original,
                });
            } else {
                state.selection = None;
            }
        }
        Tool::Text => {
            let _ = image_rect;
        }
    }

    let _ = ctx;
}

fn update_drag(ctx: &Context, state: &mut EditorState, image_pos: Point) {
    let active_tool = state.active_tool;
    let (mode, start, selection_id, handle, original, tool) = {
        let Some(drag) = state.drag_state.as_mut() else {
            return;
        };
        drag.current = image_pos;
        (
            drag.mode,
            drag.start,
            drag.selection_id,
            drag.handle,
            drag.original.clone(),
            active_tool,
        )
    };

    match mode {
        DragMode::Draw => {
            if matches!(tool, Tool::Rectangle | Tool::Ellipse)
                && ctx.input(|input| input.modifiers.shift)
            {
                if let Some(drag) = state.drag_state.as_mut() {
                    drag.current = constrain_square_point(drag.start, image_pos);
                }
            }
        }
        DragMode::Move => {
            if let (Some(id), Some(original)) = (selection_id, original) {
                let delta = start.delta(image_pos);
                if let Some(annotation) = state.find_annotation_mut(id) {
                    *annotation = original;
                    annotation.move_by(delta);
                }
            }
        }
        DragMode::Resize => {
            if let (Some(id), Some(handle), Some(original)) = (selection_id, handle, original) {
                let keep_square = ctx.input(|input| input.modifiers.shift);
                if let Some(annotation) = state.find_annotation_mut(id) {
                    *annotation = original;
                    annotation.resize_from_handle(handle, image_pos, keep_square);
                }
            }
        }
    }
}

fn finish_drag(ctx: &Context, state: &mut EditorState) {
    let Some(drag) = state.drag_state.take() else {
        return;
    };

    match drag.mode {
        DragMode::Draw => {
            let min_size = 5.0;
            let dx = (drag.current.x - drag.start.x).abs();
            let dy = (drag.current.y - drag.start.y).abs();
            if (dx * dx + dy * dy).sqrt() < min_size {
                return;
            }

            match state.active_tool {
                Tool::Arrow => {
                    let id = state.next_annotation_id();
                    let color = state.active_color;
                    let stroke = state.active_stroke;
                    state.add_annotation(Annotation {
                        id,
                        kind: AnnotationKind::Arrow {
                            from: drag.start,
                            to: drag.current,
                        },
                        color,
                        stroke_width: stroke,
                    });
                    state.set_tool(Tool::Select);
                }
                Tool::ArrowWithText => {
                    let screen_pos = Pos2::new(0.0, 0.0);
                    state.text_edit = Some(TextEditState {
                        buffer: String::new(),
                        screen_pos,
                        target: TextEditTarget::NewArrowText {
                            from: drag.start,
                            to: drag.current,
                            color: state.active_color,
                            stroke: state.active_stroke,
                        },
                        text_size: state.active_text_size,
                    });
                    state.set_tool(Tool::Select);
                }
                Tool::Rectangle => {
                    let id = state.next_annotation_id();
                    let color = state.active_color;
                    let stroke = state.active_stroke;
                    state.add_annotation(Annotation {
                        id,
                        kind: AnnotationKind::Rectangle {
                            rect: RectData {
                                min: drag.start,
                                max: drag.current,
                            }
                            .normalize(),
                        },
                        color,
                        stroke_width: stroke,
                    });
                    state.set_tool(Tool::Select);
                }
                Tool::Ellipse => {
                    let id = state.next_annotation_id();
                    let color = state.active_color;
                    let stroke = state.active_stroke;
                    state.add_annotation(Annotation {
                        id,
                        kind: AnnotationKind::Ellipse {
                            rect: RectData {
                                min: drag.start,
                                max: drag.current,
                            }
                            .normalize(),
                        },
                        color,
                        stroke_width: stroke,
                    });
                    state.set_tool(Tool::Select);
                }
                _ => {}
            }
        }
        DragMode::Move | DragMode::Resize => {
            let delta = drag.start.delta(drag.current);
            if delta.length_sq() > 0.01 {
                state.mark_changed();
                state.push_history_snapshot();
            }
        }
    }

    let _ = ctx;
}

fn handle_click(
    ctx: &Context,
    state: &mut EditorState,
    image_pos: Point,
    screen_pos: Pos2,
    image_rect: Rect,
) {
    match state.active_tool {
        Tool::Select => {
            state.selection = pick_annotation(state, image_pos);
        }
        Tool::Text => {
            state.text_edit = Some(TextEditState {
                buffer: String::new(),
                screen_pos,
                target: TextEditTarget::NewText { pos: image_pos },
                text_size: state.active_text_size,
            });
        }
        _ => {}
    }

    if let Some(text_edit) = state.text_edit.as_mut() {
        if matches!(&text_edit.target, TextEditTarget::NewArrowText { .. })
            && text_edit.screen_pos == Pos2::ZERO
        {
            text_edit.screen_pos =
                image_to_screen(image_pos.to_pos2(), image_rect, state.zoom) + vec2(10.0, -8.0);
        }
    }

    let _ = ctx;
}

fn handle_double_click(state: &mut EditorState, image_pos: Point, screen_pos: Pos2) {
    let Some(id) = pick_annotation(state, image_pos) else {
        return;
    };

    let Some(annotation) = state.annotations.iter().find(|item| item.id == id) else {
        return;
    };

    match &annotation.kind {
        AnnotationKind::Text { content, .. }
        | AnnotationKind::ArrowWithText { text: content, .. } => {
            let text_size = match &annotation.kind {
                AnnotationKind::Text { size, .. } | AnnotationKind::ArrowWithText { size, .. } => {
                    *size
                }
                _ => TextSize::M,
            };
            state.selection = Some(id);
            state.text_edit = Some(TextEditState {
                buffer: content.clone(),
                screen_pos,
                target: TextEditTarget::Existing { annotation_id: id },
                text_size,
            });
        }
        _ => {}
    }
}

fn draw_text_editor(ui: &mut Ui, state: &mut EditorState, image_rect: Rect) {
    let Some(edit) = state.text_edit.clone() else {
        return;
    };
    let target = edit.target.clone();
    let mut text_size = edit.text_size;

    let mut commit = false;
    let mut cancel = false;
    let mut buffer = edit.buffer.clone();
    let popup_id = Id::new("snapmark_text_edit");

    let mut screen_pos = edit.screen_pos;
    if matches!(&target, TextEditTarget::NewArrowText { .. }) && screen_pos == Pos2::ZERO {
        if let TextEditTarget::NewArrowText { from, to, .. } = &target {
            let anchor = arrow_text_anchor(*from, *to);
            screen_pos = image_to_screen(anchor.to_pos2(), image_rect, state.zoom);
        }
    }

    egui::Area::new(popup_id)
        .order(egui::Order::Foreground)
        .fixed_pos(screen_pos)
        .show(ui.ctx(), |ui| {
            let theme = theme::premium_dark_theme();
            egui::Frame::none()
                .fill(theme.surfaces.card_bg)
                .rounding(egui::Rounding::same(theme.controls.card_rounding))
                .stroke(Stroke::new(1.0, theme.surfaces.stroke_strong))
                .inner_margin(egui::Margin::symmetric(14.0, 12.0))
                .show(ui, |ui| {
                    ui.set_min_width(460.0);
                    ui.spacing_mut().item_spacing = vec2(10.0, 8.0);
                    ui.spacing_mut().interact_size.y = 26.0;
                    let mut text_size_points = text_size.as_u8();
                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width(), 28.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.allocate_ui_with_layout(
                                vec2(44.0, 26.0),
                                Layout::right_to_left(Align::Center),
                                |ui| {
                                    ui.label("Size");
                                },
                            );
                            ui.add_space(8.0);
                            egui::ComboBox::from_id_source(popup_id.with("font_size"))
                                .selected_text(text_size_points.to_string())
                                .width(104.0)
                                .show_ui(ui, |ui| {
                                    for size in TextSize::MIN..=TextSize::MAX {
                                        ui.selectable_value(
                                            &mut text_size_points,
                                            size,
                                            size.to_string(),
                                        );
                                    }
                                });
                            ui.add_space(6.0);
                            ui.allocate_ui_with_layout(
                                vec2(16.0, 26.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.label("pt");
                                },
                            );
                            if text_size_points != text_size.as_u8() {
                                text_size = TextSize::from_points(text_size_points);
                            }
                        },
                    );
                    ui.add_space(10.0);
                    let response = egui::Frame::none()
                        .fill(theme.surfaces.app_bg)
                        .rounding(egui::Rounding::same(theme.controls.panel_rounding))
                        .stroke(Stroke::new(1.0, theme.surfaces.accent))
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.add_sized(
                                vec2(ui.available_width(), 120.0),
                                egui::TextEdit::multiline(&mut buffer)
                                    .desired_rows(6)
                                    .desired_width(f32::INFINITY)
                                    .hint_text("Введите текст")
                                    .frame(false),
                            )
                        })
                        .inner;

                    let pressed_enter = ui.input(|input| input.key_pressed(Key::Enter));
                    let cmd_pressed =
                        ui.input(|input| input.modifiers.command || input.modifiers.ctrl);
                    let click_away =
                        response.lost_focus() && ui.input(|input| input.pointer.any_released());

                    if pressed_enter {
                        match &target {
                            TextEditTarget::NewText { .. } => {
                                if cmd_pressed {
                                    commit = true;
                                }
                            }
                            TextEditTarget::NewArrowText { .. } => {
                                commit = true;
                            }
                            TextEditTarget::Existing { .. } => {
                                if cmd_pressed {
                                    commit = true;
                                }
                            }
                        }
                    } else if click_away {
                        match &target {
                            TextEditTarget::NewArrowText { .. }
                            | TextEditTarget::Existing { .. } => {
                                commit = true;
                            }
                            TextEditTarget::NewText { .. } => {}
                        }
                    }

                    if ui.input(|input| input.key_pressed(Key::Escape)) {
                        cancel = true;
                    }
                });
        });

    if cancel {
        state.text_edit = None;
        return;
    }

    if commit {
        apply_text_edit(state, target, buffer, text_size);
        return;
    }

    state.text_edit = Some(TextEditState {
        buffer,
        screen_pos,
        target,
        text_size,
    });
}

fn apply_text_edit(
    state: &mut EditorState,
    target: TextEditTarget,
    text: String,
    text_size: TextSize,
) {
    let new_content = text.trim().to_string();
    state.set_text_size(text_size);
    match target {
        TextEditTarget::NewText { pos } => {
            if new_content.is_empty() {
                state.text_edit = None;
                return;
            }
            let id = state.next_annotation_id();
            let color = state.active_color;
            let stroke = state.active_stroke;
            state.add_annotation(Annotation {
                id,
                kind: AnnotationKind::Text {
                    pos,
                    content: new_content,
                    size: text_size,
                },
                color,
                stroke_width: stroke,
            });
            state.set_tool(Tool::Select);
        }
        TextEditTarget::NewArrowText {
            from,
            to,
            color,
            stroke,
        } => {
            let kind = if new_content.is_empty() {
                AnnotationKind::Arrow { from, to }
            } else {
                AnnotationKind::ArrowWithText {
                    from,
                    to,
                    text: new_content,
                    size: text_size,
                }
            };
            let id = state.next_annotation_id();
            state.add_annotation(Annotation {
                id,
                kind,
                color,
                stroke_width: stroke,
            });
            state.set_tool(Tool::Select);
        }
        TextEditTarget::Existing { annotation_id } => {
            let mut changed = false;
            if let Some(annotation) = state.find_annotation_mut(annotation_id) {
                match &mut annotation.kind {
                    AnnotationKind::Text { content, size, .. } => {
                        *content = new_content.clone();
                        *size = text_size;
                        changed = true;
                    }
                    AnnotationKind::ArrowWithText {
                        text: content,
                        size,
                        ..
                    } => {
                        *content = new_content.clone();
                        *size = text_size;
                        changed = true;
                    }
                    _ => {}
                }
            }
            if changed {
                state.mark_changed();
                state.push_history_snapshot();
            }
        }
    }
    state.text_edit = None;
}

fn detect_handle_hit(
    state: &EditorState,
    annotation_id: u64,
    image_pos: Point,
    image_rect: Rect,
) -> Option<(Handle, Point)> {
    let tolerance = 8.0 / state.zoom.max(0.25);
    let annotation = state
        .annotations
        .iter()
        .find(|item| item.id == annotation_id)?;

    for (handle, point) in annotation.handles() {
        let screen = image_to_screen(point.to_pos2(), image_rect, state.zoom);
        let hit = Rect::from_center_size(screen, vec2(12.0, 12.0));
        if hit.contains(image_to_screen(image_pos.to_pos2(), image_rect, state.zoom)) {
            return Some((handle, point));
        }

        if point.delta(image_pos).length() <= tolerance {
            return Some((handle, point));
        }
    }

    None
}

fn pick_annotation(state: &EditorState, image_pos: Point) -> Option<u64> {
    state
        .annotations
        .iter()
        .rev()
        .find(|annotation| annotation.contains(image_pos, 6.0 / state.zoom.max(0.25)))
        .map(|annotation| annotation.id)
}

fn ellipse_polyline(rect: Rect, segments: usize) -> Vec<Pos2> {
    let mut points = Vec::with_capacity(segments);
    let center = rect.center();
    let rx = rect.width() * 0.5;
    let ry = rect.height() * 0.5;

    for i in 0..segments {
        let t = (i as f32 / segments as f32) * std::f32::consts::TAU;
        points.push(Pos2::new(center.x + rx * t.cos(), center.y + ry * t.sin()));
    }

    points
}

fn arrow_text_anchor(from: Point, to: Point) -> Point {
    let dir = from.delta(to);
    let len = dir.length().max(1.0);
    let unit = dir / len;
    let perp_a = vec2(-unit.y, unit.x);
    let perp_b = vec2(unit.y, -unit.x);
    let up_perp = if perp_a.y < perp_b.y { perp_a } else { perp_b };
    let offset = up_perp * 12.0 + vec2(6.0, -2.0);
    Point::new(from.x + offset.x, from.y + offset.y)
}

fn image_to_screen(pos: Pos2, image_rect: Rect, zoom: f32) -> Pos2 {
    Pos2::new(
        image_rect.min.x + pos.x * zoom,
        image_rect.min.y + pos.y * zoom,
    )
}

fn screen_to_image(pos: Pos2, image_rect: Rect, zoom: f32) -> Point {
    Point::new(
        (pos.x - image_rect.min.x) / zoom,
        (pos.y - image_rect.min.y) / zoom,
    )
}

fn constrain_square_point(start: Point, current: Point) -> Point {
    let dx = current.x - start.x;
    let dy = current.y - start.y;
    let side = dx.abs().max(dy.abs());
    Point::new(start.x + side * dx.signum(), start.y + side * dy.signum())
}
