//! Shared namespace scope-tracking utilities.
//!
//! Several canonicalization rules need to know which namespace prefix->URI
//! bindings are active at each node in the tree. This module provides a
//! single implementation to avoid duplicating that logic.

use flat_tree::{
  elements::{XDecorator, XNode},
  flat_tree::{Depth, FlatTree},
};
use std::collections::HashMap;

/// For each node index, compute the active namespace scope.
///
/// Returns a `Vec` of length `tree.len()`. Each entry is a `HashMap`
/// mapping `Option<String>` (where `None` = default namespace, `Some(prefix)` =
/// prefixed namespace) to the URI string.
pub fn build_namespace_scopes(tree: &FlatTree) -> Vec<HashMap<Option<String>, String>> {
  let len = tree.len();
  let mut result: Vec<HashMap<Option<String>, String>> = Vec::with_capacity(len);

  let mut scope_stack: Vec<(Depth, HashMap<Option<String>, String>)> = Vec::new();
  let mut current: HashMap<Option<String>, String> = HashMap::new();

  for i in 0..len {
    if let Some((node, &depth)) = tree.get(i) {
      // Pop scopes that are no longer active
      while let Some(&(d, _)) = scope_stack.last() {
        if d >= depth {
          let (_, parent) = scope_stack.pop().unwrap();
          current = parent;
        } else {
          break;
        }
      }

      if let XNode::Tag { decorator, .. } = node {
        let parent = current.clone();
        if let Some(decs) = decorator {
          for dec in decs {
            if let XDecorator::XNamespace { sufix, value } = dec {
              let key = sufix.as_deref().map(|s| s.to_string());
              current.insert(key, value.to_string());
            }
          }
        }
        scope_stack.push((depth, parent));
      }

      result.push(current.clone());
    } else {
      result.push(HashMap::new());
    }
  }

  result
}
