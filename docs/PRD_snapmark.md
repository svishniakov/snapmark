# SnapMark — Screenshot Annotation Tool for macOS
## Product Requirements Document

| | |
|---|---|
| **Версия** | 1.0 — Draft |
| **Дата** | Февраль 2026 |
| **Платформа** | macOS 13 Ventura и выше |
| **Стек** | Rust · egui/eframe · arboard · tiny-skia · rfd · objc2 |
| **Режим запуска** | Menu Bar App (постоянно в фоне, иконка в статус-баре) |

---

## Содержание

1. [Цель и контекст](#1-цель-и-контекст)
2. [Пользовательский сценарий](#2-пользовательский-сценарий)
3. [Функциональные требования](#3-функциональные-требования)
4. [Технические требования](#4-технические-требования)
5. [UI / UX Спецификация](#5-ui--ux-спецификация)
6. [Нефункциональные требования](#6-нефункциональные-требования)
7. [План разработки](#7-план-разработки)
8. [Out of Scope для v1](#8-out-of-scope-для-v1)
9. [Открытые вопросы](#9-открытые-вопросы)

---

## 1. Цель и контекст

### 1.1 Проблема

Стандартный workflow при аннотировании скриншотов на macOS состоит минимум из 5 шагов: скриншот → открыть Preview → перейти в Markup → нарисовать → скопировать. Это разрывает рабочий поток. Сторонние инструменты (Skitch, Annotate Pro, CleanShot X) либо требуют подписки, либо перегружены функциями, либо не интегрируются с нативным clipboard-flow macOS.

### 1.2 Решение

SnapMark — минималистичное нативное приложение, которое живёт в Menu Bar и автоматически перехватывает скриншот из буфера обмена. Пользователь добавляет аннотации в одном окне и одним кликом возвращает результат в clipboard или сохраняет файл. Весь процесс занимает менее 10 секунд.

### 1.3 Ключевые метрики успеха

| Метрика | Целевое значение |
|---|---|
| Время от скриншота до Copy | ≤ 10 секунд для типичной аннотации |
| Время появления окна редактора | ≤ 300 мс после обнаружения изображения в clipboard |
| Размер бинарника (.app bundle) | ≤ 20 МБ |
| RAM в фоновом режиме (Menu Bar) | ≤ 25 МБ |
| CPU при polling clipboard | ≤ 0.1% (интервал 300 мс) |

---

## 2. Пользовательский сценарий

### 2.1 Основной flow (Happy Path)

1. Пользователь нажимает `Cmd+Ctrl+Shift+4` (или любую комбинацию для скриншота в clipboard) — macOS кладёт PNG в системный буфер обмена
2. SnapMark (работает в фоне) обнаруживает изображение в clipboard в течение 300 мс
3. Буфер обмена немедленно очищается — изображение «забирается» в приложение
4. Окно редактора появляется по центру активного экрана с загруженным скриншотом
5. Пользователь выбирает инструмент на тулбаре (стрелка, текст, прямоугольник и т.д.)
6. Рисует аннотации на канвасе. Использует `Cmd+Z` для отмены при необходимости
7. Нажимает кнопку **Copy** (`Cmd+C`) — изображение с аннотациями копируется в clipboard
8. Закрывает окно (`Cmd+W` / `Escape`) — приложение возвращается в фоновый режим

### 2.2 Альтернативные сценарии

#### Ручное открытие
- Клик по иконке Menu Bar → **Open Editor**
- Если clipboard содержит изображение — загружается автоматически
- Если clipboard пуст — открывается редактор с подсказкой *«Вставьте изображение (Cmd+V)»*

#### Вставка через Cmd+V
- Если окно редактора уже открыто и пусто — `Cmd+V` вставляет изображение из clipboard
- Если в редакторе уже есть изображение — диалог: **«Replace current image?»** → Replace / Cancel

#### Сохранение в файл
- Кнопка **Save** (`Cmd+S`) открывает нативный `NSSavePanel`
- Формат по умолчанию: PNG. Пользователь может выбрать расширение в диалоге (PNG / JPEG)
- После сохранения окно остаётся открытым — пользователь может также нажать Copy

#### Новый скриншот пока редактор открыт
- Если в редакторе **нет аннотаций** — изображение заменяется автоматически
- Если аннотации есть — диалог: **«New screenshot detected. Replace current image? (аннотации будут потеряны)»** → Replace / Keep Current

#### Закрытие с несохранёнными аннотациями
- Если пользователь нажимает `Cmd+W` / `Escape` и есть нарисованные (но не экспортированные) аннотации — диалог: **«Unsaved annotations. Copy to clipboard before closing?»** → Copy & Close / Close Without Saving / Cancel

---

## 3. Функциональные требования

### 3.1 Menu Bar App

| Параметр | Требование |
|---|---|
| Иконка | Монохромная SVG-иконка 18×18pt (адаптируется к светлой / тёмной теме) |
| Контекстное меню | Open Editor · *(separator)* · Quit SnapMark |
| Dock | Иконка **не** показывается (`LSUIElement = true` в Info.plist) |
| Активация окна | Окно редактора появляется поверх всех окон (`NSWindowLevel.floating`) |
| Polling clipboard | Каждые 300 мс. Проверять `changeCount` у `NSPasteboard` — не читать данные без изменения |
| Автозапуск | Опционально (v2): регистрация как LaunchAgent через `SMLoginItemSetEnabled` |

### 3.2 Окно редактора

#### Общее
- Тип окна: стандартное `NSWindow` с тайтл-баром (кнопки Close / Minimize / Zoom)
- Размер: адаптивный. `width = min(screenshot_width + 32px padding, 90% экрана)`. Аналогично по высоте
- Минимальный размер: 640 × 480 px
- Заголовок окна: `«SnapMark»`
- При открытии: центрируется на экране, где последний раз был курсор
- **Retina / HiDPI:** канвас и экспорт работают в оригинальном 2× разрешении (pixel-perfect). На экране — в логических пикселях

#### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│   Toolbar (48px)                                                │
│   [↖] [→] [→T] [T] [□] [○]  │  ● ● ● ● ● ● ● ●  [+]  ━ ━━ ━━━ │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                                                                 │
│                  Canvas (flex)                                  │
│                                                                 │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│   Action Bar (44px)                                             │
│   [↩ Undo]  [↪ Redo]                    [ Copy ]    [ Save ]   │
└─────────────────────────────────────────────────────────────────┘
```

| Зона | Описание |
|---|---|
| **Toolbar** | Горизонтальная панель вверху. Высота 48px. Инструменты · разделитель · палитра · разделитель · толщина |
| **Canvas** | Занимает всё оставшееся пространство. Скроллируется если изображение больше видимой области. Поддерживает Cmd+= / Cmd+- / Cmd+0 |
| **Action Bar** | Горизонтальная панель внизу. Высота 44px. Undo · Redo · Spacer · Copy · Save |

---

### 3.3 Toolbar — Инструменты

| Инструмент | Hotkey | Описание поведения |
|---|---|---|
| **Select** (курсор) | `V` или `Escape` | Выбор, перемещение и ресайз нарисованных элементов. Активируется автоматически после завершения рисования |
| **Arrow** (стрелка) | `A` | Drag от хвоста к острию. Линия с заливным треугольным наконечником на конце drag'а |
| **Arrow + Text** | `Shift+A` | То же что Arrow, но после отпускания мыши появляется inline text field у **хвоста** стрелки. `Enter` или клик вне поля — завершает ввод |
| **Text** | `T` | Клик на канвасе — inline text field. Шрифт: SF Pro, размер S/M/L (14/18/24pt). `Cmd+Enter` — завершить |
| **Rectangle** | `R` | Drag → прямоугольник без заливки (только stroke). `Shift+Drag` → квадрат |
| **Ellipse / Oval** | `E` | Drag → овал без заливки. `Shift+Drag` → окружность |

#### Цветовая палитра

8 предустановленных цветов в тулбаре (кружки 20px, активный обведён кольцом):

| Цвет | HEX |
|---|---|
| Красный | `#E53E3E` |
| Оранжевый | `#DD6B20` |
| Жёлтый | `#D69E2E` |
| Зелёный | `#38A169` |
| Синий | `#3182CE` |
| Фиолетовый | `#805AD5` |
| Белый | `#FFFFFF` |
| Чёрный | `#1A202C` |

Кнопка **«+»** открывает нативный `NSColorPanel` для произвольного цвета.

Выбранный цвет применяется ко всем **новым** элементам. К существующим — после выбора через Select tool.

#### Толщина линий

3 пресета (активный подсвечен):

| Пресет | Значение |
|---|---|
| Тонкая | 1.5 px |
| Средняя | 3.0 px |
| Толстая | 5.0 px |

---

### 3.4 Canvas — Поведение

#### Отображение
- Скриншот отображается по центру канваса на нейтральном фоне
- Если изображение меньше канваса — 1:1 с отступом
- Если больше — масштабируется fit-to-view при открытии (с сохранением aspect ratio)
- **Zoom:** `Cmd+=` / `Cmd+-` по шагам (25%, 33%, 50%, 67%, 75%, 100%, 150%, 200%), `Cmd+0` — fit-to-view
- При Zoom > 100%: канвас скроллируется (колесо мыши / два пальца на трекпаде)

#### Рисование
- Все координаты хранятся в пространстве изображения (не экрана) — инвариантны к зуму
- Preview элемента отображается в реальном времени во время drag
- После завершения рисования — автоматически активируется Select tool
- Минимальный размер элемента для создания: 5×5 логических px изображения (меньше — не создаётся)

#### Select Tool
- **Клик** на элемент — выбирает (синий bounding box с 8 ручками)
- **Клик** на пустое место — снимает выбор
- **Drag** за элемент — перемещает
- **Drag** за ручку — ресайз. Ручки: 4 угла + 4 середины сторон. Для стрелки: ручки на хвосте и острие
- `Delete` / `Backspace` — удалить выбранный элемент
- **Double-click** на Arrow+Text или Text — войти в режим редактирования текста

---

### 3.5 Undo / Redo

- `Cmd+Z` — отменить последнее действие
- `Cmd+Shift+Z` — повторить
- Кнопки Undo / Redo в Action Bar
- История: **неограниченная** в рамках сессии (пока окно открыто)
- В историю записывается: добавление элемента, удаление, перемещение, ресайз, изменение текста, изменение цвета/толщины
- История очищается при загрузке нового изображения

---

### 3.6 Action Bar — Экспорт

#### Кнопка Copy (`Cmd+C`)
- Выполняет **flatten**: рендерит финальное изображение (скриншот + аннотации) в памяти через `tiny-skia`
- Помещает результат в `NSPasteboard` как PNG (`com.apple.pict` / `NSPasteboardTypePNG`)
- Разрешение: **оригинальное** разрешение скриншота (Retina 2× если снят на Retina)
- После копирования: кнопка кратко показывает текст *«Copied!»* (1.5 сек)

#### Кнопка Save (`Cmd+S`)
- Открывает нативный `NSSavePanel` (через `rfd::AsyncFileDialog`)
- Форматы: **PNG** (по умолчанию), **JPEG**
- Имя по умолчанию: `Screenshot YYYY-MM-DD at HH.MM.SS`
- После сохранения: нативное уведомление macOS `«Saved to ~/Desktop/...»`

---

## 4. Технические требования

### 4.1 Стек технологий

| Компонент | Крейт / Инструмент | Обоснование |
|---|---|---|
| UI Framework | `egui + eframe 0.27` | Кастомный canvas-рендер, активно поддерживается, хорошая интеграция с macOS Metal |
| macOS интеграция | `objc2 + objc2-app-kit` | Нативный NSStatusBar, NSColorPanel, NSPasteboard changeCount, HiDPI scale factor |
| Clipboard R/W | `arboard 3.3` | Поддерживает PNG/RGBA в NSPasteboard, кросс-платформенный |
| Flatten / Render | `tiny-skia 0.11` | CPU-рендер аннотаций поверх bitmap, нет зависимости от GPU |
| Save диалог | `rfd 0.14` | Нативный NSSavePanel через Rust |
| Работа с изображениями | `image 0.25` | Декодирование PNG/TIFF из clipboard bytes, конвертация RGBA |
| Сериализация | `serde + serde_json` | Хранение пользовательских настроек |
| Обработка ошибок | `anyhow 1.0` | Единый error type для всего приложения |

### 4.2 Cargo.toml

```toml
[package]
name    = "snapmark"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe      = { version = "0.27", features = ["default"] }
egui        = "0.27"
egui_extras = "0.27"
arboard     = "3.3"
image       = { version = "0.25", default-features = false, features = ["png", "jpeg", "tiff"] }
tiny-skia   = "0.11"
rfd         = "0.14"
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
anyhow      = "1"

[target.'cfg(target_os = "macos")'.dependencies]
objc2             = "0.5"
objc2-app-kit     = "0.2"
objc2-foundation  = "0.2"
```

### 4.3 Структура проекта

```
snapmark/
├── src/
│   ├── main.rs              # Точка входа: eframe::run_native, Menu Bar setup
│   ├── app.rs               # SnapMarkApp: главный AppState, eframe::App impl
│   ├── state.rs             # EditorState: annotations, history, active_tool
│   ├── clipboard.rs         # ClipboardWatcher: polling thread + mpsc channel
│   ├── canvas.rs            # Canvas widget: egui Painter, zoom, hit-test
│   ├── toolbar.rs           # Toolbar widget: tool buttons, color, stroke
│   ├── action_bar.rs        # Action Bar: Undo/Redo, Copy, Save
│   ├── flatten.rs           # Render annotations → DynamicImage via tiny-skia
│   ├── annotation.rs        # Типы: Annotation, AnnotationKind, Handle
│   ├── history.rs           # UndoHistory<T>: stack-based undo/redo
│   └── platform/
│       ├── mod.rs
│       └── macos.rs         # NSStatusBar, NSColorPanel, HiDPI scale
├── assets/
│   └── icon.png             # Иконка приложения 1024×1024
├── Info.plist               # LSUIElement=true, CFBundleIdentifier
├── build.rs                 # Embed Info.plist, set icon
└── Cargo.toml
```

### 4.4 Ключевые структуры данных

#### Annotation

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Annotation {
    pub id:           AnnotationId,     // u64, монотонный счётчик
    pub kind:         AnnotationKind,
    pub color:        [u8; 4],          // RGBA
    pub stroke_width: StrokeWidth,      // Thin(1.5) | Medium(3.0) | Thick(5.0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnnotationKind {
    Arrow         { from: Pos2, to: Pos2 },
    ArrowWithText { from: Pos2, to: Pos2, text: String },
    Text          { pos: Pos2, content: String, size: TextSize }, // S | M | L
    Rectangle     { rect: Rect },
    Ellipse       { center: Pos2, radii: Vec2 },
}
```

#### EditorState

```rust
pub struct EditorState {
    pub image:         Option<EditorImage>,      // скриншот + egui TextureHandle
    pub annotations:   Vec<Annotation>,
    pub history:       UndoHistory<Vec<Annotation>>,
    pub active_tool:   Tool,
    pub active_color:  [u8; 4],
    pub active_stroke: StrokeWidth,
    pub selection:     Option<AnnotationId>,
    pub drag_state:    Option<DragState>,         // текущий drag в процессе
    pub text_edit:     Option<TextEditState>,
    pub zoom:          f32,                       // 0.25 ..= 4.0
    pub exported:      bool,                      // флаг для диалога при закрытии
}
```

#### UndoHistory

```rust
pub struct UndoHistory<T: Clone> {
    stack:  Vec<T>,   // снэпшоты состояния аннотаций
    cursor: usize,    // текущая позиция
}
// Каждое изменение → push_snapshot(annotations.clone())
// Undo → cursor -= 1
// Redo → cursor += 1
```

### 4.5 ClipboardWatcher

Работает в отдельном OS thread, события передаются в главный поток через `std::sync::mpsc::channel`.

```rust
fn clipboard_watcher(tx: Sender<WatcherEvent>) {
    let mut last_change_count = NSPasteboard::changeCount();
    loop {
        thread::sleep(Duration::from_millis(300));
        let current = NSPasteboard::changeCount();
        if current != last_change_count {
            last_change_count = current;
            if let Some(img) = try_read_image_from_clipboard() {
                NSPasteboard::clear();                        // очистить clipboard
                tx.send(WatcherEvent::ImageDetected(img)).ok();
            }
        }
    }
}
```

> **Важно:** `changeCount` проверяется без чтения данных — это O(1) системный вызов без аллокаций. Данные читаются только при реальном изменении.

### 4.6 Flatten (финальный рендер)

Аннотации рендерятся поверх оригинального bitmap через `tiny-skia` в **оригинальном** разрешении изображения, а не в логических пикселях экрана.

```rust
pub fn flatten(
    image:       &DynamicImage,
    annotations: &[Annotation],
    scale:       f32,             // device_pixel_ratio: 1.0 или 2.0 (Retina)
) -> DynamicImage {
    let mut pixmap = Pixmap::new(image.width(), image.height()).unwrap();
    // 1. Скопировать пиксели изображения в pixmap
    // 2. Для каждой аннотации: перевести координаты из логических px → пиксели
    //    изображения (умножить на scale)
    // 3. Нарисовать через tiny_skia::Path + Paint
    // 4. Вернуть DynamicImage::from(pixmap.data())
}
```

### 4.7 macOS App Bundle

```
SnapMark.app/
└── Contents/
    ├── MacOS/
    │   └── snapmark          # бинарник (Universal Binary: arm64 + x86_64)
    ├── Resources/
    │   └── AppIcon.icns
    └── Info.plist
```

Ключевые ключи `Info.plist`:

```xml
<key>LSUIElement</key>         <true/>          <!-- скрыть из Dock -->
<key>CFBundleIdentifier</key>  <string>com.yourname.snapmark</string>
<key>LSMinimumSystemVersion</key> <string>13.0</string>
<key>NSHighResolutionCapable</key> <true/>
```

Сборка Universal Binary:

```bash
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create \
  target/aarch64-apple-darwin/release/snapmark \
  target/x86_64-apple-darwin/release/snapmark \
  -output SnapMark.app/Contents/MacOS/snapmark
```

---

## 5. UI / UX Спецификация

### 5.1 Визуальный дизайн

- **Стиль:** нативный macOS. Использовать системные цвета egui (`egui::Visuals::dark()` / `light()`) по системной теме
- **Фон:** `NSColor.windowBackgroundColor` — автоматически адаптируется к dark/light mode
- **Тулбар и Action Bar:** отделены от канваса тонким сепаратором (1px `#E0E0E0` в light mode)
- **Кнопки тулбара:** 36×36px, `border-radius` 6px, hover effect
- **Активный инструмент:** заливка кнопки акцентным цветом системы (`NSColor.controlAccentColor`)
- **Курсоры:**

| Инструмент / действие | Курсор |
|---|---|
| Select (обычный) | `NSCursor.arrow` |
| Select + hover над элементом | `NSCursor.openHand` |
| Select + drag элемента | `NSCursor.closedHand` |
| Select + hover над ручкой | resize cursor (напр. `NSCursor.resizeUpDown`) |
| Arrow, Rectangle, Ellipse | `NSCursor.crosshair` |
| Text | `NSCursor.iBeam` |

### 5.2 Keyboard Shortcuts — полная таблица

| Shortcut | Действие |
|---|---|
| `V` / `Escape` | Активировать Select tool |
| `A` | Активировать Arrow tool |
| `Shift+A` | Активировать Arrow+Text tool |
| `T` | Активировать Text tool |
| `R` | Активировать Rectangle tool |
| `E` | Активировать Ellipse tool |
| `Cmd+Z` | Undo |
| `Cmd+Shift+Z` | Redo |
| `Cmd+C` | Copy результата в clipboard |
| `Cmd+S` | Открыть Save диалог |
| `Cmd+V` | Вставить изображение из clipboard |
| `Delete` / `Backspace` | Удалить выбранный элемент |
| `Cmd+=` | Zoom In |
| `Cmd+-` | Zoom Out |
| `Cmd+0` | Fit to view (сброс зума) |
| `Cmd+W` | Закрыть окно редактора |
| `Cmd+Q` | Завершить приложение |

### 5.3 Состояния кнопок

| Элемент | Состояние | Поведение |
|---|---|---|
| **Undo** | Disabled если история пуста | Opacity 0.4, не кликабельна |
| **Redo** | Disabled если нет forward-истории | Opacity 0.4, не кликабельна |
| **Copy** | Normal → Pressed → *«Copied!»* | Текст меняется на *«Copied!»* на 1.5 сек |
| **Save** | Normal / Pressed | Primary button style (акцентный цвет) |
| **Tool buttons** | Normal / Active / Hover | Active = заливка `controlAccentColor` |

---

## 6. Нефункциональные требования

### 6.1 Производительность

- Рендер канваса: **60 fps** при рисовании на скриншотах до 4K (3840×2160)
- Flatten для Copy/Save: **≤ 500 мс** для 4K с 20 аннотациями
- Загрузка изображения из clipboard: **≤ 100 мс** для 4K PNG
- Polling clipboard: интервал 300 мс, CPU **≤ 0.1%**

### 6.2 Совместимость

- macOS 13.0 Ventura и выше
- **Universal Binary**: arm64 (Apple Silicon) + x86_64 (Intel)
- Retina и non-Retina дисплеи
- Многомониторные конфигурации с разными scale factors
- Light mode и Dark mode (автоматически)

### 6.3 Надёжность

- Крэш приложения **не должен терять** данные clipboard — изображение «забирается» только после успешной загрузки в редактор
- Все `Result`-ошибки (clipboard, file I/O) обрабатываются через `anyhow`, показываются пользователю через нативный `NSAlert`
- При невозможности инициализировать `NSStatusBar` — приложение завершается с понятным сообщением

### 6.4 Безопасность и приватность

- Приложение **не отправляет никаких данных по сети**
- Не требует специальных entitlements кроме минимального набора
- Скриншоты хранятся только в RAM на время сессии редактора
- При закрытии окна изображение выгружается из памяти

---

## 7. План разработки

### Этапы и оценки

| Этап | Описание | Зависит от | Дней |
|---|---|---|---|
| **1** | Скелет + clipboard + отображение | — | 2–3 |
| **2** | Rect, Ellipse, Toolbar | 1 | 3–4 |
| **3** | Arrow, Text, Arrow+Text | 2 | 2–3 |
| **4** | Select, Undo/Redo, Flatten, Export | 2, 3 | 3–4 |
| **5** | Zoom, все Shortcuts, диалоги, polish | 4 | 2–3 |
| **6** | Тестирование, Bundle, релиз | 5 | 1–2 |
| **Итого** | | | **13–19 дней** |

### Детализация по этапам

#### Этап 1 — Скелет приложения (2–3 дня)
- Cargo-проект, `Info.plist`, `LSUIElement = true`
- `eframe::App` с пустым окном
- `NSStatusBar` иконка с меню (objc2)
- `ClipboardWatcher`: polling thread + `mpsc::channel`
- Чтение изображения из clipboard → отображение в egui
- Очистка clipboard после захвата

#### Этап 2 — Базовый рендер (3–4 дня)
- Структуры `Annotation`, `AnnotationKind`
- Rectangle и Ellipse (drag → bounding rect)
- Рендер аннотаций через `egui::Painter` поверх изображения
- Toolbar: кнопки инструментов, цветовая палитра, пресеты толщины
- Live preview во время drag

#### Этап 3 — Стрелки и текст (2–3 дня)
- Arrow: линия + вычисление и рендер треугольного наконечника
- Text: inline `egui::TextEdit`, завершение по `Cmd+Enter`
- Arrow+Text: Arrow + `TextEdit` у хвоста после отпускания мыши

#### Этап 4 — Select, Undo, Flatten (3–4 дня)
- `UndoHistory<Vec<Annotation>>`: push, undo, redo
- `Cmd+Z` / `Cmd+Shift+Z` + кнопки в Action Bar
- Select tool: hit-test (bounding box с padding), bounding box с 8 ручками
- Drag-to-move (delta в пространстве изображения)
- Resize handles для каждого типа элементов
- `Delete` / `Backspace` для удаления выбранного элемента
- Flatten через `tiny-skia`
- Copy → `arboard`
- Save → `rfd` → `image::save_buffer`

#### Этап 5 — Polish и macOS интеграция (2–3 дня)
- Zoom (`Cmd+=/-/0`) + скролл канваса
- Все keyboard shortcuts из таблицы
- Все confirmation диалоги (unsaved / replace / new screenshot)
- `NSColorPanel` для кастомного цвета (objc2-app-kit)
- Cursor changes в зависимости от инструмента
- Dark/Light mode автоопределение
- App icon, сборка Universal Binary

#### Этап 6 — Тестирование и релиз (1–2 дня)
- Ручное тестирование: Retina / non-Retina, Light / Dark mode, многомониторность
- Тестирование на больших изображениях (4K)
- Unit-тесты для `UndoHistory`, flatten, hit-test
- Сборка `.app` bundle (cargo-bundle или ручной скрипт)

---

## 8. Out of Scope для v1

| Фича | Причина отложить |
|---|---|
| Автозапуск при логине | `SMLoginItemSetEnabled` требует entitlements; добавить в v2 |
| Множественный выбор элементов | Усложняет hit-test и историю; редко нужно в базовом сценарии |
| Crop / кадрирование | Отдельная сложная фича |
| Blur / Pixelate (скрытие данных) | Требует отдельного рендера; популярно — в v2 |
| Рисование от руки (freehand) | Хранение пути усложняет модель данных |
| Глобальный hotkey | Требует Accessibility permission; спорный UX — v2 |
| Нумерованные callout-метки | v2 |
| Импорт из файла / drag-and-drop | v2 |
| Облачное хранилище / шаринг | Не нужно для задачи |

---

## 9. Открытые вопросы

| Вопрос | Статус / Решение по умолчанию |
|---|---|
| **macOS Sandbox: нужен ли? App Store?** | Для DMG-дистрибуции — sandbox не обязателен. Для App Store — нужен + entitlements для clipboard. Решить перед релизом. |
| **Нотаризация Apple?** | Рекомендуется для macOS 13+. Требует Developer ID ($99/год). Без нотаризации — Gatekeeper предупреждение. |
| **Шрифт для Text tool** | `NSFont.systemFont` (SF Pro) — используется через objc2-app-kit. Размеры: S=14pt, M=18pt, L=24pt. |
| **Максимальный размер изображения** | Ограничений нет. При изображениях > 4K flatten может занять > 500 мс — рассмотреть progress indicator. |
| **Хранение настроек пользователя** | JSON-файл: `~/Library/Application Support/SnapMark/settings.json` (последний цвет, толщина). |

---

*SnapMark PRD v1.0 — Draft · Февраль 2026*
