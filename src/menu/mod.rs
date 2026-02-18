/// Menu domain layer: tree building, flattening, search, path resolution.
pub mod errors;
pub mod flatten;
pub mod resolve;
pub mod search;
pub mod shortcut;
pub mod tree;

pub use errors::MenuError;
pub use flatten::flatten;
pub use resolve::resolve;
pub use search::{SearchOptions, search};
pub use tree::{MenuNode, build_tree, press_node};
