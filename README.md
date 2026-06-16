<p align="center">
  <img src="./assets/lumen.png" alt="Lumen" width="300">
</p>

# Lumen

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%2010+-blue)](https://github.com/Risuleia/Lumen/releases/latest)

[![Release](https://github.com/Risuleia/Lumen/actions/workflows/release.yml/badge.svg)](https://github.com/Risuleia/Lumen/actions/workflows/release.yml)
[![Release](https://img.shields.io/github/v/release/Risuleia/Lumen)](https://github.com/Risuleia/Lumen/releases/latest)
[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/Risuleia/Lumen/total)](https://github.com/Risuleia/Lumen/releases/)

A Dynamic Island experience for Windows 10+, built with [Rust](https://www.rust-lang.org/) and [Slint](https://slint.dev/).


https://github.com/user-attachments/assets/0ce76e0a-d8f0-42b0-b252-6c231d26d05b



> *Video: Lumen in action on Windows 11*

---

## What is Lumen?

Lumen brings a Dynamic Island-style notification and media hub to Windows. It lives at the top center of your screen as a pill-shaped overlay, surfacing system activity — media playback, notifications, microphone and camera usage — without interrupting your workflow.

<table>
  <tr>
    <td>
      
https://github.com/user-attachments/assets/d5f48992-d564-48c2-904c-f5ae72705143


  </td>
  <td>
      

https://github.com/user-attachments/assets/4ed5ca52-7810-429a-8330-51edd28b0a11


  </td>
  <td>
    

https://github.com/user-attachments/assets/167402c9-0f49-49b9-9ce6-e293aae93f75


  </td>
  </tr>
</table>

---

## Features

- **Media control** — displays current track, album art, and playback controls. Supports play/pause, next, previous, and seek.
- **Notifications** — surfaces toast notifications inline with auto-dismiss after 3 seconds.
- **Microphone & camera indicators** — shows when any app is actively using your microphone or camera.
- **Audio spectrum** — real-time FFT-based audio visualizer with 24 frequency bands.
- **Fullscreen detection** — automatically hides when a fullscreen app is in the foreground.
- **Clickthrough** — passes mouse input through when idle so it never interferes with your workflow.
- **Auto-updates** — checks for new releases on startup and notifies via Windows toast.
- **System tray** — minimal tray presence with manual update check and quit option.

---

## Installation

Download the latest installer from [Releases](https://github.com/Risuleia/Lumen/releases/latest):

```
Lumen-x.x.x-setup.exe
```

The installer will:
- Install Lumen to `%ProgramFiles%\Lumen`
- Add a Start menu entry
- Optionally create a desktop shortcut
- Optionally register Lumen to run at Windows startup
- Launch Lumen immediately after install

### Requirements

- Windows 10+
- Notification access must be granted when prompted on first launch

---

## Building from source

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.85+)
- [Inno Setup 7](https://jrsoftware.org/isdl.php) (for installer)

### Build

1. Run `cargo build --release`
2. Compile with Inno setup installer

### Development build

```powershell
cargo build
cargo run
```

---

## Architecture

Lumen is split into two crates:

### `lumen_core`

A Windows-specific library crate providing all system integrations:

| Module | Description |
|--------|-------------|
| `services/*` | Polls various Windows APIs to create |
| `core` | Top-level `IslandCore` struct implementation |
| `bus` | `crossbeam_channel` based event bus for inter-service communication |
| `event` | Event types |
| `runtime` | Shared state (`Arc<RwLock<T>>`) accessible by both services and the UI |
| `utils` | Various utilities |

### `lumen` (ui)

The Slint-based UI crate:

| Module | Description |
|--------|-------------|
| `app` | Top-level `Lumen` struct wiring core events to UI dispatches |
| `state` | `IslandState` — content, mic, camera, expanded state |
| `geometry` | Wrappers around Physical and Logical dimensions |
| `sync` | Converts core types to Slint-compatible types |
| `platform/*` | Platform level logic - window positioning, configuration, clickthrough loop, system tray, updater |

### Event flow

```
Service (poll) → EventSender → EventReceiver (background thread)
                                      ↓
                            slint::invoke_from_event_loop
                                      ↓
                              Lumen::dispatch()
                                      ↓
                             IslandState → Slint UI
```

---

## UI

The UI is written in [Slint](https://slint.dev/) and organized as follows:

```
ui/
  Shell.slint           — root window
  Island.slint          — main pill component with animation and state logic
  IndicatorLayer.slint  — mic/camera indicator dots
  layouts/
    Idle.slint          — empty state
    Media.slint         — media playback layout
    Notification.slint  — notification layout
  theme/
    Colors.slint        — color tokens
    Metrics.slint       — spacing and sizing tokens
  types.slint           — shared enums and structs
  global.slint          — IslandData global singleton
```

---

## Auto-updates

Lumen checks for new releases on startup (at most once every 24 hours). When a new version is found, a Windows toast notification is shown with an "Update Now" button. Clicking it downloads the installer and applies the update silently.

Manual update checks are available via the system tray menu.

---

## Contributing

Contributions are welcome, especially in the following areas:
- Multi-monitor support
- Additional media sources
- Accessibility improvements
- Performance improvements

> Kindly open an Issue before submitting a PR
---

## License

MIT — see [LICENSE](./LICENSE)
