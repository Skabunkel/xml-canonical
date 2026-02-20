//! This is a [pre-orderd tree](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR) of xml elements.

use crate::{elements::XNode, flat_tree_slice::FlatTreeSlice};
use std::{iter::Zip, ops::Range, slice::Iter};

/// A simple depth type definition for easy future changes<br/>
/// If i ever change this to a u16 or something i want to do that once.<br/>
/// As of right now i only support 0-255 in depth.
pub type Depth = u8;
// I should probably remove this.
//pub struct Node(usize);

/// A simple tuple for a node and depth pair.
pub type FlatNode = (XNode, Depth);
pub type FlatNodeRef<'a> = (&'a XNode, &'a Depth);
pub type FlatNodeMutRef<'a> = (&'a mut XNode, &'a mut Depth);

#[derive(Default)]
pub struct FlatTree {
  nodes: Vec<XNode>,
  depth: Vec<Depth>,
}

impl<'a> IntoIterator for &'a FlatTree {
  type Item = FlatNodeRef<'a>;
  type IntoIter = Zip<Iter<'a, XNode>, Iter<'a, Depth>>;

  fn into_iter(self) -> Self::IntoIter {
    self.nodes.iter().zip(self.depth.iter())
  }
}

impl FlatTree {
  pub fn as_slice(&self) -> FlatTreeSlice<'_> {
    FlatTreeSlice {
      nodes: &self.nodes[..],
      depth: &self.depth[..],
    }
  }

  pub fn slice(&self, range: Range<usize>) -> FlatTreeSlice<'_> {
    FlatTreeSlice {
      nodes: &self.nodes[range.clone()],
      depth: &self.depth[range],
    }
  }

  /// Appends to the end of the tree.
  pub fn push(&mut self, node: FlatNode) {
    self.nodes.push(node.0);
    self.depth.push(node.1);
  }

  /// Removes from the end of the tree.
  pub fn pop(&mut self) -> Option<FlatNode> {
    let node = self.nodes.pop()?;
    let depth = self.depth.pop()?;

    Some((node, depth))
  }

  /// Creats a node iterator for those cases we want to work with nodes in the tree based on index<br/>
  /// Example: has_children(); or when we want to mutate them inplace.
  pub fn enumerator(&self) -> impl Iterator<Item = usize> {
    0..self.nodes.len()
  }

  /// Inserts at node index, pushing the node at index forward.
  pub fn insert(&mut self, index: usize, node: FlatNode) -> bool {
    if index > self.nodes.len() {
      return false;
    }

    self.nodes.insert(index, node.0);
    self.depth.insert(index, node.1);
    true
  }

  /// Removes at node index.<br/>
  pub fn remove(&mut self, index: usize) -> Option<FlatNode> {
    if index > self.nodes.len() - 1 {
      return None;
    }

    let node = self.nodes.remove(index);
    let depth = self.depth.remove(index);

    Some((node, depth))
  }

  pub fn is_empty(&self) -> bool {
    self.nodes.is_empty()
  }

  pub fn get(&self, index: usize) -> Option<FlatNodeRef<'_>> {
    if self.nodes.is_empty() {
      return None;
    }
    let depth = self.depth.get(index)?;
    let node = self.nodes.get(index)?;

    Some((node, depth))
  }

  pub fn get_mut(&mut self, index: usize) -> Option<FlatNodeMutRef<'_>> {
    if self.nodes.is_empty() {
      return None;
    }
    let depth = self.depth.get_mut(index)?;
    let node = self.nodes.get_mut(index)?;

    Some((node, depth))
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  /// None if the current node does not exist.<br/>
  ///
  pub fn has_children(&self, index: usize) -> Option<bool> {
    let depth = self.depth.get(index)?;
    let neigbor_depth = self.depth.get(index + 1);

    match neigbor_depth {
      Some(ndepth) => Some(ndepth > depth),
      None => Some(false),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn node_depth_test() {
    let tree = test_data();
    let expected_depth: Vec<u8> = vec![0, 0, 1, 1, 2, 1, 2, 1, 2, 1, 2, 3];
    let depth_vector: Vec<u8> = tree.into_iter().map(|x| *x.1).collect();

    assert_eq!(expected_depth, depth_vector);
  }

  #[test]
  fn tree_ref_iterator() {
    let tree = test_data();

    let mut looped = false;
    for _flat_node in &tree {
      looped = true;
    }
    assert!(looped);
  }

  #[test]
  fn tree_enumerator() {
    let tree = test_data();

    let mut looped = false;
    for _flat_node in tree.enumerator() {
      looped = true;
    }
    assert!(looped);
  }

  #[test]
  fn has_children_false() {
    let tree = test_data();

    let has_child = tree.has_children(0);

    assert!(has_child.is_some());
    assert!(!has_child.unwrap());
  }

  #[test]
  fn has_children_false_end() {
    let tree = test_data();
    let len = tree.len() - 1;

    let has_child = tree.has_children(len);

    assert!(has_child.is_some());
    assert!(!has_child.unwrap());
  }

  #[test]
  fn has_children_true() {
    let tree = test_data();

    let has_child = tree.has_children(3);

    assert!(has_child.is_some());
    assert!(has_child.unwrap());
  }

  #[test]
  fn has_children_none() {
    let tree = test_data();
    let len = tree.len();

    let has_child = tree.has_children(len);

    assert!(has_child.is_none());
  }

  #[test]
  fn length_match_0() {
    let tree = FlatTree::default();

    let len = tree.len();
    assert_eq!(0, len);
    assert_eq!(tree.nodes.len(), len);
    assert_eq!(tree.depth.len(), len);
  }

  #[test]
  fn length_match_1() {
    let mut tree = FlatTree::default();
    tree.push((
      XNode::Declaration {
        version: "1.0".into(),
        encoding: None,
        standalone: None,
      },
      0,
    ));

    let len = tree.len();
    assert_eq!(1, len);
    assert_eq!(tree.nodes.len(), len);
    assert_eq!(tree.depth.len(), len);
  }

  #[test]
  fn length_match_2() {
    let tree = test_data();

    let len = tree.len();
    assert_eq!(tree.nodes.len(), len);
    assert_eq!(tree.depth.len(), len);
  }

  #[test]
  fn pop() {
    let mut tree = test_data();
    let original_len = tree.len();
    let last_node = tree.get(original_len - 1);
    assert!(last_node.is_some());
    let last_node = last_node.unwrap();

    let last_depth = *last_node.1;
    let last_node = last_node.0.clone();

    let pop_node = tree.pop();

    let len = tree.len();

    assert_ne!(original_len, len);
    assert_eq!(original_len - 1, len);
    assert_eq!(tree.depth.len(), len);
    assert_eq!(tree.nodes.len(), len);
    assert!(pop_node.is_some());
    let pop_node = pop_node.unwrap();

    assert_eq!(last_depth, pop_node.1);
    assert_eq!(last_node, pop_node.0);
  }

  #[test]
  fn remove_success() {
    let mut tree = test_data();
    let original_len = tree.len();
    let last_node = tree.get(0);
    assert!(last_node.is_some());
    let last_node = last_node.unwrap();

    let last_depth = *last_node.1;
    let last_node = last_node.0.clone();

    let remove_node = tree.remove(0);
    assert!(remove_node.is_some());
    let remove_node = remove_node.unwrap();

    let len = tree.len();

    assert_ne!(original_len, len);
    assert_eq!(original_len - 1, len);
    assert_eq!(tree.depth.len(), len);
    assert_eq!(tree.nodes.len(), len);

    assert_eq!(last_depth, remove_node.1);
    assert_eq!(last_node, remove_node.0);
  }

  #[test]
  fn remove_fail() {
    let mut tree = test_data();
    let remove_node = tree.remove(tree.len());
    assert!(remove_node.is_none());
  }

  #[test]
  fn remove_success_last() {
    let mut tree = test_data();
    let remove_node = tree.remove(tree.len() - 1);
    assert!(remove_node.is_some());
  }

  #[test]
  fn insert_success_last() {
    let mut tree = test_data();
    let insert_node = tree.insert(
      tree.len(),
      (
        XNode::Declaration {
          version: "1.0".into(),
          encoding: None,
          standalone: None,
        },
        0,
      ),
    );
    assert!(insert_node);
  }

  #[test]
  fn insert_fail() {
    let mut tree = test_data();
    let insert_node = tree.insert(
      tree.len() + 1,
      (
        XNode::Declaration {
          version: "1.0".into(),
          encoding: None,
          standalone: None,
        },
        0,
      ),
    );
    assert!(!insert_node);
  }

  fn test_data() -> FlatTree {
    let mut tree = FlatTree::default();

    //let root = tree.to_node_builder();

    // <?xml version = "1.0"><root>text1<node><!--this is a comment--></node><node><!--this is a comment again.--></node><t1>text2</t1><t2><t3>text3</t3></t2></root>

    // <?xml version = "1.0">
    tree.push((
      XNode::Declaration {
        version: "1.0".into(),
        encoding: None,
        standalone: None,
      },
      0,
    ));

    // <root>
    tree.push((
      XNode::Tag {
        namespaces: None,
        attributes: None,
        prefix: None,
        name: "root".into(),
      },
      0,
    ));
    // text1
    tree.push((XNode::Text("text1".into()), 1));
    // <node>
    tree.push((
      XNode::Tag {
        namespaces: None,
        attributes: None,
        prefix: None,
        name: "node".into(),
      },
      1,
    ));
    // <!--this is {a comment-->
    tree.push((XNode::Comment("this is a comment".into()), 2));

    // <node>
    tree.push((
      XNode::Tag {
        namespaces: None,
        attributes: None,
        prefix: None,
        name: "node".into(),
      },
      1,
    ));
    // <!--this is a comment again.--></node>
    tree.push((XNode::Comment("this is a comment again.".into()), 2));

    // <t1>text2</t1>
    tree.push((
      XNode::Tag {
        namespaces: None,
        attributes: None,
        prefix: None,
        name: "t1".into(),
      },
      1,
    ));
    // text2
    tree.push((XNode::Text("text2".into()), 2));

    //<t2><t3>text3</t3></t2></root>
    tree.push((
      XNode::Tag {
        namespaces: None,
        attributes: None,
        prefix: None,
        name: "t2".into(),
      },
      1,
    ));

    tree.push((
      XNode::Tag {
        namespaces: None,
        attributes: None,
        prefix: None,
        name: "t3".into(),
      },
      2,
    ));

    // text3
    tree.push((XNode::Text("text3".into()), 3));
    tree
  }
}
