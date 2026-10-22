use crate::types::{CprPosition, DecodeError, DecodeResult, Position};

/// Number of CPR encoding bits.
const CPR_BITS: u32 = 17;
const CPR_MAX: f64 = (1 << CPR_BITS) as f64; // 131072.0

/// dLat for even messages (NZ = 15, so 60 latitude zones).
const DLAT_EVEN: f64 = 360.0 / 60.0; // 6.0 degrees
/// dLat for odd messages (59 latitude zones).
const DLAT_ODD: f64 = 360.0 / 59.0;

/// Number of Longitude Zones lookup table.
/// NL(lat) returns the number of longitude zones for a given latitude.
/// This is based on the NL function defined in the ADS-B specification.
///
/// Transition latitudes (in degrees) where NL changes value.
/// NL(lat) = max NL value for which lat < transition_lat[NL].
const NL_TABLE: [(f64, u32); 59] = [
    (10.47047130, 59),
    (14.82817437, 58),
    (18.18626357, 57),
    (21.02939493, 56),
    (23.54504487, 55),
    (25.82924707, 54),
    (27.93898710, 53),
    (29.91135686, 52),
    (31.77209708, 51),
    (33.53993436, 50),
    (35.22899598, 49),
    (36.85025108, 48),
    (38.41241892, 47),
    (39.92256684, 46),
    (41.38651832, 45),
    (42.80914012, 44),
    (44.19454951, 43),
    (45.54626723, 42),
    (46.86733252, 41),
    (48.16039128, 40),
    (49.42776439, 39),
    (50.67150166, 38),
    (51.89342469, 37),
    (53.09516153, 36),
    (54.27817472, 35),
    (55.44378444, 34),
    (56.59318756, 33),
    (57.72747354, 32),
    (58.84763776, 31),
    (59.95459277, 30),
    (61.04917774, 29),
    (62.13216659, 28),
    (63.20427479, 27),
    (64.26616523, 26),
    (65.31845310, 25),
    (66.36171008, 24),
    (67.39646774, 23),
    (68.42322022, 22),
    (69.44242631, 21),
    (70.45451075, 20),
    (71.45986473, 19),
    (72.45884545, 18),
    (73.45177442, 17),
    (74.43893416, 16),
    (75.42056257, 15),
    (76.39684391, 14),
    (77.36789461, 13),
    (78.33374083, 12),
    (79.29428225, 11),
    (80.24923213, 10),
    (81.19801349, 9),
    (82.13956981, 8),
    (83.07199445, 7),
    (83.99173563, 6),
    (84.89166191, 5),
    (85.75541621, 4),
    (86.53536998, 3),
    (87.00000000, 2),
    (90.00000000, 1),
];

/// Compute the Number of Longitude zones for a given latitude.
#[inline]
pub fn nl(lat: f64) -> u32 {
    let abs_lat = lat.abs();
    if abs_lat >= 87.0 {
        return 1;
    }
    for &(transition, zones) in &NL_TABLE {
        if abs_lat < transition {
            return zones;
        }
    }
    1
}

/// Globally unambiguous CPR decoding.
/// Requires a matching pair of even and odd position reports.
/// `t_even` and `t_odd` are used to determine which message is more recent.
/// If `most_recent_odd` is true, the odd message is the more recent one.
pub fn decode_global(
    even: &CprPosition,
    odd: &CprPosition,
    most_recent_odd: bool,
) -> DecodeResult<Position> {
    let lat_cpr_even = even.lat_cpr as f64 / CPR_MAX;
    let lon_cpr_even = even.lon_cpr as f64 / CPR_MAX;
    let lat_cpr_odd = odd.lat_cpr as f64 / CPR_MAX;
    let lon_cpr_odd = odd.lon_cpr as f64 / CPR_MAX;

    // Compute latitude index j
    let j = (59.0 * lat_cpr_even - 60.0 * lat_cpr_odd + 0.5).floor();

    // Compute even and odd latitudes
    let mut lat_even = DLAT_EVEN * (modulus(j as i32, 60) as f64 + lat_cpr_even);
    let mut lat_odd = DLAT_ODD * (modulus(j as i32, 59) as f64 + lat_cpr_odd);

    // Adjust to [-90, 90] range
    if lat_even >= 270.0 {
        lat_even -= 360.0;
    }
    if lat_odd >= 270.0 {
        lat_odd -= 360.0;
    }

    // Check that both latitudes are in the same NL zone
    let nl_even = nl(lat_even);
    let nl_odd = nl(lat_odd);
    if nl_even != nl_odd {
        return Err(DecodeError::InvalidData(
            "CPR latitude zone mismatch between even and odd",
        ));
    }

    // Compute longitude
    let (lat, lon) = if most_recent_odd {
        // Use odd message
        let nl_val = nl(lat_odd);
        let n_i = if nl_val > 1 { nl_val - 1 } else { 1 };
        let dlon = 360.0 / n_i as f64;
        let m = (lon_cpr_even * (nl_val as f64 - 1.0) - lon_cpr_odd * nl_val as f64 + 0.5)
            .floor();
        let lon = dlon * (modulus(m as i32, n_i as i32) as f64 + lon_cpr_odd);
        (lat_odd, lon)
    } else {
        // Use even message
        let nl_val = nl(lat_even);
        let n_i = if nl_val > 1 { nl_val } else { 1 };
        let dlon = 360.0 / n_i as f64;
        let m = (lon_cpr_even * (nl_val as f64 - 1.0) - lon_cpr_odd * nl_val as f64 + 0.5)
            .floor();
        let lon = dlon * (modulus(m as i32, n_i as i32) as f64 + lon_cpr_even);
        (lat_even, lon)
    };

    let lon = if lon >= 180.0 { lon - 360.0 } else { lon };

    Ok(Position {
        latitude: lat,
        longitude: lon,
    })
}

/// Locally unambiguous CPR decoding.
/// Requires a single CPR message and a reference position (e.g., from a previous decode).
pub fn decode_local(cpr: &CprPosition, reference: &Position) -> DecodeResult<Position> {
    let dlat = if cpr.odd_flag { DLAT_ODD } else { DLAT_EVEN };

    let lat_cpr = cpr.lat_cpr as f64 / CPR_MAX;
    let lon_cpr = cpr.lon_cpr as f64 / CPR_MAX;

    // Compute latitude
    let j = (reference.latitude / dlat).floor() + ((reference.latitude % dlat) / dlat - lat_cpr + 0.5).floor();
    let lat = dlat * (j + lat_cpr);

    // Check latitude range
    if lat < -90.0 || lat > 90.0 {
        return Err(DecodeError::InvalidData("decoded latitude out of range"));
    }

    // Compute longitude
    let nl_val = nl(lat);
    let n_i = if cpr.odd_flag {
        if nl_val > 1 { nl_val - 1 } else { 1 }
    } else {
        if nl_val > 1 { nl_val } else { 1 }
    };

    let dlon = 360.0 / n_i as f64;
    let m = (reference.longitude / dlon).floor()
        + ((reference.longitude % dlon) / dlon - lon_cpr + 0.5).floor();
    let lon = dlon * (m + lon_cpr);

    let lon = if lon >= 180.0 {
        lon - 360.0
    } else if lon < -180.0 {
        lon + 360.0
    } else {
        lon
    };

    Ok(Position {
        latitude: lat,
        longitude: lon,
    })
}

/// Modulus that always returns a non-negative result.
#[inline]
fn modulus(a: i32, b: i32) -> i32 {
    ((a % b) + b) % b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nl_function() {
        assert_eq!(nl(0.0), 59);
        assert_eq!(nl(86.9), 2);
        assert_eq!(nl(87.0), 1); // exactly 87.0 -> NL=1
        assert_eq!(nl(87.1), 1);
        assert_eq!(nl(-87.1), 1);
        assert_eq!(nl(89.0), 1);
        assert_eq!(nl(45.0), 42); // 45.0 < 45.546 -> NL=42
    }

    #[test]
    fn test_global_decode() {
        // Test vectors from "The 1090 Megahertz Riddle" book
        // Even message: lat_cpr = 93000, lon_cpr = 51372
        // Odd message:  lat_cpr = 74158, lon_cpr = 50194
        let even = CprPosition {
            lat_cpr: 93000,
            lon_cpr: 51372,
            odd_flag: false,
        };
        let odd = CprPosition {
            lat_cpr: 74158,
            lon_cpr: 50194,
            odd_flag: true,
        };

        let pos = decode_global(&even, &odd, false).unwrap();

        // Expected: approximately lat=52.26, lon=3.92
        assert!((pos.latitude - 52.2572).abs() < 0.01);
        assert!((pos.longitude - 3.9194).abs() < 0.01);
    }

    #[test]
    fn test_global_decode_odd_recent() {
        let even = CprPosition {
            lat_cpr: 93000,
            lon_cpr: 51372,
            odd_flag: false,
        };
        let odd = CprPosition {
            lat_cpr: 74158,
            lon_cpr: 50194,
            odd_flag: true,
        };

        let pos = decode_global(&even, &odd, true).unwrap();

        assert!((pos.latitude - 52.2658).abs() < 0.01);
        assert!((pos.longitude - 3.9389).abs() < 0.01);
    }

    #[test]
    fn test_local_decode() {
        let reference = Position {
            latitude: 52.258,
            longitude: 3.918,
        };

        let cpr = CprPosition {
            lat_cpr: 93000,
            lon_cpr: 51372,
            odd_flag: false,
        };

        let pos = decode_local(&cpr, &reference).unwrap();
        assert!((pos.latitude - 52.2572).abs() < 0.01);
        assert!((pos.longitude - 3.9194).abs() < 0.01);
    }

    #[test]
    fn test_modulus() {
        assert_eq!(modulus(7, 3), 1);
        assert_eq!(modulus(-1, 60), 59);
        assert_eq!(modulus(0, 60), 0);
    }
}
