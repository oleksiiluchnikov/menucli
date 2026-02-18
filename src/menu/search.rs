/// Fuzzy and exact search over flat menu items.
use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};

use super::flatten::FlatItem;

/// A search result with its match score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matched item.
    pub item: FlatItem,
    /// Match score (higher = better match). 0 for exact search (unscored).
    pub score: u32,
}

/// Search options.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of results to return.
    pub limit: usize,
    /// Use exact substring match instead of fuzzy.
    pub exact: bool,
    /// Case-sensitive matching.
    pub case_sensitive: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            exact: false,
            case_sensitive: false,
        }
    }
}

/// Search menu items by query string.
///
/// Searches the `path` field (full path like "File::Save As…") which naturally
/// gives higher scores when the query matches words at boundaries.
///
/// Results are sorted by score descending (best match first).
#[must_use]
pub fn search(items: &[FlatItem], query: &str, opts: &SearchOptions) -> Vec<SearchResult> {
    if query.is_empty() {
        return items
            .iter()
            .take(opts.limit)
            .map(|item| SearchResult {
                item: item.clone(),
                score: 0,
            })
            .collect();
    }

    if opts.exact {
        return exact_search(items, query, opts);
    }

    fuzzy_search(items, query, opts)
}

fn exact_search(items: &[FlatItem], query: &str, opts: &SearchOptions) -> Vec<SearchResult> {
    let results: Vec<SearchResult> = items
        .iter()
        .filter(|item| {
            if opts.case_sensitive {
                item.path.contains(query)
            } else {
                item.path.to_lowercase().contains(&query.to_lowercase())
            }
        })
        .take(opts.limit)
        .map(|item| SearchResult {
            item: item.clone(),
            score: 0,
        })
        .collect();
    results
}

fn fuzzy_search(items: &[FlatItem], query: &str, opts: &SearchOptions) -> Vec<SearchResult> {
    let case_matching = if opts.case_sensitive {
        CaseMatching::Respect
    } else {
        CaseMatching::Smart
    };

    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT.match_paths());

    let mut scored: Vec<SearchResult> = items
        .iter()
        .filter_map(|item| {
            let mut buf = Vec::new();
            let haystack = Utf32Str::new(&item.path, &mut buf);
            pattern
                .score(haystack, &mut matcher)
                .map(|score| SearchResult {
                    item: item.clone(),
                    score,
                })
        })
        .collect();

    // Sort by score descending.
    scored.sort_by(|a, b| b.score.cmp(&a.score));
    scored.truncate(opts.limit);
    scored
}
