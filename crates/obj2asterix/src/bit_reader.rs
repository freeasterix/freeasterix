use super::{Error, InvalidSpec};

pub struct BitReader<'a, 'b> {
    reader: &'a mut &'b [u8],
    buffer: u8,
    buf_pos: u32,
    out_pos: u32,
}

impl<'a, 'b: 'a> BitReader<'a, 'b> {
    pub fn new(reader: &'a mut &'b[u8], bytes: u32) -> Self {
        Self {
            reader,
            buffer: 0,
            buf_pos: 0,
            out_pos: bytes * 8,
        }
    }

    pub fn read_bits(&mut self, start: u32, end: u32) -> Result<u64, Error> {
        if start != self.out_pos {
            return Err(InvalidSpec::NonContinousBits.into());
        }

        let nbits = start - end + 1;
        assert!(nbits <= 64, "at most 64 bits is supported in bitwriter");
        let mut rv = 0;
        let mut bits_remaining = nbits;
        while bits_remaining > 0 {
            if self.buf_pos == 0 {
                if let Some((&byte, tail)) = self.reader.split_first() {
                    *self.reader = tail;
                    self.buffer = byte;
                    self.buf_pos = 8;
                } else {
                    return Err(Error::ReadingOob);
                };
            }

            let delta = self.buf_pos.min(bits_remaining);
            bits_remaining -= delta;
            self.buf_pos -= delta;
            self.out_pos -= delta;
            let mask = 0xff >> (8 - delta);
            let right_pad = self.buf_pos;
            rv = (rv << delta) | ((self.buffer >> right_pad) & mask) as u64;
        }
        Ok(rv)
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
    use super::BitReader;

    #[test]
    fn test_bit_reader() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf: &[u8] = &[0x13, 0x37, 0x13, 0x37];
        let mut reader = BitReader::new(&mut buf, 4);
        assert_eq!(reader.read_bits(32, 17)?, 0x1337);
        assert_eq!(reader.read_bits(16, 14)?, 0x00);
        assert_eq!(reader.read_bits(13, 13)?, 0x01);
        assert_eq!(reader.read_bits(12, 5)?, 0x33);
        assert_eq!(reader.read_bits(4, 1)?, 0x07);
        reader.finish()?;

        let mut buf: &[u8] = &[0x0a, 0x02];
        let mut reader = BitReader::new(&mut buf, 2);
        assert_eq!(reader.read_bits(16, 13)?, 0);
        assert_eq!(reader.read_bits(12, 1)?, 2562);
        reader.finish()?;

        Ok(())
    }
}
