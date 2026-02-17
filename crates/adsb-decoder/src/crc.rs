/// CRC-24 computation for Mode-S messages.
///
/// Generator polynomial: x^24 + x^23 + x^22 + ... (0xFFF409).
/// The CRC is appended as the last 3 bytes (parity/interrogator identifier).
///
/// For DF 17/18 (ADS-B): the CRC remainder over all bytes must be 0.
/// For other DFs: the remainder XOR'd with the ICAO address gives the parity.

const GENERATOR: u32 = 0xFFF409;

/// Compile-time generated CRC-24 lookup table (256 entries).
const CRC_TABLE: [u32; 256] = generate_crc_table();

const fn generate_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0u32;
    while i < 256 {
        let mut crc = i << 16;
        let mut j = 0;
        while j < 8 {
            if crc & 0x800000 != 0 {
                crc = (crc << 1) ^ GENERATOR;
            } else {
                crc <<= 1;
            }
            j += 1;
        }
        table[i as usize] = crc & 0xFFFFFF;
        i += 1;
    }
    table
}

/// Compute CRC-24 remainder over the given bytes.
/// For a valid ADS-B message (DF17/18), calling this on all 14 bytes should return 0.
/// For short messages (7 bytes), same principle applies.
#[inline]
pub fn crc_remainder(data: &[u8]) -> u32 {
    let mut crc: u32 = 0;
    for &byte in data {
        let idx = ((crc >> 16) ^ (byte as u32)) & 0xFF;
        crc = ((crc << 8) ^ CRC_TABLE[idx as usize]) & 0xFFFFFF;
    }
    crc
}

/// Compute the CRC-24 over the message portion (excluding last 3 bytes)
/// and return the remainder. This can be compared with the parity field.
#[inline]
pub fn compute_checksum(msg: &[u8]) -> u32 {
    if msg.len() < 3 {
        return 0;
    }
    crc_remainder(&msg[..msg.len() - 3])
}

/// Extract the 24-bit parity/interrogator identifier from the last 3 bytes.
#[inline]
pub fn extract_parity(msg: &[u8]) -> u32 {
    let len = msg.len();
    ((msg[len - 3] as u32) << 16) | ((msg[len - 2] as u32) << 8) | (msg[len - 1] as u32)
}

/// Verify CRC for DF17/18 messages: remainder over all bytes must be 0.
#[inline]
pub fn verify_adsb_crc(msg: &[u8]) -> bool {
    crc_remainder(msg) == 0
}

/// For non-ADS-B DFs: XOR the computed checksum with the parity field
/// to recover the ICAO address.
#[inline]
pub fn recover_icao(msg: &[u8]) -> u32 {
    compute_checksum(msg) ^ extract_parity(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc_table_generation() {
        // Verify table is generated and has expected properties
        assert_eq!(CRC_TABLE[0], 0);
        assert_ne!(CRC_TABLE[1], 0);
        // All entries must be 24-bit
        for &entry in &CRC_TABLE {
            assert_eq!(entry & !0xFFFFFF, 0);
        }
    }

    #[test]
    fn test_known_adsb_message() {
        // Known DF17 message: 8D4840D6202CC371C32CE0576098
        let msg = hex_to_bytes("8D4840D6202CC371C32CE0576098");
        assert!(verify_adsb_crc(&msg));
    }

    #[test]
    fn test_crc_invalid() {
        // Corrupt a valid message
        let mut msg = hex_to_bytes("8D4840D6202CC371C32CE0576098");
        msg[5] ^= 0x01;
        assert!(!verify_adsb_crc(&msg));
    }

    #[test]
    fn test_known_adsb_velocity() {
        // Known DF17 velocity message: 8D485020994409940838175B284F
        let msg = hex_to_bytes("8D485020994409940838175B284F");
        assert!(verify_adsb_crc(&msg));
    }

    #[test]
    fn test_known_adsb_position() {
        // Known DF17 position message (even): 8D40621D58C382D690C8AC2863A7
        let msg = hex_to_bytes("8D40621D58C382D690C8AC2863A7");
        assert!(verify_adsb_crc(&msg));
    }

    fn hex_to_bytes(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }
}
