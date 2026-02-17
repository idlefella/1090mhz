use crate::types::{DecodeError, DecodeResult};

/// BDS 4,0 - Selected Vertical Intention.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedVerticalIntention {
    /// MCP/FCU selected altitude (in feet), if available.
    pub mcp_selected_altitude: Option<i32>,
    /// FMS selected altitude (in feet), if available.
    pub fms_selected_altitude: Option<i32>,
    /// Barometric pressure setting (in mbar), if available.
    pub barometric_pressure: Option<f64>,
    /// MCP/FCU mode: VNAV, ALT HOLD, APPROACH.
    pub vnav_mode: bool,
    pub alt_hold_mode: bool,
    pub approach_mode: bool,
    /// Target altitude source.
    pub target_altitude_source: u8,
}

/// Decode BDS 4,0 from 7-byte MB field.
pub fn decode(mb: &[u8]) -> DecodeResult<SelectedVerticalIntention> {
    if mb.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    // MCP/FCU selected altitude: status(1) + value(12) = 13 bits
    let mcp_status = (mb[0] >> 7) & 0x01;
    let mcp_raw = ((mb[0] as u16 & 0x7F) << 5) | ((mb[1] as u16) >> 3);
    let mcp_selected_altitude = if mcp_status != 0 {
        Some((mcp_raw as i32) * 16) // 16-ft resolution
    } else {
        None
    };

    // FMS selected altitude: status(1) + value(12) = 13 bits
    let fms_status = (mb[1] >> 2) & 0x01;
    let fms_raw = ((mb[1] as u16 & 0x03) << 10)
        | ((mb[2] as u16) << 2)
        | ((mb[3] as u16) >> 6);
    let fms_selected_altitude = if fms_status != 0 {
        Some((fms_raw as i32) * 16)
    } else {
        None
    };

    // Barometric pressure setting: status(1) + value(12) = 13 bits
    let baro_status = (mb[3] >> 5) & 0x01;
    let baro_raw = ((mb[3] as u16 & 0x1F) << 7) | ((mb[4] as u16) >> 1);
    let barometric_pressure = if baro_status != 0 {
        Some((baro_raw as f64) * 0.1 + 800.0) // 0.1 mbar resolution, offset 800
    } else {
        None
    };

    // Reserved (1 bit) + mode bits
    let vnav_mode = (mb[5] >> 3) & 0x01 != 0;
    let alt_hold_mode = (mb[5] >> 2) & 0x01 != 0;
    let approach_mode = (mb[5] >> 1) & 0x01 != 0;

    let target_altitude_source = ((mb[5] & 0x01) << 1) | ((mb[6] >> 7) & 0x01);

    Ok(SelectedVerticalIntention {
        mcp_selected_altitude,
        fms_selected_altitude,
        barometric_pressure,
        vnav_mode,
        alt_hold_mode,
        approach_mode,
        target_altitude_source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bds40() {
        let mb = [0x82, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let svi = decode(&mb).unwrap();
        assert!(svi.mcp_selected_altitude.is_some());
    }

    #[test]
    fn test_decode_bds40_no_data() {
        let mb = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let svi = decode(&mb).unwrap();
        assert!(svi.mcp_selected_altitude.is_none());
        assert!(svi.fms_selected_altitude.is_none());
        assert!(svi.barometric_pressure.is_none());
    }
}
