mod expand_empty_elements;
mod normalize_line_endings;
mod sort_attributes;
mod sort_namespaces;
mod strip_comments;
mod strip_declaration;
mod strip_redundant_namespaces;
mod visibly_utilized_namespaces;

pub use expand_empty_elements::ExpandEmptyElements;
pub use normalize_line_endings::NormalizeLineEndings;
pub use sort_attributes::SortAttributes;
pub use sort_namespaces::SortNamespaces;
pub use strip_comments::StripComments;
pub use strip_declaration::StripDeclaration;
pub use strip_redundant_namespaces::StripRedundantNamespaces;
pub use visibly_utilized_namespaces::VisiblyUtilizedNamespaces;
