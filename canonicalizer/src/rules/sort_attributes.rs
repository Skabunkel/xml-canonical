use crate::canonicalizer::Rule;
use flat_tree::{
  elements::XNode,
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
      if let Some((XNode::Tag { attributes: Some(attrs), .. }, _)) = tree.get_mut(i) {
        attrs.sort_by(|a, b| {
          let a_uri = a
            .prefix
            .as_deref()
            .and_then(|p| scope.get(p))
            .map(|s| s.as_str())
            .unwrap_or("");
          let b_uri = b
            .prefix
            .as_deref()
            .and_then(|p| scope.get(p))
            .map(|s| s.as_str())
            .unwrap_or("");
          a_uri.cmp(b_uri).then_with(|| a.local_name.cmp(&b.local_name))
        });
      }
    }
  }
}

/// For each node index, compute the active prefix→URI namespace scope.
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

      if let XNode::Tag { namespaces, .. } = node {
        let parent_scope = current_scope.clone();
        if let Some(ns_list) = namespaces {
          for ns in ns_list {
            if let Some(prefix) = &ns.prefix {
              current_scope.insert(prefix.to_string(), ns.uri.to_string());
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
  use flat_tree::elements::{XAttribute, XNamespace};

  #[test]
  fn sorts_by_namespace_uri_then_local_name() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: Some(vec![
          XAttribute {
            prefix: Some("b".into()),
            local_name: "attr1".into(),
            value: "1".into(),
          },
          XAttribute {
            prefix: Some("a".into()),
            local_name: "attr2".into(),
            value: "2".into(),
          },
          XAttribute {
            prefix: None,
            local_name: "local".into(),
            value: "3".into(),
          },
        ]),
        namespaces: Some(vec![
          XNamespace {
            prefix: Some("a".into()),
            uri: "http://aaa".into(),
          },
          XNamespace {
            prefix: Some("b".into()),
            uri: "http://bbb".into(),
          },
        ]),
      },
      0,
    ));

    SortAttributes.apply(&mut tree);

    if let Some((XNode::Tag { attributes, .. }, _)) = tree.get(0) {
      let attrs = attributes.as_ref().unwrap();
      assert_eq!(&*attrs[0].local_name, "local");
      assert_eq!(&*attrs[1].local_name, "attr2"); // a → http://aaa
      assert_eq!(&*attrs[2].local_name, "attr1"); // b → http://bbb
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
        attributes: Some(vec![
          XAttribute {
            prefix: None,
            local_name: "zebra".into(),
            value: "1".into(),
          },
          XAttribute {
            prefix: None,
            local_name: "alpha".into(),
            value: "2".into(),
          },
        ]),
        namespaces: None,
      },
      0,
    ));

    SortAttributes.apply(&mut tree);

    if let Some((XNode::Tag { attributes, .. }, _)) = tree.get(0) {
      let attrs = attributes.as_ref().unwrap();
      assert_eq!(&*attrs[0].local_name, "alpha");
      assert_eq!(&*attrs[1].local_name, "zebra");
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
        attributes: None,
        namespaces: Some(vec![XNamespace {
          prefix: Some("ns".into()),
          uri: "http://ns".into(),
        }]),
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "child".into(),
        attributes: Some(vec![
          XAttribute {
            prefix: Some("ns".into()),
            local_name: "b".into(),
            value: "2".into(),
          },
          XAttribute {
            prefix: None,
            local_name: "a".into(),
            value: "1".into(),
          },
        ]),
        namespaces: None,
      },
      1,
    ));

    SortAttributes.apply(&mut tree);

    if let Some((XNode::Tag { attributes, .. }, _)) = tree.get(1) {
      let attrs = attributes.as_ref().unwrap();
      assert_eq!(&*attrs[0].local_name, "a");
      assert_eq!(&*attrs[1].local_name, "b");
    } else {
      panic!("expected tag");
    }
  }
}
