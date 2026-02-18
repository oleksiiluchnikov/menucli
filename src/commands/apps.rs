/// `apps` command: list running applications with PIDs.
use crate::ax::list_running_apps;
use crate::cli::OutputCtx;
use crate::cli::args::AppsArgs;
use crate::cli::output::write_apps;
use crate::menu::MenuError;
use crate::types::AppInfoOutput;

/// Run `menucli apps`.
///
/// # Errors
///
/// Cannot currently fail; the list may simply be empty.
pub fn run(args: &AppsArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
    let apps = list_running_apps();

    let mut output: Vec<AppInfoOutput> = apps
        .iter()
        .map(|a| AppInfoOutput {
            name: a.name.clone(),
            pid: a.pid,
            bundle_id: a.bundle_id.clone(),
            frontmost: a.frontmost,
        })
        .collect();

    if args.frontmost {
        output.retain(|a| a.frontmost);
    }

    write_apps(&output, ctx);
    Ok(())
}
