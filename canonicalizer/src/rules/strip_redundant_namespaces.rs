use crate::canonicalizer::Rule;
use flat_tree::{
  elements::XNode,
  flat_tree::{Depth, FlatTree},
};
use std::collections::HashMap;

/// C14N: Remove namespace declarations where the same prefix→URI
/// binding is already in scope from an ancestor element.
pub struct StripRedundantNamespaces;

impl Rule for StripRedundantNamespaces {
  fn apply(&self, tree: &mut FlatTree) {
    // scope_stack: Vec<(depth, prefix→URI map at that depth)>
    let mut scope_stack: Vec<(Depth, HashMap<Option<String>, String>)> = Vec::new();
    let mut current_scope: HashMap<Option<String>, String> = HashMap::new();

    for i in 0..tree.len() {
      let depth = match tree.get(i) {
        Some((_, &d)) => d,
        None => continue,
      };

      // Pop scopes that are no longer active
      while let Some(&(d, _)) = scope_stack.last() {
        if d >= depth {
          let (_, parent_scope) = scope_stack.pop().unwrap();
          current_scope = parent_scope;
        } else {
          break;
        }
      }

      if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get_mut(i) {
        if let Some(ns_list) = namespaces {
          let parent_scope = current_scope.clone();

          // Filter out redundant declarations
          ns_list.retain(|ns| {
            let key = ns.prefix.as_deref().map(|s| s.to_string());
            let uri = ns.uri.to_string();
            let dominated = current_scope.get(&key).map(|u| u == &uri).unwrap_or(false);
            // Update current scope regardless
            current_scope.insert(key, uri);
            !dominated
          });

          if ns_list.is_empty() {
            *namespaces = None;
          }

          scope_stack.push((depth, parent_scope));
        } else {
          scope_stack.push((depth, current_scope.clone()));
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use flat_tree::elements::XNamespace;

  #[test]
  fn removes_redundant_ns() {
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
        prefix: Some("ns".into()),
        name: "child".into(),
        attributes: None,
        namespaces: Some(vec![XNamespace {
          prefix: Some("ns".into()),
          uri: "http://ns".into(),
        }]),
      },
      1,
    ));

    StripRedundantNamespaces.apply(&mut tree);

    // root should keep its ns decl, child should have it removed
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(0) {
      assert!(namespaces.is_some());
    }
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(1) {
      assert!(namespaces.is_none());
    }
  }

  #[test]
  fn keeps_overriding_ns() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: None,
        namespaces: Some(vec![XNamespace {
          prefix: Some("ns".into()),
          uri: "http://ns1".into(),
        }]),
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: Some("ns".into()),
        name: "child".into(),
        attributes: None,
        namespaces: Some(vec![XNamespace {
          prefix: Some("ns".into()),
          uri: "http://ns2".into(), // different URI
        }]),
      },
      1,
    ));

    StripRedundantNamespaces.apply(&mut tree);

    // Both should keep their ns decl since URIs differ
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(0) {
      assert!(namespaces.is_some());
    }
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(1) {
      assert!(namespaces.is_some());
    }
  }

  #[test]
  fn handles_default_namespace() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: None,
        namespaces: Some(vec![XNamespace {
          prefix: None,
          uri: "http://default".into(),
        }]),
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "child".into(),
        attributes: None,
        namespaces: Some(vec![XNamespace {
          prefix: None,
          uri: "http://default".into(),
        }]),
      },
      1,
    ));

    StripRedundantNamespaces.apply(&mut tree);

    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(1) {
      assert!(namespaces.is_none());
    }
  }

  #[test]
  fn sibling_scopes_independent() {
    let mut tree = FlatTree::new();
    // root (depth 0)
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: None,
        namespaces: None,
      },
      0,
    ));
    // child1 (depth 1) declares ns
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "a".into(),
        attributes: None,
        namespaces: Some(vec![XNamespace {
          prefix: Some("ns".into()),
          uri: "http://ns".into(),
        }]),
      },
      1,
    ));
    // child2 (depth 1) also declares ns — should NOT be stripped
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "b".into(),
        attributes: None,
        namespaces: Some(vec![XNamespace {
          prefix: Some("ns".into()),
          uri: "http://ns".into(),
        }]),
      },
      1,
    ));

    StripRedundantNamespaces.apply(&mut tree);

    // Both siblings should keep their ns decl
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(1) {
      assert!(namespaces.is_some());
    }
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(2) {
      assert!(namespaces.is_some());
    }
  }
}
