/// `search` command: fuzzy-search menu items.
use crate::ax::resolve_target;
use crate::cli::args::SearchArgs;
use crate::cli::output::write_search_results;
use crate::cli::OutputCtx;
use crate::menu::tree::{build_all_extras, build_extras_tree, TreeOptions};
use crate::menu::{build_tree_with_opts, flatten, search, MenuError, SearchOptions};
use crate::types::SearchResultOutput;

/// Run `menucli search`.
///
/// # Errors
///
/// Returns `MenuError` on AX failure, missing permissions, or unknown app.
pub fn run(args: &SearchArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
    let tree_opts = TreeOptions {
        include_alternates: ctx.alternates,
    };

    let flat = if args.extras {
        if let Some(app) = &args.app {
            let _t_resolve = ctx.timer("resolve_target");
            let pid = resolve_target(Some(app.as_str())).map_err(MenuError::from)?;
            drop(_t_resolve);

            let _t_tree = ctx.timer("build_extras_tree");
            let tree = build_extras_tree(pid, None, &tree_opts)?;
            drop(_t_tree);

            flatten(&tree)
        } else {
            let _t_tree = ctx.timer("build_all_extras");
            let results = build_all_extras(None, &tree_opts);
            drop(_t_tree);

            let mut all = Vec::new();
            for result in &results {
                all.extend(flatten(&result.nodes));
            }
            all
        }
    } else {
        let _t_resolve = ctx.timer("resolve_target");
        let pid = resolve_target(args.app.as_deref()).map_err(MenuError::from)?;
        drop(_t_resolve);

        let _t_tree = ctx.timer("build_tree");
        let tree = build_tree_with_opts(pid, None, &tree_opts)?;
        drop(_t_tree);

        let _t_flatten = ctx.timer("flatten");
        let f = flatten(&tree);
        drop(_t_flatten);
        f
    };

    let opts = SearchOptions {
        limit: args.limit,
        exact: args.exact,
        case_sensitive: args.case_sensitive,
    };

    let _t_search = ctx.timer("search");
    let results = search(&flat, &args.query, &opts);
    drop(_t_search);

    let output: Vec<SearchResultOutput> = results
        .iter()
        .map(|r| SearchResultOutput {
            title: r.item.title.clone(),
            path: r.item.path.clone(),
            enabled: r.item.enabled,
            checked: r.item.checked,
            shortcut: r.item.shortcut.clone(),
            score: r.score,
            is_alternate: r.item.is_alternate,
            alternate_of: r.item.alternate_of.clone(),
        })
        .collect();

    write_search_results(&output, ctx);
    Ok(())
}
