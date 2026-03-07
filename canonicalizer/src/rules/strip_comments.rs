use crate::canonicalizer::Rule;
use flat_tree::{elements::XNode, flat_tree::FlatTree};

pub struct StripComments;

impl Rule for StripComments {
  fn apply(&self, tree: &mut FlatTree) {
    let mut i = tree.len();
    while i > 0 {
      i -= 1;
      if let Some((node, _)) = tree.get(i)
        && matches!(node, XNode::Comment(_))
      {
        tree.remove(i);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn removes_comments() {
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
    tree.push((XNode::Comment("a comment".into()), 1));
    tree.push((XNode::Text("text".into()), 1));
    tree.push((XNode::Comment("another".into()), 1));

    StripComments.apply(&mut tree);

    assert_eq!(tree.len(), 2);
    assert!(matches!(tree.get(0).unwrap().0, XNode::Tag { .. }));
    assert!(matches!(tree.get(1).unwrap().0, XNode::Text(_)));
  }

  #[test]
  fn no_comments_noop() {
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

    StripComments.apply(&mut tree);

    assert_eq!(tree.len(), 1);
  }
}
