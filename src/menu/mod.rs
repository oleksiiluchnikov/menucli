/// Menu domain layer: tree building, flattening, search, path resolution.
pub mod errors;
pub mod flatten;
pub mod resolve;
pub mod search;
pub mod shortcut;
pub mod tree;

pub use errors::MenuError;
pub use flatten::{flatten, FlatItem};
pub use resolve::resolve;
pub use search::{search, SearchOptions};
pub use tree::{
    build_all_extras, build_extras_tree, build_tree, build_tree_with_opts, press_node,
    ExtrasResult, MenuNode, TreeOptions,
};
