/// `click` command: activate (press) a menu item.
use crate::ax::resolve_target;
use crate::cli::args::ClickArgs;
use crate::cli::output::write_menu_items;
use crate::cli::OutputCtx;
use crate::menu::tree::{build_extras_tree, TreeOptions};
use crate::menu::{build_tree_with_opts, press_node, resolve, MenuError};
use crate::types::MenuItemOutput;

/// Helper to convert a `MenuNode` to `MenuItemOutput`.
fn node_to_output(node: &crate::menu::MenuNode) -> MenuItemOutput {
    MenuItemOutput {
        title: node.title.clone(),
        path: node.path.clone(),
        enabled: node.enabled,
        checked: node.checked,
        shortcut: node.shortcut.clone(),
        role: node.role.clone(),
        children_count: node.children.len(),
        depth: node.depth,
        is_alternate: node.is_alternate,
        alternate_of: node.alternate_of.clone(),
        app_name: None,
        app_pid: None,
    }
}

/// Run `menucli click`.
///
/// # Errors
///
/// Returns `MenuError` on AX failure, missing permissions, unknown app, unresolvable path,
/// or if the item is disabled.
pub fn run(args: &ClickArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
    let tree_opts = TreeOptions {
        include_alternates: ctx.alternates,
    };

    let _t_resolve = ctx.timer("resolve_target");
    let pid = resolve_target(args.app.as_deref()).map_err(MenuError::from)?;
    drop(_t_resolve);

    let tree = if args.extras {
        let _t_tree = ctx.timer("build_extras_tree");
        let t = build_extras_tree(pid, None, &tree_opts)?;
        drop(_t_tree);
        t
    } else {
        let _t_tree = ctx.timer("build_tree");
        let t = build_tree_with_opts(pid, None, &tree_opts)?;
        drop(_t_tree);
        t
    };

    let _t_resolve_path = ctx.timer("resolve_path");
    let node = resolve(&tree, &args.path)?;
    drop(_t_resolve_path);

    let output = node_to_output(node);

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
