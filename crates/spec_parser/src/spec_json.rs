//! The purpose of this module is to provide ASTERIX spec deserialization
//! from JSON specs provided by [zoranbosnjak](https://zoranbosnjak.github.io/asterix-specs/specs.html).
//! JSON root level item for basic category is [`BasicCategory`](BasicCategory),
//! and [`ExpansionCategory`](ExpansionCategory) for the expansion.

use crate::unit::Unit;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasicCategory {
    pub catalogue: Vec<RecordDefinition>,
    pub date: Date,
    pub edition: Edition,
    pub number: u32,
    pub preamble: String,
    pub title: String,
    pub uap: Uap,
    #[serde(rename="type")]
    pub typ: BasicType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BasicType {
    Basic
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpansionCategory {
    pub date: Date,
    pub edition: Edition,
    pub number: u32,
    pub title: String,
    pub variation: FormatVariation,
    #[serde(rename="type")]
    pub typ: ExpansionType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExpansionType {
    Expansion
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordDefinition {
    pub definition: Option<String>,
    pub description: Option<String>,
    pub name: String,
    pub remark: Option<String>,
    pub spare: bool,
    pub title: String,
    pub variation: FormatVariation,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum FormatVariation {
    Element {
        size: usize,
        rule: FormatRule,
    },
    Group {
        items: Vec<DefOrSpare>,
    },
    Compound {
        fspec: Option<u64>,
        items: Vec<Option<DefOrSpare>>,
    },
    Extended {
        extents: usize,
        first: usize,
        items: Vec<DefOrSpare>,
    },
    Repetitive {
        rep: usize,
        variation: Box<FormatVariation>,
    },
    Explicit,
}

#[derive(Debug, Serialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum DefOrSpare {
    Spare { length: usize, spare: bool },
    Definition(RecordDefinition),
}

// This is needed to use `spare` as a tag for deserialization
// https://github.com/serde-rs/serde/issues/880
impl<'de> Deserialize<'de> for DefOrSpare {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = Map::deserialize(deserializer)?;

        let spare = map
            .get("spare")
            .ok_or_else(|| de::Error::missing_field("spare"))
            .map(Deserialize::deserialize)?
            .map_err(de::Error::custom)?;
        if spare {
            let _ = map.remove("spare");
            let length = map
                .remove("length")
                .ok_or_else(|| de::Error::missing_field("length"))
                .map(Deserialize::deserialize)?
                .map_err(de::Error::custom)?;
            Ok(DefOrSpare::Spare { length, spare })
        } else {
            let rest = Value::Object(map);
            RecordDefinition::deserialize(rest)
                .map(DefOrSpare::Definition)
                .map_err(de::Error::custom)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum FormatRule {
    ContextFree {
        content: RuleContent,
    },
    Dependent {
        name: Vec<String>,
        rules: Vec<(u64, RuleContent)>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum RuleContent {
    Raw,
    Bds {
        variation: BdsVariation,
    },
    Quantity {
        constraints: Vec<RuleConstraint>,
        #[serde(rename = "fractionalBits")]
        fractional_bits: u64,
        scaling: QuantityValue,
        signed: bool,
        unit: Unit,
    },
    Integer {
        constraints: Vec<RuleConstraint>,
        signed: bool,
    },
    Table {
        values: Vec<(u64, String)>,
    },
    String {
        variation: StringVariation,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleConstraint {
    #[serde(rename = "type")]
    pub typ: ConstraintType,
    pub value: QuantityValue,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ConstraintType {
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = "<=")]
    LessThanOrEqual,
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum QuantityValue {
    Integer { value: i64 },
    Real { value: f64 },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StringVariation {
    StringAscii,
    StringICAO,
    StringOctal,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum BdsVariation {
    BdsAt { address: Option<String> },
    BdsWithAddress {},
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Uap {
    Uap { items: Vec<Option<String>> },
    Uaps { variations: Vec<UapVariation> },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UapVariation {
    items: Vec<Option<String>>,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Date {
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edition {
    pub major: u32,
    pub minor: u32,
}
