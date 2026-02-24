# SnapMark

SnapMark is a fast annotation layer for screenshots on macOS.

It is explicitly designed as an extension of the native macOS Screenshot Tool, not a replacement.  
You keep using macOS capture shortcuts (`Cmd+Shift+4`, `Cmd+Shift+5`), and SnapMark picks up the result so you can mark it up immediately.

## Why SnapMark

- Built for quick communication: arrows, text, and shapes in seconds
- Lives in the menu bar and stays out of your way
- Keeps the workflow native to macOS
- Optimized for sharing: copy to clipboard or save to file

## Core Workflow

1. Take a screenshot with the native macOS Screenshot Tool.
2. SnapMark intercepts the screenshot from the clipboard.
3. Annotate and export only what you actually need.

## Clipboard-First, No Auto-Save

SnapMark works in a clipboard-first mode: it captures screenshots from your clipboard and does not automatically save every capture to disk.

This helps you:

- save disk space,
- avoid clutter from one-off screenshots,
- keep only the screenshots that are worth exporting.

## Features

- Tools: Select, Arrow, Arrow with Text, Text, Rectangle, Ellipse
- Fixed high-contrast annotation palette
- Stroke size and text size controls
- Undo/Redo
- Copy result directly to clipboard
- Save as PNG or JPEG

## Install (Homebrew)

```bash
brew tap svishniakov/snapmark
brew install --cask snapmark
```

## Manual Install

Download the latest `.dmg` from Releases and move `SnapMark.app` to Applications.

## Privacy

SnapMark works locally on your Mac and does not require a cloud account.

## For Contributors

```bash
cargo test
./scripts/build_macos_app.sh
```
