/// `list` command: list all menu items for an application.
use crate::ax::resolve_target;
use crate::cli::OutputCtx;
use crate::cli::args::ListArgs;
use crate::cli::output::{write_menu_items, write_menu_tree};
use crate::menu::{MenuError, build_tree, flatten};
use crate::types::{MenuItemOutput, MenuTreeOutput};

/// Run `menucli list`.
///
/// # Errors
///
/// Returns `MenuError` on AX failure, missing permissions, or unknown app.
pub fn run(args: &ListArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
    let _t_resolve = ctx.timer("resolve_target");
    let pid = resolve_target(args.app.as_deref()).map_err(MenuError::from)?;
    drop(_t_resolve);

    let _t_tree = ctx.timer("build_tree");
    let tree = build_tree(pid, args.depth)?;
    drop(_t_tree);

    // Decide flat vs tree output:
    // --flat forces flat; --tree forces tree; default: flat (pipe-friendly).
    let use_tree = args.tree && !args.flat;

    if use_tree {
        let nodes: Vec<MenuTreeOutput> = tree.iter().map(node_to_tree_output).collect();
        write_menu_tree(&nodes, ctx);
    } else {
        let _t_flatten = ctx.timer("flatten");
        let mut items: Vec<MenuItemOutput> = flatten(&tree)
            .into_iter()
            .map(|f| MenuItemOutput {
                title: f.title,
                path: f.path,
                enabled: f.enabled,
                checked: f.checked,
                shortcut: f.shortcut,
                role: f.role,
                children_count: f.children_count,
                depth: f.depth,
            })
            .collect();
        drop(_t_flatten);

        if args.enabled_only {
            items.retain(|i| i.enabled);
        }

        write_menu_items(&items, ctx);
    }

    Ok(())
}

fn node_to_tree_output(node: &crate::menu::MenuNode) -> MenuTreeOutput {
    MenuTreeOutput {
        title: node.title.clone(),
        path: node.path.clone(),
        enabled: node.enabled,
        checked: node.checked,
        shortcut: node.shortcut.clone(),
        role: node.role.clone(),
        children: node.children.iter().map(node_to_tree_output).collect(),
    }
}
