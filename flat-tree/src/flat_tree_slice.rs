//! Represents a slice of a tree it can be a tree or a subtree of an existing flat_tree

use crate::{
  elements::XNode,
  flat_tree::{Depth, FlatNodeRef},
};
use std::ops::Range;
use std::{iter::Zip, slice::Iter};

// I want the ability to canonicolize sub-trees
pub struct FlatTreeSlice<'a> {
  pub(crate) nodes: &'a [XNode],
  pub(crate) depth: &'a [Depth],
}

impl<'a> IntoIterator for &'a FlatTreeSlice<'a> {
  type Item = FlatNodeRef<'a>;
  type IntoIter = Zip<Iter<'a, XNode>, Iter<'a, Depth>>;

  fn into_iter(self) -> Self::IntoIter {
    self.nodes.iter().zip(self.depth.iter())
  }
}

impl FlatTreeSlice<'_> {
  pub fn slice(&self, range: Range<usize>) -> FlatTreeSlice<'_> {
    FlatTreeSlice {
      nodes: &self.nodes[range.clone()],
      depth: &self.depth[range],
    }
  }

  /// Creates a node iterator for those cases we want to work with nodes in the tree based on index<br/>
  /// Example: has_children(); or when we want to mutate them inplace.
  pub fn enumerator(&self) -> impl Iterator<Item = usize> {
    0..self.nodes.len()
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
  use crate::flat_tree::FlatTree;

  use super::*;

  #[test]
  fn node_depth_test() {
    let tree = test_data();
    let tree = tree.as_slice();
    let expected_depth: Vec<u8> = vec![0, 0, 1, 1, 2, 1, 2, 1, 2, 1, 2, 3];
    let depth_vector: Vec<u8> = tree.into_iter().map(|x| *x.1).collect();

    assert_eq!(expected_depth, depth_vector);
  }

  #[test]
  fn tree_ref_iterator() {
    let tree = test_data();
    let tree = tree.as_slice();

    let mut looped = false;
    for _flat_node in &tree {
      looped = true;
    }
    assert!(looped);
  }

  #[test]
  fn tree_enumerator() {
    let tree = test_data();
    let tree = tree.as_slice();

    let mut looped = false;
    for _flat_node in tree.enumerator() {
      looped = true;
    }
    assert!(looped);
  }

  #[test]
  fn has_children_false() {
    let tree = test_data();
    let tree = tree.as_slice();

    let has_child = tree.has_children(0);

    assert!(has_child.is_some());
    assert!(!has_child.unwrap());
  }

  #[test]
  fn has_children_false_end() {
    let tree = test_data();
    let tree = tree.as_slice();
    let len = tree.len() - 1;

    let has_child = tree.has_children(len);

    assert!(has_child.is_some());
    assert!(!has_child.unwrap());
  }

  #[test]
  fn has_children_true() {
    let tree = test_data();
    let tree = tree.as_slice();

    let has_child = tree.has_children(3);

    assert!(has_child.is_some());
    assert!(has_child.unwrap());
  }

  #[test]
  fn has_children_none() {
    let tree = test_data();
    let tree = tree.as_slice();
    let len = tree.len();

    let has_child = tree.has_children(len);

    assert!(has_child.is_none());
  }

  #[test]
  fn length_match_0() {
    let tree = FlatTree::default();
    let tree = tree.as_slice();

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

    let tree = tree.as_slice();

    let len = tree.len();
    assert_eq!(1, len);
    assert_eq!(tree.nodes.len(), len);
    assert_eq!(tree.depth.len(), len);
  }

  #[test]
  fn length_match_2() {
    let tree = test_data();
    let tree = tree.as_slice();

    let len = tree.len();
    assert_eq!(tree.nodes.len(), len);
    assert_eq!(tree.depth.len(), len);
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
