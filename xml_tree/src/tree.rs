use std::collections::BTreeMap;

// ── Node types ──────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct XAttribute {
    pub namespace: Option<u16>,
    pub value: Box<str>,
}

pub enum XNode {
    Tag {
        namespace: Option<u16>,
        name: Box<str>,
        attributes: Option<BTreeMap<Box<str>, XAttribute>>,
    },
    Text(Box<str>),
    Comment(Box<str>),
    ProcessingInstruction {
        target: Box<str>,
        data: Option<Box<str>>,
    },
}

// ── Flat tree ───────────────────────────────────────────────────────

pub struct FlatTree {
    nodes: Vec<XNode>,
    /// Parallel to `nodes` – depth of each node. Max 255 levels deep.
    depth: Vec<u8>,

    /// Namespace registry: (prefix, uri). Nodes reference by u8 index.
    // I need to move namespaces up into the nodes :/
    namespaces: Vec<(Box<str>, Box<str>)>,
    namespace_map: BTreeMap<Box<str>, usize>,
}

// Todo add a flattree iterator so i can go over each node and print them.

impl FlatTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            depth: Vec::new(),
            namespaces: Vec::new(),
            namespace_map: BTreeMap::new(),
        }
    }

    pub fn as_node(&self) -> Node {
        let size = self.len();

        if size != 0 {
            return Node { index: size - 1 };
        }
        Node { index: usize::MAX }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn depth_vector(&self) -> Vec<u8> {
        self.depth.clone()
    }

    /// Returns a `Node` cursor at the given index.
    pub fn node(&self, index: usize) -> Option<Node> {
        if index < self.len() {
            Some(Node { index })
        } else {
            None
        }
    }

    // Gets the xnode value at that index.
    pub fn value(&self, index: usize) -> Option<&XNode> {
        self.nodes.get(index)
    }

    // Looks for a node trying to figgure out if you have a s
    pub fn find_node(&self, target_name: &str) -> Option<Node> {
        match target_name.contains(':') {
            true => {
                let mut split = target_name.split(':');

                let ns = split.next()?;
                let name = split.next()?;

                let ns_index = self.find_namespace(Some(ns));

                self.find_namespaced_node_by_name(ns_index, name)
            }
            false => self.find_node_by_name(target_name),
        }
    }

    /// Looks for a node with the name ignoring namespace.
    pub fn find_node_by_name(&self, target_name: &str) -> Option<Node> {
        for (i, xnode) in self.nodes.iter().enumerate() {
            if let XNode::Tag {
                namespace: _,
                name,
                attributes: _,
            } = xnode
                && *target_name == **name
            {
                return Some(Node { index: i });
            }
        }

        None
    }

    /// Looks for a node with the name in the correct namespace.
    pub fn find_namespaced_node_by_name(
        &self,
        target_namespace: Option<u16>,
        target_name: &str,
    ) -> Option<Node> {
        for (i, xnode) in self.nodes.iter().enumerate() {
            if let XNode::Tag {
                namespace,
                name,
                attributes: _,
            } = xnode
                && *target_name == **name
                && target_namespace == *namespace
            {
                return Some(Node { index: i });
            }
        }

        None
    }

    // ── Mutation ────────────────────────────────────────────────────

    /// Append a node at the end of the tree.
    pub fn push(&mut self, node: XNode) -> Node {
        self.nodes.push(node);
        self.depth.push(1);
        let position = self.nodes.len() - 1;
        Node { index: position }
    }

    /// Append a node at the end of the tree.
    pub(crate) fn push_depth(&mut self, node: XNode, depth: u8) -> Node {
        self.nodes.push(node);
        self.depth.push(depth);
        let position = self.nodes.len() - 1;
        Node { index: position }
    }

    // ── Namespace registry ──────────────────────────────────────────

    /// Register a namespace. Returns its u8 index, or `None` if the
    /// registry is full (256 namespaces).
    pub fn add_namespace(&mut self, prefix: Box<str>, uri: Box<str>) -> Option<u16> {
        if self.namespace_map.contains_key(&prefix) {
            let index = self.namespace_map.get(&prefix).unwrap();
            return Some(*index as u16);
        }

        let id = self.namespaces.len();
        if id > u16::MAX as usize {
            return None;
        }
        self.namespaces.push((prefix.clone(), uri));
        self.namespace_map.insert(prefix, id);
        Some(id as u16)
    }

    /// Look up a namespace by its id.
    pub fn get_namespace(&self, id: Option<u16>) -> Option<(&str, &str)> {
      let id = id?;

      self.namespaces
          .get(id as usize)
          .map(|(p, u)| (p.as_ref(), u.as_ref()))
    }

    /// Find a namespace id by its prefix.
    pub fn find_namespace(&self, prefix: Option<&str>) -> Option<u16> {
      prefix?;

      self.namespace_map.get(prefix.unwrap()).map(|i| *i as u16)
    }
}

impl Default for FlatTree {
    fn default() -> Self {
        Self::new()
    }
}

// ── Node cursor ─────────────────────────────────────────────────────

/// Lightweight read-only view into a `FlatTree` at a specific index.
///
/// Navigation is derived purely from the depth array — no stored
/// parent/child pointers.

// Why is it just an usize? 
// Well I initally implemented it with a refrence to the source tree.
// It made the API better to work with but, it also added a bunch of borrows or refrences to the flattree
// And i did not like how that added alot more lifetime tracking so i removed it.
#[derive(Debug, Clone)]
pub struct Node {
    index: usize,
}

impl Node {
    pub fn is_sentinel(&self) -> bool {
        self.index == usize::MAX
    }

    /// Returns true if this node points to a valid position in the tree.
    pub fn is_valid(&self, tree: &FlatTree) -> bool {
        !self.is_sentinel() && self.index < tree.len()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn depth(&self, tree: &FlatTree) -> u8 {
        if self.is_sentinel() || self.index >= tree.len() {
            return 0;
        }
        tree.depth[self.index]
    }

    pub fn value<'a>(&self, tree: &'a FlatTree) -> Option<&'a XNode> {
        tree.value(self.index)
    }

    pub fn push(&self, tree: &mut FlatTree, node: XNode) -> Node {
        let depth = self.depth(tree) + 1;
        tree.push_depth(node, depth)
    }

    pub fn compare_name(&self, tree: &FlatTree, target_namespace: Option<u16>, target_name: &str) -> bool{
      let node = tree.value(self.index);
      match node {
        Some(node) => {
          match node {
            XNode::Tag { namespace, name, attributes: _ } => target_namespace == *namespace && *target_name == **name,
            XNode::Text(_) => false,
            XNode::Comment(_) => false,
            XNode::ProcessingInstruction { target: _, data: _ } => false,
          }
        },
        None => false,
      }
    }

    /// Scan backward to find the parent (first node with depth == self.depth - 1).
    pub fn parent(&self, tree: &FlatTree) -> Option<Node> {
        let d = self.depth(tree);
        if d == 0 {
            return None;
        }
        let target = d - 1;
        let mut i = self.index;
        while i > 0 {
            i -= 1;
            if tree.depth[i] == target {
                return Some(Node { index: i });
            }
        }
        None
    }

    /// Collect direct children (depth == self.depth + 1 within the subtree).
    pub fn children(&self, tree: &FlatTree) -> Vec<Node> {
        if !self.is_valid(tree) {
            return Vec::new();
        }
        let target = self.depth(tree) + 1;
        let end = self.subtree_end(tree);
        let mut result = Vec::new();
        let mut i = self.index + 1;
        while i < end {
            if tree.depth[i] == target {
                result.push(Node { index: i });
            }
            i += 1;
        }
        result
    }

    /// Next sibling: first node after this subtree with the same depth.
    pub fn next_sibling(&self, tree: &FlatTree) -> Option<Node> {
        if !self.is_valid(tree) {
            return None;
        }
        let d = self.depth(tree);
        let end = self.subtree_end(tree);
        if end < tree.len() && tree.depth[end] == d {
            Some(Node { index: end })
        } else {
            None
        }
    }

    /// Previous sibling: scan backward, skip nodes deeper than self,
    /// return first with same depth.
    pub fn prev_sibling(&self, tree: &FlatTree) -> Option<Node> {
        if !self.is_valid(tree) || self.index == 0 {
            return None;
        }
        let d = self.depth(tree);
        let mut i = self.index - 1;
        loop {
            let id = tree.depth[i];
            if id < d {
                return None;
            }
            if id == d {
                return Some(Node { index: i });
            }
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }

    /// All ancestors (parent, grandparent, … root).
    pub fn ancestors(&self, tree: &FlatTree) -> Vec<Node> {
        let mut result = Vec::new();
        let mut current = self.parent(tree);
        while let Some(node) = current {
            current = node.parent(tree);
            result.push(node);
        }
        result
    }

    /// All descendants (contiguous slice after self with depth > self.depth).
    pub fn descendants(&self, tree: &FlatTree) -> Vec<Node> {
        if !self.is_valid(tree) {
            return Vec::new();
        }
        let end = self.subtree_end(tree);
        (self.index + 1..end).map(|i| Node { index: i }).collect()
    }

    /// Index one past the last descendant of this node.
    pub fn subtree_end(&self, tree: &FlatTree) -> usize {
        if !self.is_valid(tree) {
            return 0;
        }
        let d = tree.depth[self.index];
        let mut i = self.index + 1;
        while i < tree.len() && tree.depth[i] > d {
            i += 1;
        }
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a small tree representing:
    /// ```xml
    /// <root>
    ///   <child attr="val">text</child>
    ///   <!-- comment -->
    /// </root>
    /// ```
    fn sample_tree() -> FlatTree {
        let mut root = FlatTree::new();

        let root_node = root.as_node();

        // <root>  depth 1
        let node = root_node.push(
            &mut root,
            XNode::Tag {
                namespace: None,
                name: "root".into(),
                attributes: None,
            },
        );

        // <child attr="val">  depth 2
        let mut attrs: BTreeMap<Box<str>, XAttribute> = BTreeMap::new();
        attrs.insert(
            "attr".into(),
            XAttribute {
                namespace: None,
                value: "val".into(),
            },
        );
        let child_node = node.push(
            &mut root,
            XNode::Tag {
                namespace: None,
                name: "child".into(),
                attributes: Some(attrs),
            },
        );

        // "text"  depth 3
        child_node.push(&mut root, XNode::Text("text".into()));

        // <!-- comment -->  depth 2
        node.push(&mut root, XNode::Comment(" comment ".into()));

        root
    }

    /// Build a small tree representing:
    /// ```xml
    /// <root1>
    /// </root1>
    /// <root2>
    /// </root2>
    /// ```
    fn empty_tree() -> FlatTree {
        let mut root = FlatTree::new();

        let root_node = root.as_node();

        // <root1>  depth 1
        _ = root_node.push(
            &mut root,
            XNode::Tag {
                namespace: None,
                name: "root1".into(),
                attributes: None,
            },
        );

        // <root2>  depth 1
        _ = root_node.push(
            &mut root,
            XNode::Tag {
                namespace: None,
                name: "root2".into(),
                attributes: None,
            },
        );

        root
    }

    #[test]
    fn depth_alignment() {
        let tree = sample_tree();
        assert_eq!(tree.len(), 4);
        let vector = tree.depth_vector();

        assert_eq!(vector, [1, 2, 3, 2]);
    }

    #[test]
    fn depth_alignment_empty() {
        let tree = empty_tree();
        assert_eq!(tree.len(), 2);
        let vector = tree.depth_vector();

        assert_eq!(vector, [1, 1]);
    }

    #[test]
    fn parent_navigation() {
        let tree = sample_tree();
        // root (depth 1) has no parent — nothing at depth 0 exists
        assert!(tree.node(0).unwrap().parent(&tree).is_none());
        // child -> root
        assert_eq!(tree.node(1).unwrap().parent(&tree).unwrap().index(), 0);
        // text -> child
        assert_eq!(tree.node(2).unwrap().parent(&tree).unwrap().index(), 1);
        // comment -> root
        assert_eq!(tree.node(3).unwrap().parent(&tree).unwrap().index(), 0);
    }

    #[test]
    fn children_navigation() {
        let tree = sample_tree();
        let root = tree.node(0).unwrap();
        let children: Vec<usize> = root.children(&tree).iter().map(|n| n.index()).collect();
        assert_eq!(children, vec![1, 3]); // <child> and <!-- comment -->

        let child = tree.node(1).unwrap();
        let grandchildren: Vec<usize> = child.children(&tree).iter().map(|n| n.index()).collect();
        assert_eq!(grandchildren, vec![2]); // "text"
    }

    #[test]
    fn sibling_navigation() {
        let tree = sample_tree();
        // child -> next sibling = comment
        assert_eq!(
            tree.node(1).unwrap().next_sibling(&tree).unwrap().index(),
            3
        );
        // comment -> no next sibling
        assert!(tree.node(3).unwrap().next_sibling(&tree).is_none());
        // comment -> prev sibling = child
        assert_eq!(
            tree.node(3).unwrap().prev_sibling(&tree).unwrap().index(),
            1
        );
        // child -> no prev sibling
        assert!(tree.node(1).unwrap().prev_sibling(&tree).is_none());
    }

    #[test]
    fn ancestors_and_descendants() {
        let tree = sample_tree();
        // text node ancestors: child, root
        let text = tree.node(2).unwrap();
        let anc: Vec<usize> = text.ancestors(&tree).iter().map(|n| n.index()).collect();
        assert_eq!(anc, vec![1, 0]);

        // root descendants: child, text, comment
        let root = tree.node(0).unwrap();
        let desc: Vec<usize> = root.descendants(&tree).iter().map(|n| n.index()).collect();
        assert_eq!(desc, vec![1, 2, 3]);
    }

    #[test]
    fn subtree_end() {
        let tree = sample_tree();
        // root's subtree covers everything
        assert_eq!(tree.node(0).unwrap().subtree_end(&tree), 4);
        // child's subtree: child + text
        assert_eq!(tree.node(1).unwrap().subtree_end(&tree), 3);
        // text is a leaf
        assert_eq!(tree.node(2).unwrap().subtree_end(&tree), 3);
        // comment is a leaf at the end
        assert_eq!(tree.node(3).unwrap().subtree_end(&tree), 4);
    }

    #[test]
    fn sentinel_node_push() {
        // The usize::MAX sentinel from as_node on an empty tree
        // should push at depth 1 (acting as a virtual root)
        let mut tree = FlatTree::new();
        let sentinel = tree.as_node();
        assert_eq!(sentinel.index(), usize::MAX);
        assert_eq!(sentinel.depth(&tree), 0);

        let root = sentinel.push(&mut tree, XNode::Text("hi".into()));
        assert_eq!(root.index(), 0);
        assert_eq!(root.depth(&tree), 1);
    }

    #[test]
    fn namespace_registry() {
        let mut tree = FlatTree::new();
        let id = tree.add_namespace("".into(), "http://example.com".into());
        assert_eq!(id, Some(0));
        let id2 = tree.add_namespace("ns".into(), "http://ns.example.com".into());
        assert_eq!(id2, Some(1));

        assert_eq!(tree.get_namespace(Some(0)), Some(("", "http://example.com")));
        assert_eq!(tree.find_namespace(Some("ns")), Some(1));
        assert_eq!(tree.find_namespace(Some("missing")), None);
    }

    #[test]
    fn sentinel_navigation_does_not_panic() {
        let tree = sample_tree();
        let sentinel = Node { index: usize::MAX };

        assert!(sentinel.is_sentinel());
        assert!(!sentinel.is_valid(&tree));
        assert_eq!(sentinel.depth(&tree), 0);
        assert!(sentinel.value(&tree).is_none());
        assert!(sentinel.parent(&tree).is_none());
        assert!(sentinel.children(&tree).is_empty());
        assert!(sentinel.next_sibling(&tree).is_none());
        assert!(sentinel.prev_sibling(&tree).is_none());
        assert!(sentinel.ancestors(&tree).is_empty());
        assert!(sentinel.descendants(&tree).is_empty());
        assert_eq!(sentinel.subtree_end(&tree), 0);
    }

    #[test]
    fn stale_node_does_not_panic() {
        let mut tree = FlatTree::new();
        let sentinel = tree.as_node();
        let node = sentinel.push(&mut tree, XNode::Text("a".into()));
        assert!(node.is_valid(&tree));

        // Remove the only node — now `node` is stale
        tree.nodes.clear();
        tree.depth.clear();

        assert!(!node.is_valid(&tree));
        assert_eq!(node.depth(&tree), 0);
        assert!(node.value(&tree).is_none());
        assert!(node.parent(&tree).is_none());
        assert!(node.children(&tree).is_empty());
        assert!(node.next_sibling(&tree).is_none());
        assert!(node.prev_sibling(&tree).is_none());
        assert!(node.descendants(&tree).is_empty());
        assert_eq!(node.subtree_end(&tree), 0);
    }
}
