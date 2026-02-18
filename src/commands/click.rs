/// `click` command: activate (press) a menu item.
use crate::ax::resolve_target;
use crate::cli::OutputCtx;
use crate::cli::args::ClickArgs;
use crate::cli::output::write_menu_items;
use crate::menu::{MenuError, build_tree, press_node, resolve};
use crate::types::MenuItemOutput;

/// Run `menucli click`.
///
/// # Errors
///
/// Returns `MenuError` on AX failure, missing permissions, unknown app, unresolvable path,
/// or if the item is disabled.
pub fn run(args: &ClickArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
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

    if args.dry_run {
        write_menu_items(&[output], ctx);
        return Ok(());
    }

    let _t_press = ctx.timer("press_node");
    press_node(node)?;
    drop(_t_press);

    write_menu_items(&[output], ctx);
    Ok(())
}
