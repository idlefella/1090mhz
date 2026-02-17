use crate::types::{DecodeError, DecodeResult};

/// BDS 1,7 - Common Usage GICB Capability Report.
/// Reports which BDS registers the transponder can provide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GicbCapability {
    pub bds05: bool,
    pub bds06: bool,
    pub bds07: bool,
    pub bds08: bool,
    pub bds09: bool,
    pub bds0a: bool,
    pub bds20: bool,
    pub bds21: bool,
    pub bds40: bool,
    pub bds41: bool,
    pub bds42: bool,
    pub bds43: bool,
    pub bds44: bool,
    pub bds45: bool,
    pub bds48: bool,
    pub bds50: bool,
    pub bds51: bool,
    pub bds52: bool,
    pub bds53: bool,
    pub bds54: bool,
    pub bds55: bool,
    pub bds56: bool,
    pub bds5f: bool,
    pub bds60: bool,
}

/// Decode BDS 1,7 from 7-byte MB field.
pub fn decode(mb: &[u8]) -> DecodeResult<GicbCapability> {
    if mb.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    Ok(GicbCapability {
        bds05: (mb[0] >> 7) & 1 != 0,
        bds06: (mb[0] >> 6) & 1 != 0,
        bds07: (mb[0] >> 5) & 1 != 0,
        bds08: (mb[0] >> 4) & 1 != 0,
        bds09: (mb[0] >> 3) & 1 != 0,
        bds0a: (mb[0] >> 2) & 1 != 0,
        bds20: (mb[0] >> 1) & 1 != 0,
        bds21: mb[0] & 1 != 0,
        bds40: (mb[1] >> 7) & 1 != 0,
        bds41: (mb[1] >> 6) & 1 != 0,
        bds42: (mb[1] >> 5) & 1 != 0,
        bds43: (mb[1] >> 4) & 1 != 0,
        bds44: (mb[1] >> 3) & 1 != 0,
        bds45: (mb[1] >> 2) & 1 != 0,
        bds48: (mb[1] >> 1) & 1 != 0,
        bds50: mb[1] & 1 != 0,
        bds51: (mb[2] >> 7) & 1 != 0,
        bds52: (mb[2] >> 6) & 1 != 0,
        bds53: (mb[2] >> 5) & 1 != 0,
        bds54: (mb[2] >> 4) & 1 != 0,
        bds55: (mb[2] >> 3) & 1 != 0,
        bds56: (mb[2] >> 2) & 1 != 0,
        bds5f: (mb[2] >> 1) & 1 != 0,
        bds60: mb[2] & 1 != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bds17() {
        // All capabilities enabled in first 3 bytes
        let mb = [0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00];
        let cap = decode(&mb).unwrap();
        assert!(cap.bds05);
        assert!(cap.bds20);
        assert!(cap.bds40);
        assert!(cap.bds50);
        assert!(cap.bds60);
    }
}
