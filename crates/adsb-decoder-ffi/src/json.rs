//! Manual JSON serialization for Mode-S/ADS-B messages.
//!
//! This avoids adding serde as a dependency while providing JSON output.

use adsb_decoder::adsb::airborne_position::AirbornePosition;
use adsb_decoder::adsb::identification::Identification;
use adsb_decoder::adsb::surface_position::SurfacePosition;
use adsb_decoder::adsb::velocity::{AirborneVelocity, VelocityData, VerticalRate};
use adsb_decoder::adsb::AdsbPayload;
use adsb_decoder::frame::{AdsbMessage, Message};
use adsb_decoder::modes::acas::AcasLongReply;
use adsb_decoder::modes::allcall::AllCallReply;
use adsb_decoder::modes::surveillance::SurveillanceReply;
use adsb_decoder::modes::CommBReply;
use adsb_decoder::types::{
    Altitude, AltitudeSource, Capability, ControlField, CprPosition, FlightStatus, GroundSpeed,
    SurveillanceStatus, WakeVortexCategory,
};

/// Convert a Message to JSON string.
pub fn message_to_json(msg: &Message) -> String {
    let mut json = String::with_capacity(512);
    json.push('{');

    match msg {
        Message::AcasSurveillance(m) => {
            json.push_str("\"df\":0,");
            json.push_str(&format!("\"icao\":\"{}\",", m.icao));
            json.push_str(&format!(
                "\"vertical_status\":\"{:?}\",",
                m.vertical_status
            ));
            if let Some(alt) = &m.altitude {
                json.push_str(&altitude_json(alt));
                json.push(',');
            }
            json.push_str(&format!("\"sensitivity_level\":{}", m.sensitivity_level.0));
        }
        Message::SurveillanceAltitude(m) => {
            json.push_str("\"df\":4,");
            surveillance_json(&mut json, m);
        }
        Message::SurveillanceIdentity(m) => {
            json.push_str("\"df\":5,");
            surveillance_json(&mut json, m);
        }
        Message::AllCallReply(m) => {
            json.push_str("\"df\":11,");
            allcall_json(&mut json, m);
        }
        Message::AcasCoordination(m) => {
            json.push_str("\"df\":16,");
            acas_long_json(&mut json, m);
        }
        Message::AdsbMessage(m) => {
            json.push_str("\"df\":17,");
            adsb_json(&mut json, m);
        }
        Message::TisbMessage(m) => {
            json.push_str("\"df\":18,");
            adsb_json(&mut json, m);
        }
        Message::CommBAltitude(m) => {
            json.push_str("\"df\":20,");
            commb_json(&mut json, m);
        }
        Message::CommBIdentity(m) => {
            json.push_str("\"df\":21,");
            commb_json(&mut json, m);
        }
    }

    json.push('}');
    json
}

fn surveillance_json(json: &mut String, m: &SurveillanceReply) {
    json.push_str(&format!("\"icao\":\"{}\",", m.icao));
    json.push_str(&format!(
        "\"flight_status\":\"{}\",",
        flight_status_str(m.flight_status)
    ));
    if let Some(alt) = &m.altitude {
        json.push_str(&altitude_json(alt));
        json.push(',');
    }
    if let Some(sq) = &m.squawk {
        json.push_str(&format!("\"squawk\":\"{}\",", sq));
    }
    // Remove trailing comma
    if json.ends_with(',') {
        json.pop();
    }
}

fn allcall_json(json: &mut String, m: &AllCallReply) {
    json.push_str(&format!("\"icao\":\"{}\",", m.icao));
    json.push_str(&format!(
        "\"capability\":\"{}\"",
        capability_str(m.capability)
    ));
}

fn acas_long_json(json: &mut String, m: &AcasLongReply) {
    json.push_str(&format!("\"icao\":\"{}\",", m.icao));
    json.push_str(&format!(
        "\"vertical_status\":\"{:?}\",",
        m.vertical_status
    ));
    if let Some(alt) = &m.altitude {
        json.push_str(&altitude_json(alt));
        json.push(',');
    }
    json.push_str(&format!("\"sensitivity_level\":{}", m.sensitivity_level.0));
}

fn adsb_json(json: &mut String, m: &AdsbMessage) {
    json.push_str(&format!("\"icao\":\"{}\",", m.icao));
    json.push_str(&format!(
        "\"capability\":\"{}\",",
        capability_str(m.capability)
    ));
    if let Some(cf) = m.control_field {
        json.push_str(&format!(
            "\"control_field\":\"{}\",",
            control_field_str(cf)
        ));
    }
    payload_json(json, &m.payload);
}

fn commb_json(json: &mut String, m: &CommBReply) {
    json.push_str(&format!("\"icao\":\"{}\",", m.icao));
    json.push_str(&format!(
        "\"flight_status\":\"{}\",",
        flight_status_str(m.flight_status)
    ));
    if let Some(alt) = &m.altitude {
        json.push_str(&altitude_json(alt));
        json.push(',');
    }
    if let Some(sq) = &m.squawk {
        json.push_str(&format!("\"squawk\":\"{}\",", sq));
    }
    // MB data as hex
    let mb_hex: String = m.mb_data.iter().map(|b| format!("{:02X}", b)).collect();
    json.push_str(&format!("\"mb_data\":\"{}\"", mb_hex));
}

fn payload_json(json: &mut String, payload: &AdsbPayload) {
    match payload {
        AdsbPayload::Identification(id) => {
            json.push_str("\"message_type\":\"identification\",");
            identification_json(json, id);
        }
        AdsbPayload::AirbornePosition(pos) => {
            json.push_str("\"message_type\":\"airborne_position\",");
            airborne_position_json(json, pos);
        }
        AdsbPayload::SurfacePosition(pos) => {
            json.push_str("\"message_type\":\"surface_position\",");
            surface_position_json(json, pos);
        }
        AdsbPayload::AirborneVelocity(vel) => {
            json.push_str("\"message_type\":\"airborne_velocity\",");
            velocity_json(json, vel);
        }
        AdsbPayload::AircraftStatus(_) => {
            json.push_str("\"message_type\":\"aircraft_status\"");
        }
        AdsbPayload::TargetState(_) => {
            json.push_str("\"message_type\":\"target_state\"");
        }
        AdsbPayload::OperationStatus(_) => {
            json.push_str("\"message_type\":\"operation_status\"");
        }
        AdsbPayload::Unknown(tc) => {
            json.push_str(&format!("\"message_type\":\"unknown\",\"type_code\":{}", tc));
        }
    }
}

fn identification_json(json: &mut String, id: &Identification) {
    json.push_str(&format!("\"type_code\":{},", id.type_code));
    json.push_str(&format!("\"callsign\":\"{}\",", id.callsign_str()));
    json.push_str(&format!(
        "\"category\":\"{}\"",
        wake_vortex_str(&id.category)
    ));
}

fn airborne_position_json(json: &mut String, pos: &AirbornePosition) {
    json.push_str(&format!("\"type_code\":{},", pos.type_code));
    json.push_str(&format!(
        "\"surveillance_status\":\"{}\",",
        surveillance_status_str(pos.surveillance_status)
    ));
    json.push_str(&format!("\"single_antenna\":{},", pos.single_antenna));
    if let Some(alt) = &pos.altitude {
        json.push_str(&altitude_json(alt));
        json.push(',');
    }
    json.push_str(&format!("\"utc_synchronized\":{},", pos.utc_synchronized));
    cpr_json(json, &pos.cpr);
}

fn surface_position_json(json: &mut String, pos: &SurfacePosition) {
    json.push_str(&format!("\"type_code\":{},", pos.type_code));
    json.push_str(&ground_speed_json(&pos.ground_speed));
    json.push(',');
    if let Some(heading) = pos.heading {
        json.push_str(&format!("\"heading\":{:.1},", heading));
    }
    json.push_str(&format!("\"heading_valid\":{},", pos.heading_valid));
    json.push_str(&format!("\"utc_synchronized\":{},", pos.utc_synchronized));
    cpr_json(json, &pos.cpr);
}

fn velocity_json(json: &mut String, vel: &AirborneVelocity) {
    json.push_str(&format!("\"subtype\":{},", vel.subtype));
    json.push_str(&format!("\"intent_change\":{},", vel.intent_change));
    json.push_str(&format!("\"ifr_capability\":{},", vel.ifr_capability));
    json.push_str(&format!("\"nac_v\":{},", vel.nac_v));

    if let Some(v) = &vel.velocity {
        match v {
            VelocityData::GroundSpeed {
                speed_knots,
                heading_degrees,
                east_west_knots,
                north_south_knots,
            } => {
                json.push_str("\"velocity_type\":\"ground_speed\",");
                json.push_str(&format!("\"speed_knots\":{:.1},", speed_knots));
                json.push_str(&format!("\"heading\":{:.1},", heading_degrees));
                json.push_str(&format!("\"east_west_knots\":{:.1},", east_west_knots));
                json.push_str(&format!("\"north_south_knots\":{:.1},", north_south_knots));
            }
            VelocityData::Airspeed {
                heading_degrees,
                airspeed_knots,
                airspeed_is_tas,
            } => {
                json.push_str("\"velocity_type\":\"airspeed\",");
                json.push_str(&format!("\"heading\":{:.1},", heading_degrees));
                json.push_str(&format!("\"airspeed_knots\":{},", airspeed_knots));
                json.push_str(&format!("\"airspeed_is_tas\":{},", airspeed_is_tas));
            }
        }
    }

    if let Some(vr) = &vel.vertical_rate {
        vertical_rate_json(json, vr);
        json.push(',');
    }

    if let Some(diff) = vel.gnss_baro_diff {
        json.push_str(&format!("\"gnss_baro_diff\":{}", diff));
    } else if json.ends_with(',') {
        json.pop();
    }
}

fn vertical_rate_json(json: &mut String, vr: &VerticalRate) {
    json.push_str(&format!("\"vertical_rate_fpm\":{},", vr.feet_per_minute));
    json.push_str(&format!(
        "\"vertical_rate_source\":\"{}\"",
        altitude_source_str(vr.source)
    ));
}

fn cpr_json(json: &mut String, cpr: &CprPosition) {
    json.push_str(&format!("\"cpr_lat\":{},", cpr.lat_cpr));
    json.push_str(&format!("\"cpr_lon\":{},", cpr.lon_cpr));
    json.push_str(&format!("\"cpr_odd\":{}", cpr.odd_flag));
}

fn altitude_json(alt: &Altitude) -> String {
    format!(
        "\"altitude_ft\":{},\"altitude_source\":\"{}\"",
        alt.feet,
        altitude_source_str(alt.source)
    )
}

fn ground_speed_json(gs: &GroundSpeed) -> String {
    match gs {
        GroundSpeed::Stopped => "\"ground_speed\":\"stopped\"".to_string(),
        GroundSpeed::Knots(k) => format!("\"ground_speed_knots\":{:.1}", k),
        GroundSpeed::GreaterThan175 => "\"ground_speed\":\">175\"".to_string(),
    }
}

fn flight_status_str(fs: FlightStatus) -> &'static str {
    match fs {
        FlightStatus::AirborneNoAlert => "airborne_no_alert",
        FlightStatus::GroundNoAlert => "ground_no_alert",
        FlightStatus::AirborneAlert => "airborne_alert",
        FlightStatus::GroundAlert => "ground_alert",
        FlightStatus::AlertSpi => "alert_spi",
        FlightStatus::Spi => "spi",
        FlightStatus::Reserved => "reserved",
        FlightStatus::NotAssigned => "not_assigned",
    }
}

fn capability_str(cap: Capability) -> &'static str {
    match cap {
        Capability::Level1 => "level1",
        Capability::Level2OnGround => "level2_on_ground",
        Capability::Level2Airborne => "level2_airborne",
        Capability::Level2GroundOrAirborne => "level2_ground_or_airborne",
        Capability::Level2OrAbove => "level2_or_above",
        Capability::Level2WithCommBCapability => "level2_commb",
        Capability::Reserved(_) => "reserved",
    }
}

fn control_field_str(cf: ControlField) -> &'static str {
    match cf {
        ControlField::AdsbIcao => "adsb_icao",
        ControlField::AdsbNonIcao => "adsb_non_icao",
        ControlField::TisbFineFormat => "tisb_fine",
        ControlField::TisbCoarseFormat => "tisb_coarse",
        ControlField::TisbRelay => "tisb_relay",
        ControlField::AdsbRebroadcast => "adsb_rebroadcast",
        ControlField::Reserved(_) => "reserved",
    }
}

fn altitude_source_str(src: AltitudeSource) -> &'static str {
    match src {
        AltitudeSource::Barometric => "barometric",
        AltitudeSource::Gnss => "gnss",
    }
}

fn surveillance_status_str(ss: SurveillanceStatus) -> &'static str {
    match ss {
        SurveillanceStatus::NoCondition => "no_condition",
        SurveillanceStatus::PermanentAlert => "permanent_alert",
        SurveillanceStatus::TemporaryAlert => "temporary_alert",
        SurveillanceStatus::SpiCondition => "spi",
    }
}

fn wake_vortex_str(wv: &WakeVortexCategory) -> &'static str {
    match wv {
        WakeVortexCategory::NoInfo => "no_info",
        WakeVortexCategory::Light => "light",
        WakeVortexCategory::Medium => "medium",
        WakeVortexCategory::Heavy => "heavy",
        WakeVortexCategory::HighPerformance => "high_performance",
        WakeVortexCategory::Rotorcraft => "rotorcraft",
        WakeVortexCategory::Glider => "glider",
        WakeVortexCategory::Lighter => "lighter_than_air",
        WakeVortexCategory::Parachutist => "parachutist",
        WakeVortexCategory::Ultralight => "ultralight",
        WakeVortexCategory::Reserved => "reserved",
        WakeVortexCategory::Uav => "uav",
        WakeVortexCategory::Space => "space",
        WakeVortexCategory::SurfaceEmergency => "surface_emergency",
        WakeVortexCategory::SurfaceService => "surface_service",
        WakeVortexCategory::GroundObstruction => "ground_obstruction",
        WakeVortexCategory::ClusterObstacle => "cluster_obstacle",
        WakeVortexCategory::LineObstacle => "line_obstacle",
        WakeVortexCategory::Unknown(_, _) => "unknown",
    }
}
