use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::Hasher,
    path::{Path, PathBuf},
    process::Command,
    time::{Instant, SystemTime},
};

use anyhow::{anyhow, Context, Result};
use arboard::Clipboard;
use image::{DynamicImage, RgbaImage};

use crate::platform;

const SCREENSHOT_SIGNAL_GRACE: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub struct ClipboardPayload {
    pub image: DynamicImage,
    pub scale_factor: f32,
}

pub enum WatcherEvent {
    ImageDetected(ClipboardPayload),
    Error(String),
}

pub struct ClipboardWatcher {
    rx: Receiver<WatcherEvent>,
    stop: Arc<AtomicBool>,
    _worker: thread::JoinHandle<()>,
}

impl ClipboardWatcher {
    pub fn new(interval_ms: u64) -> Self {
        let (tx, rx) = mpsc::channel::<WatcherEvent>();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);

        let worker = thread::spawn(move || {
            if let Err(err) = watcher_loop(tx, stop_flag, interval_ms) {
                eprintln!("clipboard watcher failed: {err:#}");
            }
        });

        Self {
            rx,
            stop,
            _worker: worker,
        }
    }

    pub fn try_recv(&self) -> Option<WatcherEvent> {
        self.rx.try_recv().ok()
    }
}

impl Drop for ClipboardWatcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn watcher_loop(tx: Sender<WatcherEvent>, stop: Arc<AtomicBool>, interval_ms: u64) -> Result<()> {
    let mut state = ScreenshotPollState::new();
    let mut clipboard = Clipboard::new().ok();
    let mut last_error: Option<String> = None;

    while !stop.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(interval_ms));

        state.observe_screencapture_signal();

        match state.poll_new_screenshot_path() {
            Ok(Some(path)) => match read_image_from_path(&path) {
                Ok(payload) => {
                    if !state.should_emit_payload(&payload) {
                        continue;
                    }
                    last_error = None;
                    if tx.send(WatcherEvent::ImageDetected(payload)).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    let message = format!(
                        "screenshot detected but failed to read {}: {err:#}",
                        path.display()
                    );
                    if last_error.as_deref() != Some(message.as_str()) {
                        let _ = tx.send(WatcherEvent::Error(message.clone()));
                        last_error = Some(message);
                    }
                }
            },
            Ok(None) => {
                last_error = None;
            }
            Err(err) => {
                let message = format!("screenshot watcher error: {err:#}");
                if last_error.as_deref() != Some(message.as_str()) {
                    let _ = tx.send(WatcherEvent::Error(message.clone()));
                    last_error = Some(message);
                }
            }
        }

        if !state.is_screenshot_signal_active() {
            continue;
        }

        if clipboard.is_none() {
            clipboard = Clipboard::new().ok();
        }

        let Some(clipboard) = clipboard.as_mut() else {
            continue;
        };

        match state.poll_clipboard_screenshot(clipboard) {
            Ok(Some(payload)) => {
                last_error = None;
                if tx.send(WatcherEvent::ImageDetected(payload)).is_err() {
                    break;
                }
            }
            Ok(None) => {}
            Err(err) => {
                let message = format!("clipboard screenshot read failed: {err:#}");
                if last_error.as_deref() != Some(message.as_str()) {
                    let _ = tx.send(WatcherEvent::Error(message.clone()));
                    last_error = Some(message);
                }
            }
        }
    }

    Ok(())
}

pub fn read_image_from_clipboard() -> Result<Option<ClipboardPayload>> {
    let mut clipboard = Clipboard::new().context("cannot initialize clipboard")?;
    read_image_from_clipboard_inner(&mut clipboard)
}

fn read_image_from_clipboard_inner(clipboard: &mut Clipboard) -> Result<Option<ClipboardPayload>> {
    let image = match clipboard.get_image() {
        Ok(data) => data,
        Err(_) => return Ok(None),
    };

    let width = image.width as u32;
    let height = image.height as u32;
    let bytes = image.bytes.into_owned();

    let rgba = RgbaImage::from_raw(width, height, bytes)
        .ok_or_else(|| anyhow!("clipboard image has invalid shape"))?;

    Ok(Some(ClipboardPayload {
        image: DynamicImage::ImageRgba8(rgba),
        scale_factor: platform::active_screen_scale_factor().unwrap_or(1.0),
    }))
}

pub fn write_png_to_clipboard(png_bytes: &[u8]) -> Result<()> {
    let mut clipboard = Clipboard::new().context("cannot initialize clipboard")?;
    let img = image::load_from_memory(png_bytes).context("cannot decode png for clipboard")?;
    let rgba = img.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;
    clipboard
        .set_image(arboard::ImageData {
            width,
            height,
            bytes: std::borrow::Cow::Owned(rgba.into_raw()),
        })
        .context("cannot write image to clipboard")
}

struct ScreenshotPollState {
    screenshot_dir: PathBuf,
    last_seen: Option<(SystemTime, PathBuf)>,
    last_clipboard_change_count: Option<i64>,
    last_emitted_fingerprint: Option<u64>,
    screencapture_running: bool,
    screenshot_signal_until: Option<Instant>,
}

impl ScreenshotPollState {
    fn new() -> Self {
        let screenshot_dir = resolve_screenshot_dir();
        let last_seen = latest_screenshot_file(&screenshot_dir, true).ok().flatten();
        Self {
            screenshot_dir,
            last_seen,
            last_clipboard_change_count: platform::clipboard_change_count(),
            last_emitted_fingerprint: None,
            screencapture_running: false,
            screenshot_signal_until: None,
        }
    }

    fn observe_screencapture_signal(&mut self) {
        let now = Instant::now();
        let running = is_screencapture_running();

        if running {
            self.screencapture_running = true;
            self.screenshot_signal_until = Some(now + SCREENSHOT_SIGNAL_GRACE);
            return;
        }

        if self.screencapture_running {
            self.screencapture_running = false;
            self.screenshot_signal_until = Some(now + SCREENSHOT_SIGNAL_GRACE);
            return;
        }

        if let Some(until) = self.screenshot_signal_until {
            if now > until {
                self.screenshot_signal_until = None;
            }
        }
    }

    fn is_screenshot_signal_active(&self) -> bool {
        self.screencapture_running
            || self
                .screenshot_signal_until
                .map(|until| Instant::now() <= until)
                .unwrap_or(false)
    }

    fn poll_new_screenshot_path(&mut self) -> Result<Option<PathBuf>> {
        let allow_loose_match = self.is_screenshot_signal_active();
        let Some((modified, path)) =
            latest_screenshot_file(&self.screenshot_dir, allow_loose_match)?
        else {
            return Ok(None);
        };

        let changed = self
            .last_seen
            .as_ref()
            .map(|(seen_modified, seen_path)| {
                modified > *seen_modified || (modified == *seen_modified && path != *seen_path)
            })
            .unwrap_or(true);

        if !changed {
            return Ok(None);
        }

        self.last_seen = Some((modified, path.clone()));
        Ok(Some(path))
    }

    fn should_emit_payload(&mut self, payload: &ClipboardPayload) -> bool {
        let fingerprint = image_fingerprint(&payload.image);
        if self.last_emitted_fingerprint == Some(fingerprint) {
            return false;
        }
        self.last_emitted_fingerprint = Some(fingerprint);
        true
    }

    fn poll_clipboard_screenshot(
        &mut self,
        clipboard: &mut Clipboard,
    ) -> Result<Option<ClipboardPayload>> {
        if let Some(current_count) = platform::clipboard_change_count() {
            if self.last_clipboard_change_count == Some(current_count) {
                return Ok(None);
            }
            self.last_clipboard_change_count = Some(current_count);
        }

        let Some(payload) = read_image_from_clipboard_inner(clipboard)? else {
            return Ok(None);
        };

        if !self.should_emit_payload(&payload) {
            return Ok(None);
        }

        Ok(Some(payload))
    }
}

fn read_image_from_path(path: &Path) -> Result<ClipboardPayload> {
    let image = image::open(path)
        .with_context(|| format!("cannot decode screenshot {}", path.display()))?;
    Ok(ClipboardPayload {
        image,
        scale_factor: platform::active_screen_scale_factor().unwrap_or(1.0),
    })
}

fn resolve_screenshot_dir() -> PathBuf {
    if let Ok(dir) = env::var("SNAPMARK_SCREENSHOT_DIR") {
        let from_env = expand_user_path(dir.trim());
        if from_env.is_dir() {
            return from_env;
        }
    }

    if let Ok(output) = Command::new("defaults")
        .args(["read", "com.apple.screencapture", "location"])
        .output()
    {
        if output.status.success() {
            let location = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !location.is_empty() {
                let path = expand_user_path(&location);
                if path.is_dir() {
                    return path;
                }
            }
        }
    }

    if let Some(home) = env::var_os("HOME") {
        let desktop = PathBuf::from(home).join("Desktop");
        if desktop.is_dir() {
            return desktop;
        }
    }

    PathBuf::from(".")
}

fn latest_screenshot_file(
    dir: &Path,
    allow_loose_match: bool,
) -> Result<Option<(SystemTime, PathBuf)>> {
    let mut latest: Option<(SystemTime, PathBuf)> = None;

    for entry in fs::read_dir(dir).with_context(|| format!("cannot read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !is_image_file(&path) {
            continue;
        }
        if !allow_loose_match && !is_screenshot_file_name(&path) {
            continue;
        }
        let modified = entry
            .metadata()
            .with_context(|| format!("cannot read metadata {}", path.display()))?
            .modified()
            .with_context(|| format!("cannot read mtime {}", path.display()))?;

        match &latest {
            Some((current_modified, current_path)) => {
                if modified > *current_modified
                    || (modified == *current_modified && path != *current_path)
                {
                    latest = Some((modified, path));
                }
            }
            None => latest = Some((modified, path)),
        }
    }

    Ok(latest)
}

fn expand_user_path(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home);
        }
    } else if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "tiff"
            )
        })
        .unwrap_or(false)
}

fn is_screenshot_file_name(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    let screenshot_prefixes = [
        "Screenshot",
        "Screen Shot",
        "Снимок экрана",
        "スクリーンショット",
        "Captura de pantalla",
        "Bildschirmfoto",
    ];

    screenshot_prefixes
        .iter()
        .any(|prefix| file_name.starts_with(prefix))
}

fn image_fingerprint(image: &DynamicImage) -> u64 {
    let rgba = image.to_rgba8();
    let mut hasher = DefaultHasher::new();
    hasher.write_u32(rgba.width());
    hasher.write_u32(rgba.height());
    hasher.write(rgba.as_raw());
    hasher.finish()
}

fn is_screencapture_running() -> bool {
    Command::new("pgrep")
        .args(["-x", "screencapture"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
