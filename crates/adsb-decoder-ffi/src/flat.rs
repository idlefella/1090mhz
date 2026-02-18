//! Flat C-compatible struct representation of decoded messages.
//!
//! This module provides a `#[repr(C)]` struct that can be directly used from C, Java (JNA),
//! Python (ctypes), etc. without any serialization overhead.

use adsb_decoder::adsb::velocity::VelocityData;
use adsb_decoder::adsb::AdsbPayload;
use adsb_decoder::frame::Message;
use adsb_decoder::types::AltitudeSource;

/// Message type discriminant for the flat struct.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Unknown or unsupported message type.
    Unknown = 0,
    /// DF 0: Short ACAS surveillance.
    AcasSurveillance = 1,
    /// DF 4: Surveillance altitude reply.
    SurveillanceAltitude = 2,
    /// DF 5: Surveillance identity reply.
    SurveillanceIdentity = 3,
    /// DF 11: All-call reply.
    AllCallReply = 4,
    /// DF 16: Long ACAS coordination.
    AcasCoordination = 5,
    /// DF 17: ADS-B message.
    AdsbMessage = 6,
    /// DF 18: TIS-B message.
    TisbMessage = 7,
    /// DF 20: Comm-B with altitude.
    CommBAltitude = 8,
    /// DF 21: Comm-B with identity.
    CommBIdentity = 9,
}

/// ADS-B payload type discriminant.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdsbType {
    /// Not an ADS-B message or unknown payload.
    None = 0,
    /// Aircraft identification (callsign).
    Identification = 1,
    /// Airborne position.
    AirbornePosition = 2,
    /// Surface position.
    SurfacePosition = 3,
    /// Airborne velocity.
    AirborneVelocity = 4,
    /// Aircraft status.
    AircraftStatus = 5,
    /// Target state and status.
    TargetState = 6,
    /// Aircraft operation status.
    OperationStatus = 7,
}

/// Velocity type discriminant.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VelocityType {
    /// No velocity data available.
    None = 0,
    /// Ground speed with heading.
    GroundSpeed = 1,
    /// Airspeed with heading.
    Airspeed = 2,
}

/// Altitude source discriminant.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltitudeSourceC {
    /// Unknown or not available.
    Unknown = 0,
    /// Barometric altitude.
    Barometric = 1,
    /// GNSS altitude.
    Gnss = 2,
}

/// Flight status (DF 4/5/20/21).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightStatusC {
    AirborneNoAlert = 0,
    GroundNoAlert = 1,
    AirborneAlert = 2,
    GroundAlert = 3,
    AlertSpi = 4,
    Spi = 5,
    Reserved = 6,
    NotAssigned = 7,
}

/// Vertical status (DF 0/16).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalStatusC {
    Airborne = 0,
    Ground = 1,
}

/// Flat C-compatible decoded message.
///
/// All fields are always present; validity is indicated by the discriminant fields
/// (`msg_type`, `adsb_type`, `velocity_type`) and `has_*` flags.
///
/// # Memory Layout
/// This struct uses `#[repr(C)]` for predictable memory layout across FFI boundaries.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FlatMessage {
    // ===== Common fields =====
    /// Downlink format (0, 4, 5, 11, 16, 17, 18, 20, 21).
    pub df: i32,
    /// Message type discriminant.
    pub msg_type: MessageType,
    /// 24-bit ICAO aircraft address.
    pub icao: u32,

    // ===== Altitude (DF 0/4/16/17/18/20) =====
    /// Whether altitude data is valid.
    pub has_altitude: i32,
    /// Altitude in feet.
    pub altitude_feet: i32,
    /// Altitude source (barometric or GNSS).
    pub altitude_source: AltitudeSourceC,

    // ===== Squawk (DF 5/21) =====
    /// Whether squawk is valid.
    pub has_squawk: i32,
    /// Squawk code as 4 octal digits packed into u16.
    /// Display as: printf("%04o", squawk)
    pub squawk: u16,
    pub _pad1: u16,

    // ===== Flight status (DF 4/5/20/21) =====
    /// Whether flight status is valid.
    pub has_flight_status: i32,
    /// Flight status.
    pub flight_status: FlightStatusC,

    // ===== Vertical status (DF 0/16) =====
    /// Whether vertical status is valid.
    pub has_vertical_status: i32,
    /// Vertical status (airborne/ground).
    pub vertical_status: VerticalStatusC,

    // ===== Capability (DF 11/17) =====
    /// Whether capability is valid.
    pub has_capability: i32,
    /// Capability code (0-7).
    pub capability: u8,
    pub _pad2: [u8; 3],

    // ===== ADS-B specific (DF 17/18) =====
    /// ADS-B payload type discriminant.
    pub adsb_type: AdsbType,
    /// Type code from ME field (1-31).
    pub type_code: u8,
    pub _pad3: [u8; 3],

    // ===== Identification (TC 1-4) =====
    /// Callsign, null-terminated (max 8 chars + null).
    pub callsign: [u8; 9],
    pub _pad4: [u8; 3],
    /// Wake vortex category code.
    pub wake_category: u8,
    pub _pad5: [u8; 3],

    // ===== Position (TC 5-22) =====
    /// Whether CPR position data is valid.
    pub has_cpr: i32,
    /// CPR odd flag (0=even, 1=odd).
    pub cpr_odd: i32,
    /// Raw 17-bit CPR latitude.
    pub cpr_lat: u32,
    /// Raw 17-bit CPR longitude.
    pub cpr_lon: u32,
    /// Whether UTC synchronized.
    pub utc_sync: i32,

    // ===== Velocity (TC 19) =====
    /// Velocity type discriminant.
    pub velocity_type: VelocityType,
    /// Ground speed in knots (for GroundSpeed type).
    pub speed_knots: f64,
    /// Heading in degrees (0-360).
    pub heading_degrees: f64,
    /// Airspeed in knots (for Airspeed type).
    pub airspeed_knots: u16,
    /// Whether airspeed is TAS (true) or IAS (false).
    pub airspeed_is_tas: i32,
    pub _pad6: u16,

    // ===== Vertical rate =====
    /// Whether vertical rate is valid.
    pub has_vertical_rate: i32,
    /// Vertical rate in feet per minute (positive=climbing).
    pub vertical_rate_fpm: i32,
    /// Vertical rate source.
    pub vertical_rate_source: AltitudeSourceC,

    // ===== Surface position specific =====
    /// Whether surface ground speed is valid.
    pub has_ground_speed: i32,
    /// Surface ground speed in knots (-1 = stopped, -2 = >175 kts).
    pub ground_speed_knots: f64,
    /// Whether surface heading is valid.
    pub has_heading: i32,
    pub _pad7: i32,

    // ===== Raw data fields =====
    /// MB field for Comm-B (DF 20/21), 7 bytes.
    pub mb_data: [u8; 7],
    /// MV field for ACAS (DF 16), 7 bytes.
    pub mv_data: [u8; 7],
    pub _pad8: [u8; 2],
}

impl Default for FlatMessage {
    fn default() -> Self {
        Self {
            df: -1,
            msg_type: MessageType::Unknown,
            icao: 0,
            has_altitude: 0,
            altitude_feet: 0,
            altitude_source: AltitudeSourceC::Unknown,
            has_squawk: 0,
            squawk: 0,
            _pad1: 0,
            has_flight_status: 0,
            flight_status: FlightStatusC::AirborneNoAlert,
            has_vertical_status: 0,
            vertical_status: VerticalStatusC::Airborne,
            has_capability: 0,
            capability: 0,
            _pad2: [0; 3],
            adsb_type: AdsbType::None,
            type_code: 0,
            _pad3: [0; 3],
            callsign: [0; 9],
            _pad4: [0; 3],
            wake_category: 0,
            _pad5: [0; 3],
            has_cpr: 0,
            cpr_odd: 0,
            cpr_lat: 0,
            cpr_lon: 0,
            utc_sync: 0,
            velocity_type: VelocityType::None,
            speed_knots: 0.0,
            heading_degrees: 0.0,
            airspeed_knots: 0,
            airspeed_is_tas: 0,
            _pad6: 0,
            has_vertical_rate: 0,
            vertical_rate_fpm: 0,
            vertical_rate_source: AltitudeSourceC::Unknown,
            has_ground_speed: 0,
            ground_speed_knots: 0.0,
            has_heading: 0,
            _pad7: 0,
            mb_data: [0; 7],
            mv_data: [0; 7],
            _pad8: [0; 2],
        }
    }
}

impl FlatMessage {
    /// Convert a decoded Message to a flat C-compatible struct.
    pub fn from_message(msg: &Message) -> Self {
        let mut flat = FlatMessage::default();

        match msg {
            Message::AcasSurveillance(m) => {
                flat.df = 0;
                flat.msg_type = MessageType::AcasSurveillance;
                flat.icao = m.icao.0;
                flat.has_vertical_status = 1;
                flat.vertical_status = match m.vertical_status {
                    adsb_decoder::types::VerticalStatus::Airborne => VerticalStatusC::Airborne,
                    adsb_decoder::types::VerticalStatus::Ground => VerticalStatusC::Ground,
                };
                if let Some(alt) = &m.altitude {
                    flat.has_altitude = 1;
                    flat.altitude_feet = alt.feet;
                    flat.altitude_source = match alt.source {
                        AltitudeSource::Barometric => AltitudeSourceC::Barometric,
                        AltitudeSource::Gnss => AltitudeSourceC::Gnss,
                    };
                }
            }

            Message::SurveillanceAltitude(m) => {
                flat.df = 4;
                flat.msg_type = MessageType::SurveillanceAltitude;
                flat.icao = m.icao.0;
                flat.has_flight_status = 1;
                flat.flight_status = convert_flight_status(&m.flight_status);
                if let Some(alt) = &m.altitude {
                    flat.has_altitude = 1;
                    flat.altitude_feet = alt.feet;
                    flat.altitude_source = match alt.source {
                        AltitudeSource::Barometric => AltitudeSourceC::Barometric,
                        AltitudeSource::Gnss => AltitudeSourceC::Gnss,
                    };
                }
            }

            Message::SurveillanceIdentity(m) => {
                flat.df = 5;
                flat.msg_type = MessageType::SurveillanceIdentity;
                flat.icao = m.icao.0;
                flat.has_flight_status = 1;
                flat.flight_status = convert_flight_status(&m.flight_status);
                if let Some(sq) = &m.squawk {
                    flat.has_squawk = 1;
                    flat.squawk = sq.0;
                }
            }

            Message::AllCallReply(m) => {
                flat.df = 11;
                flat.msg_type = MessageType::AllCallReply;
                flat.icao = m.icao.0;
                flat.has_capability = 1;
                flat.capability = convert_capability(&m.capability);
            }

            Message::AcasCoordination(m) => {
                flat.df = 16;
                flat.msg_type = MessageType::AcasCoordination;
                flat.icao = m.icao.0;
                flat.has_vertical_status = 1;
                flat.vertical_status = match m.vertical_status {
                    adsb_decoder::types::VerticalStatus::Airborne => VerticalStatusC::Airborne,
                    adsb_decoder::types::VerticalStatus::Ground => VerticalStatusC::Ground,
                };
                if let Some(alt) = &m.altitude {
                    flat.has_altitude = 1;
                    flat.altitude_feet = alt.feet;
                    flat.altitude_source = match alt.source {
                        AltitudeSource::Barometric => AltitudeSourceC::Barometric,
                        AltitudeSource::Gnss => AltitudeSourceC::Gnss,
                    };
                }
                flat.mv_data = m.mv_field;
            }

            Message::AdsbMessage(m) => {
                flat.df = 17;
                flat.msg_type = MessageType::AdsbMessage;
                flat.icao = m.icao.0;
                flat.has_capability = 1;
                flat.capability = convert_capability(&m.capability);
                convert_adsb_payload(&m.payload, &mut flat);
            }

            Message::TisbMessage(m) => {
                flat.df = 18;
                flat.msg_type = MessageType::TisbMessage;
                flat.icao = m.icao.0;
                convert_adsb_payload(&m.payload, &mut flat);
            }

            Message::CommBAltitude(m) => {
                flat.df = 20;
                flat.msg_type = MessageType::CommBAltitude;
                flat.icao = m.icao.0;
                flat.has_flight_status = 1;
                flat.flight_status = convert_flight_status(&m.flight_status);
                if let Some(alt) = &m.altitude {
                    flat.has_altitude = 1;
                    flat.altitude_feet = alt.feet;
                    flat.altitude_source = match alt.source {
                        AltitudeSource::Barometric => AltitudeSourceC::Barometric,
                        AltitudeSource::Gnss => AltitudeSourceC::Gnss,
                    };
                }
                flat.mb_data = m.mb_data;
            }

            Message::CommBIdentity(m) => {
                flat.df = 21;
                flat.msg_type = MessageType::CommBIdentity;
                flat.icao = m.icao.0;
                flat.has_flight_status = 1;
                flat.flight_status = convert_flight_status(&m.flight_status);
                if let Some(sq) = &m.squawk {
                    flat.has_squawk = 1;
                    flat.squawk = sq.0;
                }
                flat.mb_data = m.mb_data;
            }
        }

        flat
    }
}

fn convert_flight_status(fs: &adsb_decoder::types::FlightStatus) -> FlightStatusC {
    use adsb_decoder::types::FlightStatus;
    match fs {
        FlightStatus::AirborneNoAlert => FlightStatusC::AirborneNoAlert,
        FlightStatus::GroundNoAlert => FlightStatusC::GroundNoAlert,
        FlightStatus::AirborneAlert => FlightStatusC::AirborneAlert,
        FlightStatus::GroundAlert => FlightStatusC::GroundAlert,
        FlightStatus::AlertSpi => FlightStatusC::AlertSpi,
        FlightStatus::Spi => FlightStatusC::Spi,
        FlightStatus::Reserved => FlightStatusC::Reserved,
        FlightStatus::NotAssigned => FlightStatusC::NotAssigned,
    }
}

fn convert_capability(cap: &adsb_decoder::types::Capability) -> u8 {
    use adsb_decoder::types::Capability;
    match cap {
        Capability::Level1 => 0,
        Capability::Level2OnGround => 1,
        Capability::Level2Airborne => 2,
        Capability::Level2GroundOrAirborne => 3,
        Capability::Level2OrAbove => 4,
        Capability::Level2WithCommBCapability => 5,
        Capability::Reserved(v) => *v,
    }
}

fn convert_adsb_payload(payload: &AdsbPayload, flat: &mut FlatMessage) {
    match payload {
        AdsbPayload::Identification(id) => {
            flat.adsb_type = AdsbType::Identification;
            flat.type_code = id.type_code;
            // Copy callsign (up to 8 chars, null-terminated)
            let len = id.callsign_len().min(8);
            flat.callsign[..len].copy_from_slice(&id.callsign[..len]);
            flat.callsign[len] = 0; // null terminate
            flat.wake_category = convert_wake_category(&id.category);
        }

        AdsbPayload::AirbornePosition(pos) => {
            flat.adsb_type = AdsbType::AirbornePosition;
            flat.type_code = pos.type_code;
            if let Some(alt) = &pos.altitude {
                flat.has_altitude = 1;
                flat.altitude_feet = alt.feet;
                flat.altitude_source = match alt.source {
                    AltitudeSource::Barometric => AltitudeSourceC::Barometric,
                    AltitudeSource::Gnss => AltitudeSourceC::Gnss,
                };
            }
            flat.has_cpr = 1;
            flat.cpr_odd = pos.cpr.odd_flag as i32;
            flat.cpr_lat = pos.cpr.lat_cpr;
            flat.cpr_lon = pos.cpr.lon_cpr;
            flat.utc_sync = pos.utc_synchronized as i32;
        }

        AdsbPayload::SurfacePosition(pos) => {
            flat.adsb_type = AdsbType::SurfacePosition;
            flat.type_code = pos.type_code;
            flat.has_cpr = 1;
            flat.cpr_odd = pos.cpr.odd_flag as i32;
            flat.cpr_lat = pos.cpr.lat_cpr;
            flat.cpr_lon = pos.cpr.lon_cpr;
            flat.utc_sync = pos.utc_synchronized as i32;

            // Ground speed
            flat.has_ground_speed = 1;
            flat.ground_speed_knots = match pos.ground_speed {
                adsb_decoder::types::GroundSpeed::Stopped => -1.0,
                adsb_decoder::types::GroundSpeed::Knots(k) => k,
                adsb_decoder::types::GroundSpeed::GreaterThan175 => -2.0,
            };

            // Heading
            if let Some(hdg) = pos.heading {
                flat.has_heading = 1;
                flat.heading_degrees = hdg;
            }
        }

        AdsbPayload::AirborneVelocity(vel) => {
            flat.adsb_type = AdsbType::AirborneVelocity;
            flat.type_code = 19;

            match &vel.velocity {
                Some(VelocityData::GroundSpeed {
                    speed_knots,
                    heading_degrees,
                    ..
                }) => {
                    flat.velocity_type = VelocityType::GroundSpeed;
                    flat.speed_knots = *speed_knots;
                    flat.heading_degrees = *heading_degrees;
                }
                Some(VelocityData::Airspeed {
                    heading_degrees,
                    airspeed_knots,
                    airspeed_is_tas,
                }) => {
                    flat.velocity_type = VelocityType::Airspeed;
                    flat.heading_degrees = *heading_degrees;
                    flat.airspeed_knots = *airspeed_knots;
                    flat.airspeed_is_tas = *airspeed_is_tas as i32;
                }
                None => {
                    flat.velocity_type = VelocityType::None;
                }
            }

            if let Some(vr) = &vel.vertical_rate {
                flat.has_vertical_rate = 1;
                flat.vertical_rate_fpm = vr.feet_per_minute;
                flat.vertical_rate_source = match vr.source {
                    AltitudeSource::Barometric => AltitudeSourceC::Barometric,
                    AltitudeSource::Gnss => AltitudeSourceC::Gnss,
                };
            }
        }

        AdsbPayload::AircraftStatus(_) => {
            flat.adsb_type = AdsbType::AircraftStatus;
            flat.type_code = 28;
        }

        AdsbPayload::TargetState(_) => {
            flat.adsb_type = AdsbType::TargetState;
            flat.type_code = 29;
        }

        AdsbPayload::OperationStatus(_) => {
            flat.adsb_type = AdsbType::OperationStatus;
            flat.type_code = 31;
        }

        AdsbPayload::Unknown(tc) => {
            flat.adsb_type = AdsbType::None;
            flat.type_code = *tc;
        }
    }
}

fn convert_wake_category(cat: &adsb_decoder::types::WakeVortexCategory) -> u8 {
    use adsb_decoder::types::WakeVortexCategory;
    match cat {
        WakeVortexCategory::NoInfo => 0,
        WakeVortexCategory::Light => 1,
        WakeVortexCategory::Medium => 2,
        WakeVortexCategory::Heavy => 3,
        WakeVortexCategory::HighPerformance => 4,
        WakeVortexCategory::Rotorcraft => 5,
        WakeVortexCategory::Glider => 6,
        WakeVortexCategory::Lighter => 7,
        WakeVortexCategory::Parachutist => 8,
        WakeVortexCategory::Ultralight => 9,
        WakeVortexCategory::Reserved => 10,
        WakeVortexCategory::Uav => 11,
        WakeVortexCategory::Space => 12,
        WakeVortexCategory::SurfaceEmergency => 13,
        WakeVortexCategory::SurfaceService => 14,
        WakeVortexCategory::GroundObstruction => 15,
        WakeVortexCategory::ClusterObstacle => 16,
        WakeVortexCategory::LineObstacle => 17,
        WakeVortexCategory::Unknown(_, _) => 255,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adsb_decoder::frame;

    #[test]
    fn test_flat_identification() {
        let msg = frame::decode_hex("8D4840D6202CC371C32CE0576098").unwrap();
        let flat = FlatMessage::from_message(&msg);

        assert_eq!(flat.df, 17);
        assert_eq!(flat.msg_type, MessageType::AdsbMessage);
        assert_eq!(flat.icao, 0x4840D6);
        assert_eq!(flat.adsb_type, AdsbType::Identification);

        let callsign = std::str::from_utf8(&flat.callsign[..7]).unwrap();
        assert_eq!(callsign, "KLM1023");
    }

    #[test]
    fn test_flat_velocity() {
        let msg = frame::decode_hex("8D485020994409940838175B284F").unwrap();
        let flat = FlatMessage::from_message(&msg);

        assert_eq!(flat.df, 17);
        assert_eq!(flat.icao, 0x485020);
        assert_eq!(flat.adsb_type, AdsbType::AirborneVelocity);
        assert_eq!(flat.velocity_type, VelocityType::GroundSpeed);
        assert!(flat.speed_knots > 100.0);
    }

    #[test]
    fn test_flat_position() {
        let msg = frame::decode_hex("8D40621D58C382D690C8AC2863A7").unwrap();
        let flat = FlatMessage::from_message(&msg);

        assert_eq!(flat.df, 17);
        assert_eq!(flat.icao, 0x40621D);
        assert_eq!(flat.adsb_type, AdsbType::AirbornePosition);
        assert_eq!(flat.has_cpr, 1);
        assert_eq!(flat.cpr_odd, 0); // even frame
        assert_eq!(flat.has_altitude, 1);
    }

    #[test]
    fn test_flat_df11() {
        let msg = frame::decode_hex("5D4B164513B842").unwrap();
        let flat = FlatMessage::from_message(&msg);

        assert_eq!(flat.df, 11);
        assert_eq!(flat.msg_type, MessageType::AllCallReply);
        assert_eq!(flat.icao, 0x4B1645);
        assert_eq!(flat.has_capability, 1);
    }

    #[test]
    fn test_struct_size() {
        // Ensure struct size is reasonable and consistent
        let size = std::mem::size_of::<FlatMessage>();
        // Print for debugging: should be around 200-250 bytes
        println!("FlatMessage size: {} bytes", size);
        assert!(size < 300, "FlatMessage is too large: {} bytes", size);
    }
}
