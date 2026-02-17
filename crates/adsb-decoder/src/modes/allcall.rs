use crate::types::{Capability, DecodeError, DecodeResult, IcaoAddress};

/// All-call reply (DF 11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllCallReply {
    pub capability: Capability,
    pub icao: IcaoAddress,
    pub parity_interrogator: u32,
}

/// Decode a DF 11 (all-call reply) from 7 raw bytes.
///
/// Layout (56 bits):
///   [DF:5][CA:3][AA:24][PI:24]
///
/// AA = ICAO address, PI = parity/interrogator identifier.
/// For DF 11, the CRC is overlaid with the interrogator ID.
pub fn decode(msg: &[u8]) -> DecodeResult<AllCallReply> {
    if msg.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let ca = Capability::from_code(msg[0] & 0x07);
    let icao = IcaoAddress::from_bytes(&msg[1..4]);

    // Parity/interrogator identifier (last 3 bytes)
    let pi = ((msg[4] as u32) << 16) | ((msg[5] as u32) << 8) | (msg[6] as u32);

    Ok(AllCallReply {
        capability: ca,
        icao,
        parity_interrogator: pi,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_allcall() {
        // DF=11 (0x5D = 01011_101), CA=5, ICAO=4B1645
        let msg = [0x5D, 0x4B, 0x16, 0x45, 0x00, 0x00, 0x00];
        let reply = decode(&msg).unwrap();
        assert_eq!(reply.icao, IcaoAddress(0x4B1645));
        assert_eq!(
            reply.capability,
            Capability::Level2WithCommBCapability
        );
    }
}
