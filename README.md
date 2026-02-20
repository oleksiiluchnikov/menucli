# menucli

Query and interact with macOS app menu bars from the CLI via the Accessibility API.

Built for LLM agents, Hammerspoon replacements, and keyboard-driven workflows. Walks any app's menu tree, fuzzy-resolves items by partial name, clicks them, toggles checkmarks, and reads state -- all from the terminal. Structured JSON output, deterministic exit codes, and TTY-aware formatting.

## Features

- **Full menu tree walking** -- list every menu item for any running app, nested or flat
- **Fuzzy resolution** -- `menucli click "save as"` just works (smart-case, auto-disambiguation)
- **Status bar extras** -- access right-side menu bar items (Wi-Fi, Bluetooth, 1Password, Raycast) via `--extras`
- **Option-key alternates** -- reveal hidden alternate items with `--alternates`
- **7 output formats**: json, compact, ndjson, table, path, id, auto
- **TTY auto-detection** -- table for humans, JSON for pipes (zero flags needed)
- **Field projection** (`--fields title,path,shortcut`) to limit output
- **`--dry-run`** on click/toggle -- preview resolved item without acting
- **Toggle verification** -- re-reads AX state with exponential backoff to confirm toggle took effect
- **App targeting** by name, PID, or bundle ID -- defaults to frontmost app
- **Parallel tree walking** -- top-level menu bar items walked concurrently via `std::thread::scope`
- **Batch AX fetching** -- single IPC round-trip per element via `AXUIElementCopyMultipleAttributeValues`
- **Deterministic exit codes** -- `0` success, `1` error, `2` not found, `3` ambiguous, `10` no permission

## Install

Requires Rust and macOS.

```sh
git clone https://github.com/oleksiiluchnikov/menucli.git
cd menucli
cargo install --path .
```

### Prerequisites

macOS Accessibility permission is required. Grant it in:

**System Settings > Privacy & Security > Accessibility**

Add your terminal app (Ghostty, iTerm2, Terminal.app, etc.) to the list.

```sh
# Check if permission is granted
menucli check-access
```

## Usage

```sh
# List all menu items for the frontmost app (table in TTY, JSON when piped)
menucli list

# List menu items for a specific app
menucli list --app Finder

# Flat list with full paths
menucli list --flat --app Safari

# Search for a menu item
menucli search "save" --app Finder

# Click a menu item by fuzzy match
menucli click "save as" --app TextEdit

# Click by exact path
menucli click "File::Save As…" --app TextEdit --exact

# Dry run -- preview what would be clicked
menucli click "save as" --app TextEdit --dry-run

# Toggle a checkmark item and report new state
menucli toggle "View::Show Sidebar" --app Finder

# Get current state of a menu item
menucli state "View::Show Path Bar" --app Finder

# List running apps with PIDs
menucli apps
```

### Status Bar Extras

Access right-side menu bar items (Wi-Fi, Bluetooth, 1Password, etc.):

```sh
# List all status bar items from all running apps
menucli list --extras

# Filter to a specific app's status bar items
menucli list --extras --app Raycast

# Search within status bar items
menucli search "Pull" --extras --app Raycast

# Click a status bar menu item
menucli click "Open My Pull Requests" --extras --app Raycast
```

### Option-Key Alternates

Reveal hidden alternate menu items (the ones you see when holding Option):

```sh
# Show alternates alongside regular items
menucli --alternates list --app Finder

# Search including alternates
menucli --alternates search "System" --app Finder
```

### Output Formats

```sh
# JSON (explicit)
menucli list --app Finder --json

# Compact JSON (single line)
menucli list --app Finder --output compact

# Newline-delimited JSON
menucli list --app Finder --output ndjson

# Paths only (for piping)
menucli list --app Finder --output path

# Field projection
menucli list --app Finder --json --fields title,path,shortcut

# No table headers (for awk/cut)
menucli list --app Finder --output table --no-header
```

### Scripting Examples

```sh
# Click the first enabled menu item matching "new"
menucli search "new" --app Finder --output path --limit 1 | xargs -I{} menucli click "{}" --app Finder --exact

# List all keyboard shortcuts
menucli list --app Finder --flat --json --fields path,shortcut | jq '.[] | select(.shortcut != null)'

# Check if Dark Mode is on
menucli state "View::Use Dark Background" --app Terminal --json | jq '.checked'

# List all apps with status bar items
menucli list --extras --output table
```

## Architecture

```
src/
├── ax/           # macOS Accessibility API layer (AXUIElement FFI)
│   ├── element.rs  # AXElement wrapper, batch attribute fetching
│   ├── app.rs      # Running app resolution (name/PID/bundle ID)
│   └── errors.rs   # AX-level errors
├── menu/         # Domain logic
│   ├── tree.rs     # Recursive tree builder (parallel, with extras + alternates)
│   ├── flatten.rs  # Tree → flat list conversion
│   ├── search.rs   # Fuzzy + exact search
│   ├── resolve.rs  # Path/query → single node resolution
│   └── shortcut.rs # Keyboard shortcut formatting
├── commands/     # CLI command handlers
│   ├── list.rs, search.rs, click.rs, toggle.rs, state.rs, apps.rs
│   └── check_access.rs
├── cli/          # Argument parsing + output formatting
│   ├── args.rs     # clap derive definitions
│   └── output.rs   # Format dispatch (JSON, table, path, etc.)
├── types.rs      # Serializable output types (serde)
└── main.rs       # Entry point + error handling
```

## License

MIT
