use crate::types::{DecodeError, DecodeResult};

/// BDS 2,0 - Aircraft Identification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AircraftIdentification {
    pub callsign: [u8; 8],
}

impl AircraftIdentification {
    pub fn callsign_str(&self) -> &str {
        let len = self
            .callsign
            .iter()
            .rposition(|&c| c != b' ')
            .map_or(0, |i| i + 1);
        core::str::from_utf8(&self.callsign[..len]).unwrap_or("")
    }
}

/// Character lookup for BDS 2,0 (same as ADS-B identification).
const CALLSIGN_CHARS: [u8; 64] = {
    let mut table = [b' '; 64];
    table[1] = b'A';
    table[2] = b'B';
    table[3] = b'C';
    table[4] = b'D';
    table[5] = b'E';
    table[6] = b'F';
    table[7] = b'G';
    table[8] = b'H';
    table[9] = b'I';
    table[10] = b'J';
    table[11] = b'K';
    table[12] = b'L';
    table[13] = b'M';
    table[14] = b'N';
    table[15] = b'O';
    table[16] = b'P';
    table[17] = b'Q';
    table[18] = b'R';
    table[19] = b'S';
    table[20] = b'T';
    table[21] = b'U';
    table[22] = b'V';
    table[23] = b'W';
    table[24] = b'X';
    table[25] = b'Y';
    table[26] = b'Z';
    table[32] = b' ';
    table[48] = b'0';
    table[49] = b'1';
    table[50] = b'2';
    table[51] = b'3';
    table[52] = b'4';
    table[53] = b'5';
    table[54] = b'6';
    table[55] = b'7';
    table[56] = b'8';
    table[57] = b'9';
    table
};

/// Decode BDS 2,0 from 7-byte MB field.
/// Layout: [BDS2,0 code: 8 bits][C1:6][C2:6][C3:6][C4:6][C5:6][C6:6][C7:6][C8:6]
/// Actually the MB field is 56 bits (7 bytes), and all 56 bits encode 8 six-bit characters
/// (the BDS code itself is not in the MB field; it's determined by the interrogation).
pub fn decode(mb: &[u8]) -> DecodeResult<AircraftIdentification> {
    if mb.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    // 56 bits = 7 bytes, containing 8 six-bit characters + 8 spare bits
    // Actually: first 48 bits are 8 × 6-bit chars, last 8 bits spare
    let bits = u64::from_be_bytes([0, mb[0], mb[1], mb[2], mb[3], mb[4], mb[5], mb[6]]);

    let mut callsign = [b' '; 8];
    for i in 0..8u32 {
        let shift = 50 - (i * 6); // first char at bit 50, then 44, 38, ...
        let idx = ((bits >> shift) & 0x3F) as usize;
        if idx < CALLSIGN_CHARS.len() {
            callsign[i as usize] = CALLSIGN_CHARS[idx];
        }
    }

    Ok(AircraftIdentification { callsign })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bds20() {
        // Encode "KLM1023 "
        // K=11, L=12, M=13, 1=49, 0=48, 2=50, 3=51, ' '=32
        // Pack into 56 bits starting at bit 50
        let mut bits: u64 = 0;
        let chars = [11u64, 12, 13, 49, 48, 50, 51, 32];
        for (i, &c) in chars.iter().enumerate() {
            bits |= c << (50 - i as u64 * 6);
        }
        let bytes = bits.to_be_bytes();
        let mb = &bytes[1..8]; // 7 bytes

        let id = decode(mb).unwrap();
        assert_eq!(id.callsign_str(), "KLM1023");
    }
}
