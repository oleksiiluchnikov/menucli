/// `state` command: get the current state of a specific menu item.
use crate::ax::resolve_target;
use crate::cli::args::StateArgs;
use crate::cli::output::write_menu_items;
use crate::cli::OutputCtx;
use crate::menu::tree::{build_extras_tree, TreeOptions};
use crate::menu::{build_tree_with_opts, resolve, MenuError};
use crate::types::MenuItemOutput;

/// Run `menucli state`.
///
/// # Errors
///
/// Returns `MenuError` on AX failure, missing permissions, unknown app, or unresolvable path.
pub fn run(args: &StateArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
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

    let output = MenuItemOutput {
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
    };

    write_menu_items(&[output], ctx);
    Ok(())
}
