//! JNI bindings for the ADS-B decoder library.
//!
//! Provides a C-ABI interface for calling from Java via JNI.
//! Uses an opaque handle pattern: decode returns a handle,
//! getter functions extract individual fields.

use std::ffi::CStr;
use std::os::raw::c_char;

use adsb_decoder::frame::{self, Message};

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
