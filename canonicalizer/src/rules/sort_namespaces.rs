use crate::canonicalizer::Rule;
use flat_tree::{elements::XNode, flat_tree::FlatTree};

/// Sort namespace declarations: default namespace first, then alphabetical by prefix.
pub struct SortNamespaces;

impl Rule for SortNamespaces {
  fn apply(&self, tree: &mut FlatTree) {
    for i in 0..tree.len() {
      if let Some((
        XNode::Tag {
          namespaces: Some(ns),
          ..
        },
        _,
      )) = tree.get_mut(i)
      {
        ns.sort_by(|a, b| match (&a.prefix, &b.prefix) {
          (None, None) => std::cmp::Ordering::Equal,
          (None, Some(_)) => std::cmp::Ordering::Less,
          (Some(_), None) => std::cmp::Ordering::Greater,
          (Some(a), Some(b)) => a.cmp(b),
        });
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use flat_tree::elements::XNamespace;

  #[test]
  fn sorts_namespaces() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: None,
        namespaces: Some(vec![
          XNamespace {
            prefix: Some("z".into()),
            uri: "http://z".into(),
          },
          XNamespace {
            prefix: None,
            uri: "http://default".into(),
          },
          XNamespace {
            prefix: Some("a".into()),
            uri: "http://a".into(),
          },
        ]),
      },
      0,
    ));

    SortNamespaces.apply(&mut tree);

    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(0) {
      let ns = namespaces.as_ref().unwrap();
      assert!(ns[0].prefix.is_none());
      assert_eq!(&**ns[1].prefix.as_ref().unwrap(), "a");
      assert_eq!(&**ns[2].prefix.as_ref().unwrap(), "z");
    } else {
      panic!("expected tag");
    }
  }
}
