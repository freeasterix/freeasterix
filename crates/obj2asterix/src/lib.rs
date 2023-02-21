//#![allow(dead_code, unused_variables, unused_imports)]

use std::io::Write;
use spec_parser::spec_xml::{Category, DataItem, Format, Fixed};
use serde_json::{Value, Map};

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
        // NOTE(igor): although this is a linear lookup, this should not cause
        // performance problems, since all UAPs have a length < 64
        let index = lut.uap.get(key).copied()
            .ok_or_else(|| format!("Key {} is not present in UAP for CAT{}", key, spec.id))?;
        let uap_item = &uap.items[index];
        let frn = uap_item.frn.parse::<usize>()
            .map_err(|_| "Could not parse FRN, bad spec?".to_string())?
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
    _writer: &mut W,
    _fixed: &Fixed,
    _field: &Map<String, Value>,
) -> Result<(), Error> {
    Ok(())
}

fn write_field<W: Write>(
    writer: &mut W,
    data_item: &DataItem,
    field: &Map<String, Value>,
) -> Result<(), Error> {
    match &data_item.format.format {
        Format::Fixed(fixed) => write_fixed(writer, fixed, field)?,
        Format::Variable(_) => {}
        Format::Explicit(_) => {}
        Format::Repetitive(_) => {}
        Format::Compound(_) => {}
        Format::BDS => {}
    }
    Ok(())
}

pub fn write_asterix<W: Write>(
    writer: &mut W,
    lut: &SpecLut,
    spec: &Category,
    json: &Map<String, Value>,
) -> Result<(), Error> {
    writer.write_all(&[spec.id])
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

#[cfg(test)]
fn make_data() -> Value {
    serde_json::from_str(r#"
{
  "CAT": 62,
  "100": {
    "X": 0,
    "Y": 0
  },
  "105": {
    "Lat": 0,
    "Lon": 0
  },
  "110": {
    "SUM": {
      "M5": 0,
      "ID": 0,
      "DA": 0,
      "M1": 0,
      "M2": 0,
      "M3": 0,
      "MC": 0,
      "X": 0
    },
    "PMN": {
      "PIN": 0,
      "NAT": 0,
      "MIS": 0
    },
    "POS": {
      "Lat": 0,
      "Lon": 0
    },
    "GA": {
      "RES": 0,
      "GA": 0
    },
    "EM1": {
      "EM1": 0
    },
    "TOS": {
      "TOS": 0
    },
    "XP": {
      "X5": 0,
      "XC": 0,
      "X3": 0,
      "X2": 0,
      "X1": 0
    }
  },
  "120": {
    "Mode2": 0
  },
  "130": {
    "Alt": 0
  },
  "135": {
    "QNH": 0,
    "CTBA": 0
  },
  "136": {
    "MFL": 0
  },
  "185": {
    "Vx": 0,
    "Vy": 0
  },
  "200": {
    "TRANSA": 0,
    "LONGA": 0,
    "VERTA": 0,
    "ADF": 0
  },
  "210": {
    "Ax": 0,
    "Ay": 0
  },
  "220": {
    "RoC": 0
  },
  "245": {
    "STI": 0,
    "TId": 0
  },
  "270": {
    "Len": 0,
    "ORI": 0,
    "WIDTH": 0
  },
  "290": {
    "TRK": {
      "TRK": 0
    },
    "PSR": {
      "PSR": 0
    },
    "SSR": {
      "SSR": 0
    },
    "MDS": {
      "MDS": 0
    },
    "ADS": {
      "ADS": 0
    },
    "ES": {
      "ES": 0
    },
    "VDL": {
      "VDL": 0
    },
    "UAT": {
      "UAT": 0
    },
    "LOP": {
      "LOP": 0
    },
    "MLT": {
      "MLT": 0
    }
  },
  "295": {
    "MFL": {
      "MFL": 0
    },
    "MD1": {
      "MD1": 0
    },
    "MD2": {
      "MD2": 0
    },
    "MDA": {
      "MDA": 0
    },
    "MD4": {
      "MD4": 0
    },
    "MD5": {
      "MD5": 0
    },
    "MHG": {
      "MHG": 0
    },
    "IAS": {
      "IAS": 0
    },
    "TAS": {
      "TAS": 0
    },
    "SAL": {
      "SAL": 0
    },
    "FSS": {
      "FSS": 0
    },
    "TID": {
      "TID": 0
    },
    "COM": {
      "COM": 0
    },
    "SAB": {
      "SAB": 0
    },
    "ACS": {
      "ACS": 0
    },
    "BVR": {
      "BVR": 0
    },
    "GVR": {
      "GVR": 0
    },
    "RAN": {
      "RAN": 0
    },
    "TAR": {
      "TAR": 0
    },
    "TAN": {
      "TAN": 0
    },
    "GSP": {
      "GSP": 0
    },
    "VUN": {
      "VUN": 0
    },
    "MET": {
      "MET": 0
    },
    "EMC": {
      "EMC": 0
    },
    "POS": {
      "POS": 0
    },
    "GAL": {
      "GAL": 0
    },
    "PUN": {
      "PUN": 0
    },
    "MB": {
      "MB": 0
    },
    "IAR": {
      "IAR": 0
    },
    "MAC": {
      "MAC": 0
    },
    "BPS": {
      "BPS": 0
    }
  },
  "300": {
    "VFI": 0
  },
  "340": {
    "SID": {
      "SAC": 0,
      "SIC": 0
    },
    "POS": {
      "RHO": 0,
      "THETA": 0
    },
    "HEI": {
      "HEIGHT": 0
    },
    "MDC": {
      "CV": 0,
      "CG": 0,
      "ModeC": 0
    },
    "MDA": {
      "V": 0,
      "G": 0,
      "L": 0,
      "Mode3A": 0
    },
    "TYP": {
      "TYP": 0,
      "SIM": 0,
      "RAB": 0,
      "TST": 0
    }
  },
  "380": {
    "ADR": {
      "ADR": 0
    },
    "ID": {
      "ACID": 0
    },
    "MHG": {
      "MAH": 0
    },
    "IAS": {
      "IM": 0,
      "AS": 0
    },
    "TAS": {
      "TAS": 0
    },
    "SAL": {
      "SAS": 0,
      "SRC": 0,
      "Alt": 0
    },
    "FSS": {
      "MV": 0,
      "AH": 0,
      "AM": 0,
      "Alt": 0
    },
    "TIS": {
      "NAV": 0,
      "NVB": 0
    },
    "TID": [
      {
        "TCA": 0,
        "NC": 0,
        "TCPnum": 0,
        "Alt": 0,
        "Lat": 0,
        "Lon": 0,
        "PT": 0,
        "TD": 0,
        "TRA": 0,
        "TOA": 0,
        "TOV": 0,
        "TTR": 0
      }
    ],
    "COM": {
      "COM": 0,
      "STAT": 0,
      "SSC": 0,
      "ARC": 0,
      "AIC": 0,
      "B1A": 0,
      "B1B": 0
    },
    "SAB": {
      "AC": 0,
      "MN": 0,
      "DC": 0,
      "GBS": 0,
      "STAT": 0
    },
    "ACS": {
      "BDS30": 0,
      "ARA41": 0,
      "ARA42": 0,
      "ARA43": 0,
      "ARA44": 0,
      "ARA45": 0,
      "ARA46": 0,
      "ARA47": 0,
      "ACAS3": 0,
      "RAC55": 0,
      "RAC56": 0,
      "RAC57": 0,
      "RAC58": 0,
      "RAT": 0,
      "MTE": 0,
      "TTI": 0,
      "TID": 0
    },
    "BVR": {
      "BVR": 0
    },
    "GVR": {
      "GVR": 0
    },
    "RAN": {
      "RAN": 0
    },
    "TAR": {
      "TI": 0,
      "RoT": 0
    },
    "TAN": {
      "TAN": 0
    },
    "GSP": {
      "GS": 0
    },
    "VUN": {
      "VUN": 0
    },
    "MET": {
      "WS": 0,
      "WD": 0,
      "TMP": 0,
      "TRB": 0,
      "WS_D": 0,
      "WD_D": 0,
      "TMP_D": 0,
      "TRB_D": 0
    },
    "EMC": {
      "ECAT": 0
    },
    "POS": {
      "Lat": 0,
      "Lon": 0
    },
    "GAL": {
      "GAL": 0
    },
    "PUN": {
      "PUN": 0
    },
    "MB": [
      {
        "VAL": 0,
        "BDS": 0
      }
    ],
    "IAR": {
      "IAS": 0
    },
    "MAC": {
      "MNO": 0
    },
    "BPS": {
      "BPS": 0
    }
  },
  "390": {
    "TAG": {
      "SAC": 0,
      "SIC": 0
    },
    "CSN": {
      "CS": 0
    },
    "IFI": {
      "TYP": 0,
      "NBR": 0
    },
    "FCT": {
      "GATOAT": 0,
      "FR1FR2": 0,
      "RVSM": 0,
      "HPR": 0
    },
    "TAC": {
      "TYPE": 0
    },
    "WTC": {
      "WTC": 0
    },
    "DEP": {
      "DEP": 0
    },
    "DST": {
      "DES": 0
    },
    "RDS": {
      "NU1": 0,
      "NU2": 0,
      "LTR": 0
    },
    "CFL": {
      "CFL": 0
    },
    "CTL": {
      "Centre": 0,
      "Position": 0
    },
    "TOD": [
      {
        "TYP": 0,
        "DAY": 0,
        "HOR": 0,
        "MIN": 0,
        "AVS": 0,
        "SEC": 0
      }
    ],
    "AST": {
      "AS": 0
    },
    "STS": {
      "EMP": 0,
      "AVL": 0
    },
    "STD": {
      "SID": 0
    },
    "STA": {
      "STAR": 0
    },
    "PEM": {
      "VAL": 0,
      "PEM": 0
    },
    "PEC": {
      "PAC": 0
    }
  },
  "500": {
    "APC": {
      "APC_X": 0,
      "APC_Y": 0
    },
    "COV": {
      "COV_XY": 0
    },
    "APW": {
      "APW_Lat": 0,
      "APW_Long": 0
    },
    "AGA": {
      "AGA_Acc": 0
    },
    "ABA": {
      "ABA_Acc": 0
    },
    "ATV": {
      "ATV_X": 0,
      "ATV_Y": 0
    },
    "AA": {
      "AA_X": 0,
      "AA_Y": 0
    },
    "ARC": {
      "ARC_Acc": 0
    }
  },
  "510": {
    "SDPSUID": 0,
    "STN": 0
  },
  "010": {
    "SAC": 0,
    "SIC": 0
  },
  "015": {
    "SID": 0
  },
  "040": {
    "TrkN": 0
  },
  "060": {
    "V": 0,
    "G": 0,
    "CH": 0,
    "Mode3A": 0
  },
  "070": {
    "ToT": 0
  },
  "080": {
    "MON": 0,
    "SPI": 0,
    "MRH": 0,
    "SRC": 0,
    "CNF": 0,
    "SIM": 0,
    "TSE": 0,
    "TSB": 0,
    "FPC": 0,
    "AFF": 0,
    "STP": 0,
    "KOS": 0,
    "AMA": 0,
    "MD4": 0,
    "ME": 0,
    "MI": 0,
    "MD5": 0,
    "CST": 0,
    "PSR": 0,
    "SSR": 0,
    "MDS": 0,
    "ADS": 0,
    "SUC": 0,
    "AAC": 0,
    "SDS": 0,
    "EMS": 0,
    "PFT": 0,
    "FPLT": 0,
    "DUPT": 0,
    "DUPF": 0,
    "DUPM": 0,
    "SFC": 0,
    "IDD": 0,
    "IEC": 0
  },
  "RE": {
    "CST": [
      {
        "SAC": 0,
        "SIC": 0,
        "TYP": 0,
        "LTN": 0
      }
    ],
    "CSN": [
      {
        "SAC": 0,
        "SIC": 0,
        "TYP": 0
      }
    ],
    "TVS": {
      "Vx": 0,
      "Vy": 0
    },
    "STS": {
      "FDR": 0
    }
  },
  "SP": {
    "TRI": 0
  }
}
"#).unwrap()
}