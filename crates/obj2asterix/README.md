# ASTERIX packet serializer and deserializer

The purpose of this crate is to convert a JSON map to an ASTERIX packet and back. 

Example:
```rust
use spec_parser::spec_xml::Category;
use obj2asterix::read_asterix;

let xml = fs::read_to_string("../../specs-xml/asterix_cat062_1_18.xml")?;
let category = Category::parse(&xml)?;
let mut reader = &[0x3E, 0x00, 0x06, 0x80, 0xB0, 0xB1][..];
let value = read_asterix(&mut reader, &category)?;
dbg!(value);
```