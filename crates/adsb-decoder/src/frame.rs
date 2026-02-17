use crate::adsb::{self, AdsbPayload};
use crate::crc;
use crate::modes::{self, acas, allcall, surveillance, CommBReply};
use crate::types::{
    Capability, ControlField, DecodeError, DecodeResult, IcaoAddress,
};

/// Top-level decoded Mode-S message.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// DF 0: Short air-air ACAS surveillance.
    AcasSurveillance(acas::AcasShortReply),
    /// DF 4: Surveillance altitude reply.
    SurveillanceAltitude(surveillance::SurveillanceReply),
    /// DF 5: Surveillance identity reply.
    SurveillanceIdentity(surveillance::SurveillanceReply),
    /// DF 11: All-call reply.
    AllCallReply(allcall::AllCallReply),
    /// DF 16: Long air-air ACAS coordination.
    AcasCoordination(acas::AcasLongReply),
    /// DF 17: ADS-B extended squitter.
    AdsbMessage(AdsbMessage),
    /// DF 18: TIS-B / ADS-R extended squitter.
    TisbMessage(AdsbMessage),
    /// DF 20: Comm-B with altitude.
    CommBAltitude(CommBReply),
    /// DF 21: Comm-B with identity.
    CommBIdentity(CommBReply),
}

/// ADS-B / TIS-B message wrapper.
#[derive(Debug, Clone, PartialEq)]
pub struct AdsbMessage {
    pub downlink_format: u8,
    pub capability: Capability,
    pub icao: IcaoAddress,
    pub payload: AdsbPayload,
    pub control_field: Option<ControlField>,
}

/// Decode a raw Mode-S message from bytes.
///
/// Accepts either 7 bytes (56-bit short messages: DF 0, 4, 5, 11)
/// or 14 bytes (112-bit long messages: DF 16, 17, 18, 20, 21).
///
/// The input `msg` must include the 3-byte parity/AP field at the end.
pub fn decode(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let df = (msg[0] >> 3) & 0x1F;

    match df {
        0 => decode_df0(msg),
        4 => decode_df4(msg),
        5 => decode_df5(msg),
        11 => decode_df11(msg),
        16 => decode_df16(msg),
        17 => decode_df17(msg),
        18 => decode_df18(msg),
        20 => decode_df20(msg),
        21 => decode_df21(msg),
        _ => Err(DecodeError::InvalidDownlinkFormat(df)),
    }
}

/// Extract the downlink format from the first byte.
#[inline]
pub fn downlink_format(msg: &[u8]) -> u8 {
    (msg[0] >> 3) & 0x1F
}

/// Determine expected message length based on DF.
#[inline]
pub fn expected_length(df: u8) -> usize {
    match df {
        0 | 4 | 5 | 11 => 7,
        16 | 17 | 18 | 20 | 21 => 14,
        _ => 14, // assume long for unknown
    }
}

fn decode_df0(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 7 {
        return Err(DecodeError::TooShort);
    }
    let icao = IcaoAddress(crc::recover_icao(msg));
    Ok(Message::AcasSurveillance(acas::decode_short(msg, icao)?))
}

fn decode_df4(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 7 {
        return Err(DecodeError::TooShort);
    }
    let icao = IcaoAddress(crc::recover_icao(msg));
    Ok(Message::SurveillanceAltitude(
        surveillance::decode_altitude_reply(msg, icao)?,
    ))
}

fn decode_df5(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 7 {
        return Err(DecodeError::TooShort);
    }
    let icao = IcaoAddress(crc::recover_icao(msg));
    Ok(Message::SurveillanceIdentity(
        surveillance::decode_identity_reply(msg, icao)?,
    ))
}

fn decode_df11(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 7 {
        return Err(DecodeError::TooShort);
    }
    Ok(Message::AllCallReply(allcall::decode(msg)?))
}

fn decode_df16(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 14 {
        return Err(DecodeError::TooShort);
    }
    let icao = IcaoAddress(crc::recover_icao(msg));
    Ok(Message::AcasCoordination(acas::decode_long(msg, icao)?))
}

fn decode_df17(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 14 {
        return Err(DecodeError::TooShort);
    }

    // Verify CRC for DF 17
    if !crc::verify_adsb_crc(msg) {
        let remainder = crc::crc_remainder(msg);
        return Err(DecodeError::CrcFailed {
            expected: 0,
            computed: remainder,
        });
    }

    let ca = Capability::from_code(msg[0] & 0x07);
    let icao = IcaoAddress::from_bytes(&msg[1..4]);

    // ME field: bytes 4-10 (7 bytes)
    let me = &msg[4..11];
    let payload = adsb::decode_me(me)?;

    Ok(Message::AdsbMessage(AdsbMessage {
        downlink_format: 17,
        capability: ca,
        icao,
        payload,
        control_field: None,
    }))
}

fn decode_df18(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 14 {
        return Err(DecodeError::TooShort);
    }

    // Verify CRC for DF 18
    if !crc::verify_adsb_crc(msg) {
        let remainder = crc::crc_remainder(msg);
        return Err(DecodeError::CrcFailed {
            expected: 0,
            computed: remainder,
        });
    }

    let cf = ControlField::from_code(msg[0] & 0x07);
    let icao = IcaoAddress::from_bytes(&msg[1..4]);

    // ME field: bytes 4-10 (7 bytes)
    let me = &msg[4..11];
    let payload = adsb::decode_me(me)?;

    Ok(Message::TisbMessage(AdsbMessage {
        downlink_format: 18,
        capability: Capability::Level1, // CF is used instead of CA for DF18
        icao,
        payload,
        control_field: Some(cf),
    }))
}

fn decode_df20(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 14 {
        return Err(DecodeError::TooShort);
    }
    let icao = IcaoAddress(crc::recover_icao(msg));
    Ok(Message::CommBAltitude(
        modes::decode_comm_b_altitude(msg, icao)?,
    ))
}

fn decode_df21(msg: &[u8]) -> DecodeResult<Message> {
    if msg.len() < 14 {
        return Err(DecodeError::TooShort);
    }
    let icao = IcaoAddress(crc::recover_icao(msg));
    Ok(Message::CommBIdentity(
        modes::decode_comm_b_identity(msg, icao)?,
    ))
}

/// Parse a hex string into bytes and decode the message.
pub fn decode_hex(hex: &str) -> DecodeResult<Message> {
    let bytes = hex_to_bytes(hex)?;
    decode(&bytes)
}

/// Convert a hex string to bytes.
fn hex_to_bytes(hex: &str) -> DecodeResult<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(DecodeError::InvalidData("odd-length hex string"));
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|_| DecodeError::InvalidData("invalid hex character"))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_df17_identification() {
        // Known DF17 identification message
        let msg = decode_hex("8D4840D6202CC371C32CE0576098").unwrap();
        match msg {
            Message::AdsbMessage(adsb) => {
                assert_eq!(adsb.downlink_format, 17);
                assert_eq!(adsb.icao, IcaoAddress(0x4840D6));
                match &adsb.payload {
                    AdsbPayload::Identification(id) => {
                        assert_eq!(id.callsign_str(), "KLM1023");
                    }
                    _ => panic!("expected identification payload"),
                }
            }
            _ => panic!("expected ADS-B message"),
        }
    }

    #[test]
    fn test_decode_df17_velocity() {
        let msg = decode_hex("8D485020994409940838175B284F").unwrap();
        match msg {
            Message::AdsbMessage(adsb) => {
                assert_eq!(adsb.icao, IcaoAddress(0x485020));
                assert!(matches!(
                    adsb.payload,
                    AdsbPayload::AirborneVelocity(_)
                ));
            }
            _ => panic!("expected ADS-B message"),
        }
    }

    #[test]
    fn test_decode_df17_position() {
        let msg = decode_hex("8D40621D58C382D690C8AC2863A7").unwrap();
        match msg {
            Message::AdsbMessage(adsb) => {
                assert_eq!(adsb.icao, IcaoAddress(0x40621D));
                assert!(matches!(
                    adsb.payload,
                    AdsbPayload::AirbornePosition(_)
                ));
            }
            _ => panic!("expected ADS-B message"),
        }
    }

    #[test]
    fn test_decode_df11() {
        // DF=11 all-call reply
        let msg = decode_hex("5D4B164513B842").unwrap();
        match msg {
            Message::AllCallReply(reply) => {
                assert_eq!(reply.icao, IcaoAddress(0x4B1645));
            }
            _ => panic!("expected all-call reply"),
        }
    }

    #[test]
    fn test_decode_hex_invalid() {
        assert!(decode_hex("ZZ").is_err());
        assert!(decode_hex("8D4").is_err()); // odd length
    }

    #[test]
    fn test_downlink_format_extraction() {
        assert_eq!(downlink_format(&[0x8D]), 17); // 10001_101
        assert_eq!(downlink_format(&[0x5D]), 11); // 01011_101
        assert_eq!(downlink_format(&[0x20]), 4); // 00100_000
        assert_eq!(downlink_format(&[0x28]), 5); // 00101_000
        assert_eq!(downlink_format(&[0xA0]), 20); // 10100_000
        assert_eq!(downlink_format(&[0xA8]), 21); // 10101_000
    }

    #[test]
    fn test_expected_length() {
        assert_eq!(expected_length(0), 7);
        assert_eq!(expected_length(4), 7);
        assert_eq!(expected_length(5), 7);
        assert_eq!(expected_length(11), 7);
        assert_eq!(expected_length(16), 14);
        assert_eq!(expected_length(17), 14);
        assert_eq!(expected_length(18), 14);
        assert_eq!(expected_length(20), 14);
        assert_eq!(expected_length(21), 14);
    }
}
