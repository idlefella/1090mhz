//! FFI bindings for the ADS-B decoder library.
//!
//! Provides a C ABI interface for calling from C, Python, Java, and other languages.
//!
//! Two API styles are available:
//! 1. **Opaque handle pattern**: decode returns a handle, getter functions extract fields.
//! 2. **Flat struct pattern**: decode writes directly to a C-compatible struct (zero-copy).

mod flat;
mod json;

pub use flat::{
    AdsbType, AltitudeSourceC, FlatMessage, FlightStatusC, MessageType, VelocityType,
    VerticalStatusC,
};

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use adsb_decoder::adsb::AdsbPayload;
use adsb_decoder::cpr;
use adsb_decoder::frame::{self, Message};
use adsb_decoder::types::CprPosition;

/// Opaque handle to a decoded message.
/// The caller receives this as an integer and passes it back to getter functions.
pub struct DecodedMessage {
    pub message: Message,
}

/// Decode a hex-encoded Mode-S message.
///
/// Returns a pointer to a `DecodedMessage` on success, or null on failure.
/// The caller must free the returned pointer with `adsb_message_free`.
///
/// # Safety
/// `hex` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn adsb_decode_hex(hex: *const c_char) -> *mut DecodedMessage {
    if hex.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(hex) };
    let hex_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match frame::decode_hex(hex_str) {
        Ok(message) => Box::into_raw(Box::new(DecodedMessage { message })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Decode a raw byte buffer.
///
/// Returns a pointer to a `DecodedMessage` on success, or null on failure.
/// The caller must free the returned pointer with `adsb_message_free`.
///
/// # Safety
/// `data` must point to at least `len` valid bytes.
#[no_mangle]
pub unsafe extern "C" fn adsb_decode_bytes(
    data: *const u8,
    len: usize,
) -> *mut DecodedMessage {
    if data.is_null() || len == 0 {
        return std::ptr::null_mut();
    }

    let bytes = unsafe { std::slice::from_raw_parts(data, len) };

    match frame::decode(bytes) {
        Ok(message) => Box::into_raw(Box::new(DecodedMessage { message })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Get the downlink format of a decoded message.
///
/// # Safety
/// `msg` must be a valid pointer returned by `adsb_decode_hex` or `adsb_decode_bytes`.
#[no_mangle]
pub unsafe extern "C" fn adsb_message_df(msg: *const DecodedMessage) -> i32 {
    if msg.is_null() {
        return -1;
    }

    let msg = unsafe { &*msg };
    match &msg.message {
        Message::AcasSurveillance(_) => 0,
        Message::SurveillanceAltitude(_) => 4,
        Message::SurveillanceIdentity(_) => 5,
        Message::AllCallReply(_) => 11,
        Message::AcasCoordination(_) => 16,
        Message::AdsbMessage(_) => 17,
        Message::TisbMessage(_) => 18,
        Message::CommBAltitude(_) => 20,
        Message::CommBIdentity(_) => 21,
    }
}

/// Get the ICAO address as a 24-bit integer.
///
/// # Safety
/// `msg` must be a valid pointer returned by `adsb_decode_hex` or `adsb_decode_bytes`.
#[no_mangle]
pub unsafe extern "C" fn adsb_message_icao(msg: *const DecodedMessage) -> u32 {
    if msg.is_null() {
        return 0;
    }

    let msg = unsafe { &*msg };
    match &msg.message {
        Message::AcasSurveillance(m) => m.icao.0,
        Message::SurveillanceAltitude(m) => m.icao.0,
        Message::SurveillanceIdentity(m) => m.icao.0,
        Message::AllCallReply(m) => m.icao.0,
        Message::AcasCoordination(m) => m.icao.0,
        Message::AdsbMessage(m) => m.icao.0,
        Message::TisbMessage(m) => m.icao.0,
        Message::CommBAltitude(m) => m.icao.0,
        Message::CommBIdentity(m) => m.icao.0,
    }
}

/// Get the JSON representation of a decoded message.
///
/// Returns a pointer to a null-terminated JSON string on success, or null on failure.
/// The caller must free the returned string with `adsb_string_free`.
///
/// # Safety
/// `msg` must be a valid pointer returned by `adsb_decode_hex` or `adsb_decode_bytes`.
#[no_mangle]
pub unsafe extern "C" fn adsb_message_json(msg: *const DecodedMessage) -> *mut c_char {
    if msg.is_null() {
        return std::ptr::null_mut();
    }

    let msg = unsafe { &*msg };
    let json = json::message_to_json(&msg.message);

    match CString::new(json) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string returned by `adsb_message_json`.
///
/// # Safety
/// `s` must be a valid pointer returned by `adsb_message_json`,
/// or null (in which case this is a no-op).
#[no_mangle]
pub unsafe extern "C" fn adsb_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

/// Free a decoded message.
///
/// # Safety
/// `msg` must be a valid pointer returned by `adsb_decode_hex` or `adsb_decode_bytes`,
/// or null (in which case this is a no-op).
#[no_mangle]
pub unsafe extern "C" fn adsb_message_free(msg: *mut DecodedMessage) {
    if !msg.is_null() {
        unsafe {
            drop(Box::from_raw(msg));
        }
    }
}

/// Check if a message contains CPR position data.
/// Returns 1 if the message contains airborne position data, 0 otherwise.
///
/// # Safety
/// `msg` must be a valid pointer returned by `adsb_decode_hex` or `adsb_decode_bytes`.
#[no_mangle]
pub unsafe extern "C" fn adsb_message_has_cpr(msg: *const DecodedMessage) -> i32 {
    if msg.is_null() {
        return 0;
    }

    let msg = unsafe { &*msg };
    match &msg.message {
        Message::AdsbMessage(m) | Message::TisbMessage(m) => {
            matches!(m.payload, AdsbPayload::AirbornePosition(_))
        }
        _ => false,
    }
    .into()
}

/// Get the CPR odd/even flag from an airborne position message.
/// Returns 1 for odd, 0 for even, -1 if not a position message.
///
/// # Safety
/// `msg` must be a valid pointer returned by `adsb_decode_hex` or `adsb_decode_bytes`.
#[no_mangle]
pub unsafe extern "C" fn adsb_message_cpr_odd(msg: *const DecodedMessage) -> i32 {
    if msg.is_null() {
        return -1;
    }

    let msg = unsafe { &*msg };
    match &msg.message {
        Message::AdsbMessage(m) | Message::TisbMessage(m) => match &m.payload {
            AdsbPayload::AirbornePosition(pos) => pos.cpr.odd_flag.into(),
            _ => -1,
        },
        _ => -1,
    }
}

/// Get the raw CPR latitude value from an airborne position message.
/// Returns the 17-bit latitude value, or 0xFFFFFFFF if not a position message.
///
/// # Safety
/// `msg` must be a valid pointer returned by `adsb_decode_hex` or `adsb_decode_bytes`.
#[no_mangle]
pub unsafe extern "C" fn adsb_message_cpr_lat(msg: *const DecodedMessage) -> u32 {
    if msg.is_null() {
        return 0xFFFFFFFF;
    }

    let msg = unsafe { &*msg };
    match &msg.message {
        Message::AdsbMessage(m) | Message::TisbMessage(m) => match &m.payload {
            AdsbPayload::AirbornePosition(pos) => pos.cpr.lat_cpr,
            _ => 0xFFFFFFFF,
        },
        _ => 0xFFFFFFFF,
    }
}

/// Get the raw CPR longitude value from an airborne position message.
/// Returns the 17-bit longitude value, or 0xFFFFFFFF if not a position message.
///
/// # Safety
/// `msg` must be a valid pointer returned by `adsb_decode_hex` or `adsb_decode_bytes`.
#[no_mangle]
pub unsafe extern "C" fn adsb_message_cpr_lon(msg: *const DecodedMessage) -> u32 {
    if msg.is_null() {
        return 0xFFFFFFFF;
    }

    let msg = unsafe { &*msg };
    match &msg.message {
        Message::AdsbMessage(m) | Message::TisbMessage(m) => match &m.payload {
            AdsbPayload::AirbornePosition(pos) => pos.cpr.lon_cpr,
            _ => 0xFFFFFFFF,
        },
        _ => 0xFFFFFFFF,
    }
}

/// Decoded global position result.
#[repr(C)]
pub struct GlobalPosition {
    pub valid: i32,
    pub latitude: f64,
    pub longitude: f64,
}

/// Decode a global position from even and odd CPR position pairs.
///
/// Parameters:
/// - `even_lat`: CPR latitude from even message (17-bit)
/// - `even_lon`: CPR longitude from even message (17-bit)
/// - `odd_lat`: CPR latitude from odd message (17-bit)
/// - `odd_lon`: CPR longitude from odd message (17-bit)
/// - `most_recent_odd`: 1 if the odd message is more recent, 0 if even is more recent
///
/// Returns a GlobalPosition struct with valid=1 on success, valid=0 on failure.
#[no_mangle]
pub extern "C" fn adsb_cpr_decode_global(
    even_lat: u32,
    even_lon: u32,
    odd_lat: u32,
    odd_lon: u32,
    most_recent_odd: i32,
) -> GlobalPosition {
    let even = CprPosition {
        lat_cpr: even_lat,
        lon_cpr: even_lon,
        odd_flag: false,
    };
    let odd = CprPosition {
        lat_cpr: odd_lat,
        lon_cpr: odd_lon,
        odd_flag: true,
    };

    match cpr::decode_global(&even, &odd, most_recent_odd != 0) {
        Ok(pos) => GlobalPosition {
            valid: 1,
            latitude: pos.latitude,
            longitude: pos.longitude,
        },
        Err(_) => GlobalPosition {
            valid: 0,
            latitude: 0.0,
            longitude: 0.0,
        },
    }
}

// =============================================================================
// Flat struct API (zero-copy, no serialization)
// =============================================================================

/// Decode a hex-encoded Mode-S message into a flat C struct.
///
/// Returns 0 on success, -1 on failure.
/// The result is written directly to the provided `out` pointer.
///
/// # Safety
/// - `hex` must be a valid null-terminated C string.
/// - `out` must be a valid pointer to a `FlatMessage` struct.
#[no_mangle]
pub unsafe extern "C" fn adsb_decode_hex_flat(
    hex: *const c_char,
    out: *mut FlatMessage,
) -> i32 {
    if hex.is_null() || out.is_null() {
        return -1;
    }

    let c_str = unsafe { CStr::from_ptr(hex) };
    let hex_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match frame::decode_hex(hex_str) {
        Ok(message) => {
            unsafe {
                *out = FlatMessage::from_message(&message);
            }
            0
        }
        Err(_) => -1,
    }
}

/// Decode a raw byte buffer into a flat C struct.
///
/// Returns 0 on success, -1 on failure.
/// The result is written directly to the provided `out` pointer.
///
/// # Safety
/// - `data` must point to at least `len` valid bytes.
/// - `out` must be a valid pointer to a `FlatMessage` struct.
#[no_mangle]
pub unsafe extern "C" fn adsb_decode_bytes_flat(
    data: *const u8,
    len: usize,
    out: *mut FlatMessage,
) -> i32 {
    if data.is_null() || len == 0 || out.is_null() {
        return -1;
    }

    let bytes = unsafe { std::slice::from_raw_parts(data, len) };

    match frame::decode(bytes) {
        Ok(message) => {
            unsafe {
                *out = FlatMessage::from_message(&message);
            }
            0
        }
        Err(_) => -1,
    }
}

/// Get the size of the FlatMessage struct.
///
/// Useful for allocating memory from C or for verifying struct layout compatibility.
#[no_mangle]
pub extern "C" fn adsb_flat_message_size() -> usize {
    std::mem::size_of::<FlatMessage>()
}
