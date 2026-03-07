use flat_tree::flat_tree::FlatTree;

pub trait Rule {
  fn apply(&self, tree: &mut FlatTree);
}

pub struct Canonicalizer {
  rules: Vec<Box<dyn Rule>>,
}

impl Canonicalizer {
  pub fn new() -> Self {
    Self { rules: Vec::new() }
  }

  /// Add a rule. Rules are applied in order.
  pub fn with_rule(mut self, rule: impl Rule + 'static) -> Self {
    self.rules.push(Box::new(rule));
    self
  }

  /// Clone the tree, apply all rules, and return the canonical tree.
  pub fn canonicalize(&self, tree: &FlatTree) -> FlatTree {
    let mut tree = tree.clone();
    self.canonicalize_mut(&mut tree);
    tree
  }

  /// Apply all rules in-place.
  pub fn canonicalize_mut(&self, tree: &mut FlatTree) {
    for rule in &self.rules {
      rule.apply(tree);
    }
  }

  /// C14N (Canonical XML 1.0)
  pub fn c14n(with_comments: bool) -> Self {
    use crate::rules::*;

    let mut c = Self::new()
      .with_rule(StripDeclaration)
      .with_rule(NormalizeLineEndings);

    if !with_comments {
      c = c.with_rule(StripComments);
    }

    c.with_rule(SortAttributes)
      .with_rule(StripRedundantNamespaces)
      .with_rule(SortNamespaces)
      .with_rule(ExpandEmptyElements)
  }

  /// Exclusive C14N (Exclusive Canonical XML 1.0)
  pub fn exc_c14n(with_comments: bool, prefix_list: Vec<String>) -> Self {
    use crate::rules::*;

    let mut c = Self::new()
      .with_rule(StripDeclaration)
      .with_rule(NormalizeLineEndings);

    if !with_comments {
      c = c.with_rule(StripComments);
    }

    c.with_rule(SortAttributes)
      .with_rule(VisiblyUtilizedNamespaces::new(prefix_list))
      .with_rule(SortNamespaces)
      .with_rule(ExpandEmptyElements)
  }
}

impl Default for Canonicalizer {
  fn default() -> Self {
    Self::new()
  }
}
