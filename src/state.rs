use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use egui::{ColorImage, Context as EguiContext, Pos2, TextureHandle, TextureOptions, Vec2};
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::annotation::{Annotation, AnnotationId, Handle, Point, StrokeWidth, TextSize, Tool};
use crate::history::UndoHistory;

pub const ZOOM_STEPS: &[f32] = &[0.25, 0.33, 0.5, 0.67, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0];

#[derive(Default)]
pub struct AppUiFlags {
    pub copy_feedback_until: Option<f64>,
    pub ask_replace_image: Option<PendingImage>,
    pub ask_replace_from_paste: Option<PendingImage>,
}

#[derive(Clone)]
pub struct PendingImage {
    pub image: DynamicImage,
    pub scale_factor: f32,
    pub source: PendingImageSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingImageSource {
    Watcher,
    Paste,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualEffectsMode {
    Hybrid,
    EguiOnly,
}

impl Default for VisualEffectsMode {
    fn default() -> Self {
        Self::Hybrid
    }
}

#[derive(Clone, Debug)]
pub struct VisualEffectsState {
    pub mode: VisualEffectsMode,
    pub enabled: bool,
    pub fallback_reason: Option<String>,
    pub supports_vibrancy: bool,
    pub show_fallback_badge: bool,
}

impl Default for VisualEffectsState {
    fn default() -> Self {
        Self {
            mode: VisualEffectsMode::Hybrid,
            enabled: false,
            fallback_reason: None,
            supports_vibrancy: false,
            show_fallback_badge: false,
        }
    }
}

pub struct EditorImage {
    pub dynamic: DynamicImage,
    pub texture: Option<TextureHandle>,
    pub scale_factor: f32,
}

impl EditorImage {
    pub fn size_vec2(&self) -> Vec2 {
        Vec2::new(self.dynamic.width() as f32, self.dynamic.height() as f32)
    }

    pub fn ensure_texture(&mut self, ctx: &EguiContext) {
        if self.texture.is_some() {
            return;
        }
        let rgba = self.dynamic.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let color = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        let texture = ctx.load_texture("screenshot", color, TextureOptions::LINEAR);
        self.texture = Some(texture);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragMode {
    Draw,
    Move,
    Resize,
}

#[derive(Clone, Debug)]
pub struct DragState {
    pub mode: DragMode,
    pub start: Point,
    pub current: Point,
    pub selection_id: Option<AnnotationId>,
    pub handle: Option<Handle>,
    pub original: Option<Annotation>,
}

#[derive(Clone, Debug)]
pub enum TextEditTarget {
    NewText {
        pos: Point,
    },
    NewArrowText {
        from: Point,
        to: Point,
        color: [u8; 4],
        stroke: StrokeWidth,
    },
    Existing {
        annotation_id: AnnotationId,
    },
}

#[derive(Clone, Debug)]
pub struct TextEditState {
    pub buffer: String,
    pub screen_pos: Pos2,
    pub target: TextEditTarget,
    pub text_size: TextSize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct UserSettings {
    pub last_color: [u8; 4],
    pub last_stroke: StrokeWidth,
    pub last_text_size: TextSize,
    pub dock_icon_visible: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            last_color: [229, 62, 62, 255],
            last_stroke: StrokeWidth::Medium,
            last_text_size: TextSize::M,
            dock_icon_visible: true,
        }
    }
}

pub struct EditorState {
    pub image: Option<EditorImage>,
    pub annotations: Vec<Annotation>,
    pub history: UndoHistory<Vec<Annotation>>,
    pub active_tool: Tool,
    pub active_color: [u8; 4],
    pub active_stroke: StrokeWidth,
    pub active_text_size: TextSize,
    pub selection: Option<AnnotationId>,
    pub drag_state: Option<DragState>,
    pub text_edit: Option<TextEditState>,
    pub zoom: f32,
    pub view_offset: Vec2,
    pub exported: bool,
    pub has_edited: bool,
    pub next_id: AnnotationId,
    pub settings: UserSettings,
    pub window_open: bool,
    pub fit_zoom_to_view: bool,
    pub theme_density: f32,
    pub hover_intensity: f32,
    pub pressed_intensity: f32,
    pub global_spacing_scale: f32,
    pub visual_effects: VisualEffectsState,
}

impl Default for EditorState {
    fn default() -> Self {
        let settings = UserSettings::load().unwrap_or_default();
        Self {
            image: None,
            annotations: Vec::new(),
            history: UndoHistory::new(Vec::new()),
            active_tool: Tool::Select,
            active_color: settings.last_color,
            active_stroke: settings.last_stroke,
            active_text_size: settings.last_text_size,
            selection: None,
            drag_state: None,
            text_edit: None,
            zoom: 1.0,
            view_offset: Vec2::ZERO,
            exported: false,
            has_edited: false,
            next_id: 1,
            settings,
            window_open: false,
            fit_zoom_to_view: true,
            theme_density: 1.0,
            hover_intensity: 1.0,
            pressed_intensity: 1.0,
            global_spacing_scale: 1.0,
            visual_effects: VisualEffectsState::default(),
        }
    }
}

impl EditorState {
    pub fn mark_changed(&mut self) {
        self.has_edited = true;
        self.exported = false;
    }

    pub fn push_history_snapshot(&mut self) {
        self.history.push_snapshot(self.annotations.clone());
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn undo(&mut self) {
        if let Some(snapshot) = self.history.undo() {
            self.annotations = snapshot;
            self.selection = None;
            self.text_edit = None;
        }
    }

    pub fn redo(&mut self) {
        if let Some(snapshot) = self.history.redo() {
            self.annotations = snapshot;
            self.selection = None;
            self.text_edit = None;
        }
    }

    pub fn reset_for_new_image(
        &mut self,
        ctx: &EguiContext,
        image: DynamicImage,
        scale_factor: f32,
    ) {
        self.image = Some(EditorImage {
            dynamic: image,
            texture: None,
            scale_factor,
        });
        if let Some(editor_image) = self.image.as_mut() {
            editor_image.ensure_texture(ctx);
        }
        self.annotations.clear();
        self.selection = None;
        self.text_edit = None;
        self.drag_state = None;
        self.has_edited = false;
        self.exported = false;
        self.zoom = 1.0;
        self.view_offset = Vec2::ZERO;
        self.history.clear_with(Vec::new());
        self.fit_zoom_to_view = true;
    }

    pub fn next_annotation_id(&mut self) -> AnnotationId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.active_tool = tool;
        if tool == Tool::Select {
            self.drag_state = None;
        }
    }

    pub fn set_color(&mut self, rgba: [u8; 4]) {
        self.active_color = rgba;
        self.settings.last_color = rgba;
        let _ = self.settings.save();

        let mut changed_selection = false;
        if let Some(selected_id) = self.selection {
            if let Some(annotation) = self
                .annotations
                .iter_mut()
                .find(|annotation| annotation.id == selected_id)
            {
                if annotation.color != rgba {
                    annotation.color = rgba;
                    changed_selection = true;
                }
            }
        }
        if changed_selection {
            self.mark_changed();
            self.push_history_snapshot();
        }
    }

    pub fn set_stroke(&mut self, stroke: StrokeWidth) {
        self.active_stroke = stroke;
        self.settings.last_stroke = stroke;
        let _ = self.settings.save();

        let mut changed_selection = false;
        if let Some(selected_id) = self.selection {
            if let Some(annotation) = self
                .annotations
                .iter_mut()
                .find(|annotation| annotation.id == selected_id)
            {
                if annotation.stroke_width != stroke {
                    annotation.stroke_width = stroke;
                    changed_selection = true;
                }
            }
        }
        if changed_selection {
            self.mark_changed();
            self.push_history_snapshot();
        }
    }

    pub fn set_text_size(&mut self, size: TextSize) {
        self.active_text_size = size;
        self.settings.last_text_size = size;
        let _ = self.settings.save();
    }

    pub fn set_dock_icon_visible(&mut self, visible: bool) {
        if self.settings.dock_icon_visible == visible {
            return;
        }
        self.settings.dock_icon_visible = visible;
        let _ = self.settings.save();
    }

    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
        self.mark_changed();
        self.push_history_snapshot();
    }

    pub fn delete_selected(&mut self) {
        if let Some(selected) = self.selection.take() {
            self.annotations
                .retain(|annotation| annotation.id != selected);
            self.mark_changed();
            self.push_history_snapshot();
        }
    }

    pub fn nearest_zoom_step(&self) -> usize {
        let mut best_idx = 0usize;
        let mut best_diff = f32::MAX;
        for (idx, step) in ZOOM_STEPS.iter().enumerate() {
            let diff = (self.zoom - step).abs();
            if diff < best_diff {
                best_diff = diff;
                best_idx = idx;
            }
        }
        best_idx
    }

    pub fn zoom_in(&mut self) {
        let idx = self.nearest_zoom_step();
        if idx + 1 < ZOOM_STEPS.len() {
            self.zoom = ZOOM_STEPS[idx + 1];
        }
    }

    pub fn zoom_out(&mut self) {
        let idx = self.nearest_zoom_step();
        if idx > 0 {
            self.zoom = ZOOM_STEPS[idx - 1];
        }
    }

    pub fn set_fit_zoom(&mut self, image_size: Vec2, view_size: Vec2) {
        let width_scale = (view_size.x / image_size.x).max(0.1);
        let height_scale = (view_size.y / image_size.y).max(0.1);
        self.zoom = width_scale.min(height_scale).clamp(0.25, 4.0);
    }

    pub fn find_annotation_mut(&mut self, id: AnnotationId) -> Option<&mut Annotation> {
        self.annotations
            .iter_mut()
            .find(|annotation| annotation.id == id)
    }
}

impl UserSettings {
    fn file_path() -> Option<PathBuf> {
        let dirs = ProjectDirs::from("com", "snapmark", "snapmark")?;
        let config_dir = dirs.config_dir();
        std::fs::create_dir_all(config_dir).ok()?;
        Some(config_dir.join("settings.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::file_path().context("cannot resolve settings path")?;
        let raw = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::file_path().context("cannot resolve settings path")?;
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}
