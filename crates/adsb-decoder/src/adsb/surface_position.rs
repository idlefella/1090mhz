use crate::types::{CprPosition, DecodeError, DecodeResult, GroundSpeed};

/// ADS-B Surface Position message (TC 5-8).
#[derive(Debug, Clone, PartialEq)]
pub struct SurfacePosition {
    pub type_code: u8,
    pub ground_speed: GroundSpeed,
    pub heading: Option<f64>,
    pub heading_valid: bool,
    pub utc_synchronized: bool,
    pub cpr: CprPosition,
}

/// Decode a surface position message from the 7-byte ME field.
///
/// ME layout (56 bits):
///   [TC:5][MOV:7][S:1][TRK:7][T:1][F:1][LAT_CPR:17][LON_CPR:17]
pub fn decode(me: &[u8]) -> DecodeResult<SurfacePosition> {
    if me.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let tc = (me[0] >> 3) & 0x1F;
    if !(5..=8).contains(&tc) {
        return Err(DecodeError::InvalidTypeCode(tc));
    }

    // Movement field (7 bits): me[0] bits 2-0 + me[1] bits 7-4
    let mov = ((me[0] as u16 & 0x07) << 4) | ((me[1] as u16) >> 4);

    let ground_speed = decode_movement(mov);

    // Status bit for heading
    let heading_status = (me[1] >> 3) & 0x01;

    // Ground track / heading (7 bits)
    let trk_raw = ((me[1] as u16 & 0x07) << 4) | ((me[2] as u16) >> 4);

    let heading = if heading_status != 0 {
        Some(trk_raw as f64 * 360.0 / 128.0)
    } else {
        None
    };

    let t_flag = (me[2] >> 3) & 0x01;
    let f_flag = (me[2] >> 2) & 0x01;

    // 17-bit CPR latitude
    let lat_cpr = (((me[2] as u32) & 0x03) << 15)
        | ((me[3] as u32) << 7)
        | ((me[4] as u32) >> 1);

    // 17-bit CPR longitude
    let lon_cpr =
        (((me[4] as u32) & 0x01) << 16) | ((me[5] as u32) << 8) | (me[6] as u32);

    Ok(SurfacePosition {
        type_code: tc,
        ground_speed,
        heading,
        heading_valid: heading_status != 0,
        utc_synchronized: t_flag != 0,
        cpr: CprPosition {
            lat_cpr,
            lon_cpr,
            odd_flag: f_flag != 0,
        },
    })
}

/// Decode movement field to ground speed.
fn decode_movement(mov: u16) -> GroundSpeed {
    match mov {
        0 => GroundSpeed::Stopped,
        1 => GroundSpeed::Stopped,
        2..=8 => GroundSpeed::Knots(0.125 + (mov as f64 - 2.0) * 0.125),
        9..=12 => GroundSpeed::Knots(1.0 + (mov as f64 - 9.0) * 0.25),
        13..=38 => GroundSpeed::Knots(2.0 + (mov as f64 - 13.0) * 0.5),
        39..=93 => GroundSpeed::Knots(15.0 + (mov as f64 - 39.0) * 1.0),
        94..=108 => GroundSpeed::Knots(70.0 + (mov as f64 - 94.0) * 2.0),
        109..=123 => GroundSpeed::Knots(100.0 + (mov as f64 - 109.0) * 5.0),
        124 => GroundSpeed::GreaterThan175,
        _ => GroundSpeed::Stopped, // reserved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_decode() {
        assert!(matches!(decode_movement(0), GroundSpeed::Stopped));
        assert!(matches!(decode_movement(1), GroundSpeed::Stopped));
        assert!(matches!(decode_movement(124), GroundSpeed::GreaterThan175));

        if let GroundSpeed::Knots(speed) = decode_movement(12) {
            assert!((speed - 1.75).abs() < 0.01);
        } else {
            panic!("expected knots");
        }
    }

    #[test]
    fn test_decode_surface() {
        // Construct a surface position ME: TC=5
        let me = [0x28, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00]; // TC=5, minimal
        let pos = decode(&me).unwrap();
        assert_eq!(pos.type_code, 5);
    }
}
