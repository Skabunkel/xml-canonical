//! Defines elements that are stored in the tree.

/// # Attribute
/// This represents a singular attribute definition.<br/>
/// Document structure: ([`XAttribute::prefix`]:)[`XAttribute::local_name`]="[`XAttribute::value`]"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XAttribute {
  pub prefix: Option<Box<str>>,
  pub local_name: Box<str>,
  pub value: Box<str>,
}

/// # Namespace
/// This represents a single namespace definition.<br/>
/// Document structure: xmlns(:[`XNamespace::prefix`])="[`XNamespace::uri`]"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XNamespace {
  /// the prefix of the namespace.
  /// the prefix does not include "xmlns:"<br/>
  /// The prefix maybe none in which case its the default namespace.
  pub prefix: Option<Box<str>>,
  pub uri: Box<str>,
}

/// # XML Elements
/// The enum name is shamlessly stolen from microsoft.<br/>
#[derive(Debug, Clone, PartialEq)]
pub enum XNode {
  Tag {
    prefix: Option<Box<str>>,
    name: Box<str>,
    attributes: Option<Vec<XAttribute>>,
    namespaces: Option<Vec<XNamespace>>,
  },
  Text(Box<str>),
  Comment(Box<str>),
  ProcessingInstruction {
    target: Box<str>,
    data: Option<Box<str>>,
  },
  Declaration {
    version: Box<str>,
    encoding: Option<Box<str>>,
    standalone: Option<bool>,
  },
}
