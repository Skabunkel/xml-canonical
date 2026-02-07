pub mod tree;

pub use tree::{FlatTree, Node, XAttribute, XNode};

#[cfg(all(feature = "quick_xml", feature = "xml_rs"))]
compile_error!("quick_xml and xml_rs are mutually exclusive, please choose one of them.");

#[cfg(feature = "quick_xml")]
pub mod quick_reader;

#[cfg(feature = "xml_rs")]
mod xml_reader;
