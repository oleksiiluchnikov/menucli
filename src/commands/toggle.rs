/// `toggle` command: toggle a checkmark menu item and report the new state.
use crate::ax::resolve_target;
use crate::cli::OutputCtx;
use crate::cli::args::ToggleArgs;
use crate::cli::output::write_toggle;
use crate::menu::{MenuError, build_tree, press_node, resolve};
use crate::types::ToggleOutput;

/// Run `menucli toggle`.
///
/// # Errors
///
/// Returns `MenuError::NotToggleable` if the item has no checkmark state.
/// Returns `MenuError::ItemDisabled` if the item is not clickable.
/// Returns `MenuError` on AX failure, missing permissions, or unknown app.
pub fn run(args: &ToggleArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
    let _t_resolve = ctx.timer("resolve_target");
    let pid = resolve_target(args.app.as_deref()).map_err(MenuError::from)?;
    drop(_t_resolve);

    let _t_tree = ctx.timer("build_tree[1]");
    let tree = build_tree(pid, None)?;
    drop(_t_tree);

    let _t_resolve_path = ctx.timer("resolve_path");
    let node = resolve(&tree, &args.path)?;
    drop(_t_resolve_path);

    let checked_before = node.checked;
    let path = node.path.clone();

    if args.dry_run {
        let output = ToggleOutput {
            path,
            checked_before,
            checked_after: checked_before,
            dry_run: true,
        };
        write_toggle(&output, ctx);
        return Ok(());
    }

    let _t_press = ctx.timer("press_node");
    press_node(node)?;
    drop(_t_press);

    // Rebuild the tree to read the new checked state.
    let _t_tree2 = ctx.timer("build_tree[2]");
    let tree2 = build_tree(pid, None)?;
    drop(_t_tree2);

    let node2 = resolve(&tree2, &args.path)?;
    let checked_after = node2.checked;

    let output = ToggleOutput {
        path,
        checked_before,
        checked_after,
        dry_run: false,
    };

    write_toggle(&output, ctx);
    Ok(())
}
