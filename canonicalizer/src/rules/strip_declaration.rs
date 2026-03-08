use crate::canonicalizer::Rule;
use flat_tree::{elements::XNode, flat_tree::FlatTree};

pub struct StripDeclaration;

impl Rule for StripDeclaration {
  fn apply(&self, tree: &mut FlatTree) {
    let mut i = tree.len();
    while i > 0 {
      i -= 1;
      if let Some((node, _)) = tree.get(i)
        && matches!(node, XNode::Declaration { .. })
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
  fn removes_declaration() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Declaration {
        version: "1.0".into(),
        encoding: None,
        standalone: None,
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: None,
      },
      0,
    ));

    StripDeclaration.apply(&mut tree);

    assert_eq!(tree.len(), 1);
    assert!(matches!(tree.get(0).unwrap().0, XNode::Tag { .. }));
  }

  #[test]
  fn no_declaration_noop() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: None,
      },
      0,
    ));

    StripDeclaration.apply(&mut tree);

    assert_eq!(tree.len(), 1);
  }
}
