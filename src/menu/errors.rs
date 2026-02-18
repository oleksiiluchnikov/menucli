/// Errors from the menu domain layer.
use thiserror::Error;

use crate::ax::AXError;

/// Errors that can occur while working with menu trees.
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum MenuError {
    /// Accessibility permission not granted.
    #[error("Accessibility permission not granted")]
    AccessDenied,

    /// No running application matched the identifier.
    #[error("No running application matches '{identifier}'")]
    AppNotFound {
        /// The searched identifier.
        identifier: String,
    },

    /// No menu item matched the query or path.
    #[error("No menu item matches '{query}'")]
    ItemNotFound {
        /// The searched query or path.
        query: String,
    },

    /// Multiple menu items matched with similar confidence; cannot auto-resolve.
    #[error("Ambiguous match for '{query}'. Candidates:\n{}", candidates.join("\n  "))]
    AmbiguousMatch {
        /// The searched query.
        query: String,
        /// Full paths of all candidates that matched.
        candidates: Vec<String>,
    },

    /// The menu item matched but is disabled and cannot be activated.
    #[error("Menu item '{path}' is disabled")]
    ItemDisabled {
        /// Full path of the disabled item.
        path: String,
    },

    /// The menu item does not have a checkmark and cannot be toggled.
    #[error("Menu item '{path}' is not a toggleable (checkmark) item")]
    NotToggleable {
        /// Full path of the non-toggleable item.
        path: String,
    },

    /// An underlying AX API error.
    #[error("Accessibility API error: {0}")]
    AX(#[from] AXError),
}

/// Exit code mapping for `MenuError` variants.
impl MenuError {
    /// Return the CLI exit code for this error.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::AccessDenied => 3,
            Self::AppNotFound { .. } | Self::ItemNotFound { .. } | Self::AmbiguousMatch { .. } => 4,
            Self::ItemDisabled { .. } | Self::NotToggleable { .. } => 1,
            Self::AX(ax) => match ax {
                AXError::NotTrusted => 3,
                _ => 1,
            },
        }
    }
}
