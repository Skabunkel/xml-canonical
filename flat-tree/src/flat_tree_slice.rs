//! Represents a subtree of an existing flat_tree

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

  /// Creats a node iterator for those cases we want to work with nodes in the tree based on index<br/>
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
