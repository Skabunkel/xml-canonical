use crate::canonicalizer::Rule;
use flat_tree::{
  elements::{XDecorator, XNode},
  flat_tree::{Depth, FlatTree},
};
use std::collections::HashMap;

/// C14N: Remove namespace declarations where the same prefix->URI
/// binding is already in scope from an ancestor element.
pub struct StripRedundantNamespaces;

impl Rule for StripRedundantNamespaces {
  fn apply(&self, tree: &mut FlatTree) {
    // scope_stack: Vec<(depth, prefix->URI map at that depth)>
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

      if let Some((XNode::Tag { decorator, .. }, _)) = tree.get_mut(i) {
        if let Some(decs) = decorator {
          let parent_scope = current_scope.clone();

          // Filter out redundant namespace declarations
          decs.retain(|d| match d {
            XDecorator::XNamespace { sufix, value } => {
              let key = sufix.as_deref().map(|s| s.to_string());
              let uri = value.to_string();
              let dominated = current_scope.get(&key).map(|u| u == &uri).unwrap_or(false);
              // Update current scope regardless
              current_scope.insert(key, uri);
              !dominated
            }
            _ => true, // keep attributes
          });

          // If only non-namespace decorators remain or empty, handle accordingly
          if decs.is_empty() {
            *decorator = None;
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

  #[test]
  fn removes_redundant_ns() {
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
        prefix: Some("ns".into()),
        name: "child".into(),
        decorator: Some(vec![XDecorator::XNamespace {
          sufix: Some("ns".into()),
          value: "http://ns".into(),
        }]),
      },
      1,
    ));

    StripRedundantNamespaces.apply(&mut tree);

    // root should keep its ns decl, child should have it removed
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(0) {
      assert!(decorator.is_some());
    }
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(1) {
      assert!(decorator.is_none());
    }
  }

  #[test]
  fn keeps_overriding_ns() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: Some(vec![XDecorator::XNamespace {
          sufix: Some("ns".into()),
          value: "http://ns1".into(),
        }]),
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: Some("ns".into()),
        name: "child".into(),
        decorator: Some(vec![XDecorator::XNamespace {
          sufix: Some("ns".into()),
          value: "http://ns2".into(), // different URI
        }]),
      },
      1,
    ));

    StripRedundantNamespaces.apply(&mut tree);

    // Both should keep their ns decl since URIs differ
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(0) {
      assert!(decorator.is_some());
    }
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(1) {
      assert!(decorator.is_some());
    }
  }

  #[test]
  fn handles_default_namespace() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: Some(vec![XDecorator::XNamespace {
          sufix: None,
          value: "http://default".into(),
        }]),
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "child".into(),
        decorator: Some(vec![XDecorator::XNamespace {
          sufix: None,
          value: "http://default".into(),
        }]),
      },
      1,
    ));

    StripRedundantNamespaces.apply(&mut tree);

    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(1) {
      assert!(decorator.is_none());
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
        decorator: None,
      },
      0,
    ));
    // child1 (depth 1) declares ns
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "a".into(),
        decorator: Some(vec![XDecorator::XNamespace {
          sufix: Some("ns".into()),
          value: "http://ns".into(),
        }]),
      },
      1,
    ));
    // child2 (depth 1) also declares ns -- should NOT be stripped
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "b".into(),
        decorator: Some(vec![XDecorator::XNamespace {
          sufix: Some("ns".into()),
          value: "http://ns".into(),
        }]),
      },
      1,
    ));

    StripRedundantNamespaces.apply(&mut tree);

    // Both siblings should keep their ns decl
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(1) {
      assert!(decorator.is_some());
    }
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(2) {
      assert!(decorator.is_some());
    }
  }
}
