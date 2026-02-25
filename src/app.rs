use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context as _, Result};
use chrono::Local;
use eframe::egui::{self, Context as EguiContext, Key, RichText, TopBottomPanel};
use eframe::{App, Frame};
use image::ImageFormat;

use crate::action_bar;
use crate::canvas;
use crate::clipboard::{self, ClipboardPayload, ClipboardWatcher, WatcherEvent};
use crate::flatten;
use crate::platform;
use crate::state::{AppUiFlags, EditorState, PendingImage, PendingImageSource, VisualEffectsMode};
use crate::theme;
use crate::toolbar;
use crate::ui_controls;

pub struct SnapMarkApp {
    pub state: EditorState,
    clipboard_watcher: ClipboardWatcher,
    ui_flags: AppUiFlags,
    open_editor_signal: Arc<AtomicBool>,
    hide_dock_signal: Arc<AtomicBool>,
    show_dock_signal: Arc<AtomicBool>,
    status_bar: Option<platform::StatusBarHandle>,
    theme: theme::AppTheme,
    allow_app_exit: bool,
}

impl SnapMarkApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let open_editor_signal = Arc::new(AtomicBool::new(false));
        let hide_dock_signal = Arc::new(AtomicBool::new(false));
        let show_dock_signal = Arc::new(AtomicBool::new(false));
        let theme = theme::premium_dark_theme();
        theme::apply_theme(&cc.egui_ctx, &theme);

        let mut state = EditorState::default();
        state.theme_density = theme.controls.global_spacing_scale;
        state.hover_intensity = theme.controls.hover_intensity;
        state.pressed_intensity = theme.controls.pressed_intensity;
        state.global_spacing_scale = theme.controls.global_spacing_scale;
        state.visual_effects.supports_vibrancy = platform::supports_vibrancy();
        state.visual_effects.mode = if state.visual_effects.supports_vibrancy {
            VisualEffectsMode::Hybrid
        } else {
            VisualEffectsMode::EguiOnly
        };
        platform::set_dock_icon_visible(state.settings.dock_icon_visible);

        Self {
            state,
            clipboard_watcher: ClipboardWatcher::new(300),
            ui_flags: AppUiFlags::default(),
            open_editor_signal,
            hide_dock_signal,
            show_dock_signal,
            status_bar: None,
            theme,
            allow_app_exit: false,
        }
    }

    fn ensure_status_bar(&mut self) {
        if self.status_bar.is_some() {
            return;
        }
        let open_signal = Arc::clone(&self.open_editor_signal);
        let hide_dock_signal = Arc::clone(&self.hide_dock_signal);
        let show_dock_signal = Arc::clone(&self.show_dock_signal);
        self.status_bar = platform::setup_status_bar(
            self.state.settings.dock_icon_visible,
            move || {
                open_signal.store(true, Ordering::Relaxed);
            },
            move || {
                hide_dock_signal.store(true, Ordering::Relaxed);
            },
            move || {
                show_dock_signal.store(true, Ordering::Relaxed);
            },
        );
    }

    fn sync_visual_effects(&mut self) {
        if !self.state.window_open {
            platform::remove_vibrancy();
            self.state.visual_effects.enabled = false;
            self.state.visual_effects.show_fallback_badge = false;
            return;
        }

        if self.state.visual_effects.mode == VisualEffectsMode::EguiOnly {
            platform::remove_vibrancy();
            self.state.visual_effects.enabled = false;
            self.state.visual_effects.show_fallback_badge =
                self.state.visual_effects.fallback_reason.is_some();
            return;
        }

        if !self.state.visual_effects.supports_vibrancy {
            self.state.visual_effects.mode = VisualEffectsMode::EguiOnly;
            self.state.visual_effects.enabled = false;
            self.state.visual_effects.fallback_reason =
                Some("vibrancy is not supported on this runtime".to_string());
            self.state.visual_effects.show_fallback_badge = true;
            return;
        }

        if let Err(err) = platform::install_vibrancy().and_then(|_| platform::update_vibrancy()) {
            self.state.visual_effects.mode = VisualEffectsMode::EguiOnly;
            self.state.visual_effects.enabled = false;
            self.state.visual_effects.fallback_reason = Some(err.to_string());
            self.state.visual_effects.show_fallback_badge = true;
            platform::remove_vibrancy();
            return;
        }

        self.state.visual_effects.enabled = true;
        self.state.visual_effects.fallback_reason = None;
        self.state.visual_effects.show_fallback_badge = false;
    }

    fn process_watcher_events(&mut self, ctx: &EguiContext) {
        while let Some(event) = self.clipboard_watcher.try_recv() {
            match event {
                WatcherEvent::ImageDetected(payload) => {
                    if self.state.image.is_none() || self.state.annotations.is_empty() {
                        self.load_image_into_editor(ctx, payload, false);
                    } else {
                        self.ui_flags.ask_replace_image = Some(PendingImage {
                            image: payload.image,
                            scale_factor: payload.scale_factor,
                            source: PendingImageSource::Watcher,
                        });
                    }
                }
                WatcherEvent::Error(message) => {
                    platform::show_alert("Screenshot Watcher Error", &message);
                }
            }
        }
    }

    fn process_open_editor_signal(&mut self, ctx: &EguiContext) {
        if !self.open_editor_signal.swap(false, Ordering::Relaxed) {
            return;
        }

        if self.state.image.is_none() {
            if let Ok(Some(payload)) = clipboard::read_image_from_clipboard() {
                self.load_image_into_editor(ctx, payload, false);
            }
        }

        self.state.window_open = true;
    }

    fn process_hide_dock_signal(&mut self) {
        if !self.hide_dock_signal.swap(false, Ordering::Relaxed) {
            return;
        }

        self.state.set_dock_icon_visible(false);
        platform::set_dock_icon_visible(false);
        self.status_bar = None;
    }

    fn process_show_dock_signal(&mut self) {
        if !self.show_dock_signal.swap(false, Ordering::Relaxed) {
            return;
        }

        self.state.set_dock_icon_visible(true);
        platform::set_dock_icon_visible(true);
        self.status_bar = None;
    }

    fn load_image_into_editor(
        &mut self,
        ctx: &EguiContext,
        payload: ClipboardPayload,
        clear_clipboard_after_load: bool,
    ) {
        self.state
            .reset_for_new_image(ctx, payload.image, payload.scale_factor);
        if clear_clipboard_after_load {
            platform::clear_clipboard();
        }
        self.state.window_open = true;
    }

    fn handle_shortcuts(&mut self, ctx: &EguiContext, _frame: &mut Frame) {
        let cmd = ctx.input(|input| input.modifiers.command || input.modifiers.ctrl);
        let shift = ctx.input(|input| input.modifiers.shift);

        if ctx.input(|input| input.key_pressed(Key::Escape)) {
            if self.state.text_edit.is_some() {
                self.state.text_edit = None;
            } else if self.state.active_tool != crate::annotation::Tool::Select {
                self.state.set_tool(crate::annotation::Tool::Select);
            } else if self.state.image.is_some() {
                self.request_close_editor();
            }
        }

        if !cmd {
            if ctx.input(|input| input.key_pressed(Key::V)) {
                self.state.set_tool(crate::annotation::Tool::Select);
            }
            if ctx.input(|input| input.key_pressed(Key::A)) {
                if shift {
                    self.state.set_tool(crate::annotation::Tool::ArrowWithText);
                } else {
                    self.state.set_tool(crate::annotation::Tool::Arrow);
                }
            }
            if ctx.input(|input| input.key_pressed(Key::T)) {
                self.state.set_tool(crate::annotation::Tool::Text);
            }
            if ctx.input(|input| input.key_pressed(Key::R)) {
                self.state.set_tool(crate::annotation::Tool::Rectangle);
            }
            if ctx.input(|input| input.key_pressed(Key::E)) {
                self.state.set_tool(crate::annotation::Tool::Ellipse);
            }

            if ctx
                .input(|input| input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace))
            {
                self.state.delete_selected();
            }

            return;
        }

        if ctx.input(|input| input.key_pressed(Key::C)) {
            if let Err(err) = self.copy_to_clipboard(ctx) {
                platform::show_alert("Copy failed", &format!("{err:#}"));
            }
        }

        if ctx.input(|input| input.key_pressed(Key::S)) {
            if let Err(err) = self.save_to_file() {
                platform::show_alert("Save failed", &format!("{err:#}"));
            }
        }

        if ctx.input(|input| input.key_pressed(Key::V)) {
            self.paste_image(ctx);
        }

        if ctx.input(|input| input.key_pressed(Key::Z)) {
            if shift {
                self.state.redo();
            } else {
                self.state.undo();
            }
        }

        if ctx.input(|input| input.key_pressed(Key::W)) {
            self.request_close_editor();
        }

        if ctx.input(|input| input.key_pressed(Key::Q)) {
            self.allow_app_exit = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if ctx.input(|input| input.key_pressed(Key::Plus) || input.key_pressed(Key::Equals)) {
            self.state.zoom_in();
        }

        if ctx.input(|input| input.key_pressed(Key::Minus)) {
            self.state.zoom_out();
        }

        if ctx.input(|input| input.key_pressed(Key::Num0)) {
            self.state.fit_zoom_to_view = true;
        }
    }

    fn handle_window_close_request(&mut self, ctx: &EguiContext) {
        if !ctx.input(|input| input.viewport().close_requested()) {
            return;
        }

        if self.allow_app_exit {
            return;
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        self.request_close_editor();
    }

    fn paste_image(&mut self, ctx: &EguiContext) {
        match clipboard::read_image_from_clipboard() {
            Ok(Some(payload)) => {
                if self.state.image.is_some() && !self.state.annotations.is_empty() {
                    self.ui_flags.ask_replace_from_paste = Some(PendingImage {
                        image: payload.image,
                        scale_factor: payload.scale_factor,
                        source: PendingImageSource::Paste,
                    });
                } else {
                    self.load_image_into_editor(ctx, payload, false);
                }
            }
            Ok(None) => {}
            Err(err) => {
                platform::show_alert("Paste Error", &format!("Cannot paste image: {err:#}"))
            }
        }
    }

    fn request_close_editor(&mut self) {
        self.close_editor();
    }

    fn close_editor(&mut self) {
        self.state.image = None;
        self.state.annotations.clear();
        self.state.history.clear_with(Vec::new());
        self.state.selection = None;
        self.state.drag_state = None;
        self.state.text_edit = None;
        self.state.window_open = false;
        self.state.exported = false;
        self.state.has_edited = false;
        platform::remove_vibrancy();
        platform::close_native_color_panel();
        self.state.visual_effects.enabled = false;
    }

    fn copy_to_clipboard(&mut self, ctx: &EguiContext) -> Result<()> {
        let Some(image) = self.state.image.as_ref() else {
            return Ok(());
        };

        let flattened =
            flatten::flatten(&image.dynamic, &self.state.annotations, image.scale_factor)
                .context("flatten failed")?;
        let png = flatten::encode_png(&flattened)?;
        clipboard::write_png_to_clipboard(&png)?;

        self.state.exported = true;
        self.ui_flags.copy_feedback_until = Some(ctx.input(|input| input.time) + 1.5);
        Ok(())
    }

    fn save_to_file(&mut self) -> Result<()> {
        let Some(image) = self.state.image.as_ref() else {
            return Ok(());
        };

        let default_name = format!("Screenshot {}", Local::now().format("%Y-%m-%d at %H.%M.%S"));

        let file = rfd::FileDialog::new()
            .set_title("Save annotated screenshot")
            .set_file_name(&default_name)
            .add_filter("PNG", &["png"])
            .add_filter("JPEG", &["jpg", "jpeg"])
            .save_file();

        let Some(path) = file else {
            return Ok(());
        };

        let flattened =
            flatten::flatten(&image.dynamic, &self.state.annotations, image.scale_factor)
                .context("flatten failed")?;

        let ext = path
            .extension()
            .and_then(|item| item.to_str())
            .unwrap_or("png")
            .to_ascii_lowercase();

        if ext == "jpg" || ext == "jpeg" {
            flattened
                .to_rgb8()
                .save_with_format(&path, ImageFormat::Jpeg)
                .with_context(|| format!("cannot save jpeg to {}", path.display()))?;
        } else {
            flattened
                .save_with_format(&path, ImageFormat::Png)
                .with_context(|| format!("cannot save png to {}", path.display()))?;
        }

        self.state.exported = true;
        platform::show_saved_notification(&path.display().to_string());
        Ok(())
    }

    fn show_replace_dialog(
        ctx: &EguiContext,
        app_theme: &theme::AppTheme,
        title: &str,
        message: &str,
        pending: &mut Option<PendingImage>,
    ) -> Option<PendingImage> {
        let mut out = None;

        if pending.is_none() {
            return out;
        }

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .frame(ui_controls::card_frame(app_theme))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(message)
                        .color(app_theme.text.secondary)
                        .size(15.0),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui_controls::primary_button(
                        ui,
                        app_theme,
                        "Replace",
                        egui::vec2(116.0, 34.0),
                    )
                    .clicked()
                    {
                        out = pending.take();
                    }
                    if ui_controls::ghost_button(
                        ui,
                        app_theme,
                        "Keep Current",
                        egui::vec2(128.0, 34.0),
                    )
                    .clicked()
                    {
                        *pending = None;
                    }
                });
            });

        out
    }
}

impl Drop for SnapMarkApp {
    fn drop(&mut self) {
        platform::remove_vibrancy();
        platform::close_native_color_panel();
    }
}

impl App for SnapMarkApp {
    fn update(&mut self, ctx: &EguiContext, frame: &mut Frame) {
        theme::apply_theme(ctx, &self.theme);
        self.ensure_status_bar();
        self.process_watcher_events(ctx);
        self.process_open_editor_signal(ctx);
        self.process_hide_dock_signal();
        self.process_show_dock_signal();
        self.ensure_status_bar();
        self.handle_shortcuts(ctx, frame);
        self.handle_window_close_request(ctx);
        self.sync_visual_effects();
        if let Some(rgba) = platform::poll_native_color_panel_color() {
            self.state.set_color(rgba);
        }

        if !self.state.window_open {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            ctx.request_repaint_after(std::time::Duration::from_millis(
                self.theme.motion.slow_ms as u64,
            ));
            return;
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            egui::WindowLevel::AlwaysOnTop,
        ));

        if let Some(payload) = Self::show_replace_dialog(
            ctx,
            &self.theme,
            "New screenshot detected",
            "Replace current image? Existing annotations will be lost.",
            &mut self.ui_flags.ask_replace_image,
        ) {
            let clear = payload.source == PendingImageSource::Watcher;
            self.load_image_into_editor(
                ctx,
                ClipboardPayload {
                    image: payload.image,
                    scale_factor: payload.scale_factor,
                },
                clear,
            );
        }

        if let Some(payload) = Self::show_replace_dialog(
            ctx,
            &self.theme,
            "Replace current image",
            "Replace current image from clipboard?",
            &mut self.ui_flags.ask_replace_from_paste,
        ) {
            self.load_image_into_editor(
                ctx,
                ClipboardPayload {
                    image: payload.image,
                    scale_factor: payload.scale_factor,
                },
                false,
            );
        }

        TopBottomPanel::top("toolbar")
            .exact_height(self.theme.layout.toolbar_height)
            .frame(ui_controls::toolbar_frame(&self.theme))
            .show(ctx, |ui| {
                let width_class = self.theme.width_class(ui.available_width());
                toolbar::show_toolbar(ui, &mut self.state, width_class);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.theme.surfaces.app_bg)
                    .inner_margin(egui::Margin::symmetric(
                        self.theme.layout.panel_padding_x,
                        self.theme.layout.panel_padding_y + 2.0,
                    )),
            )
            .show(ctx, |ui| {
                canvas::show_canvas(ui, ctx, &mut self.state);
            });

        let copied_feedback = self
            .ui_flags
            .copy_feedback_until
            .is_some_and(|deadline| ctx.input(|input| input.time) <= deadline);

        let action_output = TopBottomPanel::bottom("action_bar")
            .exact_height(self.theme.layout.action_bar_height)
            .frame(ui_controls::action_bar_frame(&self.theme))
            .show(ctx, |ui| {
                let width_class = self.theme.width_class(ui.available_width());
                action_bar::show_action_bar(ui, &self.state, copied_feedback, width_class)
            })
            .inner;

        if action_output.undo {
            self.state.undo();
        }
        if action_output.redo {
            self.state.redo();
        }
        if action_output.copy {
            match self.copy_to_clipboard(ctx) {
                Ok(()) => self.close_editor(),
                Err(err) => {
                    platform::show_alert("Copy failed", &format!("{err:#}"));
                }
            }
        }
        if action_output.save {
            if let Err(err) = self.save_to_file() {
                platform::show_alert("Save failed", &format!("{err:#}"));
            }
        }

        platform::elevate_window(frame);
        ctx.request_repaint_after(std::time::Duration::from_millis(
            self.theme.motion.fast_ms as u64,
        ));
    }
}
