use crate::altitude;
use crate::types::{
    Altitude, AltitudeSource, CprPosition, DecodeError, DecodeResult, SurveillanceStatus,
};

/// ADS-B Airborne Position message (TC 9-18 barometric, TC 20-22 GNSS).
#[derive(Debug, Clone, PartialEq)]
pub struct AirbornePosition {
    pub type_code: u8,
    pub surveillance_status: SurveillanceStatus,
    pub single_antenna: bool,
    pub altitude: Option<Altitude>,
    pub utc_synchronized: bool,
    pub cpr: CprPosition,
}

/// Decode an airborne position message from the 7-byte ME field.
///
/// ME layout (56 bits):
///   [TC:5][SS:2][SAF:1][ALT:12][T:1][F:1][LAT_CPR:17][LON_CPR:17]
///
/// TC 9-18: barometric altitude
/// TC 20-22: GNSS altitude
pub fn decode(me: &[u8]) -> DecodeResult<AirbornePosition> {
    if me.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let tc = (me[0] >> 3) & 0x1F;

    // Validate type code range
    if !((9..=18).contains(&tc) || (20..=22).contains(&tc)) {
        return Err(DecodeError::InvalidTypeCode(tc));
    }

    let ss = SurveillanceStatus::from_code((me[0] >> 1) & 0x03);
    let saf = (me[0] & 0x01) != 0;

    // 12-bit altitude code: bits from me[1] (8 bits) + me[2] top 4 bits
    let alt_code = ((me[1] as u16) << 4) | ((me[2] as u16) >> 4);

    let altitude_source = if tc <= 18 {
        AltitudeSource::Barometric
    } else {
        AltitudeSource::Gnss
    };

    let altitude = if alt_code != 0 {
        match altitude::decode_altitude_12(alt_code) {
            Ok(mut alt) => {
                alt.source = altitude_source;
                Some(alt)
            }
            Err(_) => None,
        }
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

    Ok(AirbornePosition {
        type_code: tc,
        surveillance_status: ss,
        single_antenna: saf,
        altitude,
        utc_synchronized: t_flag != 0,
        cpr: CprPosition {
            lat_cpr,
            lon_cpr,
            odd_flag: f_flag != 0,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_position_even() {
        // Example from "The 1090MHz Riddle"
        // 8D40621D58C382D690C8AC2863A7
        // ME = 58 C3 82 D6 90 C8 AC
        let me = [0x58, 0xC3, 0x82, 0xD6, 0x90, 0xC8, 0xAC];
        let pos = decode(&me).unwrap();

        assert_eq!(pos.type_code, 11); // 0x58 >> 3 = 11
        assert!(!pos.cpr.odd_flag); // even frame
        assert!(pos.altitude.is_some());
    }

    #[test]
    fn test_decode_position_odd() {
        // 8D40621D58C386435CC412692AD6
        // ME = 58 C3 86 43 5C C4 12
        let me = [0x58, 0xC3, 0x86, 0x43, 0x5C, 0xC4, 0x12];
        let pos = decode(&me).unwrap();

        assert_eq!(pos.type_code, 11);
        assert!(pos.cpr.odd_flag); // odd frame
        assert!(pos.altitude.is_some());
    }

    #[test]
    fn test_decode_invalid_tc() {
        let me = [0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // TC=8
        assert!(decode(&me).is_err());
    }
}
