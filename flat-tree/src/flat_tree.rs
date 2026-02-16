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
