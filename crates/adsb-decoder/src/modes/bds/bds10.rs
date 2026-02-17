use crate::types::{DecodeError, DecodeResult};

/// BDS 1,0 - Data Link Capability Report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLinkCapability {
    /// BDS codes that the transponder can support (as a bit mask).
    pub bds_codes: u16,
    /// Continuation flag.
    pub continuation_flag: bool,
    /// Overlay command capability.
    pub overlay_command_capability: bool,
    /// ACAS active.
    pub acas: bool,
    /// Mode-S specific services capability.
    pub mode_s_specific_services: bool,
    /// Uplink ELM average throughput capability.
    pub uplink_elm: u8,
    /// Downlink ELM throughput capability.
    pub downlink_elm: u8,
    /// Aircraft identification capability.
    pub aircraft_id_capability: bool,
    /// Squitter capability subfield.
    pub squitter_capability: u8,
    /// Surveillance identifier code.
    pub surveillance_id: u8,
    /// Common usage GICB capability report.
    pub common_usage_gicb: bool,
}

/// Decode BDS 1,0 from 7-byte MB field.
pub fn decode(mb: &[u8]) -> DecodeResult<DataLinkCapability> {
    if mb.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    // Byte 0: BDS 1,0 config
    let continuation_flag = (mb[0] >> 7) & 0x01 != 0;
    let overlay_command_capability = (mb[0] >> 1) & 0x01 != 0;
    let acas = mb[0] & 0x01 != 0;

    let mode_s_specific_services = (mb[1] >> 7) & 0x01 != 0;
    let uplink_elm = (mb[1] >> 4) & 0x07;
    let downlink_elm = (mb[1] >> 1) & 0x07;
    let aircraft_id_capability = mb[1] & 0x01 != 0;

    let squitter_capability = (mb[2] >> 5) & 0x07;
    let surveillance_id = (mb[2] >> 4) & 0x01;
    let common_usage_gicb = (mb[2] >> 3) & 0x01 != 0;

    let bds_codes = ((mb[3] as u16) << 8) | (mb[4] as u16);

    Ok(DataLinkCapability {
        bds_codes,
        continuation_flag,
        overlay_command_capability,
        acas,
        mode_s_specific_services,
        uplink_elm,
        downlink_elm,
        aircraft_id_capability,
        squitter_capability,
        surveillance_id,
        common_usage_gicb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bds10() {
        let mb = [0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let cap = decode(&mb).unwrap();
        assert!(!cap.continuation_flag);
    }
}
