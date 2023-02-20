pub mod spec_json;

#[cfg(feature="xml_serde")]
#[path = "spec_xml_serde.rs"]
pub mod spec_xml;

#[cfg(not(feature="xml_serde"))]
#[path = "spec_xml_hard.rs"]
pub mod spec_xml;

pub mod unit;

#[cfg(test)]
mod tests;
