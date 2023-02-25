//! The purpose of this module is to provide ASTERIX XML -> Rust deserialization
//! of XML specs, produced by [CroatiaControlLtd/asterix](https://github.com/CroatiaControlLtd/asterix).
//! XML root item is [`Category`](Category)

use crate::unit::Unit;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Category {
    pub id: u8,
    pub name: String,
    pub ver: String,
    #[serde(rename = "DataItem")]
    pub data_items: Vec<DataItem>,
    #[serde(rename = "UAP")]
    pub uaps: Vec<UAP>,
}

impl Category {
    pub fn parse(s: &str) -> Result<Self, serde_xml_rs::Error> {
        serde_xml_rs::from_str(s)
    }
}

#[derive(Deserialize, Debug)]
pub struct DataItem {
    pub id: String,
    pub rule: Option<Rule>,
    #[serde(rename = "DataItemName")]
    pub name: String,
    #[serde(rename = "DataItemDefinition")]
    pub definition: String,
    #[serde(rename = "DataItemNote")]
    pub note: Option<String>,
    #[serde(rename = "DataItemFormat")]
    pub format: DataItemFormat,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Rule {
    Mandatory,
    Optional,
    Unknown,
}

#[derive(Deserialize, Debug)]
pub struct DataItemFormat {
    #[serde(rename = "$value")]
    pub format: Format,
}

#[derive(Deserialize, Debug)]
pub struct DataFormat {
    #[serde(rename = "$value")]
    pub formats: Vec<Format>,
}

#[derive(Deserialize, Debug)]
pub enum Format {
    Fixed(Fixed),
    Explicit(DataFormat),
    Repetitive(DataFormat),
    Variable(DataFormat),
    Compound(DataFormat),
    #[allow(clippy::upper_case_acronyms)]
    BDS,
}

#[derive(Deserialize, Debug)]
pub struct Fixed {
    pub length: u32,
    #[serde(rename = "$value")]
    pub bits: Vec<Bits>,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Encode {
    Signed,
    MsbSign,
    #[serde(rename = "6bitschar")]
    Sixbitschar,
    Octal,
    #[default]
    Unsigned,
    Ascii,
    Hex,
}

impl Default for Encode {
    fn default() -> Self {
        Self::Unsigned
    }
}

#[derive(Deserialize, Debug)]
pub struct Bits {
    pub bit: Option<u32>,
    pub from: Option<u32>,
    pub to: Option<u32>,
    pub fx: Option<u64>,
    pub rep: Option<u64>,
    #[serde(default)]
    pub encode: Encode,
    #[serde(rename = "BitsShortName")]
    pub short_name: String,
    #[serde(rename = "BitsName")]
    pub name: Option<String>,
    #[serde(rename = "BitsPresence")]
    pub presence: Option<u64>,
    #[serde(rename = "BitsConst")]
    pub cons: Option<u64>,
    #[serde(rename = "BitsValue")]
    pub values: Option<Vec<BitsValue>>,
    #[serde(rename = "BitsUnit")]
    pub unit: Option<BitsUnit>,
    #[serde(rename = "BitsPresence")]
    pub presence: Option<usize>,
    #[serde(rename = "BitsCondition")]
    pub condition = Option<BitsCondition>,
}

#[derive(Deserialize, Debug)]
pub struct BitsValue {
    pub val: u64,
    #[serde(rename = "$value")]
    pub desc: String,
}

#[derive(Deserialize, Debug)]
pub struct BitsUnit {
    pub scale: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub desc: Option<String>,
    #[serde(rename = "$value")]
    pub unit: Option<Unit>,
}

#[derive(Deserialize, Debug)]
pub struct BitsCondition {
    pub key: String,
    pub val: u8,
}

#[derive(Deserialize, Debug)]
pub struct UAP {
    pub use_if_bit_set: Option<u64>,
    pub use_if_byte_nr: Option<u64>,
    pub is_set_to: Option<u8>,
    #[serde(rename = "UAPItem")]
    pub items: Vec<UAPItem>,
}

#[derive(Deserialize, Debug)]
pub struct UAPItem {
    pub bit: u64,
    pub frn: String,
    pub len: Option<String>,
    #[serde(rename = "$value")]
    pub name: String,
}
