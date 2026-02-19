/// `list` command: list all menu items for an application.
use crate::ax::resolve_target;
use crate::cli::args::ListArgs;
use crate::cli::output::{write_menu_items, write_menu_tree};
use crate::cli::OutputCtx;
use crate::menu::tree::{build_all_extras, build_extras_tree, TreeOptions};
use crate::menu::{build_tree_with_opts, flatten, MenuError, MenuNode};
use crate::types::{MenuItemOutput, MenuTreeOutput};

/// Run `menucli list`.
///
/// # Errors
///
/// Returns `MenuError` on AX failure, missing permissions, or unknown app.
pub fn run(args: &ListArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
    let opts = TreeOptions {
        include_alternates: ctx.alternates,
    };

    if args.extras {
        return run_extras(args, ctx, &opts);
    }

    let _t_resolve = ctx.timer("resolve_target");
    let pid = resolve_target(args.app.as_deref()).map_err(MenuError::from)?;
    drop(_t_resolve);

    let _t_tree = ctx.timer("build_tree");
    let tree = build_tree_with_opts(pid, args.depth, &opts)?;
    drop(_t_tree);

    output_tree(&tree, args, ctx, None)
}

fn run_extras(args: &ListArgs, ctx: &OutputCtx, opts: &TreeOptions) -> Result<(), MenuError> {
    if let Some(app) = &args.app {
        // Single app extras
        let _t_resolve = ctx.timer("resolve_target");
        let pid = resolve_target(Some(app.as_str())).map_err(MenuError::from)?;
        drop(_t_resolve);

        let _t_tree = ctx.timer("build_extras_tree");
        let tree = build_extras_tree(pid, args.depth, opts)?;
        drop(_t_tree);

        output_tree(&tree, args, ctx, None)
    } else {
        // All apps extras
        let _t_tree = ctx.timer("build_all_extras");
        let results = build_all_extras(args.depth, opts);
        drop(_t_tree);

        // Flatten all results into a single list with app attribution.
        let use_tree = args.tree && !args.flat;

        if use_tree {
            // For tree output, show each app's extras separately.
            for result in &results {
                let nodes: Vec<MenuTreeOutput> =
                    result.nodes.iter().map(node_to_tree_output).collect();
                if !nodes.is_empty() {
                    println!("--- {} (pid {}) ---", result.app_name, result.app_pid);
                    write_menu_tree(&nodes, ctx);
                }
            }
            Ok(())
        } else {
            let mut items: Vec<MenuItemOutput> = Vec::new();
            for result in &results {
                let flat = flatten(&result.nodes);
                for f in flat {
                    items.push(flat_to_output(
                        f,
                        Some(&result.app_name),
                        Some(result.app_pid),
                    ));
                }
            }

            if args.enabled_only {
                items.retain(|i| i.enabled);
            }

            write_menu_items(&items, ctx);
            Ok(())
        }
    }
}

fn output_tree(
    tree: &[MenuNode],
    args: &ListArgs,
    ctx: &OutputCtx,
    app_info: Option<(&str, i32)>,
) -> Result<(), MenuError> {
    let use_tree = args.tree && !args.flat;

    if use_tree {
        let nodes: Vec<MenuTreeOutput> = tree.iter().map(node_to_tree_output).collect();
        write_menu_tree(&nodes, ctx);
    } else {
        let _t_flatten = ctx.timer("flatten");
        let mut items: Vec<MenuItemOutput> = flatten(tree)
            .into_iter()
            .map(|f| flat_to_output(f, app_info.map(|(n, _)| n), app_info.map(|(_, p)| p)))
            .collect();
        drop(_t_flatten);

        if args.enabled_only {
            items.retain(|i| i.enabled);
        }

        write_menu_items(&items, ctx);
    }

    Ok(())
}

fn flat_to_output(
    f: crate::menu::FlatItem,
    app_name: Option<&str>,
    app_pid: Option<i32>,
) -> MenuItemOutput {
    MenuItemOutput {
        title: f.title,
        path: f.path,
        enabled: f.enabled,
        checked: f.checked,
        shortcut: f.shortcut,
        role: f.role,
        children_count: f.children_count,
        depth: f.depth,
        is_alternate: f.is_alternate,
        alternate_of: f.alternate_of,
        app_name: app_name.map(str::to_owned),
        app_pid,
    }
}

fn node_to_tree_output(node: &MenuNode) -> MenuTreeOutput {
    MenuTreeOutput {
        title: node.title.clone(),
        path: node.path.clone(),
        enabled: node.enabled,
        checked: node.checked,
        shortcut: node.shortcut.clone(),
        role: node.role.clone(),
        children: node.children.iter().map(node_to_tree_output).collect(),
        is_alternate: node.is_alternate,
        alternate_of: node.alternate_of.clone(),
    }
}
