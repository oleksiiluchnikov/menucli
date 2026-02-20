# menucli

Click any macOS menu item from the terminal. Status bar too.

<!-- TODO: Add demo GIF showing `menucli click "save as" --app TextEdit --dry-run` -->

```sh
$ menucli click "save as" --app TextEdit --dry-run
```
```json
{
  "title": "Save As…",
  "path": "File::Save As…",
  "enabled": true,
  "shortcut": "⇧⌘S",
  "role": "AXMenuItem"
}
```

Type a partial name. menucli fuzzy-matches it, resolves the full path, and clicks it. Or just previews what would happen.

## Why menucli?

### Click any menu item by name

No more mousing through nested menus. Type what you want, menucli finds it.

```sh
# Fuzzy match -- "save as" resolves to "File::Save As…"
menucli click "save as" --app TextEdit

# Exact path when you need precision
menucli click "File::Save As…" --app TextEdit --exact

# Preview first, click later
menucli click "save as" --app TextEdit --dry-run

# Target any app by name, PID, or bundle ID
menucli click "Preferences…" --app com.apple.Safari
```

### Access the status bar

Wi-Fi, Bluetooth, 1Password, Raycast -- anything with a menu bar icon is scriptable.

```sh
# List every status bar item across all running apps
menucli list --extras

# Filter to a single app
menucli list --extras --app Raycast

# Search and click status bar items
menucli search "Pull" --extras --app Raycast
menucli click "Open My Pull Requests" --extras --app Raycast
```

No other CLI tool does this.

### Script toggles and read state

Toggle checkmark menu items and verify the result. menucli re-reads the actual AX state with exponential backoff -- no guessing.

```sh
# Toggle a setting and get the new state
menucli toggle "View::Show Sidebar" --app Finder

# Read current state without changing it
menucli state "View::Show Path Bar" --app Finder

# Check state in scripts
menucli state "View::Show Path Bar" --app Finder --json | jq '.checked'
```

### Reveal hidden alternate items

macOS hides Option-key alternates (e.g., "About This Mac" has a hidden "System Information…"). Surface them all:

```sh
menucli --alternates list --app Finder
menucli --alternates search "System" --app Finder
```

## Install

Requires Rust and macOS.

```sh
# From GitHub
cargo install --git https://github.com/oleksiiluchnikov/menucli.git

# Or clone and build
git clone https://github.com/oleksiiluchnikov/menucli.git
cd menucli
cargo install --path .
```

### Accessibility permission

menucli uses the macOS Accessibility API. Grant permission in:

**System Settings > Privacy & Security > Accessibility**

Add your terminal app (Ghostty, iTerm2, Terminal.app, etc.).

```sh
# Check if permission is granted
menucli check-access
```

## Quick Start

```sh
# List all menu items for the frontmost app
menucli list

# List a specific app's menus
menucli list --app Finder

# Search for a menu item
menucli search "save" --app Finder

# Click it
menucli click "save" --app Finder

# List running apps
menucli apps
```

## Agent-friendly by design

Built for LLM agents, shell scripts, and CI pipelines. Zero interactive prompts, ever.

- **Structured JSON on stdout** -- machine-parseable, no human prose mixed in
- **TTY auto-detection** -- table for humans, JSON for pipes (zero flags needed)
- **7 output formats** -- json, compact, ndjson, table, path, id, auto
- **Field projection** -- `--fields title,path,shortcut` to limit output
- **`--dry-run`** -- preview resolved items without acting
- **`--no-header`** -- strip table headers for awk/cut pipelines
- **Errors on stderr as JSON** -- agents parse errors the same way they parse results
- **Zero config** -- no setup, no auth, no config files. Install and run.

### Output formats

| Format | When to use | Example |
|--------|-------------|---------|
| `json` | Piping to jq, agent consumption | `[{"title":"Save","path":"File::Save"}]` |
| `compact` | Minimal JSON, single line | Same, no whitespace |
| `ndjson` | Streaming, large datasets | One JSON object per line |
| `table` | Human reading (default in terminal) | Aligned columns with headers |
| `path` | Piping paths to other commands | `File::Save As…\n` |
| `id` | Titles only | `Save As…\n` |

### Pipe composition

```sh
# List all keyboard shortcuts in an app
menucli list --app Safari --flat --json --fields path,shortcut \
  | jq '.[] | select(.shortcut != null)'

# Click the top search result
menucli search "new" --app Finder --output path --limit 1 \
  | xargs -I{} menucli click "{}" --app Finder --exact

# List all apps that expose status bar items
menucli list --extras --output table

# Check a toggle state in a script
if menucli state "View::Show Sidebar" --app Finder --json | jq -e '.checked' > /dev/null; then
  echo "Sidebar is visible"
fi
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Runtime error |
| 2 | Item not found |
| 3 | Ambiguous match (multiple candidates) |
| 10 | Accessibility permission not granted |

## License

[MIT](LICENSE)
