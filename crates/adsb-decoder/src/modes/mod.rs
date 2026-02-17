pub mod acas;
pub mod allcall;
pub mod bds;
pub mod surveillance;

use crate::altitude;
use crate::types::{
    Altitude, DecodeError, DecodeResult, DownlinkRequest, FlightStatus, IcaoAddress, Squawk,
    UtilityMessage,
};

/// Comm-B reply message (DF 20: altitude, DF 21: identity).
#[derive(Debug, Clone, PartialEq)]
pub struct CommBReply {
    pub downlink_format: u8,
    pub flight_status: FlightStatus,
    pub downlink_request: DownlinkRequest,
    pub utility_message: UtilityMessage,
    pub altitude: Option<Altitude>,
    pub squawk: Option<Squawk>,
    /// Raw 7-byte MB (message, Comm-B) field.
    pub mb_data: [u8; 7],
    /// Decoded BDS payload, if BDS register is known.
    pub bds: Option<bds::BdsPayload>,
    pub icao: IcaoAddress,
}

/// Decode a DF 20 (Comm-B with altitude) from 14 raw bytes.
///
/// Layout (112 bits):
///   [DF:5][FS:3][DR:5][UM:6][AC:13][MB:56][AP:24]
pub fn decode_comm_b_altitude(msg: &[u8], icao: IcaoAddress) -> DecodeResult<CommBReply> {
    if msg.len() < 14 {
        return Err(DecodeError::TooShort);
    }

    let df = (msg[0] >> 3) & 0x1F;
    let fs = FlightStatus::from_code(msg[0] & 0x07);
    let dr = DownlinkRequest::from_code((msg[1] >> 3) & 0x1F);
    let um = UtilityMessage::from_code(((msg[1] & 0x07) << 3) | ((msg[2] >> 5) & 0x07));

    // 13-bit altitude code
    let alt_code = ((msg[2] as u16 & 0x1F) << 8) | (msg[3] as u16);
    let altitude = if alt_code != 0 {
        altitude::decode_altitude_13(alt_code).ok()
    } else {
        None
    };

    // MB field: bytes 4-10
    let mut mb_data = [0u8; 7];
    mb_data.copy_from_slice(&msg[4..11]);

    Ok(CommBReply {
        downlink_format: df,
        flight_status: fs,
        downlink_request: dr,
        utility_message: um,
        altitude,
        squawk: None,
        mb_data,
        bds: None,
        icao,
    })
}

/// Decode a DF 21 (Comm-B with identity) from 14 raw bytes.
///
/// Layout (112 bits):
///   [DF:5][FS:3][DR:5][UM:6][ID:13][MB:56][AP:24]
pub fn decode_comm_b_identity(msg: &[u8], icao: IcaoAddress) -> DecodeResult<CommBReply> {
    if msg.len() < 14 {
        return Err(DecodeError::TooShort);
    }

    let df = (msg[0] >> 3) & 0x1F;
    let fs = FlightStatus::from_code(msg[0] & 0x07);
    let dr = DownlinkRequest::from_code((msg[1] >> 3) & 0x1F);
    let um = UtilityMessage::from_code(((msg[1] & 0x07) << 3) | ((msg[2] >> 5) & 0x07));

    // 13-bit identity code
    let id_code = ((msg[2] as u16 & 0x1F) << 8) | (msg[3] as u16);
    let squawk = Squawk::from_identity_code(id_code);

    // MB field: bytes 4-10
    let mut mb_data = [0u8; 7];
    mb_data.copy_from_slice(&msg[4..11]);

    Ok(CommBReply {
        downlink_format: df,
        flight_status: fs,
        downlink_request: dr,
        utility_message: um,
        altitude: None,
        squawk: Some(squawk),
        mb_data,
        bds: None,
        icao,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_comm_b_altitude() {
        let msg = [0xA0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let icao = IcaoAddress(0x123456);
        let reply = decode_comm_b_altitude(&msg, icao).unwrap();
        assert_eq!(reply.downlink_format, 20);
    }

    #[test]
    fn test_decode_comm_b_identity() {
        let msg = [0xA8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let icao = IcaoAddress(0x123456);
        let reply = decode_comm_b_identity(&msg, icao).unwrap();
        assert_eq!(reply.downlink_format, 21);
    }
}
