pub mod aircraft_status;
pub mod airborne_position;
pub mod identification;
pub mod operation_status;
pub mod surface_position;
pub mod target_state;
pub mod velocity;

use crate::types::{DecodeError, DecodeResult};

/// Decoded ADS-B payload (ME field), dispatched by type code.
#[derive(Debug, Clone, PartialEq)]
pub enum AdsbPayload {
    Identification(identification::Identification),
    AirbornePosition(airborne_position::AirbornePosition),
    SurfacePosition(surface_position::SurfacePosition),
    AirborneVelocity(velocity::AirborneVelocity),
    AircraftStatus(aircraft_status::AircraftStatus),
    TargetState(target_state::TargetState),
    OperationStatus(operation_status::OperationStatus),
    /// Type code not handled or reserved.
    Unknown(u8),
}

/// Decode the 7-byte ME (message extended) field into an `AdsbPayload`.
/// Routes to the appropriate decoder based on the type code (TC, top 5 bits of ME).
pub fn decode_me(me: &[u8]) -> DecodeResult<AdsbPayload> {
    if me.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let tc = (me[0] >> 3) & 0x1F;

    match tc {
        1..=4 => Ok(AdsbPayload::Identification(identification::decode(me)?)),
        5..=8 => Ok(AdsbPayload::SurfacePosition(surface_position::decode(me)?)),
        9..=18 => Ok(AdsbPayload::AirbornePosition(
            airborne_position::decode(me)?,
        )),
        19 => Ok(AdsbPayload::AirborneVelocity(velocity::decode(me)?)),
        20..=22 => Ok(AdsbPayload::AirbornePosition(
            airborne_position::decode(me)?,
        )),
        28 => Ok(AdsbPayload::AircraftStatus(aircraft_status::decode(me)?)),
        29 => Ok(AdsbPayload::TargetState(target_state::decode(me)?)),
        31 => Ok(AdsbPayload::OperationStatus(operation_status::decode(me)?)),
        _ => Ok(AdsbPayload::Unknown(tc)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_identification() {
        // TC=4 identification message
        let me = [0x20, 0x2C, 0xC3, 0x71, 0xC3, 0x2C, 0xE0];
        let payload = decode_me(&me).unwrap();
        assert!(matches!(payload, AdsbPayload::Identification(_)));
    }

    #[test]
    fn test_dispatch_position() {
        // TC=11 airborne position
        let me = [0x58, 0xC3, 0x82, 0xD6, 0x90, 0xC8, 0xAC];
        let payload = decode_me(&me).unwrap();
        assert!(matches!(payload, AdsbPayload::AirbornePosition(_)));
    }

    #[test]
    fn test_dispatch_velocity() {
        // TC=19 velocity
        let me = [0x99, 0x44, 0x09, 0x94, 0x08, 0x38, 0x17];
        let payload = decode_me(&me).unwrap();
        assert!(matches!(payload, AdsbPayload::AirborneVelocity(_)));
    }

    #[test]
    fn test_dispatch_unknown() {
        // TC=0 -> unknown
        let me = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let payload = decode_me(&me).unwrap();
        assert!(matches!(payload, AdsbPayload::Unknown(0)));
    }
}
