use crate::types::{DecodeError, DecodeResult, WakeVortexCategory};

/// ADS-B Aircraft Identification (TC 1-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identification {
    pub type_code: u8,
    pub category: WakeVortexCategory,
    pub callsign: [u8; 8],
}

impl Identification {
    /// Length of the decoded callsign (excluding trailing spaces).
    pub fn callsign_len(&self) -> usize {
        self.callsign
            .iter()
            .rposition(|&c| c != b' ')
            .map_or(0, |i| i + 1)
    }

    /// Return the callsign as a string slice.
    pub fn callsign_str(&self) -> &str {
        let len = self.callsign_len();
        // Safety: we only put ASCII chars into the callsign array
        core::str::from_utf8(&self.callsign[..len]).unwrap_or("")
    }
}

/// Character lookup table for ADS-B callsign encoding.
/// Index 1-26 = A-Z, 32-57 = space and 0-9, 48-57 = 0-9
const CALLSIGN_CHARS: [u8; 64] = {
    let mut table = [b' '; 64];
    // #, A-Z at indices 1-26
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
    // Space at index 32
    table[32] = b' ';
    // 0-9 at indices 48-57
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

/// Decode an identification message from the 7-byte ME field.
/// ME layout (56 bits):
///   [TC:5][CA:3][C1:6][C2:6][C3:6][C4:6][C5:6][C6:6][C7:6][C8:6]
///   TC = type code (1-4), CA = category, C1-C8 = callsign characters
pub fn decode(me: &[u8]) -> DecodeResult<Identification> {
    if me.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let tc = (me[0] >> 3) & 0x1F;
    let ca = me[0] & 0x07;

    if !(1..=4).contains(&tc) {
        return Err(DecodeError::InvalidTypeCode(tc));
    }

    // Extract 8 six-bit characters from the remaining 48 bits
    // Bits: [CA:3][C1:6][C2:6][C3:6][C4:6][C5:6][C6:6][C7:6][C8:6]
    // Starting from bit 5 of byte 0 (after TC)
    let bits = u64::from_be_bytes([0, me[0], me[1], me[2], me[3], me[4], me[5], me[6]]);

    let mut callsign = [b' '; 8];
    for i in 0..8u32 {
        let shift = 42 - (i * 6); // first char at bit 42, then 36, 30, ...
        let idx = ((bits >> shift) & 0x3F) as usize;
        callsign[i as usize] = CALLSIGN_CHARS[idx];
    }

    let category = decode_wake_vortex(tc, ca);

    Ok(Identification {
        type_code: tc,
        category,
        callsign,
    })
}

/// Decode wake vortex category from TC and CA.
fn decode_wake_vortex(tc: u8, ca: u8) -> WakeVortexCategory {
    match (tc, ca) {
        (1, _) => WakeVortexCategory::Reserved,
        (2, 0) => WakeVortexCategory::NoInfo,
        (2, 1) => WakeVortexCategory::SurfaceEmergency,
        (2, 2) | (2, 3) => WakeVortexCategory::SurfaceService,
        (2, 4) | (2, 5) | (2, 6) | (2, 7) => WakeVortexCategory::GroundObstruction,
        (3, 0) => WakeVortexCategory::NoInfo,
        (3, 1) => WakeVortexCategory::Glider,
        (3, 2) => WakeVortexCategory::Lighter,
        (3, 3) => WakeVortexCategory::Parachutist,
        (3, 4) => WakeVortexCategory::Ultralight,
        (3, 5) => WakeVortexCategory::Reserved,
        (3, 6) => WakeVortexCategory::Uav,
        (3, 7) => WakeVortexCategory::Space,
        (4, 0) => WakeVortexCategory::NoInfo,
        (4, 1) => WakeVortexCategory::Light,
        (4, 2) => WakeVortexCategory::Medium,
        (4, 3) => WakeVortexCategory::Medium,
        (4, 4) => WakeVortexCategory::HighPerformance,
        (4, 5) => WakeVortexCategory::Heavy,
        (4, 6) => WakeVortexCategory::HighPerformance,
        (4, 7) => WakeVortexCategory::Rotorcraft,
        _ => WakeVortexCategory::Unknown(tc, ca),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_callsign() {
        // Example: "KLM1023 " encoded in ADS-B
        // TC=4, CA=0 -> ME byte 0 = 0b00100_000 = 0x20
        // K=11, L=12, M=13, 1=49, 0=48, 2=50, 3=51, space=32
        // 6-bit values: K=11, L=12, M=13, 1=49, 0=48, 2=50, 3=51, ' '=32
        // Packed into 48 bits starting at bit 42
        let me = build_identification_me(4, 0, b"KLM1023 ");
        let id = decode(&me).unwrap();
        assert_eq!(id.callsign_str(), "KLM1023");
        assert_eq!(id.type_code, 4);
    }

    #[test]
    fn test_decode_from_hex() {
        // 8D4840D6202CC371C32CE0576098
        // ME = 20 2C C3 71 C3 2C E0
        // TC=4 (0x20 >> 3 = 4), CA=0
        let me = [0x20, 0x2C, 0xC3, 0x71, 0xC3, 0x2C, 0xE0];
        let id = decode(&me).unwrap();
        assert_eq!(id.type_code, 4);
        // Callsign should decode to "KLM1023 "
        assert_eq!(id.callsign_str(), "KLM1023");
    }

    /// Helper to build ME bytes for identification.
    fn build_identification_me(tc: u8, ca: u8, callsign: &[u8; 8]) -> [u8; 7] {
        let mut bits: u64 = 0;
        bits |= ((tc as u64) & 0x1F) << 51;
        bits |= ((ca as u64) & 0x07) << 48;
        for i in 0..8u64 {
            let ch = callsign[i as usize];
            let idx = match ch {
                b'A'..=b'Z' => (ch - b'A' + 1) as u64,
                b'0'..=b'9' => (ch - b'0' + 48) as u64,
                b' ' => 32,
                _ => 0,
            };
            bits |= idx << (42 - i * 6);
        }
        let bytes = bits.to_be_bytes();
        let mut me = [0u8; 7];
        me.copy_from_slice(&bytes[1..8]);
        me
    }
}
