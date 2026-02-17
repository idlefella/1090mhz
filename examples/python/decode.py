#!/usr/bin/env python3
"""
Python example for adsb-decoder via ctypes (libadsb_decoder_jni).

Build the shared library first:
    cargo build --release -p adsb-decoder-jni

Run:
    LD_LIBRARY_PATH=target/release python3 examples/python/decode.py
"""

import ctypes
import os
import sys

# Locate the shared library.
lib_name = "libadsb_decoder_jni.so"
lib_dir = os.path.join(os.path.dirname(__file__), "..", "..", "target", "release")
lib_path = os.path.join(lib_dir, lib_name)

if not os.path.isfile(lib_path):
    # Fall back to LD_LIBRARY_PATH / system search.
    lib_path = lib_name

lib = ctypes.cdll.LoadLibrary(lib_path)

# Set up function signatures for type safety.
lib.adsb_decode_hex.argtypes = [ctypes.c_char_p]
lib.adsb_decode_hex.restype = ctypes.c_void_p

lib.adsb_message_df.argtypes = [ctypes.c_void_p]
lib.adsb_message_df.restype = ctypes.c_int32

lib.adsb_message_icao.argtypes = [ctypes.c_void_p]
lib.adsb_message_icao.restype = ctypes.c_uint32

lib.adsb_message_free.argtypes = [ctypes.c_void_p]
lib.adsb_message_free.restype = None


def decode(hex_str: str) -> None:
    msg = lib.adsb_decode_hex(hex_str.encode("ascii"))
    if not msg:
        print(f"Failed to decode: {hex_str}", file=sys.stderr)
        return

    df = lib.adsb_message_df(msg)
    icao = lib.adsb_message_icao(msg)
    print(f"Hex:  {hex_str}")
    print(f"DF:   {df}")
    print(f"ICAO: {icao:06X}")

    lib.adsb_message_free(msg)


if __name__ == "__main__":
    # DF17 identification message: KLM1023
    decode("8D4840D6202CC371C32CE0576098")
    print()

    # DF11 all-call reply
    decode("5D4B164513B842")
