//#![allow(dead_code, unused_variables, unused_imports)]

use serde_json::{Value, Map};
use std::io::Write;
use spec_parser::spec_xml::{Category, DataItem, Variable, Format, Fixed};

mod bit_writer;
use bit_writer::BitWriter;

type Error = String;

fn write_fspec<'a, W: Write>(
    writer: &mut W,
    lut: &SpecLut,
    spec: &Category,
    json: &'a Map<String, Value>,
) -> Result<Vec<(usize, &'a String, &'a Value)>, String> {
    assert!(spec.uaps.len() == 1, "TODO: multiple UAPs are not supported yet");
    let uap = &spec.uaps[0];
    let mut rv = Vec::with_capacity(json.len());
    for (key, value) in json.iter() {
        if key == "CAT" { continue; }
        let index = lut.uap.get(key).copied()
            .ok_or_else(|| format!("Key {} is not present in UAP for CAT{}", key, spec.id))?;
        let uap_item = &uap.items[index];
        let frn = uap_item.frn.parse::<usize>()
            .map_err(|_| "Could not parse FRN as number, bad spec?".to_string())?
            .checked_sub(1)
            .ok_or_else(|| "Expected frn > 0".to_string())?;
        rv.push((frn, key, value));
    }
    rv.sort_by_key(|pair| pair.0);

    let mut buf = 0_u8;
    let mut next_start = 7;
    for &(frn, _key, _value) in rv.iter() {
        while frn >= next_start {
            buf |= 1;
            writer.write_all(&[buf])
                .map_err(|_| "failed to write".to_string())?;
            buf = 0;
            next_start += 7;
        }
        assert!((0..=7).contains(&(next_start - frn)));
        buf |= 1 << (next_start - frn);
    }

    if buf != 0 {
        writer.write_all(&[buf])
            .map_err(|_| "failed to write".to_string())?;
    }

    Ok(rv)
}

fn write_fixed<W: Write>(
    writer: &mut W,
    fixed: &Fixed,
    field: &Map<String, Value>,
    fx: Option<bool>,
) -> Result<bool, Error> {
    let mut fx_used = false;
    let mut bit_writer = BitWriter::new(writer, fixed.length);
    let default = Value::from(0_i64);

    for bits in &fixed.bits {
        // TODO: use encode
        let _encode = bits.encode.unwrap_or_default();
        let name = &bits.short_name;
        let value = if bits.fx == Some(1) {
            fx_used = true;
            fx.map(|b| b as i64)
                .ok_or_else(|| format!("FX bit used outside Variable"))?
        } else if name == "spare" || name == "sb" {
            0
        } else {
            let value = field.get(name).unwrap_or(&default);
            if let Some(unit) = &bits.unit {
                let scale = unit.scale.unwrap_or(1.0);
                value.as_f64().map(|value| (value * scale) as i64)
            } else {
                value.as_i64()
            }
            .ok_or_else(|| format!("Expected number for field {:?}, got: {:?}", name, value))?
        };

        let (from, to) = match (bits.bit, bits.from, bits.to) {
            (Some(bit), None, None) => (bit, bit),
            (None, Some(from), Some(to)) => (from, to),
            _ => {
                return Err(format!("Bad Spec: invalid combination of `bit`, `from` and `to` for field: {:?}", bits.name));
            }
        };
        bit_writer.write_bits((from, to), value)?;
    }
    Ok(fx_used)
}

fn write_field<W: Write>(
    writer: &mut W,
    data_item: &DataItem,
    field: &Map<String, Value>,
) -> Result<(), Error> {
    match &data_item.format.format {
        Format::Fixed(fixed) => { write_fixed(writer, fixed, field, None)?; },
        Format::Variable(_variable) => {},
        Format::Explicit(_) => {}
        Format::Repetitive(_) => {}
        Format::Compound(_) => {}
        Format::BDS => {}
    }
    Ok(())
}

pub fn write_asterix(
    writer: &mut Vec<u8>,
    lut: &SpecLut,
    spec: &Category,
    json: &Map<String, Value>,
) -> Result<(), Error> {
    let start = writer.len();
    writer.write_all(&[spec.id, 0, 0])
        .map_err(|_| "failed to write".to_string())?;
    let keys = write_fspec(writer, lut, spec, json)?;
    for (_frn, key, value) in keys {
        let field = value.as_object()
            .ok_or_else(|| "Values on root level keys must be maps".to_string())?;
        let index = lut.data_items.get(key).copied()
            .ok_or_else(|| format!("Spec is missing DataItem id={}", key))?;
        let data_item = &spec.data_items[index];
        write_field(writer, data_item, field)?;
    }
    let written_bytes = writer.len() - start;
    assert!(written_bytes < (1 << 16), "too many bytes written");
    let len_chunk = (written_bytes as u16).to_be_bytes(); 
    writer[start + 1..start + 3].copy_from_slice(&len_chunk[..]);
    Ok(())
}

use std::collections::BTreeMap;
pub struct SpecLut {
    uap: BTreeMap<String, usize>,
    data_items: BTreeMap<String, usize>,
}

impl SpecLut {
    pub fn build(spec: &Category) -> Self {
        let mut uap = BTreeMap::new();
        assert!(spec.uaps.len() == 1, "TODO: multiple UAPs are not supported yet");
        for (index, uap_item) in spec.uaps[0].items.iter().enumerate() {
            if &uap_item.frn == "FX" { continue; }
            uap.insert(uap_item.name.clone(), index);
        }
        let mut data_items = BTreeMap::new();
        for (index, data_item) in spec.data_items.iter().enumerate() {
            data_items.insert(data_item.id.clone(), index);
        }
        Self { uap, data_items }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data() -> Value {
        serde_json::from_str(r#"
        {
          "CAT": 62,
          "010": {
            "SAC": 176,
            "SIC": 177
          }
        }
       "#).unwrap()
    }

    #[test]
    fn foobar() {
        let data = make_data();
        let data = data.as_object().expect("root value must be an object");
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("Cannot fetch directory of the current crate");
        let xml_root = std::path::Path::new(&crate_dir).join("../../specs-xml");
        let spec_src = std::fs::read_to_string(xml_root.join("asterix_cat062_1_18.xml"))
            .expect("could not read spec");
        let spec = Category::parse(&spec_src)
            .expect("could not parse spec");

        let lut = SpecLut::build(&spec);

        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        let iters = 200_000;
        for _ in 0..iters {
            buffer.clear();
            write_asterix(&mut buffer, &lut, &spec, &data)
                .expect("could not write asterix");
        }
        println!("written in: {:0.1}", start.elapsed().as_secs_f64() / iters as f64 * 1e9);
        println!("buf = {:?}", buffer);
    }

}
