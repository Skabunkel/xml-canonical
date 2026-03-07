mod strip_declaration;
mod strip_comments;
mod normalize_line_endings;
mod sort_namespaces;
mod sort_attributes;
mod strip_redundant_namespaces;
mod visibly_utilized_namespaces;
mod expand_empty_elements;

pub use strip_declaration::StripDeclaration;
pub use strip_comments::StripComments;
pub use normalize_line_endings::NormalizeLineEndings;
pub use sort_namespaces::SortNamespaces;
pub use sort_attributes::SortAttributes;
pub use strip_redundant_namespaces::StripRedundantNamespaces;
pub use visibly_utilized_namespaces::VisiblyUtilizedNamespaces;
pub use expand_empty_elements::ExpandEmptyElements;
