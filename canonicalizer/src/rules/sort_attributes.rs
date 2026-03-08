use crate::canonicalizer::Rule;
use flat_tree::{
  elements::{XDecorator, XNode},
  flat_tree::{Depth, FlatTree},
};
use std::collections::HashMap;

/// Sort attributes by (namespace URI, local_name).
/// Unprefixed attributes sort first (empty namespace URI).
/// Uses scope tracking to resolve attribute prefixes to URIs.
pub struct SortAttributes;

impl Rule for SortAttributes {
  fn apply(&self, tree: &mut FlatTree) {
    let scope_map = build_scope_map(tree);

    for (i, scope) in scope_map.iter().enumerate() {
      if let Some((XNode::Tag { decorator: Some(decs), .. }, _)) = tree.get_mut(i) {
        // Sort only the XAttribute entries among themselves
        let mut attr_indices: Vec<usize> = decs
          .iter()
          .enumerate()
          .filter_map(|(idx, d)| matches!(d, XDecorator::XAttribute { .. }).then_some(idx))
          .collect();

        attr_indices.sort_by(|&a_idx, &b_idx| {
          let (a_prefix, a_local) = match &decs[a_idx] {
            XDecorator::XAttribute { prefix, local_name, .. } => (prefix, local_name),
            _ => unreachable!(),
          };
          let (b_prefix, b_local) = match &decs[b_idx] {
            XDecorator::XAttribute { prefix, local_name, .. } => (prefix, local_name),
            _ => unreachable!(),
          };

          let a_uri = a_prefix
            .as_deref()
            .and_then(|p| scope.get(p))
            .map(|s| s.as_str())
            .unwrap_or("");
          let b_uri = b_prefix
            .as_deref()
            .and_then(|p| scope.get(p))
            .map(|s| s.as_str())
            .unwrap_or("");
          a_uri.cmp(b_uri).then_with(|| a_local.cmp(b_local))
        });

        // Apply the sorted order by rebuilding the vec
        let sorted_attrs: Vec<XDecorator> = attr_indices.iter().map(|&idx| decs[idx].clone()).collect();
        let mut sorted_idx = 0;
        for d in decs.iter_mut() {
          if matches!(d, XDecorator::XAttribute { .. }) {
            *d = sorted_attrs[sorted_idx].clone();
            sorted_idx += 1;
          }
        }
      }
    }
  }
}

/// For each node index, compute the active prefix->URI namespace scope.
fn build_scope_map(tree: &FlatTree) -> Vec<HashMap<String, String>> {
  let len = tree.len();
  let mut result: Vec<HashMap<String, String>> = Vec::with_capacity(len);

  let mut scope_stack: Vec<(Depth, HashMap<String, String>)> = Vec::new();
  let mut current_scope: HashMap<String, String> = HashMap::new();

  for i in 0..len {
    if let Some((node, &depth)) = tree.get(i) {
      // Pop scopes that are no longer active
      while let Some(&(d, _)) = scope_stack.last() {
        if d >= depth {
          let (_, parent_scope) = scope_stack.pop().unwrap();
          current_scope = parent_scope;
        } else {
          break;
        }
      }

      if let XNode::Tag { decorator, .. } = node {
        let parent_scope = current_scope.clone();
        if let Some(decs) = decorator {
          for dec in decs {
            if let XDecorator::XNamespace { sufix: Some(prefix), value } = dec {
              current_scope.insert(prefix.to_string(), value.to_string());
            }
          }
        }
        scope_stack.push((depth, parent_scope));
      }

      result.push(current_scope.clone());
    } else {
      result.push(HashMap::new());
    }
  }

  result
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
            sufix: Some("a".into()),
            value: "http://aaa".into(),
          },
          XDecorator::XNamespace {
            sufix: Some("b".into()),
            value: "http://bbb".into(),
          },
          XDecorator::XAttribute {
            prefix: Some("b".into()),
            local_name: "attr1".into(),
            value: "1".into(),
          },
          XDecorator::XAttribute {
            prefix: Some("a".into()),
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

    if let Some((XNode::Tag { decorator: Some(decs), .. }, _)) = tree.get(0) {
      let attrs: Vec<_> = decs
        .iter()
        .filter_map(|d| match d {
          XDecorator::XAttribute { local_name, .. } => Some(&**local_name),
          _ => None,
        })
        .collect();
      assert_eq!(attrs, vec!["local", "attr2", "attr1"]); // "" < "http://aaa" < "http://bbb"
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

    if let Some((XNode::Tag { decorator: Some(decs), .. }, _)) = tree.get(0) {
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

    if let Some((XNode::Tag { decorator: Some(decs), .. }, _)) = tree.get(1) {
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
