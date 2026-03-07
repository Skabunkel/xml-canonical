use crate::canonicalizer::Rule;
use flat_tree::{
  elements::{XNamespace, XNode},
  flat_tree::{Depth, FlatTree},
};
use std::collections::{HashMap, HashSet};

/// exc-C14N: Keep only namespace declarations that are visibly utilized
/// by the element's own prefix or its attribute prefixes.
/// Supports InclusiveNamespaces PrefixList for forcing additional prefixes.
///
/// Uses dual scope tracking:
/// - Input scope: prefix→URI bindings from the original (input) tree
/// - Output scope: prefix→URI bindings that have actually been emitted
///
/// If an element uses a prefix that's in the input scope but not in the
/// output scope, we add the declaration to the element.
pub struct VisiblyUtilizedNamespaces {
  prefix_list: HashSet<String>,
}

impl VisiblyUtilizedNamespaces {
  pub fn new(prefix_list: Vec<String>) -> Self {
    Self {
      prefix_list: prefix_list.into_iter().collect(),
    }
  }
}

impl Rule for VisiblyUtilizedNamespaces {
  fn apply(&self, tree: &mut FlatTree) {
    // First pass: build input scope map (prefix→URI at each Tag node index)
    let input_scopes = build_input_scopes(tree);

    // Second pass: process each tag, keeping only visibly utilized ns decls
    // and adding missing ones from input scope
    let mut output_stack: Vec<(Depth, HashMap<Option<String>, String>)> = Vec::new();
    let mut output_scope: HashMap<Option<String>, String> = HashMap::new();

    for (i, input_scope) in input_scopes.iter().enumerate() {
      let depth = match tree.get(i) {
        Some((_, &d)) => d,
        None => continue,
      };

      // Pop output scopes that are no longer active
      while let Some(&(d, _)) = output_stack.last() {
        if d >= depth {
          let (_, parent) = output_stack.pop().unwrap();
          output_scope = parent;
        } else {
          break;
        }
      }

      if let Some((
        XNode::Tag {
          prefix,
          namespaces,
          attributes,
          ..
        },
        _,
      )) = tree.get_mut(i)
      {
        // Collect visibly utilized prefixes
        let mut utilized: HashSet<Option<String>> = HashSet::new();

        // Element's own prefix
        utilized.insert(prefix.as_deref().map(|s| s.to_string()));

        // Attribute prefixes (only prefixed attributes — unprefixed attrs are
        // NOT in any namespace per the XML Namespaces spec)
        if let Some(attrs) = attributes {
          for attr in attrs.iter() {
            if let Some(p) = &attr.prefix {
              utilized.insert(Some(p.to_string()));
            }
          }
        }

        // Add inclusive namespace prefixes
        for p in &self.prefix_list {
          if p == "#default" {
            utilized.insert(None);
          } else {
            utilized.insert(Some(p.clone()));
          }
        }

        let parent_output = output_scope.clone();

        // Build the new namespace list: only utilized prefixes
        let mut new_ns: Vec<XNamespace> = Vec::new();

        for prefix_key in &utilized {
          // Look up the URI in the input scope
          if let Some(uri) = input_scope.get(prefix_key) {
            // Check if already in output scope with same URI
            let already_output =
              output_scope.get(prefix_key).map(|u| u == uri).unwrap_or(false);

            if !already_output {
              new_ns.push(XNamespace {
                prefix: prefix_key.as_deref().map(|s| s.into()),
                uri: uri.clone().into(),
              });
              output_scope.insert(prefix_key.clone(), uri.clone());
            }
          }
        }

        *namespaces = if new_ns.is_empty() { None } else { Some(new_ns) };

        output_stack.push((depth, parent_output));
      }
    }
  }
}

/// Build input scope map: for each node index, the full prefix→URI bindings
/// inherited from ancestors plus the node's own declarations.
fn build_input_scopes(tree: &FlatTree) -> Vec<HashMap<Option<String>, String>> {
  let len = tree.len();
  let mut result: Vec<HashMap<Option<String>, String>> = Vec::with_capacity(len);

  let mut scope_stack: Vec<(Depth, HashMap<Option<String>, String>)> = Vec::new();
  let mut current: HashMap<Option<String>, String> = HashMap::new();

  for i in 0..len {
    if let Some((node, &depth)) = tree.get(i) {
      // Pop scopes no longer active
      while let Some(&(d, _)) = scope_stack.last() {
        if d >= depth {
          let (_, parent) = scope_stack.pop().unwrap();
          current = parent;
        } else {
          break;
        }
      }

      if let XNode::Tag { namespaces, .. } = node {
        let parent = current.clone();
        if let Some(ns_list) = namespaces {
          for ns in ns_list {
            let key = ns.prefix.as_deref().map(|s| s.to_string());
            current.insert(key, ns.uri.to_string());
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

#[cfg(test)]
mod tests {
  use super::*;
  use flat_tree::elements::{XAttribute, XNamespace};

  #[test]
  fn keeps_utilized_removes_unused() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: None,
        namespaces: Some(vec![
          XNamespace {
            prefix: Some("used".into()),
            uri: "http://used".into(),
          },
          XNamespace {
            prefix: Some("unused".into()),
            uri: "http://unused".into(),
          },
        ]),
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: Some("used".into()),
        name: "child".into(),
        attributes: None,
        namespaces: None,
      },
      1,
    ));

    VisiblyUtilizedNamespaces::new(vec![]).apply(&mut tree);

    // Root: no element prefix used, no attribute prefixes → no ns needed
    // (root has no prefix, so default ns not utilized unless prefix is None AND there's a binding)
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(0) {
      assert!(namespaces.is_none());
    }

    // Child: uses "used" prefix → should get the declaration
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(1) {
      let ns = namespaces.as_ref().unwrap();
      assert_eq!(ns.len(), 1);
      assert_eq!(&**ns[0].prefix.as_ref().unwrap(), "used");
    }
  }

  #[test]
  fn attribute_prefix_utilized() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: Some(vec![XAttribute {
          prefix: Some("ns".into()),
          local_name: "attr".into(),
          value: "val".into(),
        }]),
        namespaces: Some(vec![XNamespace {
          prefix: Some("ns".into()),
          uri: "http://ns".into(),
        }]),
      },
      0,
    ));

    VisiblyUtilizedNamespaces::new(vec![]).apply(&mut tree);

    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(0) {
      let ns = namespaces.as_ref().unwrap();
      assert!(ns.iter().any(|n| n.prefix.as_deref() == Some("ns")));
    }
  }

  #[test]
  fn prefix_list_forces_inclusion() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        attributes: None,
        namespaces: Some(vec![XNamespace {
          prefix: Some("forced".into()),
          uri: "http://forced".into(),
        }]),
      },
      0,
    ));

    VisiblyUtilizedNamespaces::new(vec!["forced".to_string()]).apply(&mut tree);

    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(0) {
      let ns = namespaces.as_ref().unwrap();
      assert!(ns.iter().any(|n| n.prefix.as_deref() == Some("forced")));
    }
  }

  #[test]
  fn output_scope_prevents_duplicate_decls() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: Some("ns".into()),
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

    VisiblyUtilizedNamespaces::new(vec![]).apply(&mut tree);

    // Root emits ns decl
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(0) {
      let ns = namespaces.as_ref().unwrap();
      assert_eq!(ns.len(), 1);
    }
    // Child should NOT re-emit since output scope already has it
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(1) {
      assert!(namespaces.is_none());
    }
  }

  #[test]
  fn adds_missing_decl_from_ancestor() {
    // Namespace declared on root, used by grandchild via prefix
    // but not directly on grandchild's namespace list
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
        name: "mid".into(),
        attributes: None,
        namespaces: None,
      },
      1,
    ));
    tree.push((
      XNode::Tag {
        prefix: Some("ns".into()),
        name: "leaf".into(),
        attributes: None,
        namespaces: None,
      },
      2,
    ));

    VisiblyUtilizedNamespaces::new(vec![]).apply(&mut tree);

    // Root: doesn't use ns prefix itself → stripped
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(0) {
      assert!(namespaces.is_none());
    }
    // Mid: doesn't use any prefix → no ns
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(1) {
      assert!(namespaces.is_none());
    }
    // Leaf: uses "ns" prefix → should have declaration added from input scope
    if let Some((XNode::Tag { namespaces, .. }, _)) = tree.get(2) {
      let ns = namespaces.as_ref().unwrap();
      assert_eq!(ns.len(), 1);
      assert_eq!(&**ns[0].prefix.as_ref().unwrap(), "ns");
      assert_eq!(&*ns[0].uri, "http://ns");
    }
  }
}
