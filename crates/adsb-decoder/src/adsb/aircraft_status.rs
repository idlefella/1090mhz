use crate::types::{DecodeError, DecodeResult, Squawk};

/// ADS-B Aircraft Status message (TC 28).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AircraftStatus {
    pub subtype: u8,
    pub status: StatusInfo,
}

/// Status information varies by subtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusInfo {
    /// Subtype 1: Emergency/priority status.
    Emergency {
        emergency_state: EmergencyState,
        squawk: Squawk,
    },
    /// Subtype 2: ACAS RA broadcast.
    AcasRa {
        /// 56-bit ACAS RA data.
        ara: u16,
        rac: u16,
        rat: bool,
        mte: bool,
    },
    /// Unknown or reserved subtype.
    Unknown(u8),
}

/// Emergency/priority state codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmergencyState {
    None,
    General,
    Medical,
    MinimumFuel,
    NoCommunication,
    UnlawfulInterference,
    DownedAircraft,
    Reserved,
}

impl EmergencyState {
    #[inline]
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => EmergencyState::None,
            1 => EmergencyState::General,
            2 => EmergencyState::Medical,
            3 => EmergencyState::MinimumFuel,
            4 => EmergencyState::NoCommunication,
            5 => EmergencyState::UnlawfulInterference,
            6 => EmergencyState::DownedAircraft,
            _ => EmergencyState::Reserved,
        }
    }
}

/// Decode aircraft status from the 7-byte ME field.
///
/// ME layout (56 bits):
///   [TC:5][ST:3][...subtype-dependent: 48 bits...]
pub fn decode(me: &[u8]) -> DecodeResult<AircraftStatus> {
    if me.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let tc = (me[0] >> 3) & 0x1F;
    if tc != 28 {
        return Err(DecodeError::InvalidTypeCode(tc));
    }

    let st = me[0] & 0x07;

    let status = match st {
        1 => {
            // Emergency/priority status
            let emergency_code = (me[1] >> 5) & 0x07;
            let squawk_raw = ((me[1] as u16 & 0x1F) << 8) | (me[2] as u16);
            StatusInfo::Emergency {
                emergency_state: EmergencyState::from_code(emergency_code),
                squawk: Squawk::from_identity_code(squawk_raw),
            }
        }
        2 => {
            // ACAS RA broadcast
            let ara = ((me[1] as u16) << 6) | ((me[2] as u16) >> 2);
            let rac = ((me[2] as u16 & 0x03) << 12)
                | ((me[3] as u16) << 4)
                | ((me[4] as u16) >> 4);
            let rat = (me[4] >> 3) & 0x01 != 0;
            let mte = (me[4] >> 2) & 0x01 != 0;
            StatusInfo::AcasRa {
                ara,
                rac,
                rat,
                mte,
            }
        }
        _ => StatusInfo::Unknown(st),
    };

    Ok(AircraftStatus {
        subtype: st,
        status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emergency_state() {
        assert_eq!(EmergencyState::from_code(0), EmergencyState::None);
        assert_eq!(EmergencyState::from_code(1), EmergencyState::General);
        assert_eq!(EmergencyState::from_code(7), EmergencyState::Reserved);
    }

    #[test]
    fn test_decode_emergency() {
        // TC=28, ST=1, emergency=0, squawk=7700 encoded
        let me = [0xE1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let status = decode(&me).unwrap();
        assert_eq!(status.subtype, 1);
    }
}
