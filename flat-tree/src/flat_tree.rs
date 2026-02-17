//! This is a [pre-orderd tree](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR) of xml elements.

use crate::elements::XNode;
use std::{iter::Zip, ops::Range, slice::Iter};

/// A simple depth type definition for easy future changes<br/>
/// If i ever change this to a u16 or something i want to do that once.<br/>
/// As of right now i only support 0-255 in depth.
pub type Depth = u8;
pub struct Node(usize);

/// A simple tuple for a node and depth pair.
pub type FlatNode = (XNode, Depth);
pub type FlatNodeRef<'a> = (&'a XNode, &'a Depth);

#[derive(Default)]
pub struct FlatTree {
  nodes: Vec<XNode>,
  depth: Vec<Depth>,
}

// I want the ability to canonicolize sub-trees
pub struct FlatTreeSlice<'a> {
  nodes: &'a [XNode],
  depth: &'a [Depth],
}

impl<'a> IntoIterator for &'a FlatTree {
  type Item = FlatNodeRef<'a>;
  type IntoIter = Zip<Iter<'a, XNode>, Iter<'a, Depth>>;

  fn into_iter(self) -> Self::IntoIter {
    self.nodes.iter().zip(self.depth.iter())
  }
}

impl<'a> IntoIterator for &'a FlatTreeSlice<'a> {
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

  pub fn push(&mut self, node: FlatNode) {
    self.nodes.push(node.0);
    self.depth.push(node.1);
  }

  pub fn pop(&mut self) -> Option<FlatNode> {
    let node = self.nodes.pop()?;
    let depth = self.depth.pop()?;

    Some((node, depth))
  }

  pub fn insert(&mut self, index: usize, node: FlatNode) {
    self.nodes.insert(index, node.0);
    self.depth.insert(index, node.1);
  }

  pub fn remove(&mut self, index: usize) -> FlatNode {
    let node = self.nodes.remove(index);
    let depth = self.depth.remove(index);

    (node, depth)
  }

  pub fn to_node_builder(&self) -> Node {
    let len = self.nodes.len();

    Node(len)
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
  fn tree_iterator() {
    let tree = test_data();

    let mut looped = false;
    for _flat_node in &tree {
      looped = true;
    }
    assert!(looped);
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
    // <!--this is a comment-->
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
