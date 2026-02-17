//! # adsb-decoder
//!
//! High-performance Mode-S and ADS-B message decoder for 1090 MHz.
//!
//! Supports all downlink formats (DF 0, 4, 5, 11, 16, 17, 18, 20, 21)
//! and all ADS-B type codes (TC 1-4, 5-8, 9-18, 19, 20-22, 28, 29, 31).
//!
//! ## Quick Start
//!
//! ```rust
//! use adsb_decoder::frame;
//!
//! // Decode from hex string
//! let msg = frame::decode_hex("8D4840D6202CC371C32CE0576098").unwrap();
//! println!("{:?}", msg);
//!
//! // Decode from raw bytes
//! let bytes = [0x8D, 0x48, 0x40, 0xD6, 0x20, 0x2C, 0xC3, 0x71,
//!              0xC3, 0x2C, 0xE0, 0x57, 0x60, 0x98];
//! let msg = frame::decode(&bytes).unwrap();
//! ```

pub mod adsb;
pub mod altitude;
pub mod cpr;
pub mod crc;
pub mod frame;
pub mod modes;
pub mod types;

// Re-export key types for convenience.
pub use frame::{decode, decode_hex, AdsbMessage, Message};
pub use types::{
    Altitude, AltitudeSource, Capability, CprPosition, DecodeError, DecodeResult, FlightStatus,
    IcaoAddress, Position, Squawk,
};
