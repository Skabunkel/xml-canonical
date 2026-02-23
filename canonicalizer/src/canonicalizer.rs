use flat_tree::flat_tree::FlatTree;

pub trait Rule {
  fn apply(&self, tree: &mut FlatTree);
}

pub struct Canonicalizer {
  rules: Vec<Box<dyn Rule>>,
}
