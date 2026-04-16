# cosmic-hot-corners

A native **Hot Corners** daemon for the [COSMIC™ Desktop](https://system76.com/cosmic) (Pop!_OS), built entirely with [libcosmic](https://github.com/pop-os/libcosmic) and [iced](https://github.com/iced-rs/iced).

## Overview

`cosmic-hot-corners` runs as a background daemon with no visible window. It places four invisible 10×10 px [wlr-layer-shell](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) surfaces — one at each screen corner — using `Layer::Overlay`. When the pointer enters a corner and remains there longer than the configured delay, a configurable action is triggered.

The project is fully integrated with the COSMIC ecosystem: configuration is stored and watched via `cosmic-config`, and all Wayland interaction is handled through libcosmic's `iced_winit` + `cctk` (COSMIC Client Toolkit) backend.

## Architecture

```
main.rs
└── cosmic::app::Settings::no_main_window(true)   // daemon mode, no visible window

app.rs
├── init()       Creates 4 SctkLayerSurfaceSettings (Overlay layer, corner anchors)
│                and issues get_layer_surface() tasks for each.
├── subscription() Listens to CursorEntered / CursorLeft via listen_with()
├── update()     On CursorEntered: increments pending_generation, schedules a
│                tokio::time::sleep(delay_ms) future.
│                On TriggerCorner: fires action only if generation matches
│                (cancellation-safe delay — rapid movement never triggers).
└── execute_action() Dispatches CornerAction variants.

config.rs
└── Config (cosmic-config v1)
    ├── delay_ms: u64          // activation delay in milliseconds
    ├── top_left: CornerAction
    ├── top_right: CornerAction
    ├── bottom_left: CornerAction
    └── bottom_right: CornerAction
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

Configuration is managed by `cosmic-config` and stored under:

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
cargo build --release
sudo cp target/release/cosmic-hotcorners /usr/local/bin/
```

Or using [just][just]:

```sh
just build-release
sudo just install
```

### 3. Configure corners

Configuration is stored under `~/.config/cosmic/io.github.cosmic-hot-corners/v1/`. Each field is a separate file in [RON](https://github.com/ron-rs/ron) format.

```sh
mkdir -p ~/.config/cosmic/io.github.cosmic-hot-corners/v1

# Delay in milliseconds (default: 300)
echo '300' > ~/.config/cosmic/io.github.cosmic-hot-corners/v1/delay_ms

# Top-left: open workspaces overview
echo 'ShowWorkspaces' > ~/.config/cosmic/io.github.cosmic-hot-corners/v1/top_left

# Top-right: open app launcher
echo 'OpenLauncher' > ~/.config/cosmic/io.github.cosmic-hot-corners/v1/top_right

# Bottom-right: run a custom command
echo 'RunCommand("notify-send Hello World")' > ~/.config/cosmic/io.github.cosmic-hot-corners/v1/bottom_right

# Bottom-left: disabled (default)
echo 'Disabled' > ~/.config/cosmic/io.github.cosmic-hot-corners/v1/bottom_left
```

### 4. Autostart

To start the daemon automatically with your session:

```sh
mkdir -p ~/.config/autostart
cat > ~/.config/autostart/cosmic-hot-corners.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=cosmic-hot-corners
Exec=/usr/local/bin/cosmic-hot-corners
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
EOF
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
