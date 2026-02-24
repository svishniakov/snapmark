# SnapMark Implementation Status

Дата: 2026-02-24

## Реализовано

- Проектная структура и модули из PRD.
- Основной editor UI: toolbar, canvas, action bar (PRD layout сохранён).
- Полный premium restyle UI под dark/glass направление:
  - добавлен модуль токенов темы `src/theme.rs` (`AppTheme`, `SurfaceTokens`, `TextTokens`, `ControlTokens`, `ShadowTokens`, `MotionTokens`);
  - добавлен набор стилизованных контролов `src/ui_controls.rs` (карточки, chip/segmented, primary/ghost/danger buttons, badge);
  - переработаны `toolbar.rs`, `action_bar.rs`, `canvas.rs`, `app.rs` под единый визуальный язык.
- Инструменты: Select, Arrow, Arrow+Text, Text, Rectangle, Ellipse.
- Цвета (8 пресетов + нативный macOS `NSColorPanel` по кнопке `+`, с fallback на egui picker вне macOS), stroke presets, text size presets.
- Выделение, перемещение, resize-handle взаимодействия, удаление.
- Undo/Redo snapshot history.
- Zoom hotkeys и fit-to-view.
- Screenshot watcher (polling 300ms), автообнаружение новых macOS screenshot-файлов в системной папке скриншотов.
- Окно скрыто по умолчанию (menu-bar mode), показывается при событии открытия редактора.
- Нативный `NSStatusBar` item с меню: `Open Editor`, `Quit SnapMark`.
- Диалоги: replace current image, unsaved annotations on close.
- Flatten через tiny-skia + text pass.
- Copy в clipboard, Save (PNG/JPEG).
- Нативные macOS API для `NSPasteboard::changeCount`, `clearContents`, `NSScreen::backingScaleFactor`, `NSAlert`.
- Реализован `hybrid` visual effects слой:
  - `src/platform/vibrancy_macos.rs` добавляет `NSVisualEffectView` на активное окно;
  - `platform` API расширен: `supports_vibrancy()`, `install_vibrancy()`, `update_vibrancy()`, `remove_vibrancy()`;
  - добавлен runtime fallback на `EguiOnly` с индикатором `blur fallback`, без падения приложения.
- Добавлен нативный color panel bridge:
  - `supports_native_color_panel()`, `open_native_color_panel()`, `poll_native_color_panel_color()`, `close_native_color_panel()`;
  - синхронизация выбранного цвета из `NSColorPanel` в активный цвет тулбара.
- Floating-уровень окна через `egui::ViewportCommand::WindowLevel(AlwaysOnTop)`.
- Конфигурация приложения: Info.plist, build.rs, app build script.
- Universal `.app` bundle (`arm64 + x86_64`) собирается скриптом.
- Совместимость с `rustc 1.93.x` в dev-режиме: отключены runtime signature assertions в `objc2/icrate` для `cargo run`, чтобы убрать падение `invalid message send ... countByEnumeratingWithState`.
- Проверки пройдены:
  - `cargo check`
  - `cargo test` (5/5 passed)
  - smoke run (`cargo run`)
  - `./scripts/build_macos_app.sh`

## Частично / технический долг

- Точная «pixel-match» под конкретный референс не заявляется; реализован перенос визуального языка в рамках текущего eframe/egui стека.
