/// Shared serializable output types for all commands.
///
/// These types are what gets written to stdout — either as JSON or rendered
/// as a table. They are decoupled from the internal `MenuNode` / `FlatItem` types.
use serde::{Deserialize, Serialize};

/// A menu item in flat (list) representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItemOutput {
    /// Display title (leaf name, e.g., "Save As…").
    pub title: String,
    /// Full path from root (e.g., "File::Save As…").
    pub path: String,
    /// Whether the item is enabled (clickable).
    pub enabled: bool,
    /// Whether the item has a checkmark (toggle state = on).
    pub checked: bool,
    /// Formatted keyboard shortcut (e.g., "⇧⌘S"), or null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    /// AX role string (e.g., "AXMenuItem", "AXMenuBarItem").
    pub role: String,
    /// Number of direct children.
    pub children_count: usize,
    /// Depth from root (1 = top-level menu bar item, 2+ = nested).
    pub depth: usize,
}

/// A menu item in tree representation (nested).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuTreeOutput {
    /// Display title.
    pub title: String,
    /// Full path from root.
    pub path: String,
    /// Whether the item is enabled.
    pub enabled: bool,
    /// Whether the item has a checkmark.
    pub checked: bool,
    /// Formatted keyboard shortcut, or null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    /// AX role string.
    pub role: String,
    /// Nested children.
    pub children: Vec<MenuTreeOutput>,
}

/// A search result with match score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultOutput {
    /// The matched item's title.
    pub title: String,
    /// The matched item's full path.
    pub path: String,
    /// Whether the item is enabled.
    pub enabled: bool,
    /// Whether the item has a checkmark.
    pub checked: bool,
    /// Formatted keyboard shortcut, or null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    /// Match score (higher = better). 0 for exact matches.
    pub score: u32,
}

/// Running application info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfoOutput {
    /// Localized app name.
    pub name: String,
    /// Process ID.
    pub pid: i32,
    /// Bundle identifier, or null if unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    /// Whether this is the frontmost application.
    pub frontmost: bool,
}

/// Result of a toggle operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleOutput {
    /// Full path of the toggled item.
    pub path: String,
    /// Checkmark state before the toggle.
    pub checked_before: bool,
    /// Checkmark state after the toggle (or same as before on `--dry-run`).
    pub checked_after: bool,
    /// Whether this was a dry-run (no actual action performed).
    pub dry_run: bool,
}

/// A structured error envelope for JSON error output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorOutput {
    /// Always `false`.
    pub ok: bool,
    /// Error details.
    pub error: ErrorDetail,
}

/// Error detail in the JSON error envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Machine-readable error code (snake_case).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional list of candidates (for ambiguous match errors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<String>>,
}

impl ErrorOutput {
    /// Construct from a `MenuError`.
    #[must_use]
    pub fn from_menu_error(err: &crate::menu::MenuError) -> Self {
        use crate::menu::MenuError;
        let (code, message, candidates) = match err {
            MenuError::AccessDenied => ("permission_denied".to_owned(), err.to_string(), None),
            MenuError::AppNotFound { .. } => ("app_not_found".to_owned(), err.to_string(), None),
            MenuError::ItemNotFound { .. } => ("item_not_found".to_owned(), err.to_string(), None),
            MenuError::AmbiguousMatch { candidates, .. } => (
                "ambiguous_match".to_owned(),
                err.to_string(),
                Some(candidates.clone()),
            ),
            MenuError::ItemDisabled { .. } => ("item_disabled".to_owned(), err.to_string(), None),
            MenuError::NotToggleable { .. } => ("not_toggleable".to_owned(), err.to_string(), None),
            MenuError::AX(_) => ("ax_error".to_owned(), err.to_string(), None),
        };
        Self {
            ok: false,
            error: ErrorDetail {
                code,
                message,
                candidates,
            },
        }
    }
}
