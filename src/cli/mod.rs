/// CLI layer: argument parsing and output formatting.
pub mod args;
pub mod output;

pub use args::{Cli, OutputFormat};
pub use output::{OutputCtx, write_error};
