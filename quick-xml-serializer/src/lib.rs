pub mod reader;
pub mod writer;

pub use flat_tree::flat_tree::FlatTree;
pub use reader::read_xml;
pub use writer::write_xml;

#[cfg(test)]
mod tests {
  use super::*;
  include!("tests.rs");
}
