pub mod bds10;
pub mod bds17;
pub mod bds20;
pub mod bds30;
pub mod bds40;
pub mod bds50;
pub mod bds60;

use crate::types::{DecodeError, DecodeResult};

/// Decoded BDS (Comm-B Data Selector) register.
#[derive(Debug, Clone, PartialEq)]
pub enum BdsPayload {
    DataLinkCapability(bds10::DataLinkCapability),
    GicbCapability(bds17::GicbCapability),
    AircraftIdentification(bds20::AircraftIdentification),
    AcasResolutionAdvisory(bds30::AcasResolutionAdvisory),
    SelectedVerticalIntention(bds40::SelectedVerticalIntention),
    TrackAndTurn(bds50::TrackAndTurnReport),
    HeadingAndSpeed(bds60::HeadingAndSpeedReport),
    /// Raw MB data when BDS register is unknown.
    Unknown([u8; 7]),
}

/// Attempt to decode a known BDS register from the 7-byte MB field.
/// Since the BDS code is generally not transmitted in the MB field
/// for DF 20/21, the caller must specify which BDS register to try.
pub fn decode_bds(bds_code: u8, mb: &[u8]) -> DecodeResult<BdsPayload> {
    if mb.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    match bds_code {
        0x10 => Ok(BdsPayload::DataLinkCapability(bds10::decode(mb)?)),
        0x17 => Ok(BdsPayload::GicbCapability(bds17::decode(mb)?)),
        0x20 => Ok(BdsPayload::AircraftIdentification(bds20::decode(mb)?)),
        0x30 => Ok(BdsPayload::AcasResolutionAdvisory(bds30::decode(mb)?)),
        0x40 => Ok(BdsPayload::SelectedVerticalIntention(bds40::decode(mb)?)),
        0x50 => Ok(BdsPayload::TrackAndTurn(bds50::decode(mb)?)),
        0x60 => Ok(BdsPayload::HeadingAndSpeed(bds60::decode(mb)?)),
        _ => {
            let mut raw = [0u8; 7];
            raw.copy_from_slice(&mb[..7]);
            Ok(BdsPayload::Unknown(raw))
        }
    }
}

/// Try all known BDS decoders and return all that succeed.
/// Useful for inferring the BDS register when it's not explicitly signaled.
pub fn try_all_bds(mb: &[u8]) -> Vec<(u8, BdsPayload)> {
    let codes = [0x10, 0x17, 0x20, 0x30, 0x40, 0x50, 0x60];
    let mut results = Vec::new();
    for &code in &codes {
        if let Ok(payload) = decode_bds(code, mb) {
            results.push((code, payload));
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_known_bds() {
        let mb = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = decode_bds(0x40, &mb);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decode_unknown_bds() {
        let mb = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE];
        let result = decode_bds(0xFF, &mb).unwrap();
        assert!(matches!(result, BdsPayload::Unknown(_)));
    }
}
