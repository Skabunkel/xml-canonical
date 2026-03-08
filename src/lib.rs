pub use flat_tree::{
  elements::XDecorator, elements::XNode, flat_tree::FlatNode,
  flat_tree::FlatNodeMutRef, flat_tree::FlatNodeRef, flat_tree::FlatTree,
  flat_tree_slice::FlatTreeSlice,
};

#[cfg(feature = "quick-serial")]
pub use quick_xml_serializer::{reader::read_xml, writer::write_xml};
