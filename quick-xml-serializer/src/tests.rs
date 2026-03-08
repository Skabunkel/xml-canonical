fn round_trip(input: &str) -> String {
  let mut tree = FlatTree::default();
  let _ = read_xml(input.as_bytes(), &mut tree);
  let mut buf: Vec<u8> = Vec::new();
  let _ = write_xml(&tree, &mut buf);
  String::from_utf8(buf).unwrap()
}

#[test]
fn simple_element() {
  let xml = "<root><child>text</child></root>";
  assert_eq!(round_trip(xml), xml);
}

#[test]
fn attributes_and_namespaces() {
  let xml = r#"<root xmlns:ns="http://example.com" ns:attr="val">text</root>"#;
  assert_eq!(round_trip(xml), xml);
}

#[test]
fn comments_and_pis() {
  let xml = "<?pi data?><root><!-- comment --></root>";
  assert_eq!(round_trip(xml), xml);
}

#[test]
fn self_closing() {
  let xml = "<root><empty/></root>";
  assert_eq!(round_trip(xml), xml);
}

#[test]
fn nested_namespaces() {
  let xml = r#"<a xmlns="http://a"><b xmlns:x="http://x"><x:c/></b></a>"#;
  assert_eq!(round_trip(xml), xml);
}

#[test]
fn attributes_namespace_out_of_order() {
  let xml = r#"<a attr="asd" xmlns="http://a"><b xmlns:x="http://x"><x:c/></b></a>"#;
  assert_eq!(round_trip(xml), xml);
}

#[test]
fn attributes_namespace_in_of_order() {
  let xml = r#"<a xmlns="http://a" attr="asd"><b xmlns:x="http://x"><x:c/></b></a>"#;
  assert_eq!(round_trip(xml), xml);
}

#[test]
fn declaration() {
  let xml = r#"<?xml version="1.0" encoding="UTF-8"?><root/>"#;
  assert_eq!(round_trip(xml), xml);
}

#[test]
fn formatted() {
  let xml = r#"<doc>
   <clean>   </clean>
   <dirty>   A   B   </dirty>
   <mixed>
      A
      <clean>   </clean>
      B
      <dirty>   A   B   </dirty>
      C
   </mixed>
</doc>"#;
  assert_eq!(round_trip(xml), xml);
}
