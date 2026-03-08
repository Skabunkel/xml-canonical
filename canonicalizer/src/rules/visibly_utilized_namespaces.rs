use crate::canonicalizer::Rule;
use flat_tree::{
  elements::{XDecorator, XNode},
  flat_tree::{Depth, FlatTree},
};
use std::collections::{HashMap, HashSet};

/// exc-C14N: Keep only namespace declarations that are visibly utilized
/// by the element's own prefix or its attribute prefixes.
/// Supports InclusiveNamespaces PrefixList for forcing additional prefixes.
///
/// Uses dual scope tracking:
/// - Input scope: prefix->URI bindings from the original (input) tree
/// - Output scope: prefix->URI bindings that have actually been emitted
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
    // First pass: build input scope map (prefix->URI at each Tag node index)
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
          prefix, decorator, ..
        },
        _,
      )) = tree.get_mut(i)
      {
        // Collect visibly utilized prefixes
        let mut utilized: HashSet<Option<String>> = HashSet::new();

        // Element's own prefix
        utilized.insert(prefix.as_deref().map(|s| s.to_string()));

        // Attribute prefixes (only prefixed attributes -- unprefixed attrs are
        // NOT in any namespace per the XML Namespaces spec)
        if let Some(decs) = decorator.as_ref() {
          for dec in decs {
            if let XDecorator::XAttribute {
              prefix: Some(p), ..
            } = dec
            {
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
        let mut new_ns: Vec<XDecorator> = Vec::new();

        for prefix_key in &utilized {
          // Look up the URI in the input scope
          if let Some(uri) = input_scope.get(prefix_key) {
            // Check if already in output scope with same URI
            let already_output = output_scope
              .get(prefix_key)
              .map(|u| u == uri)
              .unwrap_or(false);

            if !already_output {
              new_ns.push(XDecorator::XNamespace {
                sufix: prefix_key.as_deref().map(|s| s.into()),
                value: uri.clone().into(),
              });
              output_scope.insert(prefix_key.clone(), uri.clone());
            }
          }
        }

        // Keep existing attributes, replace namespaces
        if let Some(decs) = decorator {
          decs.retain(|d| matches!(d, XDecorator::XAttribute { .. }));
          decs.extend(new_ns);
          if decs.is_empty() {
            *decorator = None;
          }
        } else if !new_ns.is_empty() {
          *decorator = Some(new_ns);
        }

        output_stack.push((depth, parent_output));
      }
    }
  }
}

/// Build input scope map: for each node index, the full prefix->URI bindings
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn keeps_utilized_removes_unused() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: Some(vec![
          XDecorator::XNamespace {
            sufix: Some("used".into()),
            value: "http://used".into(),
          },
          XDecorator::XNamespace {
            sufix: Some("unused".into()),
            value: "http://unused".into(),
          },
        ]),
      },
      0,
    ));
    tree.push((
      XNode::Tag {
        prefix: Some("used".into()),
        name: "child".into(),
        decorator: None,
      },
      1,
    ));

    VisiblyUtilizedNamespaces::new(vec![]).apply(&mut tree);

    // Root: no element prefix used, no attribute prefixes -> no ns needed
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(0) {
      assert!(decorator.is_none());
    }

    // Child: uses "used" prefix -> should get the declaration
    if let Some((
      XNode::Tag {
        decorator: Some(decs),
        ..
      },
      _,
    )) = tree.get(1)
    {
      let ns: Vec<_> = decs
        .iter()
        .filter_map(|d| match d {
          XDecorator::XNamespace { sufix, .. } => Some(sufix.as_deref()),
          _ => None,
        })
        .collect();
      assert_eq!(ns.len(), 1);
      assert_eq!(ns[0], Some("used"));
    } else {
      panic!("expected tag with decorators");
    }
  }

  #[test]
  fn attribute_prefix_utilized() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: Some(vec![
          XDecorator::XNamespace {
            sufix: Some("ns".into()),
            value: "http://ns".into(),
          },
          XDecorator::XAttribute {
            prefix: Some("ns".into()),
            local_name: "attr".into(),
            value: "val".into(),
          },
        ]),
      },
      0,
    ));

    VisiblyUtilizedNamespaces::new(vec![]).apply(&mut tree);

    if let Some((
      XNode::Tag {
        decorator: Some(decs),
        ..
      },
      _,
    )) = tree.get(0)
    {
      assert!(decs.iter().any(
        |d| matches!(d, XDecorator::XNamespace { sufix, .. } if sufix.as_deref() == Some("ns"))
      ));
    }
  }

  #[test]
  fn prefix_list_forces_inclusion() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: Some(vec![XDecorator::XNamespace {
          sufix: Some("forced".into()),
          value: "http://forced".into(),
        }]),
      },
      0,
    ));

    VisiblyUtilizedNamespaces::new(vec!["forced".to_string()]).apply(&mut tree);

    if let Some((
      XNode::Tag {
        decorator: Some(decs),
        ..
      },
      _,
    )) = tree.get(0)
    {
      assert!(decs.iter().any(
        |d| matches!(d, XDecorator::XNamespace { sufix, .. } if sufix.as_deref() == Some("forced"))
      ));
    }
  }

  #[test]
  fn output_scope_prevents_duplicate_decls() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: Some("ns".into()),
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

    VisiblyUtilizedNamespaces::new(vec![]).apply(&mut tree);

    // Root emits ns decl
    if let Some((
      XNode::Tag {
        decorator: Some(decs),
        ..
      },
      _,
    )) = tree.get(0)
    {
      let ns_count = decs
        .iter()
        .filter(|d| matches!(d, XDecorator::XNamespace { .. }))
        .count();
      assert_eq!(ns_count, 1);
    }
    // Child should NOT re-emit since output scope already has it
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(1) {
      assert!(decorator.is_none());
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
        name: "mid".into(),
        decorator: None,
      },
      1,
    ));
    tree.push((
      XNode::Tag {
        prefix: Some("ns".into()),
        name: "leaf".into(),
        decorator: None,
      },
      2,
    ));

    VisiblyUtilizedNamespaces::new(vec![]).apply(&mut tree);

    // Root: doesn't use ns prefix itself -> stripped
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(0) {
      assert!(decorator.is_none());
    }
    // Mid: doesn't use any prefix -> no ns
    if let Some((XNode::Tag { decorator, .. }, _)) = tree.get(1) {
      assert!(decorator.is_none());
    }
    // Leaf: uses "ns" prefix -> should have declaration added from input scope
    if let Some((
      XNode::Tag {
        decorator: Some(decs),
        ..
      },
      _,
    )) = tree.get(2)
    {
      let ns: Vec<_> = decs
        .iter()
        .filter_map(|d| match d {
          XDecorator::XNamespace { sufix, value } => Some((sufix.as_deref(), &**value)),
          _ => None,
        })
        .collect();
      assert_eq!(ns.len(), 1);
      assert_eq!(ns[0], (Some("ns"), "http://ns"));
    } else {
      panic!("expected tag with decorators");
    }
  }
}
