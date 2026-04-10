use flat_tree::{
  elements::{XDecorator, XNode},
  flat_tree::{Depth, FlatTree},
};
use quick_xml::{Reader, events::Event};

pub fn read_xml<R: std::io::BufRead>(
  source: R,
  tree: &mut FlatTree,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut reader = Reader::from_reader(source);
  let mut depth: Depth = 0;
  let mut buf = Vec::new();

  loop {
    buf.clear();
    match reader.read_event_into(&mut buf)? {
      Event::Decl(decl) => {
        let version = decl.version()?;
        let version = std::str::from_utf8(&version)?.to_string();
        let encoding = match decl.encoding() {
          Some(enc) => Some(Box::<str>::from(std::str::from_utf8(&enc?)?)),
          None => None,
        };
        let standalone = match decl.standalone() {
          Some(s) => Some(std::str::from_utf8(&s?)? == "yes"),
          None => None,
        };
        tree.push((
          XNode::Declaration {
            version: version.into(),
            encoding,
            standalone,
          },
          0,
        ));
      }

      Event::PI(pi) => {
        let raw = std::str::from_utf8(&pi)?;
        let (target, data) = match raw.find(|c: char| c.is_ascii_whitespace()) {
          Some(i) => (&raw[..i], Some(&raw[i..])),
          None => (raw, None),
        };
        tree.push((
          XNode::ProcessingInstruction {
            target: target.into(),
            data: data.map(Into::into),
          },
          depth,
        ));
      }

      Event::Start(start) => {
        let node = parse_tag(&start)?;
        tree.push((node, depth));
        depth += 1;
      }

      Event::Empty(empty) => {
        let node = parse_tag(&empty)?;
        tree.push((node, depth));
      }

      Event::End(_) => {
        depth -= 1;
      }

      Event::Text(text) => {
        let t = std::str::from_utf8(&text)?;
        tree.push((XNode::Text(t.into()), depth));
      }

      Event::Comment(comment) => {
        let c = std::str::from_utf8(&comment)?;
        tree.push((XNode::Comment(c.into()), depth));
      }

      Event::Eof => break,
      _ => {}
    }
  }

  Ok(())
}

fn parse_tag(
  start: &quick_xml::events::BytesStart<'_>,
) -> Result<XNode, Box<dyn std::error::Error>> {
  let name_qname = start.name();
  let name_str = std::str::from_utf8(name_qname.as_ref())?;
  let (prefix, local_name) = split_prefix(name_str);

  let mut decorators: Vec<XDecorator> = Vec::new();

  for attr_result in start.attributes() {
    let attr = attr_result?;
    let key = std::str::from_utf8(attr.key.as_ref())?;
    let value = attr.unescape_value()?;
    let value: &str = &value;

    if key == "xmlns" {
      decorators.push(XDecorator::XNamespace {
        sufix: None,
        value: value.into(),
      });
    } else if let Some(ns_prefix) = key.strip_prefix("xmlns:") {
      decorators.push(XDecorator::XNamespace {
        sufix: Some(ns_prefix.into()),
        value: value.into(),
      });
    } else {
      let (attr_prefix, attr_local) = split_prefix(key);
      decorators.push(XDecorator::XAttribute {
        prefix: attr_prefix.map(Into::into),
        local_name: attr_local.into(),
        value: value.into(),
      });
    }
  }

  Ok(XNode::Tag {
    prefix: prefix.map(Into::into),
    name: local_name.into(),
    decorator: if decorators.is_empty() {
      None
    } else {
      Some(decorators)
    },
  })
}

fn split_prefix(name: &str) -> (Option<&str>, &str) {
  match name.find(':') {
    Some(i) => (Some(&name[..i]), &name[i + 1..]),
    None => (None, name),
  }
}
