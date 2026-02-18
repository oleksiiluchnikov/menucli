/// Output formatting: JSON, table, path/id modes. TTY detection.
use std::io::{IsTerminal, Write};

use comfy_table::{Cell, Table, presets::UTF8_BORDERS_ONLY};
use serde::Serialize;

use super::args::OutputFormat;
use crate::types::{
    AppInfoOutput, MenuItemOutput, MenuTreeOutput, SearchResultOutput, ToggleOutput,
};

/// Resolve the effective output format, handling `--json` flag and TTY auto-detection.
#[must_use]
pub fn resolve_format(fmt: OutputFormat, json_flag: bool) -> OutputFormat {
    if json_flag {
        return OutputFormat::Json;
    }
    if fmt == OutputFormat::Auto {
        if std::io::stdout().is_terminal() {
            OutputFormat::Table
        } else {
            OutputFormat::Json
        }
    } else {
        fmt
    }
}

/// Output context passed to all formatters.
pub struct OutputCtx {
    pub format: OutputFormat,
    pub fields: Option<Vec<String>>,
    pub no_header: bool,
    /// When true, print AX timing spans to stderr.
    pub debug: bool,
}

impl OutputCtx {
    /// Construct from CLI args.
    #[must_use]
    pub fn new(
        fmt: OutputFormat,
        json_flag: bool,
        fields: Option<&str>,
        no_header: bool,
        debug: bool,
    ) -> Self {
        let format = resolve_format(fmt, json_flag);
        let fields = fields.map(|f| f.split(',').map(str::trim).map(str::to_owned).collect());
        Self {
            format,
            fields,
            no_header,
            debug,
        }
    }

    /// Start a named debug timer. Prints elapsed on drop only when `--debug` is set.
    #[must_use]
    pub fn timer(&self, label: &'static str) -> DebugTimer {
        DebugTimer::new(label, self.debug)
    }

    /// Whether a field should be included in output.
    fn include_field(&self, name: &str) -> bool {
        self.fields
            .as_ref()
            .map_or(true, |f| f.iter().any(|n| n == name))
    }
}

// --- Flat menu item output ---

/// Write a list of `MenuItemOutput` to stdout.
pub fn write_menu_items(items: &[MenuItemOutput], ctx: &OutputCtx) {
    match ctx.format {
        OutputFormat::Json => print_json(items),
        OutputFormat::Compact => print_compact_json(items),
        OutputFormat::Ndjson => print_ndjson(items),
        OutputFormat::Path => {
            for item in items {
                println!("{}", item.path);
            }
        }
        OutputFormat::Id => {
            for item in items {
                println!("{}", item.title);
            }
        }
        OutputFormat::Table | OutputFormat::Auto => write_menu_items_table(items, ctx),
    }
}

fn write_menu_items_table(items: &[MenuItemOutput], ctx: &OutputCtx) {
    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);

    let mut headers: Vec<Cell> = Vec::new();
    if ctx.include_field("path") {
        headers.push(Cell::new("PATH"));
    }
    if ctx.include_field("enabled") {
        headers.push(Cell::new("ENABLED"));
    }
    if ctx.include_field("checked") {
        headers.push(Cell::new("CHECKED"));
    }
    if ctx.include_field("shortcut") {
        headers.push(Cell::new("SHORTCUT"));
    }
    if ctx.include_field("role") {
        headers.push(Cell::new("ROLE"));
    }

    if !ctx.no_header {
        table.set_header(headers);
    }

    for item in items {
        let mut row: Vec<Cell> = Vec::new();
        if ctx.include_field("path") {
            row.push(Cell::new(&item.path));
        }
        if ctx.include_field("enabled") {
            row.push(Cell::new(if item.enabled { "yes" } else { "no" }));
        }
        if ctx.include_field("checked") {
            row.push(Cell::new(if item.checked { "✓" } else { "" }));
        }
        if ctx.include_field("shortcut") {
            row.push(Cell::new(item.shortcut.as_deref().unwrap_or("")));
        }
        if ctx.include_field("role") {
            row.push(Cell::new(&item.role));
        }
        table.add_row(row);
    }

    println!("{table}");
}

// --- Tree output ---

/// Write a tree of `MenuTreeOutput` to stdout.
pub fn write_menu_tree(nodes: &[MenuTreeOutput], ctx: &OutputCtx) {
    match ctx.format {
        OutputFormat::Json => print_json(nodes),
        OutputFormat::Compact => print_compact_json(nodes),
        OutputFormat::Ndjson => print_ndjson(nodes),
        OutputFormat::Path => {
            for node in nodes {
                print_tree_paths(node);
            }
        }
        OutputFormat::Id => {
            for node in nodes {
                print_tree_ids(node);
            }
        }
        OutputFormat::Table | OutputFormat::Auto => {
            let count = nodes.len();
            for (i, node) in nodes.iter().enumerate() {
                print_tree_visual(node, "", i + 1 == count, ctx);
            }
        }
    }
}

fn print_tree_paths(node: &MenuTreeOutput) {
    if node.children.is_empty() {
        println!("{}", node.path);
    }
    for child in &node.children {
        print_tree_paths(child);
    }
}

fn print_tree_ids(node: &MenuTreeOutput) {
    println!("{}", node.title);
    for child in &node.children {
        print_tree_ids(child);
    }
}

fn print_tree_visual(node: &MenuTreeOutput, prefix: &str, is_last: bool, ctx: &OutputCtx) {
    let connector = if is_last { "└── " } else { "├── " };
    let shortcut_str = node
        .shortcut
        .as_deref()
        .map(|s| format!("  [{s}]"))
        .unwrap_or_default();
    let enabled_str = if !node.enabled { " (disabled)" } else { "" };
    let checked_str = if node.checked { " ✓" } else { "" };
    println!(
        "{prefix}{connector}{}{shortcut_str}{enabled_str}{checked_str}",
        node.title
    );

    let child_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        print_tree_visual(child, &child_prefix, i + 1 == child_count, ctx);
    }
}

// --- Search results ---

/// Write search results to stdout.
pub fn write_search_results(results: &[SearchResultOutput], ctx: &OutputCtx) {
    match ctx.format {
        OutputFormat::Json => print_json(results),
        OutputFormat::Compact => print_compact_json(results),
        OutputFormat::Ndjson => print_ndjson(results),
        OutputFormat::Path => {
            for r in results {
                println!("{}", r.path);
            }
        }
        OutputFormat::Id => {
            for r in results {
                println!("{}", r.title);
            }
        }
        OutputFormat::Table | OutputFormat::Auto => write_search_table(results, ctx),
    }
}

fn write_search_table(results: &[SearchResultOutput], ctx: &OutputCtx) {
    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    if !ctx.no_header {
        table.set_header(["PATH", "ENABLED", "SHORTCUT", "SCORE"]);
    }
    for r in results {
        table.add_row([
            r.path.as_str(),
            if r.enabled { "yes" } else { "no" },
            r.shortcut.as_deref().unwrap_or(""),
            &r.score.to_string(),
        ]);
    }
    println!("{table}");
}

// --- Apps ---

/// Write app list to stdout.
pub fn write_apps(apps: &[AppInfoOutput], ctx: &OutputCtx) {
    match ctx.format {
        OutputFormat::Json => print_json(apps),
        OutputFormat::Compact => print_compact_json(apps),
        OutputFormat::Ndjson => print_ndjson(apps),
        OutputFormat::Id | OutputFormat::Path => {
            for app in apps {
                println!("{}", app.name);
            }
        }
        OutputFormat::Table | OutputFormat::Auto => write_apps_table(apps, ctx),
    }
}

fn write_apps_table(apps: &[AppInfoOutput], ctx: &OutputCtx) {
    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    if !ctx.no_header {
        table.set_header(["NAME", "PID", "BUNDLE ID", "FRONTMOST"]);
    }
    for app in apps {
        table.add_row([
            app.name.as_str(),
            &app.pid.to_string(),
            app.bundle_id.as_deref().unwrap_or(""),
            if app.frontmost { "yes" } else { "" },
        ]);
    }
    println!("{table}");
}

// --- Toggle ---

/// Write toggle result to stdout.
pub fn write_toggle(result: &ToggleOutput, ctx: &OutputCtx) {
    match ctx.format {
        OutputFormat::Json | OutputFormat::Auto => print_json(result),
        OutputFormat::Compact => print_compact_json(result),
        OutputFormat::Ndjson => print_ndjson(&[result]),
        _ => {
            let state = if result.checked_after {
                "on (✓)"
            } else {
                "off"
            };
            let dry = if result.dry_run { " [dry-run]" } else { "" };
            println!("{}: {state}{dry}", result.path);
        }
    }
}

// --- Error output ---

/// Write a structured error to stderr.
pub fn write_error(err: &crate::types::ErrorOutput, format: OutputFormat, json_flag: bool) {
    let fmt = resolve_format(format, json_flag);
    let stderr = std::io::stderr();
    let mut out = stderr.lock();
    match fmt {
        OutputFormat::Json | OutputFormat::Compact | OutputFormat::Ndjson => {
            let s = serde_json::to_string_pretty(err).unwrap_or_default();
            let _ = writeln!(out, "{s}");
        }
        _ => {
            let _ = writeln!(out, "Error: {}", err.error.message);
            if let Some(candidates) = &err.error.candidates {
                let _ = writeln!(out, "  Candidates:");
                for c in candidates {
                    let _ = writeln!(out, "    {c}");
                }
            }
        }
    }
}

// --- Debug timer ---

/// A RAII timer that prints elapsed milliseconds to stderr on drop.
///
/// Created via [`OutputCtx::timer`]. Does nothing when `debug` is false.
pub struct DebugTimer {
    label: &'static str,
    start: std::time::Instant,
    active: bool,
}

impl DebugTimer {
    #[must_use]
    fn new(label: &'static str, active: bool) -> Self {
        Self {
            label,
            start: std::time::Instant::now(),
            active,
        }
    }
}

impl Drop for DebugTimer {
    fn drop(&mut self) {
        if self.active {
            let ms = self.start.elapsed().as_secs_f64() * 1000.0;
            eprintln!("[debug] {}: {ms:.2}ms", self.label);
        }
    }
}

// --- Generic JSON helpers ---

fn print_json<T: Serialize + ?Sized>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("JSON serialization error: {e}"),
    }
}

fn print_compact_json<T: Serialize + ?Sized>(value: &T) {
    match serde_json::to_string(value) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("JSON serialization error: {e}"),
    }
}

fn print_ndjson<T: Serialize>(values: &[T]) {
    for v in values {
        match serde_json::to_string(v) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("JSON serialization error: {e}"),
        }
    }
}
