use crate::read_asterix;
use crate::write_asterix;
use serde_json::{Map, Value};
use spec_parser::spec_xml::Category;
use std::collections::BTreeMap;
use std::path::Path;

fn read_specs(root: impl AsRef<Path>) -> BTreeMap<u8, Category> {
    let root = root.as_ref();
    let mut rv = BTreeMap::new();
    let xml_root = root.join("specs-xml");
    for entry in std::fs::read_dir(&xml_root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if !(path.is_file()
            && path.extension().map_or(false, |ext| ext == "xml")
            && name.starts_with("asterix_cat"))
        {
            continue;
        }
        let category = name[11..14]
            .parse::<u8>()
            .expect("could not parse spec id from file name");
        let spec_src = std::fs::read_to_string(&path).expect("could not read spec");
        let spec = Category::parse(&spec_src).expect("could not parse spec");
        rv.insert(category, spec);
    }
    rv
}

fn convert_record(record: &Value) -> Value {
    let record = record.as_object().unwrap();
    let mut result = Map::new();
    for (key, d_in) in record.iter() {
        if ["FSPEC", "index", "length"].contains(&key.as_str()) {
            continue;
        }
        let mut d_in = d_in.clone();
        if let Value::Object(obj) = &mut d_in {
            obj.retain(|s, _| !s.starts_with("FX"));
        }
        result.insert(key.into(), d_in.clone());
    }
    Value::Object(result)
}

fn convert_json(input: &Value) -> Map<String, Value> {
    let input = input.as_object().expect("input must be an object.");
    let category = input
        .get("category")
        .expect("category must be present")
        .as_u64()
        .expect("category must be an int")
        .into();
    let mut result = Map::new();
    result.insert("CAT".into(), category);
    let records = input["content"]["records"]
        .as_array()
        .expect("records must be an array");
    let mut records = records.iter().map(convert_record).collect::<Vec<Value>>();
    if records.len() == 1 {
        if let Some(Value::Object(mut record)) = records.pop() {
            record.insert("CAT".into(), result["CAT"].clone());
            record
        } else {
            panic!("bad input");
        }
    } else {
        result.insert("records".into(), records.into());
        result
    }
}

fn test_one_sample(
    bin: &[u8],
    spec: &Category,
    json: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let data_blocks = json
        .get("data_blocks")
        .ok_or_else(|| "missing data_blocks")?
        .as_array()
        .ok_or_else(|| "data_blocks is not an array")?;
    for block in data_blocks {
        let block = convert_json(block);
        let start = buffer.len();
        write_asterix(&mut buffer, spec, &block)?;
        let mut chunk = &buffer[start..];
        let data = read_asterix(&mut chunk, spec)?;
        if data != block {
            println!("ser {}", Value::Object(block).to_string());
            println!("des {}", Value::Object(data).to_string());
            return Err("mismatch between serialized and parsed data".into());
        }
    }
    if bin != &buffer {
        println!("exp={:x?}", bin);
        println!("got={:x?}", buffer);
        Err("mismatch between test data and serialize".into())
    } else {
        Ok(())
    }
}

#[test]
fn test_samples() {
    let crate_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Cannot fetch directory of the current crate");
    let root = Path::new(&crate_dir).join("../..");
    let specs = read_specs(&root);

    let sample_dir = root.join("samples");
    let mut errors = 0;
    for entry in std::fs::read_dir(&sample_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if !(path.is_dir() && name.starts_with("cat")) {
            continue;
        }
        let category = name[3..6]
            .parse::<u8>()
            .expect("could not parse spec id from dir name");
        let spec = if let Some(spec) = specs.get(&category) {
            spec
        } else {
            panic!("missing category {}", category);
        };

        for file in std::fs::read_dir(&path).unwrap() {
            let file = file.unwrap();
            let path = file.path();
            if !(path.is_file() && path.extension().map_or(false, |ext| ext == "json")) {
                continue;
            }
            let bin_path = path.with_extension("bin");
            println!("\ntesting {path:?}");
            let json = std::fs::read_to_string(&path).unwrap();
            let json = serde_json::from_str::<serde_json::Value>(&json).unwrap();
            let bin = std::fs::read(bin_path).unwrap();

            let result = test_one_sample(&bin, spec, &json);
            if let Err(err) = result {
                println!("test failed, {err:?}");
                errors += 1;
                continue;
            }
            println!("test ok");
        }
    }
    assert!(
        errors <= 2,
        "there were {errors} errors during sample testing"
    );
}
