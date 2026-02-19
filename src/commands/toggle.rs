/// `toggle` command: toggle a checkmark menu item and report the new state.
use crate::ax::resolve_target;
use crate::cli::args::ToggleArgs;
use crate::cli::output::write_toggle;
use crate::cli::OutputCtx;
use crate::menu::tree::{build_extras_tree, TreeOptions};
use crate::menu::{build_tree_with_opts, press_node, resolve, MenuError};
use crate::types::ToggleOutput;

/// Maximum number of attempts to confirm the toggle took effect.
const MAX_RETRIES: u32 = 5;

/// Initial delay (ms) between `AXPress` and the first re-read.
const INITIAL_DELAY_MS: u64 = 50;

/// Run `menucli toggle`.
///
/// After pressing the item, re-reads the menu tree up to [`MAX_RETRIES`] times
/// with exponential back-off (`50 -> 100 -> 200 -> 400 -> 800 ms`) waiting for the
/// app to update its AX checkmark state. If the state flips within that window
/// we report the observed value; otherwise we infer `!checked_before`.
///
/// # Errors
///
/// Returns `MenuError::NotToggleable` if the item has no checkmark state.
/// Returns `MenuError::ItemDisabled` if the item is not clickable.
/// Returns `MenuError` on AX failure, missing permissions, or unknown app.
pub fn run(args: &ToggleArgs, ctx: &OutputCtx) -> Result<(), MenuError> {
    let tree_opts = TreeOptions {
        include_alternates: ctx.alternates,
    };

    let _t_resolve = ctx.timer("resolve_target");
    let pid = resolve_target(args.app.as_deref()).map_err(MenuError::from)?;
    drop(_t_resolve);

    let tree = if args.extras {
        let _t_tree = ctx.timer("build_extras_tree[1]");
        let t = build_extras_tree(pid, None, &tree_opts)?;
        drop(_t_tree);
        t
    } else {
        let _t_tree = ctx.timer("build_tree[1]");
        let t = build_tree_with_opts(pid, None, &tree_opts)?;
        drop(_t_tree);
        t
    };

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

    // Poll for the AX state to flip, with exponential back-off.
    let _t_poll = ctx.timer("poll_state");
    let mut delay_ms = INITIAL_DELAY_MS;
    let mut checked_after = !checked_before; // optimistic default
    for attempt in 0..MAX_RETRIES {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));

        let tree2_result = if args.extras {
            build_extras_tree(pid, None, &tree_opts)
        } else {
            build_tree_with_opts(pid, None, &tree_opts)
        };

        if let Ok(tree2) = tree2_result {
            if let Ok(node2) = resolve(&tree2, &args.path) {
                if node2.checked != checked_before {
                    // Confirmed: the state flipped.
                    checked_after = node2.checked;
                    break;
                }
            }
        }

        if attempt + 1 < MAX_RETRIES {
            delay_ms *= 2;
        }
    }
    drop(_t_poll);

    let output = ToggleOutput {
        path,
        checked_before,
        checked_after,
        dry_run: false,
    };

    write_toggle(&output, ctx);
    Ok(())
}
