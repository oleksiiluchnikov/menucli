#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
//! menucli — query and interact with macOS app menu bars.

mod ax;
mod cli;
mod commands;
mod menu;
mod types;

use clap::Parser;

use cli::{Cli, OutputCtx, write_error};
use types::ErrorOutput;

fn main() {
    let cli = Cli::parse();

    let ctx = OutputCtx::new(
        cli.output,
        cli.json,
        cli.fields.as_deref(),
        cli.no_header,
        cli.debug,
    );

    match commands::dispatch(&cli.command, &ctx) {
        Ok(()) => {}
        Err(err) => {
            let error_output = ErrorOutput::from_menu_error(&err);
            write_error(&error_output, cli.output, cli.json);
            std::process::exit(err.exit_code());
        }
    }
}
