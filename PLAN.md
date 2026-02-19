# PLAN: Status Bar Extras + Option-Key Alternates

## Goal

Add two new capabilities to menucli, integrated as flags on existing commands:

1. **`--extras`** — access right-side status bar items (Wi-Fi, Bluetooth, 1Password, etc.) via `kAXExtrasMenuBarAttribute`
2. **`--alternates`** — reveal Option-key alternate menu items (always captured internally, hidden from output by default)

## Architecture

### AX Layer Changes (`src/ax/element.rs`)

- Import `kAXExtrasMenuBarAttribute`, `kAXVisibleChildrenAttribute`, `kAXMenuItemPrimaryUIElementAttribute`
- Add `extras_menu_bar()` method parallel to `menu_bar()`
- Add `visible_children()` method parallel to `children()`
- Add `kAXMenuItemPrimaryUIElementAttribute` to `MENU_ITEM_ATTRS` (always fetched; index `attr_idx::PRIMARY_UI_ELEMENT = 7`)

### Menu Tree Changes (`src/menu/tree.rs`)

- Add `is_alternate: bool` and `alternate_of: Option<String>` fields to `MenuNode`
- Detect alternates during `walk_element` by checking `PRIMARY_UI_ELEMENT` attribute presence
- Add `TreeOptions { include_alternates: bool }` to control filtering during walk
- Filter out alternate items during `collect_children` when `include_alternates = false`
- Add `build_extras_tree(pid, max_depth, opts)` using `extras_menu_bar()` + `visible_children()`
- Add `build_all_extras(max_depth, opts)` that iterates all running apps in parallel

### Flatten Changes (`src/menu/flatten.rs`)

- Add `is_alternate: bool` and `alternate_of: Option<String>` to `FlatItem`

### CLI Args (`src/cli/args.rs`)

- Add `--extras` flag to `ListArgs`, `SearchArgs`, `ClickArgs`, `ToggleArgs`, `StateArgs`
- Add `--alternates` global flag to `Cli`

### Output Types (`src/types.rs`)

- Add `is_alternate`, `alternate_of`, `app_name`, `app_pid` to `MenuItemOutput`
- Add `is_alternate`, `alternate_of` to `MenuTreeOutput` and `SearchResultOutput`

### Command Implementations (`src/commands/*.rs`)

- Each command checks `--extras` flag to decide `build_tree` vs `build_extras_tree`/`build_all_extras`
- Pass `alternates` flag through to tree building options
- `list` with `--extras` and no `--app`: calls `build_all_extras`, adds app attribution to output

### Output Formatting (`src/cli/output.rs`)

- Table: add APP column when extras across all apps; add `[alt]` annotation for alternates
- Tree: show `[alt]` marker for alternate items

### Re-exports (`src/menu/mod.rs`, `src/ax/mod.rs`)

- Export new functions and types

## Continuation Instructions

If implementation is incomplete, resume by:

1. Run `cargo build 2>&1` to see current compilation state
2. Check git diff to see what's already been changed
3. The implementation follows this order:
   - Phase 1: `src/ax/element.rs` — AX methods + attrs
   - Phase 2: `src/menu/tree.rs` — MenuNode fields + TreeOptions + build_extras_tree + build_all_extras
   - Phase 3: `src/menu/flatten.rs` — FlatItem fields
   - Phase 4: `src/types.rs` — output type fields
   - Phase 5: `src/cli/args.rs` — CLI flags
   - Phase 6: `src/main.rs` — pass alternates flag through OutputCtx or dispatch
   - Phase 7: `src/commands/mod.rs` — update dispatch to pass alternates
   - Phase 8: `src/commands/{list,search,click,toggle,state}.rs` — extras/alternates logic
   - Phase 9: `src/cli/output.rs` — formatting updates
   - Phase 10: `src/menu/mod.rs` + `src/ax/mod.rs` — re-exports
   - Phase 11: Update tests in flatten.rs and resolve.rs for new fields
   - Phase 12: `cargo build && cargo test` — fix all errors

## Key Constants (accessibility-sys 0.2.0)

```
kAXExtrasMenuBarAttribute    — "AXExtrasMenuBar"
kAXVisibleChildrenAttribute  — "AXVisibleChildren"  
kAXMenuItemPrimaryUIElementAttribute — "AXMenuItemPrimaryUIElement"
kAXShowAlternateUIAction     — "AXShowAlternateUI"
kAXShowDefaultUIAction       — "AXShowDefaultUI"
```

## Edge Cases

- Apps with no extras: `extras_menu_bar()` returns `AXError::AttributeUnsupported` — skip silently
- Extras that open windows (not menus): click still works, tree walk may find no submenu children
- Alternate items may share titles with primary: use `alternate_of` to disambiguate
- `kAXMenuItemPrimaryUIElementAttribute` returns an element ref, not a path: check for non-None = alternate
- Background helper processes may own extras but not appear in NSWorkspace — accepted limitation
