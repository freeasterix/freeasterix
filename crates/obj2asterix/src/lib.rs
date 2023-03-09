#![deny(missing_docs)]

//! A crate for converting JSON to asterix data packets and back.
//!
//! This crate exposes three functions:
//! - [`read_asterix`] - used to convert one asterix data packet to a JSON map.
//! - [`read_asterix_multi`] - used to parse multiple data packets.
//! - [`write_asterix`] - used to convert a JSON map to an asterix data packet.
//!
//! Example:
//! ```
//! # use std::fs;
//! use spec_parser::spec_xml::Category;
//! use obj2asterix::read_asterix;
//!
//! let xml = fs::read_to_string("../../specs-xml/asterix_cat062_1_18.xml")?;
//! let category = Category::parse(&xml)?;
//! let mut reader = &[0x3E, 0x00, 0x06, 0x80, 0xB0, 0xB1][..];
//! let value = read_asterix(&mut reader, &category)?;
//! dbg!(value);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`read_asterix`]: crate::read_asterix()
//! [`read_asterix_multi`]: crate::read_asterix_multi()
//! [`write_asterix`]: crate::write_asterix()

mod ais_code;
mod bit_reader;
mod bit_writer;
mod error;
mod read_asterix;
mod write_asterix;

#[cfg(test)]
mod test_samples;

pub use error::{Error, InvalidSpec};
pub use read_asterix::{read_asterix, read_asterix_multi};
pub use write_asterix::write_asterix;
