
#[test]
fn check_json() {
    use crate::spec_json::{BasicCategory, ExpansionCategory};
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("Cannot fetch directory of the current crate");
    let json_root = std::path::Path::new(&crate_dir).join("../../specs-json");
    let list_path = json_root.join("list.txt");
    let list = std::fs::read_to_string(list_path)
        .expect("Cannot read list of JSON spec files");

    for fname in list.lines() {
        eprintln!("Reading spec `{fname}`");
        let spec_path = json_root.join(fname);
        let spec_json = std::fs::read_to_string(spec_path)
            .expect("Cannot read spec JSON");
        if fname.contains("refs") {
            let _item: ExpansionCategory = serde_json::from_str(&spec_json)
                .expect("Cannot parse ExpansionCategory");
        } else {
            let _item: BasicCategory = serde_json::from_str(&spec_json)
                .expect("Cannot parse BasicCategory");
        }
    }
}

#[test]
fn check_xml() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("Cannot fetch directory of the current crate");
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
        let from_str = serde_xml_rs::from_str::<crate::spec_xml::Category>;
        #[cfg(not(feature = "xml_serde"))]
        let from_str = <crate::spec_xml::Category as hard_xml::XmlRead>::from_str;

        let src = std::fs::read_to_string(entry.path())
            .expect("Cannot read XML spec");
        let _item = from_str(&src).expect("Cannot parse XML spec");
    } 
}
