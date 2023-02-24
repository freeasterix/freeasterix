use crate::bit_writer::BitWriter;
use crate::error::{Error, InvalidSpec};
use serde_json::{Map, Value};
use spec_parser::spec_xml::{Category, Compound, Explicit, Fixed, Format, Repetitive, Variable};

struct PresentItem<'a> {
    frn: usize,
    #[allow(dead_code)]
    key: &'a String,
    value: &'a Value,
    index: usize,
}

fn calculate_fspec<'a>(
    spec: &Category,
    json: &'a Map<String, Value>,
) -> Result<Vec<PresentItem<'a>>, Error> {
    if spec.uaps.len() != 1 {
        return Err(Error::MultipleUaps);
    }

    let uap = &spec.uaps[0];
    let mut present_items = Vec::with_capacity(json.len());

    for (key, value) in json.iter() {
        if key == "CAT" {
            continue;
        }
        let uap_item = uap
            .items
            .iter()
            .find(|uap| &uap.name == key)
            .ok_or_else(|| Error::InvalidCategoryField {
                category: spec.id,
                field: key.clone(),
            })?;
        let frn = uap_item
            .frn
            .parse::<usize>()
            .map_err(|_| InvalidSpec::FrnNotANumber { field: key.clone() })?
            .checked_sub(1)
            .ok_or_else(|| InvalidSpec::FrnIsZero { field: key.clone() })?;
        let index = spec
            .data_items
            .iter()
            .position(|item| &item.id == key)
            .ok_or_else(|| InvalidSpec::MissingDataItem { field: key.clone() })?;
        present_items.push(PresentItem {
            frn,
            key,
            value,
            index,
        });
    }

    present_items.sort_by_key(|item| item.frn);

    Ok(present_items)
}

fn write_fspec(writer: &mut Vec<u8>, items: &[PresentItem]) {
    let mut buf = 0_u8;
    let mut next_start = 7;
    for &PresentItem { frn, .. } in items.iter() {
        while frn >= next_start {
            buf |= 1;
            writer.push(buf);
            buf = 0;
            next_start += 7;
        }
        assert!(next_start >= frn && next_start - frn <= 7);
        buf |= 1 << (next_start - frn);
    }

    writer.push(buf);
}

fn write_fixed(
    writer: &mut Vec<u8>,
    fixed: &Fixed,
    field: &Value,
    fx: Option<bool>,
) -> Result<(), Error> {
    let field = field.as_object().ok_or_else(|| Error::ExpectedMap)?;
    let mut fx_used = false;
    let mut bit_writer = BitWriter::new(writer, fixed.length);
    let default = Value::from(0_i64);

    for bits in &fixed.bits {
        // TODO(igor): use encode
        let _encode = bits.encode.unwrap_or_default();
        let name = &bits.short_name;
        let value = if bits.fx == Some(1) {
            if fx_used {
                return Err(InvalidSpec::FxUsedTwice.into());
            }
            fx_used = true;
            fx.map(|b| b as u64)
                .ok_or_else(|| InvalidSpec::FxOutsideVariable)?
        } else if name == "spare" || name == "sb" {
            0
        } else {
            let value = field.get(name).unwrap_or(&default);
            if let Some(unit) = &bits.unit {
                let scale = unit.scale.unwrap_or(1.0);
                value.as_f64().map(|value| (value / scale).round() as i64)
            } else {
                value.as_i64()
            }
            .map(|x| x as u64)
            .ok_or_else(|| Error::ExpectedNumber {
                field: name.clone(),
            })?
        };

        let (from, to) = match (bits.bit, bits.from, bits.to) {
            (Some(bit), None, None) => (bit, bit),
            (None, Some(from), Some(to)) => (from, to),
            (bit, from, to) => {
                return Err(InvalidSpec::BadBitCombination { bit, from, to }.into());
            }
        };
        bit_writer.write_bits((from, to), value)?;
    }

    bit_writer.finish()?;

    if fx.is_some() && !fx_used {
        Err(InvalidSpec::FxNotUsed.into())
    } else {
        Ok(())
    }
}

fn expect_fixed(format: &Format) -> Result<&Fixed, Error> {
    if let Format::Fixed(fixed) = format {
        Ok(fixed)
    } else {
        Err(InvalidSpec::ExpectedFixedInVariable.into())
    }
}

fn write_variable(writer: &mut Vec<u8>, variable: &Variable, field: &Value) -> Result<(), Error> {
    let object = field.as_object().ok_or_else(|| Error::ExpectedMap)?;
    let mut max_subitem = None;
    for (idx, item) in variable.formats.iter().enumerate() {
        let fixed = expect_fixed(item)?;
        for bits in &fixed.bits {
            if object.contains_key(&bits.short_name) {
                max_subitem = Some(idx);
                break;
            }
        }
    }

    let max_subitem = max_subitem.ok_or_else(|| Error::NoSubitems)?;

    for (idx, item) in variable.formats.iter().enumerate() {
        let fixed = expect_fixed(item)?;
        write_fixed(writer, fixed, field, Some(idx < max_subitem))?;
    }

    Ok(())
}

fn expect_variable(format: &Format) -> Result<&Variable, Error> {
    if let Format::Variable(variable) = format {
        Ok(variable)
    } else {
        Err(InvalidSpec::ExpectedVariableInCompound.into())
    }
}

fn write_compound(writer: &mut Vec<u8>, compound: &Compound, field: &Value) -> Result<(), Error> {
    let field = field.as_object().ok_or_else(|| Error::ExpectedMap)?;
    let mut present_items = Vec::new();
    let head = expect_variable(&compound.formats[0])?;

    for (byte_no, item) in head.formats.iter().enumerate() {
        let fixed = expect_fixed(item)?;
        for bits in fixed.bits.iter() {
            let key = &bits.short_name;
            if bits.fx == Some(1) || key == "spare" || key == "sb" {
                continue;
            }

            if let Some(value) = field.get(key) {
                let bit = bits
                    .bit
                    .ok_or_else(|| InvalidSpec::InvalidCompoundSubitem)?;
                let frn = byte_no * 7 + (8 - bit as usize);
                let index = bits
                    .presence
                    .ok_or_else(|| InvalidSpec::InvalidCompoundSubitem)?;
                present_items.push(PresentItem {
                    frn,
                    key,
                    value,
                    index,
                });
            }
        }
    }

    write_fspec(writer, &present_items);

    for &PresentItem { value, index, .. } in &present_items {
        let format = compound
            .formats
            .get(index)
            .ok_or_else(|| InvalidSpec::CompoundSubitemOob)?;
        write_field(writer, format, value)?;
    }

    Ok(())
}

fn write_explicit(writer: &mut Vec<u8>, explicit: &Explicit, field: &Value) -> Result<(), Error> {
    let start = writer.len();
    writer.push(0);

    for format in &explicit.formats {
        write_field(writer, format, field)?;
    }

    let written = writer.len() - start;
    let written_bytes: u8 = written.try_into().map_err(|_| Error::TooManyBytesWritten {
        written,
        limit: 1 << 8,
    })?;
    writer[start] = written_bytes;

    Ok(())
}

fn write_repetitive(
    writer: &mut Vec<u8>,
    repetitive: &Repetitive,
    field: &Value,
) -> Result<(), Error> {
    let items = field
        .as_array()
        .ok_or_else(|| Error::RepetitiveExpectsArray)?;
    // TODO(igor): REP length is NOT ALWAYS 1 octet in ASTERIX protocol!
    // However, in all checked use-cases it was always 1.
    let len: u8 = items
        .len()
        .try_into()
        .map_err(|_| Error::TooManyRepetitiveItems {
            count: items.len(),
            limit: 1 << 8,
        })?;
    writer.push(len);
    for item in items {
        for format in &repetitive.formats {
            write_field(writer, format, item)?;
        }
    }
    Ok(())
}

fn write_field(writer: &mut Vec<u8>, format: &Format, field: &Value) -> Result<(), Error> {
    match &format {
        Format::Fixed(fixed) => write_fixed(writer, fixed, field, None),
        Format::Variable(variable) => write_variable(writer, variable, field),
        Format::Compound(compound) => write_compound(writer, compound, field),
        Format::Explicit(explicit) => write_explicit(writer, explicit, field),
        Format::Repetitive(rep) => write_repetitive(writer, rep, field),
        Format::BDS => Err(Error::BdsNotImplemented),
    }
}

fn write_record(
    writer: &mut Vec<u8>,
    spec: &Category,
    json: &Map<String, Value>,
) -> Result<(), Error> {
    let present_items = calculate_fspec(spec, json)?;

    write_fspec(writer, &present_items);

    for PresentItem { value, index, .. } in present_items {
        let data_item = &spec.data_items[index];
        write_field(writer, &data_item.format.format, value)?;
    }

    Ok(())
}

pub fn write_asterix(
    writer: &mut Vec<u8>,
    spec: &Category,
    json: &Map<String, Value>,
) -> Result<(), Error> {
    let start = writer.len();

    let spec_category = json.get("CAT").and_then(|v| v.as_u64());
    if Some(spec.id as u64) != spec_category {
        return Err(Error::MismatchedCategory {
            category: spec.id,
            got: spec_category,
        });
    }

    writer.extend_from_slice(&[spec.id, 0, 0]);

    write_record(writer, spec, json)?;

    let written = writer.len() - start;
    let written_bytes: u16 = written.try_into().map_err(|_| Error::TooManyBytesWritten {
        written,
        limit: 1 << 16,
    })?;

    let len_chunk = written_bytes.to_be_bytes();
    writer[start + 1..start + 3].copy_from_slice(&len_chunk[..]);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data() -> Value {
        serde_json::json! {
           {
             "CAT":62,
             "010": {"SAC": 176,"SIC": 177},
             "210": {"Ax": -30.0,"Ay": -25.75}
           }
        }
    }

    #[test]
    fn test_write_asterix() {
        let data = make_data();
        let data = data.as_object().expect("must be an object");
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("Cannot fetch directory of the current crate");
        let xml_root = std::path::Path::new(&crate_dir).join("../../specs-xml");
        let spec_src = std::fs::read_to_string(xml_root.join("asterix_cat062_1_18.xml"))
            .expect("could not read spec");
        let spec = Category::parse(&spec_src).expect("could not parse spec");

        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        let iters = 200_000;
        for _ in 0..iters {
            buffer.clear();
            write_asterix(&mut buffer, &spec, &data).expect("could not write asterix");
        }
        let elapsed = start.elapsed();
        println!(
            "written in: {:0.1}",
            elapsed.as_secs_f64() / iters as f64 * 1e9
        );
        println!(
            "per second: {:0.1}K",
            iters as f64 / elapsed.as_secs_f64() / 1e3
        );
        println!("buf = {:x?}", buffer);
    }
}
