/// CLI layer: argument parsing and output formatting.
pub mod args;
pub mod output;

pub use args::{Cli, OutputFormat};
pub use output::{write_error, OutputCtx};
