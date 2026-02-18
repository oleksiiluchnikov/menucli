/// `state` command: get the current state of a specific menu item.
use crate::ax::resolve_target;
use crate::cli::OutputCtx;
use crate::cli::args::StateArgs;
use crate::cli::output::write_menu_items;
use crate::menu::{MenuError, build_tree, resolve};
use crate::types::MenuItemOutput;

/// Run `menucli state`.
///
/// # Errors
///
/// Returns `MenuError` on AX failure, missing permissions, unknown app, or unresolvable path.
pub fn run(args: &StateArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
    let _t_resolve = ctx.timer("resolve_target");
    let pid = resolve_target(args.app.as_deref()).map_err(MenuError::from)?;
    drop(_t_resolve);

    let _t_tree = ctx.timer("build_tree");
    let tree = build_tree(pid, None)?;
    drop(_t_tree);

    let _t_resolve_path = ctx.timer("resolve_path");
    let node = resolve(&tree, &args.path)?;
    drop(_t_resolve_path);

    let output = MenuItemOutput {
        title: node.title.clone(),
        path: node.path.clone(),
        enabled: node.enabled,
        checked: node.checked,
        shortcut: node.shortcut.clone(),
        role: node.role.clone(),
        children_count: node.children.len(),
        depth: node.depth,
    };

    write_menu_items(&[output], ctx);
    Ok(())
}
