/// Command dispatch: routes `Command` enum variants to their implementations.
pub mod apps;
pub mod check_access;
pub mod click;
pub mod list;
pub mod search;
pub mod state;
pub mod toggle;

use crate::cli::OutputCtx;
use crate::cli::args::Command;
use crate::menu::MenuError;

/// Dispatch a parsed `Command` to its handler.
///
/// # Errors
///
/// Returns `MenuError` on any command failure.
pub fn dispatch(command: &Command, ctx: &OutputCtx) -> Result<(), MenuError> {
    match command {
        Command::CheckAccess => check_access::run(ctx),
        Command::Apps(args) => apps::run(args, ctx),
        Command::List(args) => list::run(args, ctx),
        Command::Search(args) => search::run(args, ctx),
        Command::State(args) => state::run(args, ctx),
        Command::Click(args) => click::run(args, ctx),
        Command::Toggle(args) => toggle::run(args, ctx),
    }
}
