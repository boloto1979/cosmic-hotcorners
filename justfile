# Name of the application's binary.
name := 'cosmic-hot-corners'
# The unique ID of the application.
appid := 'io.github.cosmic-hot-corners'

# Path to root file system, which defaults to `/`.
rootdir := ''
# The prefix for the `/usr` directory.
prefix := '/usr'
# The location of the cargo target directory.
cargo-target-dir := env('CARGO_TARGET_DIR', 'target')

# Application's appstream metadata
appdata := appid + '.metainfo.xml'
# Application's desktop entry
desktop := appid + '.desktop'
# Settings app desktop entry
desktop-settings := appid + '.settings.desktop'
# Application's icon.
icon-svg := appid + '.svg'

# Install destinations
base-dir := absolute_path(clean(rootdir / prefix))
appdata-dst := base-dir / 'share' / 'appdata' / appdata
bin-dst := base-dir / 'bin' / name
desktop-dst := base-dir / 'share' / 'applications' / desktop
desktop-settings-dst := base-dir / 'share' / 'applications' / desktop-settings
icons-dst := base-dir / 'share' / 'icons' / 'hicolor'
icon-svg-dst := icons-dst / 'scalable' / 'apps'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Compiles with debug profile
build-debug *args:
    cargo build --locked {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features --locked {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

# Run the application for testing purposes
run *args:
    env RUST_BACKTRACE=full cargo run --release --locked {{args}}

# Run the settings GUI for development
run-settings:
    env RUST_BACKTRACE=full cargo run --release --locked

# Run the daemon for development
run-daemon:
    env RUST_BACKTRACE=full cargo run --release --locked -- --daemon

# Installs files
install:
    install -Dm0755 {{ cargo-target-dir / 'release' / name }} {{bin-dst}}
    install -Dm0644 {{ 'resources' / desktop }} {{desktop-dst}}
    install -Dm0644 {{ 'resources' / desktop-settings }} {{desktop-settings-dst}}
    install -Dm0644 {{ 'resources' / appdata }} {{appdata-dst}}
    install -Dm0644 {{ 'resources' / 'icons' / 'hicolor' / 'scalable' / 'apps' / 'icon.svg' }} {{icon-svg-dst}}

# Enable autostart for the current user
autostart:
    mkdir -p ~/.config/autostart
    install -Dm0644 {{ 'resources' / desktop }} ~/.config/autostart/{{desktop}}

# Disables autostart for the current user
autostart-disable:
    rm -f ~/.config/autostart/{{desktop}}

# Uninstalls installed files
uninstall:
    rm {{bin-dst}} {{desktop-dst}} {{desktop-settings-dst}} {{icon-svg-dst}}

# Vendor dependencies locally
vendor:
    mkdir -p .cargo
    cargo vendor | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    tar pcf vendor.tar vendor
    rm -rf vendor

# Extracts vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar

# Bump cargo version, create git commit, and create tag
tag version:
    find -type f -name Cargo.toml -exec sed -i '0,/^version/s/^version.*/version = "{{version}}"/' '{}' \; -exec git add '{}' \;
    cargo check
    cargo clean
    git add Cargo.lock
    git commit -m 'release: {{version}}'
    git commit --amend
    git tag -a {{version}} -m ''

# Generate Flatpak cargo sources (requires Python 3 + aiohttp: pip install aiohttp toml)
# Downloads flatpak-cargo-generator from flatpak-builder-tools and runs it.
flatpak-sources:
    mkdir -p build-aux
    curl -fsSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py \
        -o build-aux/flatpak-cargo-generator.py
    python3 build-aux/flatpak-cargo-generator.py Cargo.lock -o generated-sources.json

# Build and install the Flatpak locally for testing (requires flatpak-builder)
flatpak-build:
    flatpak-builder --install --user --force-clean flatpak-build-dir \
        io.github.cosmic-hot-corners.yml

# Run the installed Flatpak (daemon mode)
flatpak-run:
    flatpak run io.github.cosmic-hot-corners --daemon

# Run the installed Flatpak (settings GUI)
flatpak-run-settings:
    flatpak run io.github.cosmic-hot-corners

