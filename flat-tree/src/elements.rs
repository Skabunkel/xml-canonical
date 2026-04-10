//! Defines elements that are stored in the tree.

#[derive(Debug, Clone, PartialEq)]
pub enum XDecorator {
  /// # Attribute
  /// This represents a singular attribute definition.<br/>
  /// Document structure: ([`XDecorator::XAttribute::prefix`]:)[`XDecorator::XAttribute::local_name`]="[`XDecorator::XAttribute::value`]"
  XAttribute {
    prefix: Option<Box<str>>,
    local_name: Box<str>,
    value: Box<str>,
  },

  /// # Namespace
  /// This represents a single namespace definition.<br/>
  /// Document structure: xmlns(:[`XDecorator::XNamespace::sufix`])="[`XDecorator::XNamespace::value`]"
  XNamespace {
    /// the sufix of the namespace.
    /// the sufix does not include "xmlns:"<br/>
    /// The sufix maybe none in which case its the default namespace.
    sufix: Option<Box<str>>,
    value: Box<str>,
  },
}

/// # XML Elements
/// The enum name is shamlessly stolen from microsoft.<br/>
#[derive(Debug, Clone, PartialEq)]
pub enum XNode {
  Tag {
    prefix: Option<Box<str>>,
    name: Box<str>,
    decorator: Option<Vec<XDecorator>>,
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
