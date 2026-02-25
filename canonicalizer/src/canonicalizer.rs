use flat_tree::flat_tree::{FlatNodeRef, FlatTree};

pub enum RuleResult {
  Remove = 0,
}

pub trait Rule {
  fn apply(&self, node: FlatNodeRef, tree: &FlatTree);
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
    todo!()
  }

  /// Apply all rules in-place.                                                                                                       
  pub fn canonicalize_mut(&self, tree: &mut FlatTree) {
    todo!()
  }
}

impl Default for Canonicalizer {
  fn default() -> Self {
    Self::new()
  }
}
