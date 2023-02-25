use super::{Error, InvalidSpec};

pub struct BitWriter<'a> {
    writer: &'a mut Vec<u8>,
    buffer: u8,
    buf_pos: u32,
    out_pos: u32,
}

impl<'a> BitWriter<'a> {
    pub fn new(writer: &'a mut Vec<u8>, bytes: u32) -> Self {
        Self {
            writer,
            buffer: 0,
            buf_pos: 8,
            out_pos: bytes * 8,
        }
    }

    pub fn write_bits(&mut self, (start, end): (u32, u32), bits: u64) -> Result<(), Error> {
        if start != self.out_pos {
            return Err(InvalidSpec::NonContinousBits.into());
        }

        let nbits = start - end + 1;
        assert!(nbits <= 64, "at most 64 bits is supported in bitwriter");
        let mut bits_remaining = nbits;
        while bits_remaining > 0 {
            let delta = self.buf_pos.min(bits_remaining);
            bits_remaining -= delta;
            self.buf_pos -= delta;
            self.out_pos -= delta;
            let mask = 0xff >> (8 - delta);
            let right_pad = self.buf_pos;
            self.buffer |= (((bits >> bits_remaining) & mask) << right_pad) as u8;
            if self.buf_pos == 0 {
                self.writer.push(self.buffer);
                self.buffer = 0;
                self.buf_pos = 8;
            }
        }
        Ok(())
    }

    pub fn finish(self) -> Result<(), Error> {
        if self.out_pos != 0 {
            Err(InvalidSpec::NonContinousBits.into())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BitWriter;

    #[test]
    fn test_bit_writer() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let mut writer = BitWriter::new(&mut buf, 4);
        writer.write_bits((32, 17), 0x1337)?;
        writer.write_bits((16, 14), 0x00)?;
        writer.write_bits((13, 13), 0x01)?;
        writer.write_bits((12, 5), 0x33)?;
        writer.write_bits((4, 1), 0x07)?;
        writer.finish()?;
        assert_eq!(buf, [0x13, 0x37, 0x13, 0x37]);

        buf.clear();
        let mut writer = BitWriter::new(&mut buf, 2);
        writer.write_bits((16, 13), 0)?;
        writer.write_bits((12, 1), 39426)?;
        assert_eq!(buf, [0x0a, 0x02]);
        Ok(())
    }
}
