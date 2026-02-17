use crate::types::{DecodeError, DecodeResult};

/// BDS 6,0 - Heading and Speed Report.
#[derive(Debug, Clone, PartialEq)]
pub struct HeadingAndSpeedReport {
    /// Magnetic heading in degrees, if available.
    pub magnetic_heading: Option<f64>,
    /// Indicated airspeed in knots, if available.
    pub indicated_airspeed: Option<u16>,
    /// Mach number, if available.
    pub mach_number: Option<f64>,
    /// Barometric altitude rate in ft/min, if available.
    pub barometric_altitude_rate: Option<i32>,
    /// Inertial vertical velocity in ft/min, if available.
    pub inertial_vertical_velocity: Option<i32>,
}

/// Decode BDS 6,0 from 7-byte MB field.
pub fn decode(mb: &[u8]) -> DecodeResult<HeadingAndSpeedReport> {
    if mb.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    // Magnetic heading: status(1) + sign(1) + value(9) = 11 bits
    let hdg_status = (mb[0] >> 7) & 0x01;
    let hdg_sign = (mb[0] >> 6) & 0x01;
    let hdg_raw = ((mb[0] as u16 & 0x3F) << 3) | ((mb[1] as u16) >> 5);
    let magnetic_heading = if hdg_status != 0 {
        let heading = (hdg_raw as f64) * 90.0 / 256.0;
        Some(if hdg_sign != 0 {
            360.0 - heading
        } else {
            heading
        })
    } else {
        None
    };

    // Indicated airspeed: status(1) + value(10) = 11 bits
    let ias_status = (mb[1] >> 4) & 0x01;
    let ias_raw = ((mb[1] as u16 & 0x0F) << 6) | ((mb[2] as u16) >> 2);
    let indicated_airspeed = if ias_status != 0 {
        Some(ias_raw as u16)
    } else {
        None
    };

    // Mach number: status(1) + value(10) = 11 bits
    let mach_status = (mb[2] >> 1) & 0x01;
    let mach_raw = ((mb[2] as u16 & 0x01) << 9) | ((mb[3] as u16) << 1) | ((mb[4] as u16) >> 7);
    let mach_number = if mach_status != 0 {
        Some((mach_raw as f64) * 2.048 / 1024.0) // Resolution: 0.004 Mach
    } else {
        None
    };

    // Barometric altitude rate: status(1) + sign(1) + value(9) = 11 bits
    let bar_status = (mb[4] >> 6) & 0x01;
    let bar_sign = (mb[4] >> 5) & 0x01;
    let bar_raw = ((mb[4] as u16 & 0x1F) << 4) | ((mb[5] as u16) >> 4);
    let barometric_altitude_rate = if bar_status != 0 {
        let rate = (bar_raw as i32) * 32; // 32 ft/min resolution
        Some(if bar_sign != 0 { -rate } else { rate })
    } else {
        None
    };

    // Inertial vertical velocity: status(1) + sign(1) + value(9) = 11 bits
    let ivv_status = (mb[5] >> 3) & 0x01;
    let ivv_sign = (mb[5] >> 2) & 0x01;
    let ivv_raw = ((mb[5] as u16 & 0x03) << 7) | ((mb[6] as u16) >> 1);
    let inertial_vertical_velocity = if ivv_status != 0 {
        let rate = (ivv_raw as i32) * 32;
        Some(if ivv_sign != 0 { -rate } else { rate })
    } else {
        None
    };

    Ok(HeadingAndSpeedReport {
        magnetic_heading,
        indicated_airspeed,
        mach_number,
        barometric_altitude_rate,
        inertial_vertical_velocity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bds60_empty() {
        let mb = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let report = decode(&mb).unwrap();
        assert!(report.magnetic_heading.is_none());
        assert!(report.indicated_airspeed.is_none());
        assert!(report.mach_number.is_none());
        assert!(report.barometric_altitude_rate.is_none());
        assert!(report.inertial_vertical_velocity.is_none());
    }
}
