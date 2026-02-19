/// Public API for the macOS Accessibility layer.
pub mod app;
pub mod element;
pub mod errors;
pub mod permissions;

pub use app::{list_running_apps, resolve_target};
pub use element::{attr_idx, AXElement, AttributeValue, MENU_ITEM_ATTRS};
pub use errors::AXError;
pub use permissions::{ensure_trusted, permission_instructions};
