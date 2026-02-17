use crate::types::{DecodeError, DecodeResult};

/// ADS-B Aircraft Operation Status message (TC 31).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationStatus {
    pub subtype: u8,
    pub data: OperationStatusData,
}

/// Operation status data varies by subtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationStatusData {
    /// Subtype 0: airborne operational status.
    Airborne {
        capability_class: AirborneCapability,
        operational_mode: OperationalMode,
        version: u8,
        nic_supplement_a: bool,
        nac_p: u8,
        gva: u8,
        sil: u8,
        nic_baro: bool,
        sil_supplement: bool,
        hrd: HeadingReference,
    },
    /// Subtype 1: surface operational status.
    Surface {
        capability_class: SurfaceCapability,
        operational_mode: OperationalMode,
        version: u8,
        nic_supplement_a: bool,
        nac_p: u8,
        sil: u8,
        sil_supplement: bool,
        trk_hdg: TrackHeadingInfo,
        hrd: HeadingReference,
    },
    Unknown(u8),
}

/// Airborne capability class codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AirborneCapability {
    pub tcas_operational: bool,
    pub adsb_1090_in: bool,
    pub adsb_uat_in: bool,
    pub arv: bool,
    pub ts: bool,
    pub tc: u8,
}

/// Surface capability class codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceCapability {
    pub poa: bool,
    pub cdti: bool,
    pub adsb_1090_in: bool,
    pub adsb_uat_in: bool,
    pub nac_v: u8,
    pub nic_supplement_c: bool,
}

/// Operational mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationalMode {
    pub tcas_ra_active: bool,
    pub ident_active: bool,
    pub receiving_atc_services: bool,
}

/// Heading reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingReference {
    TrueNorth,
    MagneticNorth,
}

/// Track/heading information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackHeadingInfo {
    Track,
    Heading,
}

/// Decode operation status from the 7-byte ME field.
///
/// ME layout (56 bits):
///   [TC:5][ST:3][CC:16][OM:16][version-dependent fields]
pub fn decode(me: &[u8]) -> DecodeResult<OperationStatus> {
    if me.len() < 7 {
        return Err(DecodeError::TooShort);
    }

    let tc = (me[0] >> 3) & 0x1F;
    if tc != 31 {
        return Err(DecodeError::InvalidTypeCode(tc));
    }

    let st = me[0] & 0x07;

    let data = match st {
        0 => decode_airborne(me),
        1 => decode_surface(me),
        _ => OperationStatusData::Unknown(st),
    };

    Ok(OperationStatus { subtype: st, data })
}

fn decode_airborne(me: &[u8]) -> OperationStatusData {
    // Capability class (16 bits): me[1..3]
    let cc = ((me[1] as u16) << 8) | me[2] as u16;
    let capability_class = AirborneCapability {
        tcas_operational: (cc >> 14) & 0x01 != 0,
        adsb_1090_in: (cc >> 12) & 0x01 != 0,
        adsb_uat_in: (cc >> 11) & 0x01 != 0,
        arv: (cc >> 9) & 0x01 != 0,
        ts: (cc >> 8) & 0x01 != 0,
        tc: ((cc >> 6) & 0x03) as u8,
    };

    // Operational mode (16 bits): me[3..5]
    let om = ((me[3] as u16) << 8) | me[4] as u16;
    let operational_mode = OperationalMode {
        tcas_ra_active: (om >> 13) & 0x01 != 0,
        ident_active: (om >> 12) & 0x01 != 0,
        receiving_atc_services: (om >> 11) & 0x01 != 0,
    };

    // Version and remaining fields
    let version = (me[5] >> 5) & 0x07;
    let nic_supplement_a = (me[5] >> 4) & 0x01 != 0;
    let nac_p = me[5] & 0x0F;
    let gva = (me[6] >> 6) & 0x03;
    let sil = (me[6] >> 4) & 0x03;
    let nic_baro = (me[6] >> 3) & 0x01 != 0;
    let sil_supplement = (me[6] >> 2) & 0x01 != 0;
    let hrd = if (me[6] >> 1) & 0x01 != 0 {
        HeadingReference::MagneticNorth
    } else {
        HeadingReference::TrueNorth
    };

    OperationStatusData::Airborne {
        capability_class,
        operational_mode,
        version,
        nic_supplement_a,
        nac_p,
        gva,
        sil,
        nic_baro,
        sil_supplement,
        hrd,
    }
}

fn decode_surface(me: &[u8]) -> OperationStatusData {
    // Capability class (16 bits): me[1..3]
    let cc = ((me[1] as u16) << 8) | me[2] as u16;
    let capability_class = SurfaceCapability {
        poa: (cc >> 13) & 0x01 != 0,
        cdti: (cc >> 12) & 0x01 != 0,
        adsb_1090_in: (cc >> 11) & 0x01 != 0,
        adsb_uat_in: (cc >> 10) & 0x01 != 0,
        nac_v: ((cc >> 7) & 0x07) as u8,
        nic_supplement_c: (cc >> 6) & 0x01 != 0,
    };

    // Operational mode (16 bits): me[3..5]
    let om = ((me[3] as u16) << 8) | me[4] as u16;
    let operational_mode = OperationalMode {
        tcas_ra_active: (om >> 13) & 0x01 != 0,
        ident_active: (om >> 12) & 0x01 != 0,
        receiving_atc_services: (om >> 11) & 0x01 != 0,
    };

    let version = (me[5] >> 5) & 0x07;
    let nic_supplement_a = (me[5] >> 4) & 0x01 != 0;
    let nac_p = me[5] & 0x0F;
    let sil = (me[6] >> 4) & 0x03;
    let sil_supplement = (me[6] >> 3) & 0x01 != 0;
    let trk_hdg = if (me[6] >> 2) & 0x01 != 0 {
        TrackHeadingInfo::Heading
    } else {
        TrackHeadingInfo::Track
    };
    let hrd = if (me[6] >> 1) & 0x01 != 0 {
        HeadingReference::MagneticNorth
    } else {
        HeadingReference::TrueNorth
    };

    OperationStatusData::Surface {
        capability_class,
        operational_mode,
        version,
        nic_supplement_a,
        nac_p,
        sil,
        sil_supplement,
        trk_hdg,
        hrd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_airborne_status() {
        // TC=31, ST=0
        let me = [0xF8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let status = decode(&me).unwrap();
        assert_eq!(status.subtype, 0);
        assert!(matches!(
            status.data,
            OperationStatusData::Airborne { .. }
        ));
    }

    #[test]
    fn test_decode_surface_status() {
        // TC=31, ST=1
        let me = [0xF9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let status = decode(&me).unwrap();
        assert_eq!(status.subtype, 1);
        assert!(matches!(status.data, OperationStatusData::Surface { .. }));
    }
}
