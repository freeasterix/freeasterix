use crate::error::Error;
const AIS_CODING: [(u8, u8, u8, u8); 3] = [
    (b'A', b'Z', 0b000001, 0b011010),
    (b'0', b'9', 0b110000, 0b111001),
    (b' ', b' ', 0b100000, 0b100000),
];

pub fn encode_ais(s: &str, length: u32) -> Result<u64, Error> {
    let bytes = s.as_bytes();
    if s.len() > 8.min(length as usize) {
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

pub fn decode_ais(bits: u64, count: u32) -> Result<String, Error> {
    let mut rv = String::new();

    let mut pos = count;
    while pos > 0 {
        pos -= 1;
        let mask = (1 << 6) - 1;
        let code = ((bits >> (pos * 6)) & mask) as u8;
        let &(start_byte, _, start_coded, _) = AIS_CODING
            .iter()
            .find(|&&(_, _, sc, ec)| code >= sc && code <= ec)
            .ok_or(Error::InvalidAisCode { code })?;
        let chr = code - start_coded + start_byte;
        rv.push(chr as char);
    }

    Ok(rv)
}
