#![allow(dead_code, unused_imports)]
use crate::bit_reader::BitReader;
use crate::error::{Error, InvalidSpec};
use serde_json::{Map, Value};
use spec_parser::spec_xml::{
    Category, Compound, Encode, Explicit, Fixed, Format, Repetitive, Variable,
};

fn plonk(reader: &mut &[u8]) -> Result<u8, Error> {
    if let Some((&byte, tail)) = reader.split_first() {
        *reader = tail;
        Ok(byte)
    } else {
        Err(Error::ReadingOob)
    }
}

fn plonk_u16(reader: &mut &[u8]) -> Result<u16, Error> {
    let hi = plonk(reader)?;
    let lo = plonk(reader)?;
    Ok(((hi as u16) << 8) | (lo as u16))
}

#[derive(Debug)]
struct PresentItem<'a> {
    frn: usize,
    key: &'a String,
    index: usize,
}

fn switch_uap(data: &[u8], spec: &Category) -> Result<usize, Error> {
    let rv = match spec.id {
        1 => {
            let head = data.first().ok_or(Error::ReadingOob)?;
            if (head >> 7) == 0 {
                1
            } else {
                0
            }
        }
        129 => {
            let head = data.first().ok_or(Error::ReadingOob)?;
            if (head >> 7) == 1 {
                1
            } else {
                0
            }
        }
        _ => 0,
    };
    Ok(rv)
}

fn read_fspec<'a, 'b: 'a>(reader: &'a mut &'b [u8]) -> Result<&'b [u8], Error> {
    let mut fspec_len = 0;
    loop {
        let byte = reader.get(fspec_len).ok_or(Error::ReadingOob)?;
        fspec_len += 1;
        if byte & 1 == 0 {
            break;
        }
    }

    let fspec = &reader[..fspec_len];
    *reader = &reader[fspec_len..];
    Ok(fspec)
}

fn read_present_items<'a>(
    reader: &mut &[u8],
    spec: &'a Category,
) -> Result<Vec<PresentItem<'a>>, Error> {
    let fspec = read_fspec(reader)?;
    let mut fspec = fspec.to_vec();
    let idx = switch_uap(reader, spec)?;
    let uap = spec.uaps.get(idx).ok_or(InvalidSpec::UapIndexOob)?;

    let mut present_items = Vec::new();
    for uap_item in &uap.items {
        if uap_item.frn == "FX" || uap_item.name == "-" {
            continue;
        }
        let key = &uap_item.name;
        let frn = uap_item
            .frn
            .parse::<usize>()
            .map_err(|_| InvalidSpec::FrnNotANumber { field: key.clone() })?
            .checked_sub(1)
            .ok_or_else(|| InvalidSpec::FrnIsZero { field: key.clone() })?;

        let byte = frn / 7;
        let bit = 7 - (frn % 7);
        let mask = 1 << bit;
        if byte >= fspec.len() || fspec[byte] & mask == 0 {
            continue;
        }
        fspec[byte] &= !mask;

        let index = spec
            .data_items
            .iter()
            .position(|item| &item.id == key)
            .ok_or_else(|| InvalidSpec::MissingDataItem { field: key.clone() })?;
        present_items.push(PresentItem { frn, key, index });
    }

    let not_fx = !1;
    for (index, byte) in fspec.into_iter().enumerate() {
        if byte & not_fx != 0 {
            return Err(Error::UnknownFspecField { byte, index });
        }
    }

    Ok(present_items)
}

fn read_fixed<'a, 'b: 'a>(
    reader: &'a mut &'b [u8],
    fixed: &Fixed,
) -> Result<(Map<String, Value>, Option<bool>), Error> {
    let mut fx = None;
    let mut bit_reader = BitReader::new(reader, fixed.length);
    let mut rv = Map::new();

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
            if rv.get(&condition.key) != Some(&cond_val) {
                continue;
            }
        }

        let mut value = bit_reader.read_bits(from, to)?;

        if bits.fx == Some(1) {
            if fx.is_some() {
                return Err(InvalidSpec::FxUsedTwice.into());
            }
            fx = Some(value != 0);
            continue;
        } else if name == "spare" || name == "sb" {
            continue;
        }

        match encode {
            Encode::MsbSign => {
                let mask = 1 << (length - 1);
                if value & mask != 0 {
                    let signed = (value & !mask) as i64;
                    value = (-signed) as u64;
                }
            }
            Encode::Signed => {
                let padding = 64 - length;
                // This is not a noop, this is a sign extension
                let signed = ((value as i64) << padding) >> padding;
                value = signed as u64;
            }
            _ => {}
        }

        let value: Value = if let Some(unit) = &bits.unit {
            let scale = unit.scale.unwrap_or(1.0);
            ((value as i64 as f64) * scale).into()
        } else if matches!(encode, Encode::MsbSign | Encode::Signed) {
            (value as i64).into()
        } else {
            value.into()
        };
        rv.insert(name.to_string(), value);
    }

    bit_reader.finish()?;

    Ok((rv, fx))
}

fn expect_fixed(format: &Format) -> Result<&Fixed, Error> {
    if let Format::Fixed(fixed) = format {
        Ok(fixed)
    } else {
        Err(InvalidSpec::ExpectedFixedInVariable.into())
    }
}

fn read_variable<'a, 'b: 'a>(
    reader: &'a mut &'b [u8],
    variable: &Variable,
) -> Result<Value, Error> {
    if variable.formats.len() == 1 {
        let mut rv = Vec::new();
        let fixed = expect_fixed(&variable.formats[0])?;
        loop {
            let (value, fx) = read_fixed(reader, fixed)?;
            rv.push(value);
            match fx {
                Some(true) => continue,
                Some(false) => break,
                None => return Err(InvalidSpec::FxNotUsed.into()),
            }
        }

        return Ok(rv.into());
    }

    let mut rv = Map::new();
    for item in &variable.formats {
        let fixed = expect_fixed(item)?;
        let (value, fx) = read_fixed(reader, fixed)?;
        rv.extend(value);
        match fx {
            Some(true) => continue,
            Some(false) => break,
            None => return Err(InvalidSpec::FxNotUsed.into()),
        }
    }

    Ok(rv.into())
}

fn expect_variable(format: &Format) -> Result<&Variable, Error> {
    if let Format::Variable(variable) = format {
        Ok(variable)
    } else {
        Err(InvalidSpec::ExpectedVariableInCompound.into())
    }
}

fn read_compound<'a, 'b: 'a>(
    reader: &'a mut &'b [u8],
    compound: &Compound,
) -> Result<Value, Error> {
    let fspec = read_fspec(reader)?;
    let mut present_items = Vec::new();
    let head = compound
        .formats
        .first()
        .ok_or(InvalidSpec::ExpectedVariableInCompound)?;
    let head = expect_variable(head)?;

    for (byte_no, item) in head.formats.iter().enumerate() {
        let fixed = expect_fixed(item)?;
        for bits in fixed.bits.iter() {
            let key = &bits.short_name;
            if bits.fx == Some(1) || key == "spare" || key == "sb" {
                continue;
            }
            let bit = bits.bit.ok_or(InvalidSpec::InvalidCompoundSubitem)?;
            let frn = byte_no * 7 + (8 - bit as usize);
            let mask = 1 << (8 - bit);
            if byte_no >= fspec.len() || fspec[byte_no] & mask == 0 {
                continue;
            }
            let index = bits.presence.ok_or(InvalidSpec::InvalidCompoundSubitem)?;
            present_items.push(PresentItem { frn, key, index });
        }
    }

    let mut rv = Map::new();
    for &PresentItem { index, key, .. } in &present_items {
        let format = compound
            .formats
            .get(index)
            .ok_or(InvalidSpec::CompoundSubitemOob)?;
        let value = read_field(reader, format)?;
        rv.insert(key.to_string(), value);
    }

    Ok(rv.into())
}

fn read_explicit<'a, 'b: 'a>(
    reader: &'a mut &'b [u8],
    explicit: &Explicit,
) -> Result<Value, Error> {
    let length = plonk(reader)? as usize;
    let mut local_reader = reader.get(..length).ok_or(Error::ReadingOob)?;
    *reader = reader.get(length..).ok_or(Error::ReadingOob)?;

    let mut rv = Map::new();
    for format in &explicit.formats {
        let value = read_field(&mut local_reader, format)?;
        if let Value::Object(object) = value {
            rv.extend(object);
        } else {
            return Err(Error::ExpectedMap);
        }
    }

    if !local_reader.is_empty() {
        return Err(Error::ExplicitHasDataLeft);
    }

    Ok(rv.into())
}

fn read_repetitive<'a, 'b: 'a>(
    reader: &'a mut &'b [u8],
    repetitive: &Repetitive,
) -> Result<Value, Error> {
    let count = plonk(reader)? as usize;
    let mut result = Vec::new();
    for _ii in 0..count {
        let mut value = Map::new();
        for format in &repetitive.formats {
            let item = read_field(reader, format)?;
            if let Value::Object(object) = item {
                value.extend(object);
            } else {
                return Err(Error::ExpectedMap);
            }
        }
        result.push(value);
    }
    Ok(result.into())
}

fn read_field<'a, 'b: 'a>(reader: &'a mut &'b [u8], format: &Format) -> Result<Value, Error> {
    match &format {
        Format::Fixed(fixed) => {
            let (result, fx) = read_fixed(reader, fixed)?;
            if fx.is_some() {
                return Err(InvalidSpec::FxOutsideVariable.into());
            }
            Ok(result.into())
        }
        Format::Variable(variable) => read_variable(reader, variable),
        Format::Compound(compound) => read_compound(reader, compound),
        Format::Explicit(explicit) => read_explicit(reader, explicit),
        Format::Repetitive(rep) => read_repetitive(reader, rep),
        Format::BDS => Err(Error::BdsNotImplemented),
    }
}

fn read_record<'a>(reader: &'a mut &'a [u8], spec: &Category) -> Result<Map<String, Value>, Error> {
    let present_items = read_present_items(reader, spec)?;

    let mut rv = Map::new();
    for PresentItem { index, key, .. } in present_items {
        let data_item = &spec.data_items[index];
        let value = read_field(reader, &data_item.format.format)?;
        rv.insert(key.to_string(), value);
    }

    Ok(rv)
}

pub fn read_asterix(reader: &mut &[u8], spec: &Category) -> Result<Map<String, Value>, Error> {
    let orig_reader = *reader;
    let category = plonk(reader)?;
    let length = plonk_u16(reader)?;

    if category != spec.id {
        return Err(Error::MismatchedCategory {
            category: spec.id,
            got: Some(category as u64),
        });
    }

    let mut local_reader = orig_reader
        .get(3..length as usize)
        .ok_or(Error::ReadingOob)?;
    *reader = orig_reader
        .get(length as usize..)
        .ok_or(Error::ReadingOob)?;

    let mut rv = read_record(&mut local_reader, spec)?;
    rv.insert("CAT".to_string(), category.into());

    Ok(rv)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_asterix() -> Result<(), Box<dyn std::error::Error>> {
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR")?;
        let xml_root = std::path::Path::new(&crate_dir).join("../../specs-xml");
        let spec_src = std::fs::read_to_string(xml_root.join("asterix_cat062_1_18.xml"))?;
        let spec = Category::parse(&spec_src)?;

        let mut buf: &[u8] = &[62, 0, 9, 129, 128, 176, 177, 136, 153];
        let result = read_asterix(&mut buf, &spec)?;
        let expect = serde_json::json! {
           {
             "CAT":62,
             "010": {"SAC": 176,"SIC": 177},
             "210": {"Ax": -30.0,"Ay": -25.75}
           }
        };
        let expect = expect.as_object().ok_or("must be object")?;
        assert_eq!(&result, expect);
        Ok(())
    }
}
