use crate::canonicalizer::Rule;
use flat_tree::{elements::XNode, flat_tree::FlatTree};

pub struct ExpandEmptyElements;

impl Rule for ExpandEmptyElements {
  fn apply(&self, tree: &mut FlatTree) {
    let mut i = 0;
    while i < tree.len() {
      if let Some((node, depth)) = tree.get(i)
        && matches!(node, XNode::Tag { .. })
      {
        let has_children = tree.has_children(i).unwrap_or(false);
        if !has_children {
          let child_depth = depth + 1;
          tree.insert(i + 1, (XNode::Text("".into()), child_depth));
          i += 1; // skip the inserted node
        }
      }
      i += 1;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn expands_empty_element() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: None,
        namespaces: None,
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "empty".into(),
        attributes: None,
        namespaces: None,
      },
      1,
    ));

    ExpandEmptyElements.apply(&mut tree);

    assert_eq!(tree.len(), 3);
    assert!(matches!(tree.get(2).unwrap().0, XNode::Text(_)));
    assert_eq!(*tree.get(2).unwrap().1, 2);
  }

  #[test]
  fn leaves_nonempty_alone() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: None,
        namespaces: None,
      },
      0,
    ));
    tree.push((XNode::Text("content".into()), 1));

    ExpandEmptyElements.apply(&mut tree);

    assert_eq!(tree.len(), 2);
  }
}
