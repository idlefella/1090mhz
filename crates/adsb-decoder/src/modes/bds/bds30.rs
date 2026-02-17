use crate::types::{DecodeError, DecodeResult};

/// BDS 3,0 - ACAS Resolution Advisory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcasResolutionAdvisory {
    /// Active Resolution Advisory (14 bits).
    pub ara: u16,
    /// Resolution Advisory Complement (4 bits for record 1).
    pub rac: u8,
    /// RA terminated indicator.
    pub ra_terminated: bool,
    /// Multiple threat encounter.
    pub multiple_threat: bool,
    /// Threat type indicator.
    pub threat_type: u8,
    /// Threat identity (ICAO address or altitude + range, 26 bits).
    pub threat_identity: u32,
}

/// Decode BDS 3,0 from 7-byte MB field.
pub fn decode(mb: &[u8]) -> DecodeResult<AcasResolutionAdvisory> {
    if mb.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    // ARA: 14 bits (mb[0] full + mb[1] bits 7-2)
    let ara = ((mb[0] as u16) << 6) | ((mb[1] as u16) >> 2);

    // RAC: 4 bits (mb[1] bits 1-0 + mb[2] bits 7-6)
    let rac = ((mb[1] & 0x03) << 2) | ((mb[2] >> 6) & 0x03);

    // RA terminated: 1 bit
    let ra_terminated = (mb[2] >> 5) & 0x01 != 0;

    // Multiple threat: 1 bit
    let multiple_threat = (mb[2] >> 4) & 0x01 != 0;

    // Threat type: 2 bits
    let threat_type = (mb[2] >> 2) & 0x03;

    // Threat identity: 26 bits
    let threat_identity = ((mb[2] as u32 & 0x03) << 24)
        | ((mb[3] as u32) << 16)
        | ((mb[4] as u32) << 8)
        | (mb[5] as u32);

    Ok(AcasResolutionAdvisory {
        ara,
        rac,
        ra_terminated,
        multiple_threat,
        threat_type,
        threat_identity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bds30() {
        let mb = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let ra = decode(&mb).unwrap();
        assert_eq!(ra.ara, 0);
        assert!(!ra.ra_terminated);
        assert!(!ra.multiple_threat);
    }
}
