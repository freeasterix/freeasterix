mod ais_code;
mod bit_reader;
mod bit_writer;
mod error;
mod read_asterix;
mod write_asterix;
pub use error::{Error, InvalidSpec};
pub use write_asterix::write_asterix;

#[cfg(test)]
mod test_129;
#[cfg(test)]
mod test_samples;
