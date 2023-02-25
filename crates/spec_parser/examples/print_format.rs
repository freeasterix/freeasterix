//! This program prints all category formats in Python form for ASTERIX

use spec_parser::spec_xml::{Category, Format};

#[derive(Debug)]
enum Visit<'a> {
    CompoundEnter(&'a str, bool),
    CompoundExit,
    Prop(&'a str, &'a str),
}

fn visit_fields(format: &Format, visitor: &mut dyn FnMut(Visit)) {
    let children = match format {
        Format::Fixed(fixed) => {
            for bits in &fixed.bits {
                visitor(Visit::Prop(
                    &bits.short_name,
                    bits.name.as_ref().map(String::as_str).unwrap_or(""),
                ));
            }
            None
        }
        Format::Variable(variable) => Some(&variable.formats[..]),
        Format::Repetitive(repetitive) => Some(&repetitive.formats[..]),
        Format::Explicit(expl) => Some(&expl.formats[..]),
        Format::Compound(compound) => {
            let sspec = if let Format::Variable(sspec) = &compound.formats[0] {
                sspec
            } else {
                panic!("expected Variable to be subitem specifier");
            };
            let mut fields = Vec::new();
            for format in &sspec.formats {
                visit_fields(&format, &mut |visit| {
                    if let Visit::Prop(name, _) = visit {
                        if ["fx", "FX", "spare", "sb"].contains(&name) {
                            return;
                        }
                        fields.push(name.to_string());
                    } else {
                        panic!("unexpected visit {:?}", visit);
                    }
                });
            }
            assert_eq!(fields.len(), compound.formats.len() - 1, "{:?}", fields);
            for (field, format) in fields.iter().zip(&compound.formats[1..]) {
                let is_repetitive = matches!(&format, Format::Repetitive(_));
                visitor(Visit::CompoundEnter(field, is_repetitive));
                visit_fields(format, visitor);
                visitor(Visit::CompoundExit);
            }
            None
        }
        Format::BDS => {
            visitor(Visit::Prop("VAL", "BDS value"));
            visitor(Visit::Prop("BDS", "BDS Register"));
            None
        }
    };
    if let Some(children) = children {
        for format in children {
            visit_fields(format, visitor);
        }
    }
}

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string());
    let xml_root = std::path::Path::new(&crate_dir).join("../../specs-xml");

    for entry in std::fs::read_dir(xml_root).expect("Cannot read XML spec dir") {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name: &std::path::Path = name.as_ref();

        let name = name.file_name().unwrap().to_str().unwrap();
        if !name.starts_with("asterix_cat") {
            continue;
        }
        eprintln!("Reading spec `{name}`");

        #[cfg(feature = "xml_serde")]
        let from_str = serde_xml_rs::from_str::<Category>;
        #[cfg(not(feature = "xml_serde"))]
        let from_str = <Category as hard_xml::XmlRead>::from_str;

        let start = std::time::Instant::now();
        let src = std::fs::read_to_string(entry.path()).expect("Cannot read XML spec");
        let read_time = start.elapsed();
        let start = std::time::Instant::now();
        let cat = from_str(&src).expect("Cannot parse XML spec");

        let parse_time = start.elapsed();
        println!("# read in {:?} parsed in {:?}", read_time, parse_time);
        let cat_id = cat.id;
        println!("# category {cat_id:03}");
        println!("cat_{cat_id:03} = {{\n    \"CAT\": {cat_id},");
        let mut indent = "        ".to_string();
        fn bra_ket(is_array: bool) -> (&'static str, &'static str) {
            if is_array {
                ("[{", "}]")
            } else {
                ("{", "}")
            }
        }
        for data_item in &cat.data_items {
            let data_item_id = &data_item.id;
            let (open, close) = bra_ket(matches!(&data_item.format.format, Format::Repetitive(_)));
            if data_item.name.contains("\n") {
                panic!("newline in {}", data_item.name);
            }
            println!("    # name=       {}", data_item.name);
            for line in data_item.definition.trim().lines() {
                let line = line.trim();
                println!("    # definition= {}", line);
            }
            println!(
                "    # mandatory=  {:?}",
                data_item
                    .rule
                    .unwrap_or(spec_parser::spec_xml::Rule::Unknown)
            );
            println!("    \"{data_item_id}\": {open}");
            let mut close_stack = Vec::new();
            let format = &data_item.format.format;
            visit_fields(format, &mut |visit| match visit {
                Visit::CompoundEnter(compound, is_repetitive) => {
                    let (open, close) = bra_ket(is_repetitive);
                    println!("{indent}\"{compound}\": {open}");
                    indent.push_str("    ");
                    close_stack.push(close);
                }
                Visit::CompoundExit => {
                    indent.truncate(indent.len() - 4);
                    let close = close_stack.pop().unwrap();
                    println!("{indent}{close},");
                }
                Visit::Prop(prop, name) => {
                    if ["fx", "FX", "spare"].contains(&prop) {
                        return;
                    }
                    if !name.is_empty() {
                        println!("{indent}# {name}");
                    }
                    println!("{indent}\"{prop}\": 0,");
                }
            });
            println!("    {close},");
        }
        println!("}}");
    }
}
