use crate::altitude;
use crate::types::{
    Altitude, DecodeError, DecodeResult, IcaoAddress, ReplyInformation, SensitivityLevel,
    VerticalStatus,
};

/// ACAS short reply (DF 0).
#[derive(Debug, Clone, PartialEq)]
pub struct AcasShortReply {
    pub vertical_status: VerticalStatus,
    pub cross_link: bool,
    pub sensitivity_level: SensitivityLevel,
    pub reply_information: ReplyInformation,
    pub altitude: Option<Altitude>,
    pub icao: IcaoAddress,
}

/// ACAS long reply (DF 16).
#[derive(Debug, Clone, PartialEq)]
pub struct AcasLongReply {
    pub vertical_status: VerticalStatus,
    pub sensitivity_level: SensitivityLevel,
    pub reply_information: ReplyInformation,
    pub altitude: Option<Altitude>,
    pub mv_field: [u8; 7],
    pub icao: IcaoAddress,
}

/// Decode a DF 0 (short air-air ACAS) from 7 raw bytes.
///
/// Layout (56 bits):
///   [DF:5][VS:1][CC:1][unused:1][SL:3][unused:2][RI:4][unused:2][AC:13][AP:24]
pub fn decode_short(msg: &[u8], icao: IcaoAddress) -> DecodeResult<AcasShortReply> {
    if msg.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let vs = if (msg[0] >> 2) & 0x01 != 0 {
        VerticalStatus::Ground
    } else {
        VerticalStatus::Airborne
    };
    let cc = (msg[0] >> 1) & 0x01 != 0;
    let sl = SensitivityLevel((msg[1] >> 5) & 0x07);
    let ri = ReplyInformation(((msg[1] >> 1) & 0x0F) as u8);

    // 13-bit altitude code
    let alt_code = ((msg[2] as u16 & 0x1F) << 8) | (msg[3] as u16);
    let altitude = if alt_code != 0 {
        altitude::decode_altitude_13(alt_code).ok()
    } else {
        None
    };

    Ok(AcasShortReply {
        vertical_status: vs,
        cross_link: cc,
        sensitivity_level: sl,
        reply_information: ri,
        altitude,
        icao,
    })
}

/// Decode a DF 16 (long air-air ACAS) from 14 raw bytes.
///
/// Layout (112 bits):
///   [DF:5][VS:1][unused:2][SL:3][unused:2][RI:4][unused:2][AC:13][MV:56][AP:24]
pub fn decode_long(msg: &[u8], icao: IcaoAddress) -> DecodeResult<AcasLongReply> {
    if msg.len() < 14 {
        return Err(DecodeError::TooShort);
    }

    let vs = if (msg[0] >> 2) & 0x01 != 0 {
        VerticalStatus::Ground
    } else {
        VerticalStatus::Airborne
    };
    let sl = SensitivityLevel((msg[1] >> 5) & 0x07);
    let ri = ReplyInformation(((msg[1] >> 1) & 0x0F) as u8);

    // 13-bit altitude code
    let alt_code = ((msg[2] as u16 & 0x1F) << 8) | (msg[3] as u16);
    let altitude = if alt_code != 0 {
        altitude::decode_altitude_13(alt_code).ok()
    } else {
        None
    };

    // MV field: 7 bytes starting at byte 4
    let mut mv_field = [0u8; 7];
    mv_field.copy_from_slice(&msg[4..11]);

    Ok(AcasLongReply {
        vertical_status: vs,
        sensitivity_level: sl,
        reply_information: ri,
        altitude,
        mv_field,
        icao,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_df0() {
        // DF=0 (0x02 >> 3 = 0), VS=airborne
        let msg = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let icao = IcaoAddress(0x123456);
        let reply = decode_short(&msg, icao).unwrap();
        assert_eq!(reply.vertical_status, VerticalStatus::Airborne);
    }

    #[test]
    fn test_decode_df16() {
        let msg = [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let icao = IcaoAddress(0x123456);
        let reply = decode_long(&msg, icao).unwrap();
        assert_eq!(reply.vertical_status, VerticalStatus::Airborne);
    }
}
