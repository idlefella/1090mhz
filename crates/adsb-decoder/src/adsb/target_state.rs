use crate::types::{AltitudeSource, DecodeError, DecodeResult};

/// ADS-B Target State and Status message (TC 29).
#[derive(Debug, Clone, PartialEq)]
pub struct TargetState {
    pub subtype: u8,
    pub data: TargetStateData,
}

/// Target state data varies by subtype.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetStateData {
    /// Subtype 0: target state (version 1).
    V1 {
        vertical_data_available: bool,
        target_altitude_type: AltitudeSource,
        target_altitude_capability: u8,
        vertical_mode: u8,
        target_altitude: i32,
        horizontal_data_available: bool,
        target_heading: f64,
        target_heading_valid: bool,
        horizontal_mode: u8,
        nac_p: u8,
        nic_baro: bool,
        sil: u8,
    },
    /// Subtype 1: target state (version 2).
    V2 {
        sil_supplement: bool,
        selected_altitude_type: AltitudeSource,
        selected_altitude: Option<i32>,
        barometric_pressure_setting: Option<f64>,
        selected_heading: Option<f64>,
        nac_p: u8,
        nic_baro: bool,
        sil: u8,
        autopilot: Option<bool>,
        vnav: Option<bool>,
        alt_hold: Option<bool>,
        approach: Option<bool>,
        tcas_operational: Option<bool>,
        lnav: Option<bool>,
    },
    Unknown(u8),
}

/// Decode target state from the 7-byte ME field.
pub fn decode(me: &[u8]) -> DecodeResult<TargetState> {
    if me.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let tc = (me[0] >> 3) & 0x1F;
    if tc != 29 {
        return Err(DecodeError::InvalidTypeCode(tc));
    }

    let st = me[0] & 0x07;

    let data = match st {
        0 => decode_v1(me),
        1 => decode_v2(me),
        _ => TargetStateData::Unknown(st),
    };

    Ok(TargetState { subtype: st, data })
}

fn decode_v1(me: &[u8]) -> TargetStateData {
    let vertical_data = (me[1] >> 7) & 0x01 != 0;
    let alt_type = if (me[1] >> 6) & 0x01 != 0 {
        AltitudeSource::Barometric
    } else {
        AltitudeSource::Gnss
    };
    let alt_cap = (me[1] >> 4) & 0x03;
    let vertical_mode = (me[1] >> 2) & 0x03;

    let alt_raw = ((me[1] as u16 & 0x03) << 8) | (me[2] as u16);
    let target_altitude = (alt_raw as i32) * 100 - 1000;

    let horizontal_data = (me[3] >> 7) & 0x01 != 0;
    let heading_raw = ((me[3] as u16 & 0x7F) << 2) | ((me[4] as u16) >> 6);
    let target_heading = heading_raw as f64 * 360.0 / 512.0;
    let heading_valid = heading_raw != 0;
    let horizontal_mode = (me[4] >> 4) & 0x03;
    let nac_p = (me[4] >> 0) & 0x0F;
    let nic_baro = (me[5] >> 7) & 0x01 != 0;
    let sil = (me[5] >> 5) & 0x03;

    TargetStateData::V1 {
        vertical_data_available: vertical_data,
        target_altitude_type: alt_type,
        target_altitude_capability: alt_cap,
        vertical_mode,
        target_altitude,
        horizontal_data_available: horizontal_data,
        target_heading,
        target_heading_valid: heading_valid,
        horizontal_mode,
        nac_p,
        nic_baro,
        sil,
    }
}

fn decode_v2(me: &[u8]) -> TargetStateData {
    let sil_supplement = (me[1] >> 7) & 0x01 != 0;
    let fms_selected = (me[1] >> 6) & 0x01 != 0;
    let alt_type = if fms_selected {
        AltitudeSource::Gnss
    } else {
        AltitudeSource::Barometric
    };

    let alt_raw = ((me[1] as u16 & 0x3F) << 5) | ((me[2] as u16) >> 3);
    let selected_altitude = if alt_raw != 0 {
        Some((alt_raw as i32 - 1) * 32)
    } else {
        None
    };

    let baro_raw = ((me[2] as u16 & 0x07) << 6) | ((me[3] as u16) >> 2);
    let barometric_pressure = if baro_raw != 0 {
        Some((baro_raw as f64 - 1.0) * 0.8 + 800.0)
    } else {
        None
    };

    let heading_status = (me[3] >> 1) & 0x01;
    let heading_raw = ((me[3] as u16 & 0x01) << 8) | (me[4] as u16);
    let selected_heading = if heading_status != 0 {
        Some(heading_raw as f64 * 360.0 / 512.0)
    } else {
        None
    };

    let nac_p = (me[5] >> 4) & 0x0F;
    let nic_baro = (me[5] >> 3) & 0x01 != 0;
    let sil = (me[5] >> 1) & 0x03;

    let mode_status = (me[5] >> 0) & 0x01;
    let autopilot = if mode_status != 0 {
        Some((me[6] >> 7) & 0x01 != 0)
    } else {
        None
    };
    let vnav = if mode_status != 0 {
        Some((me[6] >> 6) & 0x01 != 0)
    } else {
        None
    };
    let alt_hold = if mode_status != 0 {
        Some((me[6] >> 5) & 0x01 != 0)
    } else {
        None
    };
    let approach = if mode_status != 0 {
        Some((me[6] >> 3) & 0x01 != 0)
    } else {
        None
    };
    let tcas_operational = if mode_status != 0 {
        Some((me[6] >> 2) & 0x01 != 0)
    } else {
        None
    };
    let lnav = if mode_status != 0 {
        Some((me[6] >> 1) & 0x01 != 0)
    } else {
        None
    };

    TargetStateData::V2 {
        sil_supplement,
        selected_altitude_type: alt_type,
        selected_altitude,
        barometric_pressure_setting: barometric_pressure,
        selected_heading,
        nac_p,
        nic_baro,
        sil,
        autopilot,
        vnav,
        alt_hold,
        approach,
        tcas_operational,
        lnav,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_target_state_v2() {
        // TC=29, ST=1
        let me = [0xE9, 0x80, 0x40, 0x00, 0x00, 0x00, 0x00];
        let ts = decode(&me).unwrap();
        assert_eq!(ts.subtype, 1);
        assert!(matches!(ts.data, TargetStateData::V2 { .. }));
    }
}
