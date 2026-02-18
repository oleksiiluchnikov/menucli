/// CLI argument definitions via clap derive.
use clap::{Parser, Subcommand, ValueEnum};

/// menucli — query and interact with macOS app menu bars.
#[derive(Debug, Parser)]
#[command(
    name = "menucli",
    about = "Query and interact with macOS app menu bars from the CLI",
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Output format. Auto-detects: table when TTY, json when piped.
    #[arg(long, global = true, value_name = "FORMAT", default_value = "auto")]
    pub output: OutputFormat,

    /// Shorthand for --output json.
    #[arg(long, global = true, conflicts_with = "output")]
    pub json: bool,

    /// Comma-separated field names to include in output (projection).
    /// Available fields vary by command (see --help for each subcommand).
    #[arg(long, global = true, value_name = "FIELDS")]
    pub fields: Option<String>,

    /// Omit table headers (useful for awk/cut processing).
    #[arg(long, global = true)]
    pub no_header: bool,

    /// Print AX API call timing to stderr for debugging.
    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Command,
}

/// Output format variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum OutputFormat {
    /// Auto-detect: table when stdout is a TTY, json when piped.
    #[default]
    Auto,
    /// JSON array or object (pretty-printed).
    Json,
    /// Compact single-line JSON.
    Compact,
    /// Newline-delimited JSON (one object per line).
    Ndjson,
    /// Aligned table with headers (human-readable).
    Table,
    /// Full path only, one per line (for piping to other commands).
    Path,
    /// ID/title only, one per line.
    Id,
}

/// All subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// List all menu items for an application.
    List(ListArgs),
    /// Fuzzy-search menu items by title.
    Search(SearchArgs),
    /// Click (activate) a menu item.
    Click(ClickArgs),
    /// Toggle a checkmark menu item and report the new state.
    Toggle(ToggleArgs),
    /// Get the current state of a specific menu item.
    State(StateArgs),
    /// List running applications with their PIDs.
    Apps(AppsArgs),
    /// Check if Accessibility permission is granted.
    CheckAccess,
}

/// Arguments for `menucli list`.
#[derive(Debug, Parser)]
pub struct ListArgs {
    /// Target application: name, PID, or bundle ID.
    /// Defaults to the frontmost application.
    #[arg(long, value_name = "NAME|PID|BUNDLE_ID")]
    pub app: Option<String>,

    /// Output as flat list with full path notation (default when not a TTY).
    #[arg(long)]
    pub flat: bool,

    /// Output as nested tree (default when a TTY).
    #[arg(long, conflicts_with = "flat")]
    pub tree: bool,

    /// Only include enabled (clickable) items.
    #[arg(long)]
    pub enabled_only: bool,

    /// Maximum recursion depth (default: unlimited).
    #[arg(long, value_name = "N")]
    pub depth: Option<usize>,
}

/// Arguments for `menucli search`.
#[derive(Debug, Parser)]
pub struct SearchArgs {
    /// Search query string.
    pub query: String,

    /// Target application.
    #[arg(long, value_name = "NAME|PID|BUNDLE_ID")]
    pub app: Option<String>,

    /// Maximum number of results to return.
    #[arg(long, value_name = "N", default_value = "10")]
    pub limit: usize,

    /// Use exact substring match instead of fuzzy.
    #[arg(long)]
    pub exact: bool,

    /// Case-sensitive matching (default: smart-case).
    #[arg(long)]
    pub case_sensitive: bool,
}

/// Arguments for `menucli click`.
#[derive(Debug, Parser)]
pub struct ClickArgs {
    /// Menu item path or partial match.
    /// Examples: "File > Save As…", "Save As", "save as"
    pub path: String,

    /// Target application.
    #[arg(long, value_name = "NAME|PID|BUNDLE_ID")]
    pub app: Option<String>,

    /// Preview the resolved item without clicking it.
    #[arg(long)]
    pub dry_run: bool,

    /// Require exact path match (no fuzzy resolution).
    #[arg(long)]
    pub exact: bool,
}

/// Arguments for `menucli toggle`.
#[derive(Debug, Parser)]
pub struct ToggleArgs {
    /// Menu item path or partial match.
    pub path: String,

    /// Target application.
    #[arg(long, value_name = "NAME|PID|BUNDLE_ID")]
    pub app: Option<String>,

    /// Show current state without toggling.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for `menucli state`.
#[derive(Debug, Parser)]
pub struct StateArgs {
    /// Menu item path or partial match.
    pub path: String,

    /// Target application.
    #[arg(long, value_name = "NAME|PID|BUNDLE_ID")]
    pub app: Option<String>,
}

/// Arguments for `menucli apps`.
#[derive(Debug, Parser)]
pub struct AppsArgs {
    /// Show only the frontmost application.
    #[arg(long)]
    pub frontmost: bool,
}
