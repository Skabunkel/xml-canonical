use xml_tree::{FlatTree, Node, XAttribute, XNode};
use quick_xml::events::{BytesStart, Event};
use std::collections::BTreeMap;
use quick_xml::{Reader};
use std::io::BufRead;

/// Parse XML from a `quick_xml::Reader` into a `FlatTree`.
///
/// The caller provides the reader (configured however they want) and
/// a reusable event buffer.
pub fn read<R: BufRead>(mut reader: Reader<R>, buf: &mut Vec<u8>) -> Result<FlatTree, quick_xml::Error> {
  let mut tree = FlatTree::new();
  let mut node_stack: Vec<Node> = Vec::new();
  let mut current_node = tree.as_node();

  loop {
    buf.clear();
    match reader.read_event_into(buf)? {
        Event::Start(ref e) => {
          let xnode = build_tag(&mut tree, e, &reader);
          node_stack.push(current_node.clone());
          current_node = current_node.push(&mut tree, xnode);
        }
        Event::End(ref e) => {
          let (local_name, prefix) = e.name().decompose();
          let local = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
          let prefix_owned = prefix.map(|p| std::str::from_utf8(p.as_ref()).unwrap_or("").to_string());
          let ns_id = tree.find_namespace(prefix_owned.as_deref());

          if current_node.compare_name(&tree, ns_id, local){
            let node = node_stack.pop();

            if node.is_none(){
              continue;
            }

            current_node = node.unwrap();  
          } /*else { // Handling broken xml, like <root><e1></root>... quick_xml returns an error when this happens... Sadness.
              for (i, node) in node_stack.iter().enumerate().rev()  {
                if node.compare_name(&tree, ns_id, local){

                  current_node = node.clone();
                  node_stack.truncate(i);
                  break;
                }
              }
          }*/
        }
        Event::Empty(ref e) => {
          let node = build_tag(&mut tree, e, &reader);
          _ = current_node.push(&mut tree, node);
        }
        Event::Text(ref e) => {
          let text = e.decode()?.into_owned().into_boxed_str();
          _ = current_node.push(&mut tree, XNode::Text(text));
        }
        Event::Comment(ref e) => {
          let text = e.decode()?.into_owned().into_boxed_str();
          _ = current_node.push(&mut tree, XNode::Comment(text));
        }
        Event::PI(ref e) => {
          let target = std::str::from_utf8(e.target())
            .unwrap_or("")
            .to_string()
            .into_boxed_str();
          let raw_content = e.content();
          let data = if raw_content.is_empty() {
            None
          } else {
            Some(
              std::str::from_utf8(raw_content)
                .unwrap_or("")
                .trim()
                .to_string()
                .into_boxed_str(),
            )
          };
          _ = current_node.push(&mut tree, XNode::ProcessingInstruction { target, data });
        }
        Event::Eof => break,
        _ => {} // I need to think about how i want to support some of the other nodes i have neglected here. 
    }
  }

  Ok(tree)
}

/// Build an `XNode::Tag` from a `BytesStart` event, registering any
/// xmlns declarations into the tree's namespace registry.
fn build_tag<R: BufRead>(tree: &mut FlatTree, e: &BytesStart, reader: &Reader<R>) -> XNode {
  let (local_name, prefix) = e.name().decompose();
  let local = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
  let prefix_owned = prefix.map(|p| std::str::from_utf8(p.as_ref()).unwrap_or("").to_string());

  let decoder = reader.decoder();
  let mut attributes = BTreeMap::new();
  let mut ns_id: Option<u16> = None;

  for attr_result in e.attributes() {
    let Ok(attr) = attr_result else { continue };
    let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
    let value = attr.decode_and_unescape_value(decoder).unwrap_or_default();

    if key == "xmlns" {
      ns_id = tree
          .add_namespace("".into(), value.into_owned().into_boxed_str());
    } else if let Some(ns_prefix) = key.strip_prefix("xmlns:") {
      tree.add_namespace(
        ns_prefix.to_string().into_boxed_str(),
        value.into_owned().into_boxed_str(),
      );
    } else {
    let (prefix, name) = format_tag_name(key);
      attributes.insert(
        name.to_string().into_boxed_str(),
        XAttribute {
          namespace: tree.find_namespace(prefix),
          value: value.into_owned().into_boxed_str(),
        },
      );
    }
  }

  XNode::Tag {
    namespace: ns_id.or(tree.find_namespace(prefix_owned.as_deref())),
    name: local.to_string().into_boxed_str(),
    attributes: if attributes.is_empty() {None} else {Some(attributes)},
  }
}

fn format_tag_name(key: &str) -> (Option<&str>, &str){
  let mut split = key.split(':');

  let prefix = split.next();
  let name = split.next();

  if name.is_none(){
    return (None, prefix.unwrap());
  }

  (prefix, name.unwrap())
}

#[cfg(test)]
mod tests {
  use super::*;
  use quick_xml::Reader;

  #[test]
  fn read_simple_xml() {
    let xml = r#"<root><child attr="val">text</child><!-- comment --></root>"#;
    let reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let tree = read(reader, &mut buf).unwrap();

    assert_eq!(tree.len(), 4);
    assert_eq!(tree.depth_vector(), [1, 2, 3, 2]);

    // root element
    let root = tree.node(0).unwrap();
    assert!(matches!(root.value(&tree), Some(XNode::Tag { name, .. }) if &**name == "root"));

    // child element with attribute
    let child = tree.node(1).unwrap();
    if let Some(XNode::Tag {
      name, attributes, ..
    }) = child.value(&tree)
    {
      assert_eq!(&**name, "child");
      assert!(attributes.is_some());
      let attributes = attributes.as_ref().unwrap();

      assert_eq!(&*attributes.get("attr" as &str).unwrap().value, "val");
    } else {
      panic!("expected Tag");
    }

    // text node
    assert!(matches!(tree.value(2), Some(XNode::Text(t)) if &**t == "text"));

    // comment
    assert!(matches!(tree.value(3), Some(XNode::Comment(c)) if &**c == " comment "));
  }

  #[test]
  fn read_with_namespaces() {
    let xml = r#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"><soap:Body/></soap:Envelope>"#;
    let reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let tree = read(reader, &mut buf).unwrap();

    assert_eq!(tree.len(), 2);
    assert_eq!(tree.depth_vector(), [1, 2]);

    // soap namespace should be registered
    let ns_id = tree.find_namespace(Some("soap"));
    assert_eq!(
      tree.get_namespace(ns_id),
      Some(("soap", "http://schemas.xmlsoap.org/soap/envelope/"))
    );

    // Both elements should reference the soap namespace
    if let Some(XNode::Tag {
        namespace, name, ..
    }) = tree.value(0)
    {
      assert_eq!(&**name, "Envelope");
      assert_eq!(*namespace, ns_id);
    }
    if let Some(XNode::Tag {
      namespace, name, ..
    }) = tree.value(1)
    {
      assert_eq!(&**name, "Body");
      assert_eq!(*namespace, ns_id);
    }
  }

  #[test]
  fn read_empty_elements() {
    let xml = r#"<root><a/><b/><c/></root>"#;
    let reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let tree = read(reader, &mut buf).unwrap();

    assert_eq!(tree.len(), 4);
    // root=1, all empty children=2
    assert_eq!(tree.depth_vector(), [1, 2, 2, 2]);
  }

  #[test]
  fn read_processing_instruction() {
    let xml = r#"<?xml-stylesheet href="style.css"?><root/>"#;
    let reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let tree = read(reader, &mut buf).unwrap();

    // PI at depth 1, root at depth 1
    assert!(!tree.is_empty());
    let has_pi = (0..tree.len())
      .any(|i| matches!(tree.value(i), Some(XNode::ProcessingInstruction { .. })));
    assert!(has_pi);
  }

  #[test]
  fn read_attribute_namespace() {
    let xml = r#"<root xmlns:ns="example"><ns:a ns:attr="spoon"/><b/><c/></root>"#;
    let reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let tree = read(reader, &mut buf).unwrap();

    assert_eq!(tree.len(), 4);
    // root=1, all empty children=2
    assert_eq!(tree.depth_vector(), [1, 2, 2, 2]);
    let node = tree.value(1);
    assert!(node.is_some());
    let node = node.unwrap();

    match node {
        XNode::Tag { namespace, name: _, attributes } => {
          assert!(namespace.is_some());
          assert_eq!(namespace.unwrap(), 0);

          assert!(attributes.is_some());
          let attributes = attributes.as_ref().unwrap();

          let attr = attributes.get("attr");

          assert!(attr.is_some());
          let attr = attr.unwrap();
          assert!(attr.namespace.is_some());
        },
        _ => unreachable!()
    }
  }

  /*#[test] Turns out quick_xml returns an error when this happens... Sadness.
  fn read_broken_xml() {
    let xml = r#"<root><e1><e2></e1></root>"#;
    let reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let tree = read(reader, &mut buf).unwrap();

    // PI at depth 1, root at depth 1
    assert_eq!(tree.len(), 2);
    assert_eq!(tree.depth_vector(), [1, 2]);
  }*/

  /* 
  #[test]
  fn advanced_xml_test() {
    let xml = r#"<!DOCTYPE doc [<!ATTLIST e9 attr CDATA "default">]>
<doc>
<e1   />
<e2   ></e2>
<e3   name = "elem3"   id="elem3"   />
<e4   name="elem4"   id="elem4"   ></e4>
<e5 a:attr="out" b:attr="sorted" attr2="all" attr="I'm"
  xmlns:b="http://www.ietf.org"
  xmlns:a="http://www.w3.org"
  xmlns="http://example.org"/>
<e6 xmlns="" xmlns:a="http://www.w3.org">
  <e7 xmlns="http://www.ietf.org">
      <e8 xmlns="" xmlns:a="http://www.w3.org">
        <e9 xmlns="" xmlns:a="http://www.ietf.org"/>
      </e8>
  </e7>
</e6>
</doc>"#;

    let reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let tree = read(reader, &mut buf).unwrap();

    print!("{}", tree.len());

    let len = tree.len(); // Sometimes when i set a breakpoint on the assert_eq bellow i end up breakin on a panic.
    assert_eq!(len, 11);
  }*/
}
