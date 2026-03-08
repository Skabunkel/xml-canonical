use flat_tree::{
  elements::XNode,
  flat_tree::{Depth, FlatTree},
};
use quick_xml::{
  Writer,
  events::{BytesDecl, BytesPI, BytesStart, BytesText, Event},
};
use std::io::{self, Write};

pub fn write_xml<W: Write>(tree: &FlatTree, writer: &mut W) -> Result<(), io::Error> {
  let mut writer = Writer::new(writer);

  let mut open_tags: Vec<(usize, Depth)> = Vec::new();
  let mut depth: Depth;

  for index in tree.enumerator() {
    if let Some(node) = tree.get(index) {
      depth = *node.1;

      while let Some(&(_, open_depth)) = open_tags.last() {
        if open_depth >= depth {
          let (node_index, _) = open_tags.pop().unwrap();
          let (tag, _) = tree.get(node_index).unwrap();
          write_closing_tag(tag, &mut writer);
        } else {
          break;
        }
      }

      match node.0 {
        XNode::Tag {
          prefix,
          name,
          decorator,
        } => {
          let has_children = tree.has_children(index).unwrap_or(false);
          write_tag(has_children, prefix, name, decorator, &mut writer);

          if has_children {
            open_tags.push((index, depth));
          }
        }
        XNode::Text(text) => {
          writer.write_event(Event::Text(BytesText::new(text)))?;
        }
        XNode::Comment(text) => {
          writer.write_event(Event::Comment(BytesText::new(text)))?;
        }
        XNode::ProcessingInstruction { target, data } => {
          let content = match data {
            Some(d) => format!("{target}{d}"), // This threw me for a loop
            None => target.to_string(),
          };
          writer.write_event(Event::PI(BytesPI::new(&content)))?;
        }
        XNode::Declaration {
          version,
          encoding,
          standalone,
        } => {
          let enc = encoding.as_deref();
          let sa = standalone.map(|b| if b { "yes" } else { "no" });
          writer.write_event(Event::Decl(BytesDecl::new(version, enc, sa)))?;
        }
      }
    }
  }

  while let Some((node_index, _)) = open_tags.pop() {
    let (tag, _) = tree.get(node_index).unwrap();
    write_closing_tag(tag, &mut writer);
  }

  Ok(())
}

fn write_closing_tag<W: Write>(node: &XNode, writer: &mut Writer<W>) {
  if let XNode::Tag {
    prefix,
    name,
    decorator: _,
  } = node
  {
    let name = make_qname(prefix, name);
    let _ = writer.write_event(Event::End(quick_xml::events::BytesEnd::new(name)));
  }
}

fn write_tag<W: Write>(
  has_children: bool,
  prefix: &Option<Box<str>>,
  name: &str,
  decorators: &Option<Vec<flat_tree::elements::XDecorator>>,
  writer: &mut Writer<&mut W>,
) {
  let qname = make_qname(prefix, name);

  let mut element = BytesStart::new(qname);

  if let Some(decorators) = decorators {
    for decorator in decorators {
      match decorator {
        flat_tree::elements::XDecorator::XAttribute {
          prefix,
          local_name,
          value,
        } => {
          let name = make_qname(&prefix, &local_name);
          element.push_attribute((name.as_str(), value.as_ref()));
        }
        flat_tree::elements::XDecorator::XNamespace { sufix, value } => match sufix {
          Some(p) => element.push_attribute((format!("xmlns:{p}").as_str(), value.as_ref())),
          None => element.push_attribute(("xmlns", value.as_ref())),
        },
      }
    }
  }

  let _ = match has_children {
    true => writer.write_event(Event::Start(element)),
    false => writer.write_event(Event::Empty(element)),
  };
}

fn make_qname(prefix: &Option<Box<str>>, name: &str) -> String {
  match prefix {
    Some(p) => format!("{p}:{name}"),
    None => name.to_string(),
  }
}
