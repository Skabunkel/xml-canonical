use crate::canonicalizer::Rule;
use flat_tree::{
  elements::{XDecorator, XNode},
  flat_tree::FlatTree,
};

/// Sort namespace declarations: default namespace first, then alphabetical by prefix.
pub struct SortNamespaces;

impl Rule for SortNamespaces {
  fn apply(&self, tree: &mut FlatTree) {
    for i in 0..tree.len() {
      if let Some((XNode::Tag { decorator: Some(decs), .. }, _)) = tree.get_mut(i) {
        // Collect namespace indices
        let mut ns_indices: Vec<usize> = decs
          .iter()
          .enumerate()
          .filter_map(|(idx, d)| matches!(d, XDecorator::XNamespace { .. }).then_some(idx))
          .collect();

        ns_indices.sort_by(|&a_idx, &b_idx| {
          let a_sufix = match &decs[a_idx] {
            XDecorator::XNamespace { sufix, .. } => sufix,
            _ => unreachable!(),
          };
          let b_sufix = match &decs[b_idx] {
            XDecorator::XNamespace { sufix, .. } => sufix,
            _ => unreachable!(),
          };
          match (a_sufix, b_sufix) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
          }
        });

        // Apply sorted order: one clone to extract, then swap back (avoids second clone)
        let mut sorted_ns: Vec<XDecorator> = ns_indices.iter().map(|&idx| decs[idx].clone()).collect();
        let mut sorted_idx = 0;
        for d in decs.iter_mut() {
          if matches!(d, XDecorator::XNamespace { .. }) {
            std::mem::swap(d, &mut sorted_ns[sorted_idx]);
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
  fn sorts_namespaces() {
    let mut tree = FlatTree::new();
    tree.push((
      XNode::Tag {
        prefix: None,
        name: "root".into(),
        decorator: Some(vec![
          XDecorator::XNamespace {
            sufix: Some("z".into()),
            value: "http://z".into(),
          },
          XDecorator::XNamespace {
            sufix: None,
            value: "http://default".into(),
          },
          XDecorator::XNamespace {
            sufix: Some("a".into()),
            value: "http://a".into(),
          },
        ]),
      },
      0,
    ));

    SortNamespaces.apply(&mut tree);

    if let Some((XNode::Tag { decorator: Some(decs), .. }, _)) = tree.get(0) {
      let ns: Vec<_> = decs
        .iter()
        .filter_map(|d| match d {
          XDecorator::XNamespace { sufix, .. } => Some(sufix.as_deref()),
          _ => None,
        })
        .collect();
      assert_eq!(ns, vec![None, Some("a"), Some("z")]);
    } else {
      panic!("expected tag");
    }
  }
}
