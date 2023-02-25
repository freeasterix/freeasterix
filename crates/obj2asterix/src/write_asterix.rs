use crate::ais_code::encode_ais;
use crate::bit_writer::BitWriter;
use crate::error::{Error, InvalidSpec};
use serde_json::{Map, Value};
use spec_parser::spec_xml::{
    Category, Compound, Encode, Explicit, Fixed, Format, Repetitive, Variable,
};

struct PresentItem<'a> {
    frn: usize,
    #[allow(dead_code)]
    key: &'a String,
    value: &'a Value,
    index: usize,
}

fn switch_uap(spec: &Category, json: &Map<String, Value>) -> usize {
    fn read_path(json: &Map<String, Value>, path: &[&str]) -> u64 {
        let mut value = json.get(path[0]).unwrap_or(&Value::Null);
        for &chunk in &path[1..] {
            value = value.get(chunk).unwrap_or(&Value::Null);
        }
        value.as_u64().unwrap_or(0)
    }
    match spec.id {
        1 => {
            let typ = read_path(json, &["010", "TYP"]);
            if typ == 0 {
                1
            } else {
                0
            }
        }
        129 => {
            let typ = read_path(json, &["010", "TYP"]);
            if typ == 1 {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn calculate_fspec<'a>(
    spec: &Category,
    json: &'a Map<String, Value>,
) -> Result<Vec<PresentItem<'a>>, Error> {
    let idx = switch_uap(spec, json);
    let uap = &spec.uaps[idx];
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

fn write_ascii(
    writer: &mut BitWriter,
    (from, to): (u32, u32),
    name: &str,
    value: &Value,
) -> Result<(), Error> {
    let length = from - to + 1;
    if length % 8 != 0 {
        return Err(InvalidSpec::BadAsciiLength {
            field: name.to_string(),
        }
        .into());
    }
    let s = if let Value::String(s) = value {
        s
    } else {
        return Err(Error::ExpectedStringForAscii {
            field: name.to_string(),
        });
    };
    if s.len() > length as usize / 8 {
        return Err(Error::AsciiStringTooLong {
            string: s.to_string(),
        });
    }
    let bytes = s.as_bytes();
    let mut ii = 0;
    let mut pos = from;
    while pos > to {
        let chr = bytes.get(ii).copied().unwrap_or(b' ');
        if !(chr.is_ascii_graphic() || chr == b' ') {
            return Err(Error::InvalidAsciiChar {
                chr: chr as char,
                string: s.to_string(),
            });
        }
        writer.write_bits((pos, pos - 7), chr as u64)?;
        ii += 1;
        pos -= 8;
    }
    Ok(())
}

fn write_fixed(
    writer: &mut Vec<u8>,
    fixed: &Fixed,
    field: &Value,
    fx: Option<bool>,
) -> Result<(), Error> {
    let field = field.as_object().ok_or(Error::ExpectedMap)?;
    let mut fx_used = false;
    let mut bit_writer = BitWriter::new(writer, fixed.length);
    let default = Value::from(0_i64);

    for bits in &fixed.bits {
        let encode = bits.encode.unwrap_or_default();
        let name = &bits.short_name;
        let (from, to) = match (bits.bit, bits.from, bits.to) {
            (Some(bit), None, None) => (bit, bit),
            // Min/max Because XML spec doesn't respect this!
            (None, Some(from), Some(to)) => (from.max(to), from.min(to)),
            (bit, from, to) => {
                return Err(InvalidSpec::BadBitCombination { bit, from, to }.into());
            }
        };
        let length = from - to + 1;

        if let Some(condition) = &bits.condition {
            let cond_val: Value = condition.val.into();
            if field.get(&condition.key) != Some(&cond_val) {
                continue;
            }
        }

        let mut value = if bits.fx == Some(1) {
            if fx_used {
                return Err(InvalidSpec::FxUsedTwice.into());
            }
            fx_used = true;
            fx.map(|b| b as u64)
                .ok_or(InvalidSpec::FxOutsideVariable)?
        } else if name == "spare" || name == "sb" {
            0
        } else {
            let mut value = field.get(name).unwrap_or(&default);

            if encode == Encode::Ascii {
                write_ascii(&mut bit_writer, (from, to), name, value)?;
                continue;
            }

            let tmp: Value;
            if let (Value::String(s), Encode::SixBitsChar) = (value, encode) {
                tmp = encode_ais(s)?.into();
                value = &tmp;
            }

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

        if encode == Encode::Octal {
            if length == 5 {
                let a = value / 10;
                let b = value % 10;
                if a >= 8 || b >= 4 {
                    return Err(Error::InvalidOcatlCode { code: value });
                }
                value = (a << 2) | b;
            } else {
                let mut tmp = value;
                let mut pow = 1;
                let mut rv = 0;
                while tmp > 0 {
                    let chr = tmp % 10;
                    if chr >= 8 {
                        return Err(Error::InvalidOcatlCode { code: value });
                    }
                    rv += pow * chr;
                    tmp /= 10;
                    pow <<= 3;
                }
                value = rv;
            }
        }

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
    if variable.formats.len() == 1 {
        let array = field.as_array().ok_or(Error::ExpectedArray)?;
        let last = array.len() - 1;
        let fixed = expect_fixed(&variable.formats[0])?;
        for (idx, item) in array.iter().enumerate() {
            write_fixed(writer, fixed, item, Some(idx < last))?;
        }
        return Ok(());
    }

    let object = field.as_object().ok_or(Error::ExpectedMap)?;
    let mut max_subitem = None;
    for (idx, item) in variable.formats.iter().enumerate() {
        let fixed = expect_fixed(item)?;
        for bits in &fixed.bits {
            let name = &bits.short_name;
            if bits.fx == Some(1) || name == "spare" || name == "sb" {
                continue;
            }

            if object.contains_key(&bits.short_name) {
                max_subitem = Some(idx);
                break;
            }
        }
    }

    let max_subitem = if let Some(max_subitem) = max_subitem {
        max_subitem
    } else {
        return Ok(());
    };
    //let max_subitem = max_subitem.ok_or_else(|| Error::NoSubitems)?;

    for (idx, item) in variable.formats[..max_subitem + 1].iter().enumerate() {
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
    let field = field.as_object().ok_or(Error::ExpectedMap)?;
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
                    .ok_or(InvalidSpec::InvalidCompoundSubitem)?;
                let frn = byte_no * 7 + (8 - bit as usize);
                let index = bits
                    .presence
                    .ok_or(InvalidSpec::InvalidCompoundSubitem)?;
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
            .ok_or(InvalidSpec::CompoundSubitemOob)?;
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
        .ok_or(Error::RepetitiveExpectsArray)?;
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

    if let Some(Value::Array(records)) = json.get("records") {
        for record in records {
            let record = record.as_object().ok_or(Error::ExpectedMap)?;
            write_record(writer, spec, record)?;
        }
    } else {
        write_record(writer, spec, json)?;
    }

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
    fn test_write_asterix() -> Result<(), Box<dyn std::error::Error>> {
        let data = make_data();
        let data = data.as_object().expect("must be an object");
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR")?;
        let xml_root = std::path::Path::new(&crate_dir).join("../../specs-xml");
        let spec_src = std::fs::read_to_string(xml_root.join("asterix_cat062_1_18.xml"))?;
        let spec = Category::parse(&spec_src)?;

        let mut buffer = Vec::new();
        let start = std::time::Instant::now();
        let iters = 200_000;
        for _ in 0..iters {
            buffer.clear();
            write_asterix(&mut buffer, &spec, &data)?;
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
        Ok(())
    }
}
