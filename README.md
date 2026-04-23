# cosmic-hot-corners

A native **Hot Corners** daemon for the [COSMIC™ Desktop](https://system76.com/cosmic) (Pop!_OS), built entirely with [libcosmic](https://github.com/pop-os/libcosmic) and [iced](https://github.com/iced-rs/iced).

## Overview

`cosmic-hot-corners` runs as a background daemon with no visible window. It places four invisible 10×10 px [wlr-layer-shell](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) surfaces — one at each screen corner — using `Layer::Overlay`. When the pointer enters a corner and remains there longer than the configured delay, a configurable action is triggered.

The project is fully integrated with the COSMIC ecosystem: configuration is stored and watched via `cosmic-config`, and all Wayland interaction is handled through libcosmic's `iced_winit` + `cctk` (COSMIC Client Toolkit) backend.

## Architecture

```
main.rs
├── cosmic-hot-corners            // daemon mode (no visible window)
└── cosmic-hot-corners --settings // GUI settings window

app.rs
├── init()         Loads config; waits for Wayland output events to create surfaces.
├── subscription() Listens to OutputEvent (Added/Removed) to manage surfaces per monitor,
│                  CursorMoved/CursorLeft for pointer detection, and watch_config() for
│                  live config reload without restarting the daemon.
├── update()
│   ├── OutputAdded   → creates 4 layer-shell surfaces (10×10 px, Overlay) for that output
│   ├── OutputRemoved → destroys the 4 surfaces for that output
│   ├── CursorMoved   → increments pending_generation, schedules tokio::time::sleep(delay_ms)
│   ├── TriggerCorner → fires action only if generation still matches
│   │                   (cancellation-safe — rapid movement never triggers)
│   └── ConfigUpdated → hot-reloads config; next trigger uses new settings
└── execute_action() Dispatches CornerAction variants via D-Bus or sh -c.

config.rs
└── Config (cosmic-config v1)
    ├── delay_ms: u64          // activation delay in milliseconds
    ├── top_left: CornerAction
    ├── top_right: CornerAction
    ├── bottom_left: CornerAction
    └── bottom_right: CornerAction

settings_app.rs
└── GUI settings window (libcosmic, follows COSMIC light/dark theme automatically)
    ├── Activation delay field
    └── 2×2 grid of corner cards, each with action dropdown + optional command input
```

## Corner Actions

| Variant | Behavior |
|---|---|
| `Disabled` | No-op |
| `ShowWorkspaces` | Opens the workspaces overview via D-Bus (`com.system76.CosmicWorkspaces`) |
| `ShowDesktop` | *(not yet available in COSMIC)* |
| `OpenLauncher` | Opens the app launcher via D-Bus (`com.system76.CosmicLauncher`) |
| `ToggleNightLight` | *(not yet available in COSMIC)* |
| `RunCommand(String)` | Executes an arbitrary shell command via `sh -c` |

## Configuration

### GUI (recommended)

After installing, open **Hot Corners Settings** from the COSMIC app drawer, or right-click the daemon's entry in the launcher and choose **Configure Hot Corners**.

The settings window lets you assign an action to each corner and adjust the activation delay. Changes are saved instantly — no restart required.

### Manual (advanced)

Configuration is also stored as plain files managed by `cosmic-config`, under:

```
~/.config/cosmic/io.github.cosmic-hot-corners/v1/
```

Each field is a separate file. Example — enable workspace overview on the top-left corner with a 400 ms delay:

```sh
mkdir -p ~/.config/cosmic/io.github.cosmic-hot-corners/v1
echo '400' > ~/.config/cosmic/io.github.cosmic-hot-corners/v1/delay_ms
echo 'ShowWorkspaces' > ~/.config/cosmic/io.github.cosmic-hot-corners/v1/top_left
```

## Requirements

- COSMIC Desktop (Wayland compositor with `wlr-layer-shell` support)
- `libxkbcommon-dev`, `libwayland-dev`, `pkg-config`
- Rust toolchain (edition 2024)

## Installation

### 1. Install build dependencies

```sh
sudo apt install libxkbcommon-dev libwayland-dev pkg-config
```

### 2. Clone and install

```sh
git clone https://github.com/your-username/cosmic-hotcorners
cd cosmic-hotcorners
just build-release
sudo just install
```

### 3. Open the settings

Launch the settings GUI to configure each corner:

```sh
cosmic-hot-corners --settings
```

Or open **Hot Corners Settings** from the COSMIC app drawer.

### 4. Autostart

To start the daemon automatically on login:

```sh
just autostart
```

To disable autostart:

```sh
just autostart-disable
```

This installs/removes a `.desktop` file in `~/.config/autostart/`. The daemon does not run as a systemd service — it is launched by the COSMIC session manager alongside other autostart applications.

Configuration changes made in the settings GUI are applied instantly — the running daemon reloads config automatically without needing a restart.

To stop a running instance:

```sh
pkill -f cosmic-hot-corners
```

For distribution packaging:

```sh
just vendor
just build-vendored
just rootdir=debian/cosmic-hot-corners prefix=/usr install
```

## Development

Install [rustup][rustup] and configure your editor with [rust-analyzer][rust-analyzer]. To reduce compile times, disable LTO in the release profile and use [mold][mold] + [sccache][sccache].

## License

[MPL-2.0](./LICENSE)

[just]: https://github.com/casey/just
[rustup]: https://rustup.rs/
[rust-analyzer]: https://rust-analyzer.github.io/
[mold]: https://github.com/rui314/mold
[sccache]: https://github.com/mozilla/sccache
