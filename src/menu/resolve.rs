/// Path resolution: convert user-provided strings to specific `MenuNode`s.
///
/// Resolution strategy (in priority order):
///
/// 1. **Exact path match**: If input contains "::", walk the tree level-by-level
///    with exact title matching.
/// 2. **Exact title match (leaf)**: Search all leaf items for an exact title match.
///    Succeeds only if exactly one item matches.
/// 3. **Fuzzy match**: Run fuzzy search. Auto-resolve if the top result has a
///    significantly higher score than the second (confidence > threshold).
/// 4. **Ambiguity error**: If multiple items match with similar scores.
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Matcher, Utf32Str,
};

use super::{
    errors::MenuError,
    tree::{split_path, unescape_segment, MenuNode, PATH_SEP},
};

/// Minimum score ratio between 1st and 2nd result to auto-resolve fuzzy match.
const FUZZY_AUTO_RESOLVE_RATIO: f32 = 2.0;

/// Resolve a user-provided path/query to a single `MenuNode`.
///
/// The node is found by reference in the tree; the returned node is cloned
/// (including its `element` ref which is `Clone`-able via CF retain).
///
/// # Errors
///
/// - `MenuError::ItemNotFound` — no item matches
/// - `MenuError::AmbiguousMatch` — multiple items match with similar confidence
pub fn resolve<'a>(nodes: &'a [MenuNode], query: &str) -> Result<&'a MenuNode, MenuError> {
    // Strategy 1: Exact path match (query contains separator)
    if query.contains(PATH_SEP) {
        return resolve_by_exact_path(nodes, query);
    }

    // Strategy 2: Exact title match (case-insensitive)
    let exact_matches: Vec<&MenuNode> = collect_leaves(nodes)
        .into_iter()
        .filter(|n| n.title.to_lowercase() == query.to_lowercase())
        .collect();

    match exact_matches.len() {
        1 => return Ok(exact_matches[0]),
        n if n > 1 => {
            return Err(MenuError::AmbiguousMatch {
                query: query.to_owned(),
                candidates: exact_matches.iter().map(|n| n.path.clone()).collect(),
            });
        }
        _ => {}
    }

    // Strategy 3: Fuzzy match
    resolve_fuzzy(nodes, query)
}

/// Walk the tree level-by-level using the path segments split by `::`.
///
/// Handles escaped `\::` in segments via [`split_path`] / [`unescape_segment`].
fn resolve_by_exact_path<'a>(nodes: &'a [MenuNode], path: &str) -> Result<&'a MenuNode, MenuError> {
    let segments = split_path(path);
    let mut current = nodes;
    let mut found: Option<&MenuNode> = None;

    for segment in &segments {
        let unescaped = unescape_segment(segment);
        let seg_lower = unescaped.to_lowercase();
        let matched = current.iter().find(|n| n.title.to_lowercase() == seg_lower);
        match matched {
            Some(node) => {
                found = Some(node);
                current = &node.children;
            }
            None => {
                return Err(MenuError::ItemNotFound {
                    query: path.to_owned(),
                });
            }
        }
    }

    found.ok_or_else(|| MenuError::ItemNotFound {
        query: path.to_owned(),
    })
}

/// Collect all leaf nodes (items with no children) from the tree.
fn collect_leaves(nodes: &[MenuNode]) -> Vec<&MenuNode> {
    let mut leaves = Vec::new();
    for node in nodes {
        if node.children.is_empty() {
            leaves.push(node);
        } else {
            leaves.extend(collect_leaves(&node.children));
        }
    }
    leaves
}

/// Collect all nodes (including non-leaves) for fuzzy search.
fn collect_all<'a>(nodes: &'a [MenuNode], out: &mut Vec<&'a MenuNode>) {
    for node in nodes {
        out.push(node);
        collect_all(&node.children, out);
    }
}

fn resolve_fuzzy<'a>(nodes: &'a [MenuNode], query: &str) -> Result<&'a MenuNode, MenuError> {
    let mut all = Vec::new();
    collect_all(nodes, &mut all);

    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT.match_paths());

    let mut scored: Vec<(&MenuNode, u32)> = all
        .iter()
        .filter_map(|&node| {
            let mut buf = Vec::new();
            let haystack = Utf32Str::new(&node.path, &mut buf);
            pattern.score(haystack, &mut matcher).map(|s| (node, s))
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));

    match scored.as_slice() {
        [] => Err(MenuError::ItemNotFound {
            query: query.to_owned(),
        }),
        [(node, _)] => Ok(node),
        [(best_node, best_score), (_, second_score), ..] => {
            // Auto-resolve if best is significantly ahead of second.
            let ratio = *best_score as f32 / (*second_score as f32).max(1.0);
            if ratio >= FUZZY_AUTO_RESOLVE_RATIO {
                Ok(best_node)
            } else {
                Err(MenuError::AmbiguousMatch {
                    query: query.to_owned(),
                    candidates: scored.iter().take(5).map(|(n, _)| n.path.clone()).collect(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(title: &str, path: &str, children: Vec<MenuNode>) -> MenuNode {
        MenuNode {
            title: title.to_owned(),
            path: path.to_owned(),
            enabled: true,
            checked: false,
            shortcut: None,
            role: "AXMenuItem".to_owned(),
            depth: 1,
            children,
            element: None,
            is_alternate: false,
            alternate_of: None,
        }
    }

    fn tree() -> Vec<MenuNode> {
        vec![
            node(
                "File",
                "File",
                vec![
                    node("New", "File::New", vec![]),
                    node("Save As…", "File::Save As…", vec![]),
                    node("Close", "File::Close", vec![]),
                ],
            ),
            node(
                "Edit",
                "Edit",
                vec![
                    node("Copy", "Edit::Copy", vec![]),
                    node("Paste", "Edit::Paste", vec![]),
                ],
            ),
        ]
    }

    #[test]
    fn test_exact_path() {
        let t = tree();
        let result = resolve(&t, "File::Save As…").unwrap();
        assert_eq!(result.path, "File::Save As…");
    }

    #[test]
    fn test_exact_title_unique() {
        let t = tree();
        let result = resolve(&t, "Paste").unwrap();
        assert_eq!(result.path, "Edit::Paste");
    }

    #[test]
    fn test_exact_title_ambiguous() {
        // "New" and "Copy" don't collide, but let's test ambiguity with a custom tree.
        let t = vec![
            node("File", "File", vec![node("Save", "File::Save", vec![])]),
            node("Edit", "Edit", vec![node("Save", "Edit::Save", vec![])]),
        ];
        let result = resolve(&t, "save");
        assert!(matches!(result, Err(MenuError::AmbiguousMatch { .. })));
    }

    #[test]
    fn test_not_found() {
        let t = tree();
        let result = resolve(&t, "File::NonExistent");
        assert!(matches!(result, Err(MenuError::ItemNotFound { .. })));
    }
}
