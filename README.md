# adsb-decoder

High-performance Mode-S and ADS-B message decoder for 1090 MHz transponders, written in pure Rust with zero external dependencies.

## Features

- Decodes all standard downlink formats: DF 0, 4, 5, 11, 16, 17, 18, 20, 21
- Full ADS-B type code support: identification (TC 1-4), surface position (TC 5-8), airborne position (TC 9-18, 20-22), velocity (TC 19), aircraft status (TC 28), target state (TC 29), operation status (TC 31)
- Mode-S: ACAS, surveillance replies, all-call, Comm-B with BDS registers
- CPR position decoding (global and local)
- CRC-24 validation and ICAO address recovery
- Gillham/Gray code altitude decoding
- Zero dependencies — only the Rust standard library
- JNI/C ABI bindings for Java/Android integration

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
adsb-decoder = { path = "crates/adsb-decoder" }
```

### Decode from a hex string

```rust
use adsb_decoder::{decode_hex, Message, AdsbMessage};
use adsb_decoder::adsb::AdsbPayload;

let msg = decode_hex("8D4840D6202CC371C32CE0576098").unwrap();

match msg {
    Message::AdsbMessage(adsb) => {
        println!("ICAO: {}", adsb.icao);        // "4840D6"
        println!("DF:   {}", adsb.downlink_format); // 17

        match &adsb.payload {
            AdsbPayload::Identification(id) => {
                println!("Callsign: {}", id.callsign_str()); // "KLM1023"
            }
            AdsbPayload::AirbornePosition(pos) => {
                if let Some(alt) = &pos.altitude {
                    println!("Altitude: {} ft", alt.feet);
                }
                // Use pos.cpr with cpr::decode_global() or cpr::decode_local()
                // to resolve lat/lon from even/odd message pairs
            }
            AdsbPayload::AirborneVelocity(vel) => {
                if let Some(vd) = &vel.velocity {
                    println!("Velocity: {:?}", vd);
                }
            }
            _ => {}
        }
    }
    _ => {}
}
```

### Decode from raw bytes

```rust
use adsb_decoder::decode;

let bytes = [
    0x8D, 0x48, 0x40, 0xD6, 0x20, 0x2C, 0xC3, 0x71,
    0xC3, 0x2C, 0xE0, 0x57, 0x60, 0x98,
];
let msg = decode(&bytes).unwrap();
```

### CPR position decoding

Resolve aircraft latitude/longitude from a pair of even and odd position messages:

```rust
use adsb_decoder::cpr;
use adsb_decoder::types::{CprPosition, Position};

let even = CprPosition { lat_cpr: 92095, lon_cpr: 39846, odd_flag: false };
let odd  = CprPosition { lat_cpr: 88385, lon_cpr: 125818, odd_flag: true };

// Global decode from an even/odd pair
let position = cpr::decode_global(&even, &odd, false).unwrap();
println!("Lat: {:.4}, Lon: {:.4}", position.latitude, position.longitude);

// Local decode using a known reference position
let reference = Position { latitude: 52.0, longitude: 3.5 };
let position = cpr::decode_local(&even, &reference).unwrap();
```

### Handling all message types

```rust
use adsb_decoder::{decode_hex, Message};

let msg = decode_hex("5D4B164513B842").unwrap();

match msg {
    Message::AcasSurveillance(acas) => { /* DF 0 */ }
    Message::SurveillanceAltitude(reply) => { /* DF 4 */ }
    Message::SurveillanceIdentity(reply) => { /* DF 5 */ }
    Message::AllCallReply(reply) => {
        println!("ICAO: {}", reply.icao); // DF 11
    }
    Message::AcasCoordination(acas) => { /* DF 16 */ }
    Message::AdsbMessage(adsb) => { /* DF 17 */ }
    Message::TisbMessage(tisb) => { /* DF 18 */ }
    Message::CommBAltitude(comm_b) => { /* DF 20 */ }
    Message::CommBIdentity(comm_b) => { /* DF 21 */ }
}
```

## Project Structure

```
crates/
  adsb-decoder/       Core decoding library
  adsb-decoder-jni/   C ABI / JNI bindings for Java/Android
```

## Building

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo test                   # Run all tests
cargo clippy                 # Lint
```

## License

See [LICENSE](LICENSE) for details.
