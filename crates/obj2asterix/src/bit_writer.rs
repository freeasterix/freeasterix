use std::io::Write;

use super::Error;

pub struct BitWriter<'a, W: Write> {
    writer: &'a mut W,
    buffer: u8,
    buf_pos: u32,
    out_pos: u32,
}

impl<'a, W: Write> BitWriter<'a, W> {
    pub fn new(writer: &'a mut W, bytes: u32) -> Self {
        Self {
            writer,
            buffer: 0,
            buf_pos: 8,
            out_pos: bytes * 8,
        }
    }

    pub fn write_bits(
        &mut self,
        (start, end): (u32, u32),
        bits: i64,
    ) -> Result<(), Error> {
        let nbits = start - end + 1;
        assert!(nbits <= 64, "at most 64 bits is supported in bitwriter");
        let mut bits_remaining = nbits;
        while bits_remaining > 0 {
            let delta = self.buf_pos.min(bits_remaining);
            bits_remaining -= delta;
            self.buf_pos -= delta;
            self.out_pos -= delta;
            let mask = (!0) >> (8 - delta);
            let right_pad = self.buf_pos;
            self.buffer |= ((((bits >> bits_remaining) & mask) << right_pad)) as u8;
            if self.buf_pos == 0 {
                self.writer.write_all(&[self.buffer])
                    .map_err(|_| "failed to write".to_string())?;
                self.buffer = 0;
                self.buf_pos = 8;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BitWriter;

    #[test]
    fn test_writer() {
        let mut buf = Vec::new();
        let mut writer = BitWriter::new(&mut buf, 2);
        writer.write_bits((16, 12), 0xff).expect("failed write");
        writer.write_bits((11, 5), 0xff).expect("failed write");
        writer.write_bits((4, 1), 0xff).expect("failed write");
        drop(writer);
        println!("{:02x?}", buf);
    }
}