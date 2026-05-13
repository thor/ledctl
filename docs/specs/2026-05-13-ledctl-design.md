# ledctl — macOS LED Control Tool

**Date:** 2026-05-13  
**Status:** Approved

---

## Overview

A macOS-only Rust application that reads and toggles keyboard indicator LEDs (Caps Lock, Num Lock, Scroll Lock) on a per-device basis using IOKit HID APIs. Ships as a universal binary (arm64 + x86_64). When invoked with no arguments it shows a graphical UI; with arguments it operates as a CLI tool.

---

## Architecture

```
ledctl/
├── Cargo.toml          # eframe + egui + core-foundation-sys
├── Makefile            # universal binary target via lipo
├── build.rs            # links IOKit.framework + CoreFoundation.framework
└── src/
    ├── main.rs         # CLI arg parsing, dispatch to GUI or CLI mode
    ├── hid.rs          # IOKit HID FFI bindings + all LED logic (safe public API)
    └── ui.rs           # egui application
```

Three modules with a single direction of dependency: `main.rs` and `ui.rs` call into `hid.rs`; `hid.rs` has no knowledge of the UI or CLI layers.

---

## IOKit HID Layer (`hid.rs`)

### FFI Bindings

Manual `unsafe extern "C"` bindings against `IOKit.framework`. Uses `core-foundation-sys` for `CFMutableDictionaryRef`, `CFSetRef`, `CFArrayRef`, `CFStringRef`, and `CFNumberRef`. Approximately 15 extern functions bound:

- `IOHIDManagerCreate`, `IOHIDManagerOpen`, `IOHIDManagerSetDeviceMatching`, `IOHIDManagerCopyDevices`
- `IOHIDDeviceGetProperty`, `IOHIDDeviceConformsTo`, `IOHIDDeviceCopyMatchingElements`
- `IOHIDDeviceGetValue`, `IOHIDDeviceSetValue`
- `IOHIDValueCreateWithIntegerValue`, `IOHIDValueGetIntegerValue`
- `IOHIDElementGetUsagePage`, `IOHIDElementGetUsage`

Key HID constants:

| Constant | Value |
|---|---|
| `kHIDPage_GenericDesktop` | 0x01 |
| `kHIDUsage_GD_Keyboard` | 0x06 |
| `kHIDPage_LEDs` | 0x08 |
| `kHIDUsage_LED_NumLock` | 0x01 |
| `kHIDUsage_LED_CapsLock` | 0x02 |
| `kHIDUsage_LED_ScrollLock` | 0x03 |

### Public Safe API

```rust
pub enum Led { Caps, Num, Scroll }

pub struct KeyboardDevice {
    pub name: String,
    pub location_id: u32,
    ref_: IOHIDDeviceRef,   // non-Send; only used on main thread
}

pub struct LedState {
    pub caps: bool,
    pub num: bool,
    pub scroll: bool,
}

pub fn list_keyboards() -> Vec<KeyboardDevice>
pub fn get_led_state(dev: &KeyboardDevice) -> Result<LedState, LedError>
pub fn set_led(dev: &KeyboardDevice, led: Led, on: bool) -> Result<(), LedError>
pub fn toggle_led(dev: &KeyboardDevice, led: Led, count: u32) -> Result<(), LedError>
```

`toggle_led` calls `set_led` in a tight loop `count` times, alternating state on each iteration starting from the current state. A count of 1 is a normal toggle; even counts return the LED to its original state.

### Device Enumeration

1. `IOHIDManagerCreate` with `kIOHIDOptionsTypeNone`
2. Set matching dictionary: `kHIDPage_GenericDesktop` / `kHIDUsage_GD_Keyboard`
3. `IOHIDManagerOpen` + `IOHIDManagerCopyDevices`
4. For each device: verify with `IOHIDDeviceConformsTo(kHIDPage_GenericDesktop, kHIDUsage_GD_Keyboard)`
5. Read `kIOHIDProductKey` property for display name; `kIOHIDLocationIDKey` as stable identifier
6. LED elements found by filtering `IOHIDDeviceCopyMatchingElements` to `kHIDPage_LEDs`

---

## CLI Mode (`main.rs`)

### Invocation

```
ledctl --list
ledctl [--device <name>] <led> <state> [--count <n>]
```

- `<led>`: `caps` | `num` | `scroll`
- `<state>`: `on` | `off` | `toggle`
- `--device`: matches by product name substring (case-insensitive); if omitted, targets all keyboards
- `--count <n>`: repeat the operation N times (default: 1, min: 1, max: 9999)

### Behavior

- `--list` prints one line per keyboard: `[location_id] Product Name`
- On success: exits 0, no output
- On error: message to stderr, exits 1
- Ambiguous `--device` match (multiple devices match substring): prints matches and exits 1

---

## GUI Mode (`ui.rs`)

### Window

Single `egui` window, title "LED Control", ~420×220px, not resizable.

### Layout (top to bottom)

1. **Device row**: label "Device:" + dropdown listing keyboards by name, with "All Devices" as the first entry. Refreshed once at startup.
2. **Count row**: label "Count:" + numeric integer input (range 1–9999, default 1). This controls how many times each LED toggle operation fires.
3. **LED button row**: three equal-width buttons — `Caps Lock`, `Num Lock`, `Scroll Lock`. Each button is visually highlighted (accent fill) when the LED is currently on, dimmed when off.

### Interaction

- On startup: enumerate keyboards, select "All Devices", read LED state from first real device for display.
- Clicking a button: calls `toggle_led(dev, led, count)` for the selected device (or all devices if "All Devices"), then re-reads and refreshes displayed state.
- State is read synchronously (no background threads); IOKit reads are fast enough (~1ms) that no async needed.
- Count input validates on change (clamps to 1–9999); non-numeric input is rejected.

---

## Universal Binary

Two separate `cargo build --release` invocations targeting `aarch64-apple-darwin` and `x86_64-apple-darwin`, combined with `lipo`:

```makefile
.PHONY: universal clean

universal:
	cargo build --release --target aarch64-apple-darwin
	cargo build --release --target x86_64-apple-darwin
	lipo -create \
	    -output target/ledctl \
	    target/aarch64-apple-darwin/release/ledctl \
	    target/x86_64-apple-darwin/release/ledctl

clean:
	cargo clean
	rm -f target/ledctl
```

The binary requires macOS 11.0+ (Big Sur) for arm64 support.

---

## Error Handling

`LedError` is a simple enum covering: `DeviceNotFound`, `ElementNotFound`, `IoKitError(IOReturn)`, `NoSuchLed`. The CLI prints the `Display` impl and exits 1. The GUI shows the error string below the buttons in red, replacing it with the refreshed state on the next successful click.

---

## Dependencies

| Crate | Purpose |
|---|---|
| `eframe` | egui + Metal/wgpu backend for macOS |
| `egui` | immediate-mode GUI |
| `core-foundation-sys` | CF type definitions for IOKit FFI |

No other third-party crates. CLI argument parsing is done manually (the interface is simple enough that `clap` adds more weight than value).

---

## Out of Scope

- Packaging as a `.app` bundle (the binary runs standalone)
- Support for compose/kana LEDs (present in HID spec but rarely wired on real hardware)
- Persistence of any settings
- Background polling / auto-refresh
