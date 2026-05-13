# ledctl

Control keyboard indicator LEDs (Caps Lock, Num Lock, Scroll Lock) on macOS.

## Background

Some hardware uses keyboard LED state as a signaling channel. Certain programmable keyboards, stream decks, and other input peripherals watch for Num Lock, Scroll Lock, or Caps Lock changes to trigger behaviour — essentially treating the LED as a software-controlled output pin. macOS provides no built-in way to drive these LEDs programmatically; they're only toggled as a side effect of pressing the physical key.

`ledctl` talks directly to the IOKit HID layer, letting you set LED state without a keypress. The `--count N` toggle option covers hardware that distinguishes a sequence of N toggles from a single one.

## Install

Download the universal binary from the [latest release](https://github.com/thor/ledctl/releases/latest) and put it somewhere on your `$PATH`.

## Usage

```
ledctl --list
ledctl [--device <name>] <caps|num|scroll> <on|off|toggle> [--count <n>]
```

Running without arguments opens a small GUI.

```sh
# Turn Num Lock on across all keyboards
ledctl num on

# Toggle Scroll Lock 3 times on a specific keyboard
ledctl --device "Keychron" scroll toggle --count 3

# List connected keyboards with their location IDs
ledctl --list
```

## Requirements

macOS 11.0 or later.
