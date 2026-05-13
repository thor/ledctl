# Design: Replace egui with iced

**Date:** 2026-05-13  
**Scope:** `src/ui.rs` + `Cargo.toml` — all other files unchanged  
**Goal:** Replace the egui/eframe GUI with idiomatic iced 0.13 using the Elm architecture

---

## Context

`ledctl` is a macOS-only tool for toggling keyboard LEDs (Caps Lock, Num Lock, Scroll Lock) via IOKit/HID. When invoked with no CLI arguments it opens a small native window. The entire GUI lives in `src/ui.rs` (169 lines). The rest of the codebase (`hid.rs`, `cli.rs`, `main.rs`, `lib.rs`) is unaffected by this migration.

---

## Architecture

### Framework

- **Remove:** `eframe = "0.28"`, `egui = "0.28"`
- **Add:** `iced = "0.13"` (with default features; no extra feature flags required for a basic native window)
- All other dependencies (`core-foundation-sys`) stay the same

### Paradigm shift

egui is immediate-mode: `update()` is called every frame and the UI is rebuilt from scratch each time.

iced uses the Elm architecture:
- **Model** — `LedApp` struct holds all state
- **Message** — enum describing every event that can change state
- **`update`** — takes a `&mut self` and a `Message`, mutates state
- **`view`** — takes `&self`, returns the widget tree (called after each update)

### `run()` entry point

```rust
pub fn run() {
    iced::application("LED Control", LedApp::update, LedApp::view)
        .window(iced::window::Settings {
            resizable: false,
            size: iced::Size::new(380.0, 160.0),
            ..Default::default()
        })
        .run_with(|| (LedApp::new(), iced::Task::none()))
        .expect("Failed to launch LED Control window");
}
```

`run_with` provides the initial state and an initial `Task`. `main.rs` requires no changes since `run()` returns `()`.

---

## State

`LedApp` fields differ slightly from the egui version to accommodate `pick_list`:

| Field | Type | Purpose |
|---|---|---|
| `session` | `Option<HidSession>` | IOKit session; `None` if open failed |
| `selected` | `usize` | 0 = All Devices; 1..N = keyboards[selected-1] |
| `selected_name` | `String` | Display name for the currently selected device (drives `pick_list`) |
| `state` | `Option<LedState>` | Current caps/num/scroll state of selected device |
| `error` | `Option<String>` | Last HID error, shown in red |
| `count` | `u32` | Toggle repeat count (1–9999) |
| `count_str` | `String` | Raw text input for count |

`LedApp::new()` opens the session and reads initial LED state from the first keyboard, same as the egui version. `selected` starts at 0 and `selected_name` starts as `"All Devices"`.

---

## Messages

```rust
enum Message {
    DeviceSelected(String),  // User picked a device name from the list (mapped back to index in update)
    CountChanged(String),    // User edited the count text input
    ToggleLed(Led),          // User clicked a LED button
}
```

`pick_list` requires items that implement `Display + Clone + PartialEq`. Device selection uses `Vec<String>` display names (index 0 = "All Devices", 1..N = keyboard names), with a `selected_name: String` field tracked in state. The `usize` index is computed from the name on each `DeviceSelected` message.

---

## Update logic

All HID calls happen synchronously inside `update`. IOKit operations complete in microseconds; there is no need for async commands.

| Message | Effect |
|---|---|
| `DeviceSelected(name)` | Update `selected_name`; compute `selected` index by finding `name` in device list; call `refresh_state()` |
| `CountChanged(s)` | Update `count_str`; parse and update `count` if valid (1–9999) |
| `ToggleLed(led)` | Call `session.toggle_led(...)` on target devices, refresh state, set/clear `error` |

`refresh_state()` is unchanged: reads LED state from the currently selected device.

---

## View

A `Column` with fixed padding:

1. **Heading** — `text("LED Control")` at a larger size
2. **Device row** — `Row` with `text("Device:")` label + `pick_list(device_names, selected_name, Message::DeviceSelected)`
3. **Count row** — `Row` with `text("Count:")` label + `text_input("", &count_str).on_input(Message::CountChanged)` with `width(60)`
4. **LED buttons row** — `Row` with three `led_button("Caps Lock", caps_on)`, `led_button("Num Lock", num_on)`, `led_button("Scroll Lock", scroll_on)` widgets, each producing `Message::ToggleLed(led)` on press
5. **Error row** — if `self.error.is_some()`, `text(err).style(iced::theme::Text::Color(RED))`

### LED button styling

iced 0.13 uses `button(...).style(...)` with a closure or a type implementing `StyleSheet`. Each LED button passes a `bool` for its on/off state and applies:

- **On:** green background (`#50b45a`) with white text
- **Off:** dark grey background (`#373738`) with white text
- Both: minimum size 110×36, bold label text

---

## Error handling

- If `HidSession::new()` fails at startup, `session` is `None` and an initial error message is shown
- `ToggleLed` messages when `session` is `None` set `self.error` to a static "No HID session available" message (same as egui version)
- HID errors from `toggle_led` are displayed as `self.error`; successful operations clear it

---

## Window sizing

iced 0.13 does not have a direct equivalent of egui's `send_viewport_cmd` auto-resize trick. The window is fixed at a size that comfortably fits all widgets (380×160 px). This is a minor regression from the egui version's dynamic resize-to-content, but is acceptable for a fixed-layout window.

If precise fit-to-content is needed in future, iced supports `window::resize` commands from inside `update`, but this is out of scope.

---

## Files changed

| File | Change |
|---|---|
| `Cargo.toml` | Remove `eframe`, `egui`; add `iced = "0.13"` |
| `src/ui.rs` | Complete rewrite (~140 lines) |

No changes to `src/main.rs`, `src/hid.rs`, `src/cli.rs`, `src/lib.rs`, `tests/unit.rs`, or build scripts.
