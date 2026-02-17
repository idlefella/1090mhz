use crate::types::{AltitudeSource, DecodeError, DecodeResult};

/// ADS-B Airborne Velocity message (TC 19).
#[derive(Debug, Clone, PartialEq)]
pub struct AirborneVelocity {
    pub subtype: u8,
    pub intent_change: bool,
    pub ifr_capability: bool,
    pub nac_v: u8,
    pub velocity: Option<VelocityData>,
    pub vertical_rate: Option<VerticalRate>,
    pub gnss_baro_diff: Option<i16>,
}

/// Velocity information (ground speed or airspeed).
#[derive(Debug, Clone, PartialEq)]
pub enum VelocityData {
    /// Subtype 1/2: ground speed from east/north components.
    GroundSpeed {
        east_west_knots: f64,
        north_south_knots: f64,
        speed_knots: f64,
        heading_degrees: f64,
    },
    /// Subtype 3/4: airspeed with heading.
    Airspeed {
        heading_degrees: f64,
        airspeed_knots: u16,
        airspeed_is_tas: bool,
    },
}

/// Vertical rate information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalRate {
    pub feet_per_minute: i32,
    pub source: AltitudeSource,
}

/// Decode an airborne velocity message from the 7-byte ME field.
///
/// ME layout (56 bits):
///   [TC:5][ST:3][IC:1][IFR:1][NACv:3][...subtype-dependent...]
pub fn decode(me: &[u8]) -> DecodeResult<AirborneVelocity> {
    if me.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let tc = (me[0] >> 3) & 0x1F;
    if tc != 19 {
        return Err(DecodeError::InvalidTypeCode(tc));
    }

    let st = me[0] & 0x07;
    let ic = (me[1] >> 7) & 0x01;
    let ifr = (me[1] >> 6) & 0x01;
    let nac_v = (me[1] >> 3) & 0x07;

    let velocity = match st {
        1 | 2 => decode_ground_speed(me, st)?,
        3 | 4 => decode_airspeed(me, st)?,
        _ => None,
    };

    // Vertical rate: sign(1) + source(1) + value(9) starting at bit 36
    let vr_source = (me[4] >> 4) & 0x01;
    let vr_sign = (me[4] >> 3) & 0x01;
    let vr_value = (((me[4] as u16) & 0x07) << 6) | ((me[5] as u16) >> 2);

    let vertical_rate = if vr_value != 0 {
        let rate = ((vr_value as i32) - 1) * 64;
        let rate = if vr_sign != 0 { -rate } else { rate };
        Some(VerticalRate {
            feet_per_minute: rate,
            source: if vr_source != 0 {
                AltitudeSource::Barometric
            } else {
                AltitudeSource::Gnss
            },
        })
    } else {
        None
    };

    // GNSS/baro altitude difference
    let diff_sign = (me[5] >> 1) & 0x01;
    let diff_value = (((me[5] as u16) & 0x01) << 6) | ((me[6] as u16) >> 2);

    let gnss_baro_diff = if diff_value != 0 {
        let diff = ((diff_value as i16) - 1) * 25;
        Some(if diff_sign != 0 { -diff } else { diff })
    } else {
        None
    };

    Ok(AirborneVelocity {
        subtype: st,
        intent_change: ic != 0,
        ifr_capability: ifr != 0,
        nac_v,
        velocity,
        vertical_rate,
        gnss_baro_diff,
    })
}

/// Decode ground speed from subtype 1/2 velocity message.
fn decode_ground_speed(me: &[u8], subtype: u8) -> DecodeResult<Option<VelocityData>> {
    // East-West: direction(1) + value(10)
    let ew_dir = (me[1] >> 2) & 0x01;
    let ew_vel = (((me[1] as u16) & 0x03) << 8) | (me[2] as u16);

    // North-South: direction(1) + value(10)
    let ns_dir = (me[3] >> 7) & 0x01;
    let ns_vel = (((me[3] as u16) & 0x7F) << 3) | ((me[4] as u16) >> 5);

    if ew_vel == 0 || ns_vel == 0 {
        return Ok(None);
    }

    let scale = if subtype == 2 { 4.0 } else { 1.0 };

    let ew_speed = ((ew_vel as f64) - 1.0) * scale;
    let ns_speed = ((ns_vel as f64) - 1.0) * scale;

    // Apply direction
    let ew = if ew_dir != 0 { -ew_speed } else { ew_speed }; // 0=east, 1=west
    let ns = if ns_dir != 0 { -ns_speed } else { ns_speed }; // 0=north, 1=south

    let speed = (ew * ew + ns * ns).sqrt();
    let heading = ew.atan2(ns).to_degrees();
    let heading = if heading < 0.0 {
        heading + 360.0
    } else {
        heading
    };

    Ok(Some(VelocityData::GroundSpeed {
        east_west_knots: ew,
        north_south_knots: ns,
        speed_knots: speed,
        heading_degrees: heading,
    }))
}

/// Decode airspeed from subtype 3/4 velocity message.
fn decode_airspeed(me: &[u8], subtype: u8) -> DecodeResult<Option<VelocityData>> {
    // Heading status + heading (1+10 bits)
    let heading_status = (me[1] >> 2) & 0x01;
    let heading_raw = (((me[1] as u16) & 0x03) << 8) | (me[2] as u16);

    // Airspeed type + airspeed (1+10 bits)
    let airspeed_type = (me[3] >> 7) & 0x01;
    let airspeed_raw = (((me[3] as u16) & 0x7F) << 3) | ((me[4] as u16) >> 5);

    if airspeed_raw == 0 {
        return Ok(None);
    }

    let scale = if subtype == 4 { 4 } else { 1 };
    let airspeed_knots = ((airspeed_raw) - 1) * scale;

    let heading_degrees = if heading_status != 0 {
        heading_raw as f64 * 360.0 / 1024.0
    } else {
        return Ok(None); // heading not available
    };

    Ok(Some(VelocityData::Airspeed {
        heading_degrees,
        airspeed_knots,
        airspeed_is_tas: airspeed_type != 0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_velocity_subtype1() {
        // Example: 8D485020994409940838175B284F
        // ME = 99 44 09 94 08 38 17
        let me = [0x99, 0x44, 0x09, 0x94, 0x08, 0x38, 0x17];
        let vel = decode(&me).unwrap();

        assert_eq!(vel.subtype, 1);
        assert!(vel.velocity.is_some());

        if let Some(VelocityData::GroundSpeed {
            speed_knots,
            heading_degrees,
            ..
        }) = &vel.velocity
        {
            // Expected: ~159 kts, heading ~183 deg
            assert!(*speed_knots > 100.0 && *speed_knots < 250.0);
            assert!(*heading_degrees > 150.0 && *heading_degrees < 220.0);
        } else {
            panic!("expected ground speed");
        }
    }

    #[test]
    fn test_decode_invalid_tc() {
        let me = [0xA0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // TC=20
        assert!(decode(&me).is_err());
    }
}
