/**
 * Example: Using the flat struct API for zero-copy ADS-B decoding in Java.
 *
 * This approach is faster than JSON because JNA directly maps the C struct.
 *
 * Compile: javac -cp .:jna.jar AdsbFlatExample.java
 * Run: java -cp .:../../jna.jar -Djna.library.path=../../target/release AdsbFlatExample
 */

import com.sun.jna.*;
import com.sun.jna.ptr.*;
import java.util.Arrays;
import java.util.List;

public class AdsbFlatExample {

    /**
     * Flat C-compatible decoded message.
     * This struct matches the Rust FlatMessage exactly.
     */
    public static class FlatMessage extends Structure {
        // Common fields
        public int df;
        public int msg_type;
        public int icao;

        // Altitude
        public int has_altitude;
        public int altitude_feet;
        public int altitude_source;

        // Squawk
        public int has_squawk;
        public short squawk;
        public short _pad1;

        // Flight status
        public int has_flight_status;
        public int flight_status;

        // Vertical status
        public int has_vertical_status;
        public int vertical_status;

        // Capability
        public int has_capability;
        public byte capability;
        public byte[] _pad2 = new byte[3];

        // ADS-B specific
        public int adsb_type;
        public byte type_code;
        public byte[] _pad3 = new byte[3];

        // Identification
        public byte[] callsign = new byte[9];
        public byte[] _pad4 = new byte[3];
        public byte wake_category;
        public byte[] _pad5 = new byte[3];

        // Position
        public int has_cpr;
        public int cpr_odd;
        public int cpr_lat;
        public int cpr_lon;
        public int utc_sync;

        // Velocity
        public int velocity_type;
        public double speed_knots;
        public double heading_degrees;
        public short airspeed_knots;
        public int airspeed_is_tas;
        public short _pad6;

        // Vertical rate
        public int has_vertical_rate;
        public int vertical_rate_fpm;
        public int vertical_rate_source;

        // Surface position
        public int has_ground_speed;
        public double ground_speed_knots;
        public int has_heading;
        public int _pad7;

        // Raw data
        public byte[] mb_data = new byte[7];
        public byte[] mv_data = new byte[7];
        public byte[] _pad8 = new byte[2];

        @Override
        protected List<String> getFieldOrder() {
            return Arrays.asList(
                "df", "msg_type", "icao",
                "has_altitude", "altitude_feet", "altitude_source",
                "has_squawk", "squawk", "_pad1",
                "has_flight_status", "flight_status",
                "has_vertical_status", "vertical_status",
                "has_capability", "capability", "_pad2",
                "adsb_type", "type_code", "_pad3",
                "callsign", "_pad4", "wake_category", "_pad5",
                "has_cpr", "cpr_odd", "cpr_lat", "cpr_lon", "utc_sync",
                "velocity_type", "speed_knots", "heading_degrees",
                "airspeed_knots", "airspeed_is_tas", "_pad6",
                "has_vertical_rate", "vertical_rate_fpm", "vertical_rate_source",
                "has_ground_speed", "ground_speed_knots", "has_heading", "_pad7",
                "mb_data", "mv_data", "_pad8"
            );
        }

        public String getCallsignString() {
            int len = 0;
            for (int i = 0; i < callsign.length && callsign[i] != 0; i++) {
                len++;
            }
            return new String(callsign, 0, len).trim();
        }

        public String getMsgTypeName() {
            switch (msg_type) {
                case 1: return "ACAS Surveillance (DF0)";
                case 2: return "Surveillance Altitude (DF4)";
                case 3: return "Surveillance Identity (DF5)";
                case 4: return "All-Call Reply (DF11)";
                case 5: return "ACAS Coordination (DF16)";
                case 6: return "ADS-B (DF17)";
                case 7: return "TIS-B (DF18)";
                case 8: return "Comm-B Altitude (DF20)";
                case 9: return "Comm-B Identity (DF21)";
                default: return "Unknown";
            }
        }

        public String getAdsbTypeName() {
            switch (adsb_type) {
                case 1: return "Identification";
                case 2: return "Airborne Position";
                case 3: return "Surface Position";
                case 4: return "Airborne Velocity";
                case 5: return "Aircraft Status";
                case 6: return "Target State";
                case 7: return "Operation Status";
                default: return "None";
            }
        }
    }

    /**
     * JNA interface to the ADS-B decoder FFI library.
     */
    public interface AdsbLib extends Library {
        AdsbLib INSTANCE = Native.load("adsb_decoder_ffi", AdsbLib.class);

        int adsb_decode_hex_flat(String hex, FlatMessage out);
        NativeLong adsb_flat_message_size();
    }

    public static void decodeAndPrint(String hex) {
        FlatMessage msg = new FlatMessage();

        if (AdsbLib.INSTANCE.adsb_decode_hex_flat(hex, msg) != 0) {
            System.out.println("Failed to decode: " + hex + "\n");
            return;
        }

        System.out.println("Hex: " + hex);
        System.out.println("  Type: " + msg.getMsgTypeName());
        System.out.printf("  ICAO: %06X%n", msg.icao);
        System.out.println("  DF: " + msg.df);

        if (msg.has_altitude != 0) {
            String source = msg.altitude_source == 1 ? "baro" : "gnss";
            System.out.printf("  Altitude: %d ft (%s)%n", msg.altitude_feet, source);
        }

        if (msg.has_squawk != 0) {
            System.out.printf("  Squawk: %04o%n", msg.squawk & 0xFFFF);
        }

        if (msg.adsb_type != 0) {
            System.out.printf("  ADS-B Type: %s (TC=%d)%n", msg.getAdsbTypeName(), msg.type_code);
        }

        if (msg.adsb_type == 1) { // Identification
            System.out.println("  Callsign: " + msg.getCallsignString());
        }

        if (msg.has_cpr != 0) {
            String frame = msg.cpr_odd != 0 ? "odd" : "even";
            System.out.printf("  CPR: %s frame, lat=%d, lon=%d%n", frame, msg.cpr_lat, msg.cpr_lon);
        }

        if (msg.velocity_type == 1) { // Ground speed
            System.out.printf("  Ground Speed: %.1f kts, Heading: %.1f deg%n",
                msg.speed_knots, msg.heading_degrees);
        } else if (msg.velocity_type == 2) { // Airspeed
            String type = msg.airspeed_is_tas != 0 ? "TAS" : "IAS";
            System.out.printf("  Airspeed: %d kts (%s), Heading: %.1f deg%n",
                msg.airspeed_knots, type, msg.heading_degrees);
        }

        if (msg.has_vertical_rate != 0) {
            System.out.printf("  Vertical Rate: %d fpm%n", msg.vertical_rate_fpm);
        }

        System.out.println();
    }

    public static void main(String[] args) {
        // Verify struct size matches
        long rustSize = AdsbLib.INSTANCE.adsb_flat_message_size().longValue();
        FlatMessage msg = new FlatMessage();
        int javaSize = msg.size();
        System.out.printf("FlatMessage struct size: %d bytes (Java) vs %d bytes (Rust)%n%n",
            javaSize, rustSize);

        if (javaSize != rustSize) {
            System.err.println("WARNING: Struct size mismatch! Layout may be incorrect.");
        }

        // Test messages
        decodeAndPrint("8D4840D6202CC371C32CE0576098");  // Identification: KLM1023
        decodeAndPrint("8D485020994409940838175B284F");  // Velocity
        decodeAndPrint("8D40621D58C382D690C8AC2863A7");  // Position (even)
        decodeAndPrint("8D40621D58C386435CC412692AD6");  // Position (odd)
        decodeAndPrint("5D4B164513B842");                // All-call reply (DF11)
    }
}
