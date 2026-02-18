# menucli justfile

# Default: show available recipes
default:
    @just --list

# Build (debug)
build:
    cargo build

# Build (release)
release:
    cargo build --release

# Build release and copy to /usr/local/bin (requires sudo)
deploy: release
    sudo cp target/release/menucli /usr/local/bin/menucli
    @echo "Deployed to /usr/local/bin/menucli"

# Run unit tests
test:
    cargo test

# Run clippy lints (matches CI settings)
lint:
    cargo clippy -- -D clippy::all -D clippy::pedantic

# Auto-fix clippy warnings where possible
fix:
    cargo clippy --fix --allow-dirty

# Check for compile errors without producing artifacts
check:
    cargo check

# List all menu items for the frontmost app (table view)
run-list:
    cargo run -- list --tree

# List all menu items as flat JSON
run-list-json:
    cargo run -- list --json

# List running apps
run-apps:
    cargo run -- apps

# Check Accessibility permission
run-check-access:
    cargo run -- check-access

# Fuzzy search example (edit QUERY as needed)
run-search QUERY="save":
    cargo run -- search "{{QUERY}}"

# Click a menu item by path (edit PATH as needed)
run-click PATH="File > Save":
    cargo run -- click "{{PATH}}" --dry-run

# Show state of a menu item by path
run-state PATH="File > Save":
    cargo run -- state "{{PATH}}"

# Clean build artifacts
clean:
    cargo clean

# Time a release build cold-start
bench: release
    hyperfine --warmup 3 \
        'target/release/menucli apps --json' \
        'target/release/menucli list --app Finder --json' \
        'target/release/menucli search "New" --app Finder --json'
