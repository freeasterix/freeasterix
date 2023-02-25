mod ais_code;
mod bit_writer;
mod bit_reader;
mod error;
mod write_asterix;
mod read_asterix;
pub use error::{Error, InvalidSpec};
pub use write_asterix::write_asterix;

#[cfg(test)]
mod test_129;
#[cfg(test)]
mod test_samples;
