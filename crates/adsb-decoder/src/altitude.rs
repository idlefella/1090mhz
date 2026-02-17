use crate::types::{Altitude, AltitudeSource, DecodeError, DecodeResult};

/// Decode altitude from a 13-bit altitude code (DF 0/4/16/20 and ADS-B position).
///
/// The 13-bit code has the following layout:
///   M-bit at position 6 (bit index from MSB), Q-bit at position 4.
///
/// - If Q-bit = 1: altitude = 25 * N - 1000 (N is the 11-bit value after removing M and Q bits)
/// - If Q-bit = 0: Gray code decoding (Gillham code for 100-ft increments)
#[inline]
pub fn decode_altitude_13(code: u16) -> DecodeResult<Altitude> {
    if code == 0 {
        return Err(DecodeError::InvalidData("altitude code is zero"));
    }

    let m_bit = (code >> 6) & 1;
    let q_bit = (code >> 4) & 1;

    if m_bit != 0 {
        // Metric altitude (not commonly used)
        return Err(DecodeError::InvalidData("metric altitude not supported"));
    }

    if q_bit == 1 {
        decode_altitude_q_bit(code)
    } else {
        decode_altitude_gray(code)
    }
}

/// Q-bit altitude decoding: remove Q and M bits, form 11-bit value, multiply by 25 and subtract 1000.
#[inline]
fn decode_altitude_q_bit(code: u16) -> DecodeResult<Altitude> {
    // Remove Q bit (bit 4) and M bit (bit 6) from the 13-bit code.
    // Bits: [12 11 10 9 8 7] [6=M] [5] [4=Q] [3 2 1 0]
    // Result 11 bits: [12 11 10 9 8 7 5 3 2 1 0]
    let high = (code >> 7) & 0x3F; // bits 12-7 (6 bits)
    let mid = (code >> 5) & 0x01; // bit 5 (1 bit, between M and Q)
    let low = code & 0x0F; // bits 3-0 (4 bits, below Q)

    let n = (high << 5) | (mid << 4) | low;

    if n == 0 {
        return Err(DecodeError::InvalidData("altitude N value is zero"));
    }

    let feet = (n as i32) * 25 - 1000;
    Ok(Altitude {
        feet,
        source: AltitudeSource::Barometric,
    })
}

/// Gray code altitude decoding (Q-bit = 0, M-bit = 0).
/// Uses Gillham code: 500-ft increments encoded in Gray code.
fn decode_altitude_gray(code: u16) -> DecodeResult<Altitude> {
    // Extract the individual bits for Gillham decoding.
    // The 13-bit altitude code (after removing M and Q bits) maps to:
    // C1 A1 C2 A2 C4 A4 [M=0] B1 [Q=0] B2 D2 B4 D4
    let c1 = (code >> 12) & 1;
    let a1 = (code >> 11) & 1;
    let c2 = (code >> 10) & 1;
    let a2 = (code >> 9) & 1;
    let c4 = (code >> 8) & 1;
    let a4 = (code >> 7) & 1;
    // bit 6 = M (already checked = 0)
    let b1 = (code >> 5) & 1;
    // bit 4 = Q (already checked = 0)
    let b2 = (code >> 3) & 1;
    let d2 = (code >> 2) & 1;
    let b4 = (code >> 1) & 1;
    let d4 = code & 1;

    // Only D2 and D4 are present; D1 is always 0
    let d_gray = (d2 << 1) | d4;
    let d_val = gray_to_binary(d_gray, 2);

    // Gray to binary for the 100-ft part (A and B bits)
    let a_gray = (a1 << 2) | (a2 << 1) | a4;
    let b_gray = (b1 << 2) | (b2 << 1) | b4;

    let a_val = gray_to_binary(a_gray, 3);
    let b_val = gray_to_binary(b_gray, 3);

    // C bits for the 500-ft group Gray code
    let c_gray = (c1 << 2) | (c2 << 1) | c4;
    let c_val = gray_to_binary(c_gray, 3);

    if c_val == 0 || c_val == 5 || c_val == 6 {
        return Err(DecodeError::InvalidData("invalid Gillham altitude code"));
    }

    // Full Gillham decode using the standard algorithm
    let alt = gillham_decode(c_val, a_val, b_val, d_val);

    match alt {
        Some(feet) => Ok(Altitude {
            feet,
            source: AltitudeSource::Barometric,
        }),
        None => Err(DecodeError::InvalidData("invalid Gillham altitude code")),
    }
}

/// Convert Gray code to binary.
#[inline]
fn gray_to_binary(gray: u16, _bits: u32) -> u16 {
    let mut val = gray;
    let mut mask = val >> 1;
    while mask != 0 {
        val ^= mask;
        mask >>= 1;
    }
    val
}

/// Standard Gillham altitude decode.
/// Takes decoded C, A, B, D values (already converted from Gray code)
/// and returns altitude in feet.
fn gillham_decode(c: u16, a: u16, b: u16, d: u16) -> Option<i32> {
    // The 500-ft increment is derived from C bits
    // Valid C values: 1,2,3,4,7 (Gray decoded)
    let five_hundreds: i32 = match c {
        1 => 1,
        2 => 3,
        3 => 2,
        4 => 6, // not standard but possible
        7 => 4,
        _ => return None,
    };

    // The 100-ft increment is derived from A, B, D
    // There are 5 possible 100-ft values: 0, 100, 200, 300, 400
    let one_hundreds: i32 = if d == 0 {
        match (a, b) {
            (0, 0) => 0,
            (0, 1) => 1,
            (0, 3) => 2,
            (0, 2) => 3,
            (1, 2) => 4,
            _ => return None,
        }
    } else {
        // D=1 means 500-ft offset within the group
        match (a, b) {
            (1, 2) => 5,
            (1, 3) => 6,
            (1, 1) => 7,
            (1, 0) => 8,
            (0, 0) => 9,
            _ => return None,
        }
    };

    // Each C increment = 500ft, each A/B/D increment = 100ft
    // Altitude = (500 * five_hundreds + 100 * one_hundreds) - 1300
    let alt = five_hundreds * 500 + one_hundreds * 100 - 1300;

    Some(alt)
}

/// Decode altitude from a 12-bit altitude code in ADS-B messages (TC 9-18, 20-22).
/// This is the same as the 13-bit code but without the M-bit.
/// Layout: [Q C1 A1 C2 A2 C4 A4 B1 B2 D2 B4 D4]
/// Wait -- actually the ADS-B altitude field is 12 bits:
///   bits: [12bits total] with Q-bit at bit 4 (from the right, 0-indexed at bit 4 of the 12-bit field)
///
/// Actually the ME field for airborne position has the altitude encoded as 12 bits,
/// identical layout to the 13-bit code minus the M-bit.
#[inline]
pub fn decode_altitude_12(code: u16) -> DecodeResult<Altitude> {
    if code == 0 {
        return Err(DecodeError::InvalidData("altitude code is zero"));
    }

    // In the 12-bit ADS-B altitude code, Q-bit is bit 4 (counting from bit 0 at LSB).
    let q_bit = (code >> 4) & 1;

    if q_bit == 1 {
        // Remove Q-bit: high part (bits 11-5) and low part (bits 3-0)
        let high = (code >> 5) & 0x7F; // 7 bits
        let low = code & 0x0F; // 4 bits
        let n = (high << 4) | low;
        if n == 0 {
            return Err(DecodeError::InvalidData("altitude N value is zero"));
        }
        let feet = (n as i32) * 25 - 1000;
        Ok(Altitude {
            feet,
            source: AltitudeSource::Barometric,
        })
    } else {
        // Gray code / Gillham for the 12-bit code
        // Map 12-bit to 13-bit by inserting M=0 at bit 6
        // 12-bit: [C1 A1 C2 A2 C4 A4 B1 Q=0 B2 D2 B4 D4]
        // 13-bit: [C1 A1 C2 A2 C4 A4 M=0 B1 Q=0 B2 D2 B4 D4]
        let upper = (code >> 6) & 0x3F; // bits 11-6 -> C1 A1 C2 A2 C4 A4
        let lower = code & 0x3F; // bits 5-0 -> B1 Q B2 D2 B4 D4
        let code13 = (upper << 7) | lower;
        decode_altitude_gray(code13)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_q_bit_altitude() {
        // Example: altitude code with Q=1
        // N = 60 (after removing Q), altitude = 60*25 - 1000 = 500 ft
        // Construct 12-bit code: high=0000011 Q=1 low=1100
        // n = 0000011_1100 = 0x3C = 60... let me compute properly
        // We want N=60 => high=3 (bits 10-4), low=12 (bits 3-0) => 3<<5 | 1<<4 | 12 = 0x06C + 0x10 = 0x07C
        // Actually: code = (high << 5) | (1 << 4) | low
        // high = N >> 4 = 60 >> 4 = 3
        // low = N & 0x0F = 60 & 0x0F = 12
        // code = (3 << 5) | (1 << 4) | 12 = 96 | 16 | 12 = 124 = 0x7C
        let alt = decode_altitude_12(0x07C).unwrap();
        assert_eq!(alt.feet, 500);
        assert_eq!(alt.source, AltitudeSource::Barometric);
    }

    #[test]
    fn test_q_bit_altitude_38000() {
        // 38000 ft: N = (38000 + 1000) / 25 = 1560
        // high = 1560 >> 4 = 97, low = 1560 & 0xF = 8
        // code = (97 << 5) | (1 << 4) | 8 = 3104 | 16 | 8 = 3128 = 0xC38
        let alt = decode_altitude_12(0xC38).unwrap();
        assert_eq!(alt.feet, 38000);
    }

    #[test]
    fn test_altitude_zero_code() {
        assert!(decode_altitude_12(0).is_err());
    }

    #[test]
    fn test_gray_to_binary() {
        assert_eq!(gray_to_binary(0b000, 3), 0);
        assert_eq!(gray_to_binary(0b001, 3), 1);
        assert_eq!(gray_to_binary(0b011, 3), 2);
        assert_eq!(gray_to_binary(0b010, 3), 3);
        assert_eq!(gray_to_binary(0b110, 3), 4);
        assert_eq!(gray_to_binary(0b111, 3), 5);
        assert_eq!(gray_to_binary(0b101, 3), 6);
        assert_eq!(gray_to_binary(0b100, 3), 7);
    }
}
