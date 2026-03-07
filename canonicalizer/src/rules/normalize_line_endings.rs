use crate::canonicalizer::Rule;
use flat_tree::{elements::XNode, flat_tree::FlatTree};

pub struct NormalizeLineEndings;

impl Rule for NormalizeLineEndings {
  fn apply(&self, tree: &mut FlatTree) {
    for i in 0..tree.len() {
      if let Some((node, _)) = tree.get_mut(i) {
        match node {
          XNode::Text(text) => {
            if text.contains('\r') {
              *text = normalize(text);
            }
          }
          XNode::Comment(text) => {
            if text.contains('\r') {
              *text = normalize(text);
            }
          }
          _ => {}
        }
      }
    }
  }
}

fn normalize(s: &str) -> Box<str> {
  s.replace("\r\n", "\n").replace('\r', "\n").into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalizes_crlf_in_text() {
    let mut tree = FlatTree::new();
    tree.push((XNode::Text("a\r\nb\rc".into()), 0));

    NormalizeLineEndings.apply(&mut tree);

    if let Some((XNode::Text(t), _)) = tree.get(0) {
      assert_eq!(&**t, "a\nb\nc");
    } else {
      panic!("expected text node");
    }
  }

  #[test]
  fn normalizes_crlf_in_comment() {
    let mut tree = FlatTree::new();
    tree.push((XNode::Comment("a\r\nb".into()), 0));

    NormalizeLineEndings.apply(&mut tree);

    if let Some((XNode::Comment(t), _)) = tree.get(0) {
      assert_eq!(&**t, "a\nb");
    } else {
      panic!("expected comment node");
    }
  }

  #[test]
  fn no_cr_noop() {
    let mut tree = FlatTree::new();
    tree.push((XNode::Text("no carriage returns".into()), 0));

    NormalizeLineEndings.apply(&mut tree);

    if let Some((XNode::Text(t), _)) = tree.get(0) {
      assert_eq!(&**t, "no carriage returns");
    } else {
      panic!("expected text node");
    }
  }
}
