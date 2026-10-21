//! Work breakdown structure (WBS) tree.
//!
//! A [`WbsTree`] always has exactly one root node. Nodes may be nested
//! arbitrarily; every mutation validates structure so that the tree can never
//! contain a cycle, an orphan, or a duplicate identifier.
//!
//! Deletion policy (see `DECISIONS.md` D7): a node may only be deleted when it
//! has no children. Activity assignments are validated by the command layer,
//! which owns knowledge of activities, and are reported as
//! [`WbsError::HasActivities`].

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Stable identifier for a [`WbsNode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WbsId(pub u64);

impl std::fmt::Display for WbsId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// One node in the work breakdown structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WbsNode {
    /// Identifier assigned when the node was created.
    pub id: WbsId,
    /// Display name (never empty).
    pub name: String,
    /// Parent node; `None` only for the root.
    pub parent: Option<WbsId>,
    /// Direct children in display order.
    #[serde(default)]
    pub children: Vec<WbsId>,
}

/// Errors produced by WBS mutations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WbsError {
    /// The referenced node does not exist.
    #[error("WBS node {0} does not exist")]
    UnknownNode(WbsId),
    /// A node with the same identifier already exists.
    #[error("WBS node {0} already exists")]
    DuplicateNode(WbsId),
    /// A node name was empty or whitespace-only.
    #[error("WBS node name must not be empty")]
    EmptyName,
    /// The move would make a node its own ancestor.
    #[error("moving WBS node {node} under {parent} would create a cycle")]
    Cycle {
        /// Node being moved.
        node: WbsId,
        /// Requested new parent.
        parent: WbsId,
    },
    /// The node still has children and cannot be deleted.
    #[error("WBS node {0} still has children")]
    HasChildren(WbsId),
    /// The node still has activities assigned and cannot be deleted.
    #[error("WBS node {0} still has activities assigned")]
    HasActivities(WbsId),
    /// The root node cannot be moved or deleted.
    #[error("the WBS root node cannot be moved or deleted")]
    RootImmutable,
}

/// A validated hierarchical work breakdown structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WbsTree {
    root: WbsId,
    nodes: Vec<WbsNode>,
    #[serde(default)]
    next: u64,
}

impl WbsTree {
    /// Create a tree containing only a root node named `Project`.
    pub fn new() -> Self {
        let root = WbsId(1);
        Self {
            root,
            nodes: vec![WbsNode {
                id: root,
                name: "Project".to_string(),
                parent: None,
                children: Vec::new(),
            }],
            next: 2,
        }
    }

    /// Identifier of the root node.
    pub fn root(&self) -> WbsId {
        self.root
    }

    /// All nodes in creation order.
    pub fn nodes(&self) -> &[WbsNode] {
        &self.nodes
    }

    /// Look up a node.
    pub fn node(&self, id: WbsId) -> Option<&WbsNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// Look up a node mutably.
    pub fn node_mut(&mut self, id: WbsId) -> Option<&mut WbsNode> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    /// Next unused node identifier.
    pub fn next_id(&self) -> WbsId {
        let highest = self.nodes.iter().map(|node| node.id.0).max().unwrap_or(0);
        WbsId(highest.max(self.next.saturating_sub(1)).saturating_add(1))
    }

    /// Add a node under `parent`.
    ///
    /// # Errors
    ///
    /// Returns [`WbsError::DuplicateNode`] when `id` is taken,
    /// [`WbsError::UnknownNode`] when `parent` does not exist, or
    /// [`WbsError::EmptyName`] for a blank name.
    pub fn add(
        &mut self,
        id: WbsId,
        name: impl Into<String>,
        parent: WbsId,
    ) -> Result<(), WbsError> {
        if self.node(id).is_some() {
            return Err(WbsError::DuplicateNode(id));
        }
        if self.node(parent).is_none() {
            return Err(WbsError::UnknownNode(parent));
        }
        let name = name.into();
        let name = name.trim();
        if name.is_empty() {
            return Err(WbsError::EmptyName);
        }

        self.nodes.push(WbsNode {
            id,
            name: name.to_string(),
            parent: Some(parent),
            children: Vec::new(),
        });
        // The parent was validated above, so this cannot fail.
        if let Some(parent_node) = self.node_mut(parent) {
            parent_node.children.push(id);
        }
        self.next = self.next.max(id.0.saturating_add(1));
        Ok(())
    }

    /// Rename a node, returning the previous name.
    ///
    /// # Errors
    ///
    /// Returns [`WbsError::UnknownNode`] or [`WbsError::EmptyName`].
    pub fn rename(&mut self, id: WbsId, name: impl Into<String>) -> Result<String, WbsError> {
        let name = name.into();
        let name = name.trim();
        if name.is_empty() {
            return Err(WbsError::EmptyName);
        }
        let node = self.node_mut(id).ok_or(WbsError::UnknownNode(id))?;
        let previous = std::mem::replace(&mut node.name, name.to_string());
        Ok(previous)
    }

    /// Move a node (and its subtree) under a new parent, returning the previous
    /// parent.
    ///
    /// # Errors
    ///
    /// Rejects moving the root, unknown nodes, and moves that would create a
    /// cycle ([`WbsError::Cycle`]).
    pub fn move_node(&mut self, id: WbsId, new_parent: WbsId) -> Result<Option<WbsId>, WbsError> {
        if id == self.root {
            return Err(WbsError::RootImmutable);
        }
        if self.node(new_parent).is_none() {
            return Err(WbsError::UnknownNode(new_parent));
        }
        let previous_parent = self.node(id).ok_or(WbsError::UnknownNode(id))?.parent;
        if id == new_parent || self.is_ancestor(id, new_parent) {
            return Err(WbsError::Cycle {
                node: id,
                parent: new_parent,
            });
        }
        if previous_parent == Some(new_parent) {
            return Ok(previous_parent);
        }

        if let Some(parent) = previous_parent {
            if let Some(parent_node) = self.node_mut(parent) {
                parent_node.children.retain(|child| *child != id);
            }
        }
        if let Some(node) = self.node_mut(id) {
            node.parent = Some(new_parent);
        }
        if let Some(parent_node) = self.node_mut(new_parent) {
            parent_node.children.push(id);
        }
        Ok(previous_parent)
    }

    /// Delete a leaf node, returning its snapshot for undo.
    ///
    /// # Errors
    ///
    /// Rejects the root ([`WbsError::RootImmutable`]) and nodes with children
    /// ([`WbsError::HasChildren`]). Activity assignments must be checked by the
    /// caller.
    pub fn delete(&mut self, id: WbsId) -> Result<WbsNode, WbsError> {
        if id == self.root {
            return Err(WbsError::RootImmutable);
        }
        let node = self.node(id).cloned().ok_or(WbsError::UnknownNode(id))?;
        if !node.children.is_empty() {
            return Err(WbsError::HasChildren(id));
        }
        if let Some(parent) = node.parent {
            if let Some(parent_node) = self.node_mut(parent) {
                parent_node.children.retain(|child| *child != id);
            }
        }
        self.nodes.retain(|entry| entry.id != id);
        Ok(node)
    }

    /// Restore a deleted leaf node snapshot, used by undo.
    ///
    /// # Errors
    ///
    /// Returns [`WbsError::DuplicateNode`] when the identifier is taken or
    /// [`WbsError::UnknownNode`] when the recorded parent no longer exists.
    pub fn restore(&mut self, node: WbsNode) -> Result<(), WbsError> {
        if self.node(node.id).is_some() {
            return Err(WbsError::DuplicateNode(node.id));
        }
        let parent = node.parent.ok_or(WbsError::RootImmutable)?;
        self.add(node.id, node.name, parent)
    }

    /// Whether `ancestor` is `node` itself or an ancestor of it.
    pub fn is_ancestor(&self, ancestor: WbsId, node: WbsId) -> bool {
        let mut current = Some(node);
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            current = self.node(id).and_then(|entry| entry.parent);
        }
        false
    }

    /// Depth of a node, with the root at depth zero.
    pub fn depth(&self, id: WbsId) -> Option<usize> {
        let mut depth = 0;
        let mut current = self.node(id)?;
        while let Some(parent) = current.parent {
            depth += 1;
            current = self.node(parent)?;
        }
        Some(depth)
    }

    /// All descendants of a node in depth-first order, excluding the node.
    pub fn descendants(&self, id: WbsId) -> Vec<WbsId> {
        let mut collected = Vec::new();
        let Some(node) = self.node(id) else {
            return collected;
        };
        let mut stack: Vec<WbsId> = node.children.iter().rev().copied().collect();
        while let Some(child) = stack.pop() {
            collected.push(child);
            if let Some(entry) = self.node(child) {
                stack.extend(entry.children.iter().rev().copied());
            }
        }
        collected
    }
}

impl Default for WbsTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree_with_children() -> (WbsTree, WbsId, WbsId, WbsId) {
        let mut tree = WbsTree::new();
        let design = WbsId(2);
        let build = WbsId(3);
        let concrete = WbsId(4);
        tree.add(design, "Design", tree.root()).expect("add design");
        tree.add(build, "Construction", tree.root())
            .expect("add construction");
        tree.add(concrete, "Concrete works", build)
            .expect("add concrete");
        (tree, design, build, concrete)
    }

    #[test]
    fn new_tree_has_single_root() {
        let tree = WbsTree::new();
        assert_eq!(tree.root(), WbsId(1));
        assert_eq!(tree.node(tree.root()).expect("root").name, "Project");
        assert_eq!(tree.nodes().len(), 1);
        assert_eq!(tree.depth(tree.root()), Some(0));
    }

    #[test]
    fn add_builds_hierarchy_and_ids() {
        let (tree, design, build, concrete) = tree_with_children();
        assert_eq!(tree.depth(design), Some(1));
        assert_eq!(tree.depth(concrete), Some(2));
        assert_eq!(tree.node(build).expect("build").children, vec![concrete]);
        assert_eq!(tree.next_id(), WbsId(5));
        assert_eq!(tree.descendants(tree.root()), vec![design, build, concrete]);
    }

    #[test]
    fn add_rejects_duplicate_and_unknown_parent_and_empty_name() {
        let (mut tree, _, build, _) = tree_with_children();
        assert_eq!(
            tree.add(build, "Again", tree.root()),
            Err(WbsError::DuplicateNode(build))
        );
        assert_eq!(
            tree.add(WbsId(9), "Orphan", WbsId(99)),
            Err(WbsError::UnknownNode(WbsId(99)))
        );
        assert_eq!(
            tree.add(WbsId(10), "   ", tree.root()),
            Err(WbsError::EmptyName)
        );
    }

    #[test]
    fn rename_returns_previous_name() {
        let (mut tree, design, _, _) = tree_with_children();
        assert_eq!(
            tree.rename(design, "Engineering").expect("rename"),
            "Design"
        );
        assert_eq!(tree.node(design).expect("design").name, "Engineering");
        assert_eq!(tree.rename(design, ""), Err(WbsError::EmptyName));
        assert_eq!(
            tree.rename(WbsId(99), "Ghost"),
            Err(WbsError::UnknownNode(WbsId(99)))
        );
    }

    #[test]
    fn move_reparents_and_preserves_subtree() {
        let (mut tree, _, build, concrete) = tree_with_children();
        let previous = tree.move_node(concrete, tree.root()).expect("move");
        assert_eq!(previous, Some(build));
        assert!(tree.node(build).expect("build").children.is_empty());
        assert_eq!(
            tree.node(concrete).expect("concrete").parent,
            Some(tree.root())
        );
        assert_eq!(tree.depth(concrete), Some(1));
    }

    #[test]
    fn move_rejects_cycles_and_root() {
        let (mut tree, design, build, _) = tree_with_children();
        assert_eq!(
            tree.move_node(build, WbsId(3)),
            Err(WbsError::Cycle {
                node: build,
                parent: build
            })
        );
        assert_eq!(
            tree.move_node(design, design),
            Err(WbsError::Cycle {
                node: design,
                parent: design
            })
        );
        // Moving a parent under its own descendant is a cycle too.
        let design_tree = {
            let mut tree = WbsTree::new();
            tree.add(WbsId(2), "Parent", tree.root()).expect("parent");
            tree.add(WbsId(3), "Child", WbsId(2)).expect("child");
            tree.add(WbsId(4), "Grandchild", WbsId(3))
                .expect("grandchild");
            tree
        };
        let mut design_tree = design_tree;
        assert_eq!(
            design_tree.move_node(WbsId(2), WbsId(4)),
            Err(WbsError::Cycle {
                node: WbsId(2),
                parent: WbsId(4)
            })
        );
        assert_eq!(
            tree.move_node(tree.root(), build),
            Err(WbsError::RootImmutable)
        );
    }

    #[test]
    fn delete_rejects_non_leaf_and_root() {
        let (mut tree, _, build, concrete) = tree_with_children();
        assert_eq!(tree.delete(build), Err(WbsError::HasChildren(build)));
        assert_eq!(tree.delete(tree.root()), Err(WbsError::RootImmutable));
        let removed = tree.delete(concrete).expect("delete leaf");
        assert_eq!(removed.id, concrete);
        assert!(tree.node(concrete).is_none());
        assert!(tree.node(build).expect("build").children.is_empty());
    }

    #[test]
    fn restore_undoes_delete() {
        let (mut tree, _, build, concrete) = tree_with_children();
        let snapshot = tree.delete(concrete).expect("delete");
        tree.restore(snapshot).expect("restore");
        assert_eq!(tree.node(concrete).expect("concrete").parent, Some(build));
        assert_eq!(tree.node(build).expect("build").children, vec![concrete]);
    }
}
