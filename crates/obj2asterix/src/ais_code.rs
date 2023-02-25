use crate::error::Error;
const AIS_CODING: [(u8, u8, u8, u8); 3] = [
    (b'A', b'Z', 0b000001, 0b011010),
    (b'0', b'9', 0b110000, 0b111001),
    (b' ', b' ', 0b100000, 0b100000),
];

pub fn encode_ais(s: &str) -> Result<u64, Error> {
    let bytes = s.as_bytes();
    if s.len() > 8 {
        return Err(Error::AisTooLong {
            string: s.to_string(),
        });
    }
    let mut rv = 0;
    for &chr in bytes.iter() {
        let &(start_byte, _, start_coded, _end) = AIS_CODING
            .iter()
            .find(|&&(sb, eb, _, _)| chr >= sb && chr <= eb)
            .ok_or_else(|| Error::InvalidAisChar {
                chr: chr as char,
                string: s.to_string(),
            })?;
        let out = chr - start_byte + start_coded;
        rv = (rv << 6) | (out as u64);
    }
    for _ in s.len()..8 {
        rv = (rv << 6) | 0b100000;
    }
    Ok(rv)
}
