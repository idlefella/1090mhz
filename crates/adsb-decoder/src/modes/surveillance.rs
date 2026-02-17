use crate::altitude;
use crate::types::{
    Altitude, DecodeError, DecodeResult, DownlinkRequest, FlightStatus, IcaoAddress, Squawk,
    UtilityMessage,
};

/// Surveillance reply (DF 4: altitude reply, DF 5: identity reply).
#[derive(Debug, Clone, PartialEq)]
pub struct SurveillanceReply {
    pub downlink_format: u8,
    pub flight_status: FlightStatus,
    pub downlink_request: DownlinkRequest,
    pub utility_message: UtilityMessage,
    pub altitude: Option<Altitude>,
    pub squawk: Option<Squawk>,
    pub icao: IcaoAddress,
}

/// Decode a DF 4 (surveillance altitude reply) from 7 raw bytes.
///
/// Layout (56 bits):
///   [DF:5][FS:3][DR:5][UM:6][AC:13][AP:24]
pub fn decode_altitude_reply(msg: &[u8], icao: IcaoAddress) -> DecodeResult<SurveillanceReply> {
    if msg.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let df = (msg[0] >> 3) & 0x1F;
    let fs = FlightStatus::from_code(msg[0] & 0x07);
    let dr = DownlinkRequest::from_code((msg[1] >> 3) & 0x1F);
    let um = UtilityMessage::from_code(((msg[1] & 0x07) << 3) | ((msg[2] >> 5) & 0x07));

    // 13-bit altitude code: bits from msg[2] (bottom 5 bits) + msg[3] (top 8 bits)
    let alt_code = ((msg[2] as u16 & 0x1F) << 8) | (msg[3] as u16);

    let altitude = if alt_code != 0 {
        altitude::decode_altitude_13(alt_code).ok()
    } else {
        None
    };

    Ok(SurveillanceReply {
        downlink_format: df,
        flight_status: fs,
        downlink_request: dr,
        utility_message: um,
        altitude,
        squawk: None,
        icao,
    })
}

/// Decode a DF 5 (surveillance identity reply) from 7 raw bytes.
///
/// Layout (56 bits):
///   [DF:5][FS:3][DR:5][UM:6][ID:13][AP:24]
pub fn decode_identity_reply(msg: &[u8], icao: IcaoAddress) -> DecodeResult<SurveillanceReply> {
    if msg.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let df = (msg[0] >> 3) & 0x1F;
    let fs = FlightStatus::from_code(msg[0] & 0x07);
    let dr = DownlinkRequest::from_code((msg[1] >> 3) & 0x1F);
    let um = UtilityMessage::from_code(((msg[1] & 0x07) << 3) | ((msg[2] >> 5) & 0x07));

    // 13-bit identity code
    let id_code = ((msg[2] as u16 & 0x1F) << 8) | (msg[3] as u16);
    let squawk = Squawk::from_identity_code(id_code);

    Ok(SurveillanceReply {
        downlink_format: df,
        flight_status: fs,
        downlink_request: dr,
        utility_message: um,
        altitude: None,
        squawk: Some(squawk),
        icao,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_altitude_reply() {
        // DF4 message with known altitude
        let msg = [0x20, 0x00, 0x1A, 0xC8, 0x00, 0x00, 0x00];
        let icao = IcaoAddress(0x123456);
        let reply = decode_altitude_reply(&msg, icao).unwrap();
        assert_eq!(reply.downlink_format, 4);
        assert_eq!(reply.flight_status, FlightStatus::AirborneNoAlert);
    }

    #[test]
    fn test_decode_identity_reply() {
        let msg = [0x28, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00];
        let icao = IcaoAddress(0x123456);
        let reply = decode_identity_reply(&msg, icao).unwrap();
        assert_eq!(reply.downlink_format, 5);
        assert!(reply.squawk.is_some());
    }
}
