use crate::{canonicalizer::Rule, scope::build_namespace_scopes};
use flat_tree::{
  elements::{XDecorator, XNode},
  flat_tree::FlatTree,
};

/// Sort attributes by (namespace URI, local_name).
/// Unprefixed attributes sort first (empty namespace URI).
/// Uses shared scope tracking to resolve attribute prefixes to URIs.
pub struct SortAttributes;

impl Rule for SortAttributes {
  fn apply(&self, tree: &mut FlatTree) {
    let scopes = build_namespace_scopes(tree);

    for (i, scope) in scopes.iter().enumerate() {
      if let Some((
        XNode::Tag {
          decorator: Some(decs),
          ..
        },
        _,
      )) = tree.get_mut(i)
      {
        // Sort only the XAttribute entries among themselves
        let mut attr_indices: Vec<usize> = decs
          .iter()
          .enumerate()
          .filter_map(|(idx, d)| matches!(d, XDecorator::XAttribute { .. }).then_some(idx))
          .collect();

        attr_indices.sort_by(|&a_idx, &b_idx| {
          let (a_prefix, a_local) = match &decs[a_idx] {
            XDecorator::XAttribute {
              prefix, local_name, ..
            } => (prefix, local_name),
            _ => unreachable!(),
          };
          let (b_prefix, b_local) = match &decs[b_idx] {
            XDecorator::XAttribute {
              prefix, local_name, ..
            } => (prefix, local_name),
            _ => unreachable!(),
          };

          // Resolve prefix -> URI via the shared scope map (keyed by Option<String>)
          let a_uri = a_prefix
            .as_deref()
            .and_then(|p| scope.get(&Some(p.to_string())))
            .map(|s| s.as_str())
            .unwrap_or("");
          let b_uri = b_prefix
            .as_deref()
            .and_then(|p| scope.get(&Some(p.to_string())))
            .map(|s| s.as_str())
            .unwrap_or("");
          a_uri.cmp(b_uri).then_with(|| a_local.cmp(b_local))
        });

        // Apply sorted order: one clone to extract, then swap back (avoids second clone)
        let mut sorted_attrs: Vec<XDecorator> =
          attr_indices.iter().map(|&idx| decs[idx].clone()).collect();
        let mut sorted_idx = 0;
        for d in decs.iter_mut() {
          if matches!(d, XDecorator::XAttribute { .. }) {
            std::mem::swap(d, &mut sorted_attrs[sorted_idx]);
            sorted_idx += 1;
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sorts_by_namespace_uri_then_local_name() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: Some(vec![
          XDecorator::XNamespace {
            sufix: Some("ns1".into()),
            value: "http://aaa".into(),
          },
          XDecorator::XNamespace {
            sufix: Some("ns2".into()),
            value: "http://bbb".into(),
          },
          XDecorator::XAttribute {
            prefix: Some("ns2".into()),
            local_name: "attr1".into(),
            value: "1".into(),
          },
          XDecorator::XAttribute {
            prefix: Some("ns1".into()),
            local_name: "attr2".into(),
            value: "2".into(),
          },
          XDecorator::XAttribute {
            prefix: None,
            local_name: "local".into(),
            value: "3".into(),
          },
        ]),
      },
      0,
    ));

    SortAttributes.apply(&mut tree);

    if let Some((
      XNode::Tag {
        decorator: Some(decs),
        ..
      },
      _,
    )) = tree.get(0)
    {
      let attrs: Vec<_> = decs
        .iter()
        .map(|d| match d {
          XDecorator::XAttribute { local_name, .. } => local_name.as_ref(),
          XDecorator::XNamespace { sufix, .. } => sufix.as_deref().unwrap(),
        })
        .collect();
      assert_eq!(attrs, vec!["ns1", "ns2", "local", "attr2", "attr1"]); // "" < "http://aaa" < "http://bbb"
    } else {
      panic!("expected tag");
    }
  }

  #[test]
  fn sorts_unprefixed_alphabetically() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: Some(vec![
          XDecorator::XAttribute {
            prefix: None,
            local_name: "zebra".into(),
            value: "1".into(),
          },
          XDecorator::XAttribute {
            prefix: None,
            local_name: "alpha".into(),
            value: "2".into(),
          },
        ]),
      },
      0,
    ));

    SortAttributes.apply(&mut tree);

    if let Some((
      XNode::Tag {
        decorator: Some(decs),
        ..
      },
      _,
    )) = tree.get(0)
    {
      let attrs: Vec<_> = decs
        .iter()
        .filter_map(|d| match d {
          XDecorator::XAttribute { local_name, .. } => Some(&**local_name),
          _ => None,
        })
        .collect();
      assert_eq!(attrs, vec!["alpha", "zebra"]);
    } else {
      panic!("expected tag");
    }
  }

  #[test]
  fn inherits_namespace_scope() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: Some(vec![XDecorator::XNamespace {
          sufix: Some("ns".into()),
          value: "http://ns".into(),
        }]),
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "child".into(),
        decorator: Some(vec![
          XDecorator::XAttribute {
            prefix: Some("ns".into()),
            local_name: "b".into(),
            value: "2".into(),
          },
          XDecorator::XAttribute {
            prefix: None,
            local_name: "a".into(),
            value: "1".into(),
          },
        ]),
      },
      1,
    ));

    SortAttributes.apply(&mut tree);

    if let Some((
      XNode::Tag {
        decorator: Some(decs),
        ..
      },
      _,
    )) = tree.get(1)
    {
      let attrs: Vec<_> = decs
        .iter()
        .filter_map(|d| match d {
          XDecorator::XAttribute { local_name, .. } => Some(&**local_name),
          _ => None,
        })
        .collect();
      assert_eq!(attrs, vec!["a", "b"]);
    } else {
      panic!("expected tag");
    }
  }
}
