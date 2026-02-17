// Runnable example for adsb-decoder.
//
// cargo run --example decode -p adsb-decoder

use adsb_decoder::adsb::AdsbPayload;
use adsb_decoder::cpr;
use adsb_decoder::{decode_hex, Message};

fn main() {
    // --- 1. Decode a DF17 identification message (callsign) ---
    let msg = decode_hex("8D4840D6202CC371C32CE0576098").unwrap();
    if let Message::AdsbMessage(ref adsb) = msg {
        println!("DF:   {}", adsb.downlink_format);
        println!("ICAO: {}", adsb.icao);
        if let AdsbPayload::Identification(ref id) = adsb.payload {
            println!("Callsign: {}", id.callsign_str());
        }
    }
    println!();

    // --- 2. Decode a DF11 all-call reply ---
    let msg = decode_hex("5D4B164513B842").unwrap();
    if let Message::AllCallReply(ref ac) = msg {
        println!("DF11 All-Call — ICAO: {}, Capability: {:?}", ac.icao, ac.capability);
    }
    println!();

    // --- 3. CPR global position decoding from an even/odd pair ---
    let even = decode_hex("8D40621D58C382D690C8AC2863A7").unwrap();
    let odd = decode_hex("8D40621D58C386435CC412692AD6").unwrap();

    if let (
        Message::AdsbMessage(ref e),
        Message::AdsbMessage(ref o),
    ) = (&even, &odd)
    {
        if let (
            AdsbPayload::AirbornePosition(ref ep),
            AdsbPayload::AirbornePosition(ref op),
        ) = (&e.payload, &o.payload)
        {
            let pos = cpr::decode_global(&ep.cpr, &op.cpr, false).unwrap();
            println!(
                "ICAO {} position: {:.4}N, {:.4}E",
                e.icao, pos.latitude, pos.longitude
            );
            if let Some(alt) = &ep.altitude {
                println!("Altitude: {} ft ({:?})", alt.feet, alt.source);
            }
        }
    }
    println!();

    // --- 4. Airborne velocity ---
    let msg = decode_hex("8D485020994409940838175B284F").unwrap();
    if let Message::AdsbMessage(ref adsb) = msg {
        if let AdsbPayload::AirborneVelocity(ref vel) = adsb.payload {
            if let Some(ref v) = vel.velocity {
                match v {
                    adsb_decoder::adsb::velocity::VelocityData::GroundSpeed {
                        speed_knots,
                        heading_degrees,
                        ..
                    } => {
                        println!(
                            "ICAO {} ground speed: {:.1} kts, heading: {:.1} deg",
                            adsb.icao, speed_knots, heading_degrees
                        );
                    }
                    adsb_decoder::adsb::velocity::VelocityData::Airspeed {
                        airspeed_knots,
                        heading_degrees,
                        ..
                    } => {
                        println!(
                            "ICAO {} airspeed: {} kts, heading: {:.1} deg",
                            adsb.icao, airspeed_knots, heading_degrees
                        );
                    }
                }
            }
            if let Some(vr) = &vel.vertical_rate {
                println!(
                    "Vertical rate: {} ft/min ({:?})",
                    vr.feet_per_minute, vr.source
                );
            }
        }
    }
}
