# SnapMark

SnapMark is a macOS screenshot annotation tool built with Rust + egui.

## Implemented

- Menu-bar oriented app architecture with dedicated platform module (`src/platform/*`)
- Native macOS status bar menu (`Open Editor`, `Quit SnapMark`)
- Screenshot watcher loop (300ms polling) with automatic detection of new macOS screenshots
- Editor layout: toolbar + canvas + action bar
- Premium dark/glass theme tokens + styled controls
- Hidden-by-default editor window for menu-bar workflow
- Tools: Select, Arrow, Arrow+Text, Text, Rectangle, Ellipse
- Palette + native macOS `NSColorPanel` (`+`) with fallback picker + stroke presets + text size presets
- Selection, move, resize handles, delete
- Undo / Redo snapshot history
- Zoom in / out / fit-to-view
- Flatten (tiny-skia) + text rendering pass (imageproc)
- Copy to clipboard and Save (PNG / JPEG)
- Unsaved-close confirmation, replace-image confirmation dialogs
- Hybrid visual effects mode (`NSVisualEffectView`), with automatic fallback to pure egui (`EguiOnly`) on failure
- App metadata files (`Info.plist`, `build.rs`, icon placeholder)

## Build

Requires Rust toolchain and Cargo installed.

```bash
cargo check
cargo test
cargo run
```

Universal macOS `.app` build:

```bash
./scripts/build_macos_app.sh
```

Output bundle:

- `build/SnapMark.app`

Release DMG build:

```bash
./scripts/build_release_dmg.sh
```

Output artifact:

- `build/SnapMark-<version>.dmg`

Homebrew cask generation for your own tap:

```bash
./scripts/generate_cask.sh --repo <github-user-or-org>/snapmark --dmg build/SnapMark-<version>.dmg
```

Detailed steps:

- `docs/HOMEBREW_CASK_RELEASE.md`

Implementation tracking:

- `docs/IMPLEMENTATION_STATUS.md`

## Project structure

Matches PRD target layout:

- `src/main.rs`
- `src/app.rs`
- `src/state.rs`
- `src/clipboard.rs`
- `src/canvas.rs`
- `src/toolbar.rs`
- `src/action_bar.rs`
- `src/flatten.rs`
- `src/annotation.rs`
- `src/history.rs`
- `src/platform/mod.rs`
- `src/platform/macos.rs`
