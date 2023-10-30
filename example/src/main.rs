fn main() -> Result<(), Box<dyn std::error::Error>> {
    use spec_parser::spec_xml::Category;
    use obj2asterix::{read_asterix, write_asterix};
    use serde_json::{Map, Value};

    // 1) Read and parse the spec for a given category
    let xml = std::fs::read_to_string("../specs-xml/asterix_cat062_1_18.xml")
        .or_else(|_| std::fs::read_to_string("specs-xml/asterix_cat062_1_18.xml"))?;
    let category = Category::parse(&xml)?;

    // 2) Demonstration of parsing/deserialization
    let input = &[0x3E, 0x00, 0x06, 0x80, 0xB0, 0xB1][..];
    let mut reader = input;
    let value: Map<String, Value> = read_asterix(&mut reader, &category)?;
    println!("ASTERIX decoded: {:?}", value);

    // 3) Demonstration of serialization
    let mut writer = Vec::new();
    write_asterix(&mut writer, &category, &value)?;
    println!("ASTERIX encoded: {:x?}", writer);
    assert_eq!(&writer, input);
    Ok(())
}
