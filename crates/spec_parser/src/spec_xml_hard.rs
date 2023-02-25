//! The purpose of this module is to provide ASTERIX XML -> Rust deserialization
//! of XML specs, produced by [CroatiaControlLtd/asterix](https://github.com/CroatiaControlLtd/asterix).
//! XML root item is [`Category`](Category)

// TODO: this lint originates from hard_xml, maybe fix it?
#![allow(clippy::needless_late_init)]

use crate::unit::Unit;
use hard_xml::{XmlRead, XmlWrite};

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "Category")]
pub struct Category {
    #[xml(attr = "id")]
    pub id: u8,
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "ver")]
    pub ver: String,
    #[xml(child = "DataItem")]
    pub data_items: Vec<DataItem>,
    #[xml(child = "UAP")]
    pub uaps: Vec<UAP>,
}

impl Category {
    pub fn parse(s: &str) -> Result<Self, hard_xml::XmlError> {
        Self::from_str(s)
    }
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "DataItem")]
pub struct DataItem {
    #[xml(attr = "id")]
    pub id: String,
    #[xml(attr = "rule")]
    pub rule: Option<Rule>,
    #[xml(flatten_text = "DataItemName")]
    pub name: String,
    #[xml(flatten_text = "DataItemDefinition")]
    pub definition: String,
    #[xml(child = "DataItemFormat")]
    pub format: DataItemFormat,
    #[xml(flatten_text = "DataItemNote")]
    pub note: Option<String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Rule {
    Mandatory,
    Optional,
    #[default]
    Unknown,
}

impl std::str::FromStr for Rule {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Rule::*;
        let rv = match s {
            "mandatory" => Mandatory,
            "optional" => Optional,
            "unknown" => Unknown,
            _ => return Err("bad DataItem rule attribute"),
        };
        Ok(rv)
    }
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Rule::*;
        let s = match self {
            Mandatory => "mandatory",
            Optional => "optional",
            Unknown => "unknown",
        };
        f.write_str(s)
    }
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "DataItemFormat")]
pub struct DataItemFormat {
    #[xml(attr = "desc")]
    pub desc: Option<String>,
    #[xml(
        child = "Fixed",
        child = "Explicit",
        child = "Repetitive",
        child = "Variable",
        child = "Compound",
        child = "BDS"
    )]
    pub format: Format,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Format {
    #[xml(tag = "Fixed")]
    Fixed(Fixed),
    #[xml(tag = "Variable")]
    Variable(Variable),
    #[xml(tag = "Explicit")]
    Explicit(Explicit),
    #[xml(tag = "Repetitive")]
    Repetitive(Repetitive),
    #[xml(tag = "Compound")]
    Compound(Compound),
    #[xml(tag = "BDS")]
    BDS,
}

// TODO(igorm): is it possible to make hard_xml run in strict mode where
// unexpected tags return an error?  Currently unexpected tags are skipped.
// The purpose of this code is to create an intentionally broad child matching
// in formats, and to check whether stricter definitions of format tags
// Variable/Explicit/Repetitive/Compound do not miss any data.
/*
macro_rules! make_wrap {
    ($tag:expr, $name:ident) => {
        #[derive(XmlRead, Debug)]
        #[xml(tag=$tag)]
        pub struct $name {
            #[xml(
                child = "Fixed",
                child = "Compound",
                child = "Explicit",
                child = "Repetitive",
                child = "Variable",
            )]
            pub formats: Vec<Format>,
        }
    };
}

make_wrap!("Variable", Variable);
make_wrap!("Explicit", Explicit);
make_wrap!("Repetitive", Repetitive);
make_wrap!("Compound", Compound);
*/

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "Variable")]
pub struct Variable {
    #[xml(child = "Fixed")]
    pub formats: Vec<Format>,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "Explicit")]
pub struct Explicit {
    #[xml(
        child = "Fixed",
        child = "Explicit",
        child = "Repetitive",
        child = "Variable",
        child = "Compound"
    )]
    pub formats: Vec<Format>,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "Repetitive")]
pub struct Repetitive {
    #[xml(child = "Fixed", child = "BDS")]
    pub formats: Vec<Format>,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "Compound")]
pub struct Compound {
    #[xml(
        child = "Variable",
        child = "Fixed",
        child = "Repetitive",
        child = "Compound"
    )]
    pub formats: Vec<Format>,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "Fixed")]
pub struct Fixed {
    #[xml(attr = "length")]
    pub length: u32,
    #[xml(child = "Bits")]
    pub bits: Vec<Bits>,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "Bits")]
pub struct Bits {
    #[xml(attr = "bit")]
    pub bit: Option<u32>,
    #[xml(attr = "from")]
    pub from: Option<u32>,
    #[xml(attr = "to")]
    pub to: Option<u32>,
    #[xml(attr = "fx")]
    pub fx: Option<u64>,
    #[xml(attr = "rep")]
    pub rep: Option<u64>,
    #[xml(attr = "encode")]
    pub encode: Option<Encode>,
    #[xml(flatten_text = "BitsShortName")]
    pub short_name: String,
    #[xml(flatten_text = "BitsConst")]
    pub cons: Option<String>,
    #[xml(flatten_text = "BitsName")]
    pub name: Option<String>,
    #[xml(child = "BitsValue")]
    pub values: Vec<BitsValue>,
    #[xml(child = "BitsUnit")]
    pub unit: Option<BitsUnit>,
    #[xml(flatten_text = "BitsPresence")]
    pub presence: Option<usize>,
    #[xml(child = "BitsCondition")]
    pub condition: Option<BitsCondition>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Encode {
    #[default]
    Unsigned,
    Signed,
    MsbSign,
    Octal,
    Hex,
    SixBitsChar,
    Ascii,
}

impl std::str::FromStr for Encode {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Encode::*;
        let rv = match s {
            "unsigned" => Unsigned,
            "signed" => Signed,
            "msbsign" => MsbSign,
            "octal" => Octal,
            "hex" => Hex,
            "6bitschar" => SixBitsChar,
            "ascii" => Ascii,
            _ => return Err("bad Bits encode attr"),
        };
        Ok(rv)
    }
}

impl std::fmt::Display for Encode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Encode::*;
        let s = match self {
            Unsigned => "unsigned",
            Signed => "signed",
            MsbSign => "msbsign",
            Octal => "octal",
            Hex => "hex",
            SixBitsChar => "6bitschar",
            Ascii => "ascii",
        };
        f.write_str(s)
    }
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "BitsValue")]
pub struct BitsValue {
    #[xml(attr = "val")]
    pub val: u64,
    #[xml(text)]
    pub desc: String,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "BitsUnit")]
pub struct BitsUnit {
    #[xml(attr = "min")]
    pub min: Option<f64>,
    #[xml(attr = "max")]
    pub max: Option<f64>,
    #[xml(attr = "scale")]
    pub scale: Option<f64>,
    #[xml(text)]
    pub unit: Unit,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "BitsCondition")]
pub struct BitsCondition {
    #[xml(attr = "key")]
    pub key: String,
    #[xml(attr = "val")]
    pub val: u8,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "UAP")]
#[allow(clippy::upper_case_acronyms)]
pub struct UAP {
    #[xml(attr = "use_if_bit_set")]
    pub use_if_bit_set: Option<u64>,
    #[xml(attr = "use_if_byte_nr")]
    pub use_if_byte_nr: Option<u64>,
    #[xml(attr = "is_set_to")]
    pub is_set_to: Option<u8>,
    #[xml(child = "UAPItem")]
    pub items: Vec<UAPItem>,
}

#[derive(XmlRead, XmlWrite, Debug)]
#[xml(tag = "UAPItem")]
pub struct UAPItem {
    #[xml(attr = "bit")]
    pub bit: u64,
    #[xml(attr = "frn")]
    pub frn: String,
    #[xml(attr = "len")]
    pub len: Option<String>,
    #[xml(text)]
    pub name: String,
}
