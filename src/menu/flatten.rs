/// Flatten a menu tree into a list of `FlatItem`s with full path notation.
use super::tree::MenuNode;

/// A flat representation of a menu item (no children).
#[derive(Debug, Clone)]
pub struct FlatItem {
    /// Display title (leaf name only).
    pub title: String,
    /// Full path from root (e.g., "File::Save As…").
    pub path: String,
    /// Whether the item is enabled.
    pub enabled: bool,
    /// Whether the item has a checkmark.
    pub checked: bool,
    /// Formatted keyboard shortcut.
    pub shortcut: Option<String>,
    /// AX role string.
    pub role: String,
    /// Depth in the menu hierarchy.
    pub depth: usize,
    /// Number of direct children (0 for leaf items).
    pub children_count: usize,
    /// Whether this item is an Option-key alternate.
    pub is_alternate: bool,
    /// Title of the primary item this alternate replaces, if any.
    pub alternate_of: Option<String>,
}

/// Flatten a tree of `MenuNode`s into a `Vec<FlatItem>`.
///
/// Traversal is depth-first, pre-order (parent before children).
#[must_use]
pub fn flatten(nodes: &[MenuNode]) -> Vec<FlatItem> {
    let mut result = Vec::new();
    for node in nodes {
        flatten_node(node, &mut result);
    }
    result
}

fn flatten_node(node: &MenuNode, out: &mut Vec<FlatItem>) {
    out.push(FlatItem {
        title: node.title.clone(),
        path: node.path.clone(),
        enabled: node.enabled,
        checked: node.checked,
        shortcut: node.shortcut.clone(),
        role: node.role.clone(),
        depth: node.depth,
        children_count: node.children.len(),
        is_alternate: node.is_alternate,
        alternate_of: node.alternate_of.clone(),
    });
    for child in &node.children {
        flatten_node(child, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_node(title: &str, path: &str, children: Vec<MenuNode>) -> MenuNode {
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

    #[test]
    fn test_flatten_single_level() {
        let nodes = vec![
            mock_node("File", "File", vec![]),
            mock_node("Edit", "Edit", vec![]),
        ];
        let flat = flatten(&nodes);
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].path, "File");
        assert_eq!(flat[1].path, "Edit");
    }

    #[test]
    fn test_flatten_nested() {
        let child = mock_node("New", "File::New", vec![]);
        let parent = mock_node("File", "File", vec![child]);
        let flat = flatten(&[parent]);
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].path, "File");
        assert_eq!(flat[1].path, "File::New");
        assert_eq!(flat[0].children_count, 1);
        assert_eq!(flat[1].children_count, 0);
    }
}
