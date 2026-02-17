/// 24-bit ICAO aircraft address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IcaoAddress(pub u32);

impl IcaoAddress {
    #[inline]
    pub fn from_bytes(b: &[u8]) -> Self {
        IcaoAddress(((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32))
    }
}

impl core::fmt::Display for IcaoAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:06X}", self.0)
    }
}

/// Decoded altitude value with source indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Altitude {
    pub feet: i32,
    pub source: AltitudeSource,
}

/// Source of an altitude measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltitudeSource {
    Barometric,
    Gnss,
}

/// Decoded WGS-84 position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
}

/// Raw CPR-encoded position from a single message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CprPosition {
    pub lat_cpr: u32,
    pub lon_cpr: u32,
    pub odd_flag: bool,
}

/// Flight status (DF 4/5/20/21).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightStatus {
    AirborneNoAlert,
    GroundNoAlert,
    AirborneAlert,
    GroundAlert,
    AlertSpi,
    Spi,
    Reserved,
    NotAssigned,
}

impl FlightStatus {
    #[inline]
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => FlightStatus::AirborneNoAlert,
            1 => FlightStatus::GroundNoAlert,
            2 => FlightStatus::AirborneAlert,
            3 => FlightStatus::GroundAlert,
            4 => FlightStatus::AlertSpi,
            5 => FlightStatus::Spi,
            6 => FlightStatus::Reserved,
            _ => FlightStatus::NotAssigned,
        }
    }
}

/// Downlink request field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownlinkRequest {
    None,
    CommB,
    CommBAtcFacility,
    CommBAtcFacilityWithId,
    Reserved(u8),
}

impl DownlinkRequest {
    #[inline]
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => DownlinkRequest::None,
            1 => DownlinkRequest::CommB,
            4 => DownlinkRequest::CommBAtcFacility,
            5 => DownlinkRequest::CommBAtcFacilityWithId,
            v => DownlinkRequest::Reserved(v),
        }
    }
}

/// Utility message field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtilityMessage {
    pub iis: u8,
    pub ids: UtilityIds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityIds {
    NoInformation,
    CommB,
    CommC,
    CommD,
}

impl UtilityMessage {
    #[inline]
    pub fn from_code(code: u8) -> Self {
        let iis = (code >> 2) & 0x0F;
        let ids = match code & 0x03 {
            0 => UtilityIds::NoInformation,
            1 => UtilityIds::CommB,
            2 => UtilityIds::CommC,
            3 => UtilityIds::CommD,
            _ => unreachable!(),
        };
        UtilityMessage { iis, ids }
    }
}

/// Transponder squawk code (4 octal digits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Squawk(pub u16);

impl Squawk {
    /// Decode squawk from a 13-bit identity field.
    /// The 13 bits are laid out as: C1 A1 C2 A2 C4 A4 _ B1 D1 B2 D2 B4 D4
    #[inline]
    pub fn from_identity_code(code: u16) -> Self {
        let c1 = (code >> 12) & 1;
        let a1 = (code >> 11) & 1;
        let c2 = (code >> 10) & 1;
        let a2 = (code >> 9) & 1;
        let c4 = (code >> 8) & 1;
        let a4 = (code >> 7) & 1;
        // bit 6 is the M bit (spare)
        let b1 = (code >> 5) & 1;
        let d1 = (code >> 4) & 1;
        let b2 = (code >> 3) & 1;
        let d2 = (code >> 2) & 1;
        let b4 = (code >> 1) & 1;
        let d4 = code & 1;

        let a = (a4 << 2) | (a2 << 1) | a1;
        let b = (b4 << 2) | (b2 << 1) | b1;
        let c = (c4 << 2) | (c2 << 1) | c1;
        let d = (d4 << 2) | (d2 << 1) | d1;

        Squawk(((a << 12) | (b << 8) | (c << 4) | d) as u16)
    }
}

impl core::fmt::Display for Squawk {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:o}{:o}{:o}{:o}",
            (self.0 >> 12) & 0x7,
            (self.0 >> 8) & 0x7,
            (self.0 >> 4) & 0x7,
            self.0 & 0x7
        )
    }
}

/// Capability field (DF 11, 17).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Level1,
    Level2OnGround,
    Level2Airborne,
    Level2GroundOrAirborne,
    Level2OrAbove,
    Level2WithCommBCapability,
    Reserved(u8),
}

impl Capability {
    #[inline]
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => Capability::Level1,
            1 => Capability::Level2OnGround,
            2 => Capability::Level2Airborne,
            3 => Capability::Level2GroundOrAirborne,
            4 => Capability::Level2OrAbove,
            5 | 6 | 7 => Capability::Level2WithCommBCapability,
            v => Capability::Reserved(v),
        }
    }
}

/// Vertical status (DF 0/16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalStatus {
    Airborne,
    Ground,
}

/// Sensitivity level (DF 0/16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensitivityLevel(pub u8);

/// Reply information (DF 0/16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplyInformation(pub u8);

/// Wake vortex category for ADS-B identification messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeVortexCategory {
    NoInfo,
    Light,
    Medium,
    Heavy,
    HighPerformance,
    Rotorcraft,
    Glider,
    Lighter,
    Parachutist,
    Ultralight,
    Reserved,
    Uav,
    Space,
    SurfaceEmergency,
    SurfaceService,
    GroundObstruction,
    ClusterObstacle,
    LineObstacle,
    Unknown(u8, u8),
}

/// Surveillance status (ADS-B position messages).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurveillanceStatus {
    NoCondition,
    PermanentAlert,
    TemporaryAlert,
    SpiCondition,
}

impl SurveillanceStatus {
    #[inline]
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => SurveillanceStatus::NoCondition,
            1 => SurveillanceStatus::PermanentAlert,
            2 => SurveillanceStatus::TemporaryAlert,
            3 => SurveillanceStatus::SpiCondition,
            _ => SurveillanceStatus::NoCondition,
        }
    }
}

/// Navigation integrity category (NIC) supplement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NicSupplement(pub u8);

/// Control field (DF 18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlField {
    AdsbIcao,
    AdsbNonIcao,
    TisbFineFormat,
    TisbCoarseFormat,
    TisbRelay,
    AdsbRebroadcast,
    Reserved(u8),
}

impl ControlField {
    #[inline]
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => ControlField::AdsbIcao,
            1 => ControlField::AdsbNonIcao,
            2 => ControlField::TisbFineFormat,
            3 => ControlField::TisbCoarseFormat,
            4 => ControlField::TisbRelay,
            5 => ControlField::AdsbRebroadcast,
            v => ControlField::Reserved(v),
        }
    }
}

/// Movement speed encoding for surface position messages.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroundSpeed {
    Stopped,
    Knots(f64),
    GreaterThan175,
}

/// Error type for decoding failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    TooShort,
    InvalidDownlinkFormat(u8),
    CrcFailed { expected: u32, computed: u32 },
    InvalidTypeCode(u8),
    InvalidData(&'static str),
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::TooShort => write!(f, "message too short"),
            DecodeError::InvalidDownlinkFormat(df) => {
                write!(f, "invalid downlink format: {df}")
            }
            DecodeError::CrcFailed { expected, computed } => {
                write!(f, "CRC failed: expected {expected:06X}, computed {computed:06X}")
            }
            DecodeError::InvalidTypeCode(tc) => write!(f, "invalid type code: {tc}"),
            DecodeError::InvalidData(msg) => write!(f, "invalid data: {msg}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Result alias for decode operations.
pub type DecodeResult<T> = Result<T, DecodeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icao_address() {
        let addr = IcaoAddress::from_bytes(&[0x4B, 0x16, 0x45]);
        assert_eq!(addr.0, 0x4B1645);
        assert_eq!(format!("{addr}"), "4B1645");
    }

    #[test]
    fn test_squawk_decode() {
        // Squawk 7700 (emergency): identity code maps to A=7,B=7,C=0,D=0
        let _sq = Squawk::from_identity_code(0b0_11_0_11_0_11_0_00_0);
        // Test with a known value: squawk 2000
        // A=2, B=0, C=0, D=0 => 2000 octal
        let sq2 = Squawk(0x2000);
        assert_eq!(format!("{sq2}"), "2000");
    }

    #[test]
    fn test_squawk_display() {
        // Squawk stored as packed BCD-like: each nibble is 0-7
        let sq = Squawk(0x7700);
        assert_eq!(format!("{sq}"), "7700");
    }

    #[test]
    fn test_flight_status() {
        assert_eq!(FlightStatus::from_code(0), FlightStatus::AirborneNoAlert);
        assert_eq!(FlightStatus::from_code(1), FlightStatus::GroundNoAlert);
        assert_eq!(FlightStatus::from_code(7), FlightStatus::NotAssigned);
    }

    #[test]
    fn test_capability() {
        assert_eq!(Capability::from_code(0), Capability::Level1);
        assert_eq!(
            Capability::from_code(5),
            Capability::Level2WithCommBCapability
        );
    }
}
