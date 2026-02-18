/// `search` command: fuzzy-search menu items.
use crate::ax::resolve_target;
use crate::cli::OutputCtx;
use crate::cli::args::SearchArgs;
use crate::cli::output::write_search_results;
use crate::menu::{MenuError, SearchOptions, build_tree, flatten, search};
use crate::types::SearchResultOutput;

/// Run `menucli search`.
///
/// # Errors
///
/// Returns `MenuError` on AX failure, missing permissions, or unknown app.
pub fn run(args: &SearchArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
    let _t_resolve = ctx.timer("resolve_target");
    let pid = resolve_target(args.app.as_deref()).map_err(MenuError::from)?;
    drop(_t_resolve);

    let _t_tree = ctx.timer("build_tree");
    let tree = build_tree(pid, None)?;
    drop(_t_tree);

    let _t_flatten = ctx.timer("flatten");
    let flat = flatten(&tree);
    drop(_t_flatten);

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
        })
        .collect();

    write_search_results(&output, ctx);
    Ok(())
}
