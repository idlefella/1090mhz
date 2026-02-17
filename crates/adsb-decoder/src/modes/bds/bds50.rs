use crate::types::{DecodeError, DecodeResult};

/// BDS 5,0 - Track and Turn Report.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackAndTurnReport {
    /// Roll angle in degrees (positive = right wing down), if available.
    pub roll_angle: Option<f64>,
    /// True track angle in degrees, if available.
    pub true_track_angle: Option<f64>,
    /// Ground speed in knots, if available.
    pub ground_speed: Option<f64>,
    /// Track angle rate in degrees/second, if available.
    pub track_angle_rate: Option<f64>,
    /// True airspeed in knots, if available.
    pub true_airspeed: Option<u16>,
}

/// Decode BDS 5,0 from 7-byte MB field.
pub fn decode(mb: &[u8]) -> DecodeResult<TrackAndTurnReport> {
    if mb.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    // Roll angle: status(1) + sign(1) + value(9) = 11 bits
    let roll_status = (mb[0] >> 7) & 0x01;
    let roll_sign = (mb[0] >> 6) & 0x01;
    let roll_raw = ((mb[0] as u16 & 0x3F) << 3) | ((mb[1] as u16) >> 5);
    let roll_angle = if roll_status != 0 {
        let angle = (roll_raw as f64) * 45.0 / 256.0;
        Some(if roll_sign != 0 { -angle } else { angle })
    } else {
        None
    };

    // True track angle: status(1) + sign(1) + value(10) = 12 bits
    let track_status = (mb[1] >> 4) & 0x01;
    let track_sign = (mb[1] >> 3) & 0x01;
    let track_raw = ((mb[1] as u16 & 0x07) << 7) | ((mb[2] as u16) >> 1);
    let true_track_angle = if track_status != 0 {
        let angle = (track_raw as f64) * 90.0 / 512.0;
        Some(if track_sign != 0 {
            360.0 - angle
        } else {
            angle
        })
    } else {
        None
    };

    // Ground speed: status(1) + value(10) = 11 bits
    let gs_status = mb[2] & 0x01;
    let gs_raw = ((mb[3] as u16) << 2) | ((mb[4] as u16) >> 6);
    let ground_speed = if gs_status != 0 {
        Some((gs_raw as f64) * 1024.0 / 1024.0) // 1-kt resolution up to 1023
    } else {
        None
    };

    // Track angle rate: status(1) + sign(1) + value(9) = 11 bits
    let tar_status = (mb[4] >> 5) & 0x01;
    let tar_sign = (mb[4] >> 4) & 0x01;
    let tar_raw = ((mb[4] as u16 & 0x0F) << 5) | ((mb[5] as u16) >> 3);
    let track_angle_rate = if tar_status != 0 {
        let rate = (tar_raw as f64) * 8.0 / 256.0;
        Some(if tar_sign != 0 { -rate } else { rate })
    } else {
        None
    };

    // True airspeed: status(1) + value(10) = 11 bits
    let tas_status = (mb[5] >> 2) & 0x01;
    let tas_raw = ((mb[5] as u16 & 0x03) << 8) | (mb[6] as u16);
    let true_airspeed = if tas_status != 0 {
        Some((tas_raw * 2) as u16) // 2-kt resolution
    } else {
        None
    };

    Ok(TrackAndTurnReport {
        roll_angle,
        true_track_angle,
        ground_speed,
        track_angle_rate,
        true_airspeed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bds50_all_none() {
        let mb = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let report = decode(&mb).unwrap();
        assert!(report.roll_angle.is_none());
        assert!(report.true_track_angle.is_none());
        assert!(report.ground_speed.is_none());
        assert!(report.track_angle_rate.is_none());
        assert!(report.true_airspeed.is_none());
    }

    #[test]
    fn test_decode_bds50_with_data() {
        // Set roll_status=1, roll_sign=0, roll_value=256 -> 45 degrees
        let mb = [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let report = decode(&mb).unwrap();
        assert!(report.roll_angle.is_some());
    }
}
