/// Recursive menu tree builder using the AX layer.
///
/// Performance strategy:
/// 1. Use batch attribute fetching (`AXUIElementCopyMultipleAttributeValues`) to
///    read all needed attributes per item in one IPC round-trip.
/// 2. Walk top-level menu bar items in parallel using `std::thread::scope`.
/// 3. Recurse into submenus only within each thread.
use accessibility_sys::kAXPressAction;

use crate::ax::app::{list_running_apps, RunningApp};
use crate::ax::{attr_idx, AXElement, AttributeValue, MENU_ITEM_ATTRS};
use crate::menu::shortcut::format_shortcut;

use super::errors::MenuError;

/// Path separator used in full item paths.
///
/// Double-colon `::` is compact, shell-friendly (no quoting needed for simple
/// paths like `File::Save`), and unambiguous in practice — real menu titles
/// almost never contain `::`. When they do, `escape_title` replaces the
/// literal `::` with `\::` so round-tripping is lossless.
pub const PATH_SEP: &str = "::";

/// Escape literal `::` in a menu title so it won't be confused with [`PATH_SEP`].
///
/// Titles containing `::` (extremely rare) get it replaced with `\::`.
/// Titles without `::` are returned as-is (zero allocation).
#[must_use]
pub fn escape_title(title: &str) -> std::borrow::Cow<'_, str> {
    if title.contains(PATH_SEP) {
        std::borrow::Cow::Owned(title.replace("::", "\\::"))
    } else {
        std::borrow::Cow::Borrowed(title)
    }
}

/// Split a full menu path on the unescaped `::` separator.
///
/// `\::` inside a segment is preserved (not treated as a split point).
/// After splitting, call [`unescape_segment`] on each piece to get the raw title.
#[must_use]
pub fn split_path(path: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let bytes = path.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if i + 2 <= len && &bytes[i..i + 2] == b"::" {
            // Check if preceded by backslash (escaped).
            if i > 0 && bytes[i - 1] == b'\\' {
                // Escaped `\::` — skip, not a separator.
                i += 2;
            } else {
                segments.push(&path[start..i]);
                i += 2;
                start = i;
            }
        } else {
            i += 1;
        }
    }
    segments.push(&path[start..]);
    segments
}

/// Remove escape sequences from a single path segment.
///
/// Converts `\::` back to `::`.
#[must_use]
pub fn unescape_segment(seg: &str) -> std::borrow::Cow<'_, str> {
    if seg.contains("\\::") {
        std::borrow::Cow::Owned(seg.replace("\\::", "::"))
    } else {
        std::borrow::Cow::Borrowed(seg)
    }
}

/// A node in the menu tree.
#[derive(Debug, Clone)]
pub struct MenuNode {
    /// Display title of the item (e.g., "Save As…").
    pub title: String,
    /// Full path from root (e.g., "File::Save As…").
    pub path: String,
    /// Whether the item is enabled (clickable).
    pub enabled: bool,
    /// Whether the item has a checkmark (toggle state = on).
    pub checked: bool,
    /// Formatted keyboard shortcut (e.g., "⇧⌘S"), if any.
    pub shortcut: Option<String>,
    /// AX role string (e.g., "AXMenuBarItem", "AXMenuItem").
    pub role: String,
    /// Depth from root (menu bar = 0, top-level items = 1, submenu items = 2+).
    pub depth: usize,
    /// Child nodes (empty for leaf items).
    pub children: Vec<MenuNode>,
    /// The underlying AX element, kept for `click` / `toggle` operations.
    /// `None` only in unit-test fixtures that never call press/toggle.
    pub element: Option<AXElement>,
    /// Whether this item is an Option-key alternate of another item.
    pub is_alternate: bool,
    /// If this item is an alternate, the title of the primary item it replaces.
    pub alternate_of: Option<String>,
}

/// Options for tree building.
#[derive(Debug, Clone)]
pub struct TreeOptions {
    /// Whether to include alternate (Option-key) items in the output.
    /// Alternates are always detected internally; this controls filtering.
    pub include_alternates: bool,
}

/// Build the full menu tree for an application, given its PID.
///
/// Convenience wrapper around [`build_tree_with_opts`] with alternates excluded.
/// Top-level menu bar items are walked in parallel threads.
///
/// # Errors
///
/// Returns `MenuError` if the AX API fails or permissions are missing.
#[allow(dead_code)]
pub fn build_tree(pid: i32, max_depth: Option<usize>) -> Result<Vec<MenuNode>, MenuError> {
    build_tree_with_opts(
        pid,
        max_depth,
        &TreeOptions {
            include_alternates: false,
        },
    )
}

/// Build the full menu tree with options controlling alternate item inclusion.
///
/// # Errors
///
/// Returns `MenuError` if the AX API fails or permissions are missing.
pub fn build_tree_with_opts(
    pid: i32,
    max_depth: Option<usize>,
    opts: &TreeOptions,
) -> Result<Vec<MenuNode>, MenuError> {
    let app = AXElement::application(pid);
    let menubar = app.menu_bar()?;
    let top_level = menubar.children()?;

    if top_level.is_empty() {
        return Ok(Vec::new());
    }

    let include_alternates = opts.include_alternates;

    // Walk each top-level item in parallel (one thread per top-level menu).
    let mut trees: Vec<Option<MenuNode>> = vec![None; top_level.len()];

    std::thread::scope(|s| {
        let handles: Vec<_> = top_level
            .into_iter()
            .enumerate()
            .map(|(i, element)| {
                s.spawn(move || {
                    let node =
                        walk_element(element, String::new(), 1, max_depth, include_alternates);
                    (i, node)
                })
            })
            .collect();

        for handle in handles {
            if let Ok((i, Ok(node))) =
                handle.join() as Result<(usize, Result<MenuNode, MenuError>), _>
            {
                trees[i] = Some(node);
            }
        }
    });

    Ok(trees.into_iter().flatten().collect())
}

/// Recursively walk a menu element and its children.
fn walk_element(
    element: AXElement,
    parent_path: String,
    depth: usize,
    max_depth: Option<usize>,
    include_alternates: bool,
) -> Result<MenuNode, MenuError> {
    // Batch-fetch all needed attributes in one IPC call.
    let attrs = element.batch_attributes(MENU_ITEM_ATTRS)?;

    let title = extract_string(&attrs, attr_idx::TITLE).unwrap_or_default();
    let enabled = extract_bool(&attrs, attr_idx::ENABLED).unwrap_or(true);
    let mark_char = extract_string(&attrs, attr_idx::MARK_CHAR);
    let cmd_char = extract_string(&attrs, attr_idx::CMD_CHAR);
    let cmd_mods = extract_number(&attrs, attr_idx::CMD_MODIFIERS);
    let role = extract_string(&attrs, attr_idx::ROLE).unwrap_or_default();

    // Detect alternate items: if PRIMARY_UI_ELEMENT is present (non-None),
    // this item is an Option-key alternate of another item.
    let is_alternate = attrs
        .get(attr_idx::PRIMARY_UI_ELEMENT)
        .is_some_and(|v| v.is_some());

    // A checkmark is indicated by a non-empty mark character (typically "✓" or "–").
    let checked = mark_char.as_deref().is_some_and(|s| !s.is_empty());

    let shortcut = format_shortcut(cmd_char.as_deref(), cmd_mods);

    let escaped = escape_title(&title);
    let path = if parent_path.is_empty() {
        escaped.into_owned()
    } else {
        format!("{parent_path}{PATH_SEP}{escaped}")
    };

    // Recurse into children unless at max depth.
    let children = if max_depth.is_none_or(|max| depth < max) {
        collect_children(&element, &path, depth, max_depth, include_alternates)
    } else {
        Vec::new()
    };

    Ok(MenuNode {
        title,
        path,
        enabled,
        checked,
        shortcut,
        role,
        depth,
        children,
        element: Some(element),
        is_alternate,
        alternate_of: None, // Populated during collect_children
    })
}

/// Collect child menu nodes from an element.
///
/// AXMenu containers (role = "AXMenu") are transparent: we skip the container
/// node itself and recurse directly into its children. This handles the macOS
/// AX menu hierarchy:
///
/// ```text
/// AXMenuBarItem ("File")
///   └── AXMenu          ← transparent container, skipped
///         ├── AXMenuItem ("New")
///         ├── AXMenuItem ("Open")
///         └── AXMenuItem ("Save")
///               └── AXMenu    ← nested submenu container, also skipped
///                     └── AXMenuItem ("Save As…")
/// ```
fn collect_children(
    element: &AXElement,
    parent_path: &str,
    parent_depth: usize,
    max_depth: Option<usize>,
    include_alternates: bool,
) -> Vec<MenuNode> {
    let child_elements = match element.children() {
        Ok(children) => children,
        Err(_) => return Vec::new(),
    };

    let mut child_nodes: Vec<MenuNode> = Vec::with_capacity(child_elements.len());
    // Track the last non-alternate item title so we can set `alternate_of`.
    let mut last_primary_title: Option<String> = None;

    for child in child_elements {
        // Peek at the role to detect AXMenu containers.
        let role = child
            .batch_attributes(&[accessibility_sys::kAXRoleAttribute])
            .ok()
            .and_then(|a| extract_string(&a, 0));

        if role.as_deref() == Some("AXMenu") {
            // AXMenu is a transparent container — recurse through it without
            // incrementing depth or creating a node.
            let grandchildren = collect_children(
                &child,
                parent_path,
                parent_depth,
                max_depth,
                include_alternates,
            );
            child_nodes.extend(grandchildren);
            // Reset last_primary_title since we merged grandchildren.
            last_primary_title = None;
        } else if let Ok(mut node) = walk_element(
            child,
            parent_path.to_owned(),
            parent_depth + 1,
            max_depth,
            include_alternates,
        ) {
            // Skip separator items (empty title or role AXSeparator).
            if !node.title.is_empty() && node.role != "AXSeparator" {
                if node.is_alternate {
                    // Set alternate_of to the last primary item's title.
                    node.alternate_of = last_primary_title.clone();
                    if include_alternates {
                        child_nodes.push(node);
                    }
                    // Don't update last_primary_title for alternates.
                } else {
                    last_primary_title = Some(node.title.clone());
                    child_nodes.push(node);
                }
            }
        }
    }

    child_nodes
}

/// Perform the AX press action on a `MenuNode`.
///
/// # Errors
///
/// Returns `MenuError::ItemDisabled` if the item is disabled.
/// Returns `MenuError::AX` for underlying AX failures.
pub fn press_node(node: &MenuNode) -> Result<(), MenuError> {
    if !node.enabled {
        return Err(MenuError::ItemDisabled {
            path: node.path.clone(),
        });
    }
    let element = node
        .element
        .as_ref()
        .ok_or(MenuError::AX(crate::ax::errors::AXError::InvalidElement))?;
    // SAFETY: kAXPressAction is a valid action constant.
    element.perform_action(kAXPressAction)?;
    Ok(())
}

/// An extras tree result, associating menu nodes with the owning app.
#[derive(Debug, Clone)]
pub struct ExtrasResult {
    /// Name of the app that owns these extras.
    pub app_name: String,
    /// PID of the owning app.
    pub app_pid: i32,
    /// Menu nodes for the extras items.
    pub nodes: Vec<MenuNode>,
}

/// Build the extras (status bar) tree for a single app, given its PID.
///
/// Uses `visible_children` to respect menu bar managers (Bartender/Ice).
///
/// # Errors
///
/// Returns `MenuError` if the AX API fails or the app has no extras.
pub fn build_extras_tree(
    pid: i32,
    max_depth: Option<usize>,
    opts: &TreeOptions,
) -> Result<Vec<MenuNode>, MenuError> {
    let app = AXElement::application(pid);
    let extras_bar = app.extras_menu_bar()?;
    // Use visible_children to respect system hiding (Bartender/Ice).
    let top_level = extras_bar
        .visible_children()
        .or_else(|_| extras_bar.children())?;

    if top_level.is_empty() {
        return Ok(Vec::new());
    }

    let include_alternates = opts.include_alternates;

    let mut nodes = Vec::with_capacity(top_level.len());
    for element in top_level {
        match walk_element(element, String::new(), 1, max_depth, include_alternates) {
            Ok(node) => {
                if !node.title.is_empty() {
                    nodes.push(node);
                }
            }
            Err(_) => continue,
        }
    }

    Ok(nodes)
}

/// Build extras trees for all running apps.
///
/// Iterates all running apps, collecting extras from each. Apps without extras
/// are silently skipped.
pub fn build_all_extras(max_depth: Option<usize>, opts: &TreeOptions) -> Vec<ExtrasResult> {
    let apps: Vec<RunningApp> = list_running_apps();

    let mut results = Vec::new();
    for app in &apps {
        if let Ok(nodes) = build_extras_tree(app.pid, max_depth, opts) {
            if !nodes.is_empty() {
                results.push(ExtrasResult {
                    app_name: app.name.clone(),
                    app_pid: app.pid,
                    nodes,
                });
            }
        }
    }

    results
}

// --- Attribute extraction helpers ---

fn extract_string(attrs: &[Option<AttributeValue>], idx: usize) -> Option<String> {
    match attrs.get(idx)?.as_ref()? {
        AttributeValue::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn extract_bool(attrs: &[Option<AttributeValue>], idx: usize) -> Option<bool> {
    match attrs.get(idx)?.as_ref()? {
        AttributeValue::Bool(b) => Some(*b),
        _ => None,
    }
}

fn extract_number(attrs: &[Option<AttributeValue>], idx: usize) -> Option<i64> {
    match attrs.get(idx)?.as_ref()? {
        AttributeValue::Number(n) => Some(*n),
        _ => None,
    }
}
