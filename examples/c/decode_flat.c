/**
 * Example: Using the flat struct API for zero-copy ADS-B decoding.
 *
 * This approach is faster than JSON because there's no serialization—
 * the struct is written directly by Rust and read directly by C.
 *
 * Compile: gcc -o decode_flat decode_flat.c -L../../target/release -ladsb_decoder_ffi -Wl,-rpath,../../target/release
 * Run: ./decode_flat
 */

#include <stdio.h>
#include <stdint.h>
#include <string.h>

/* Message type discriminant */
typedef enum {
    MSG_TYPE_UNKNOWN = 0,
    MSG_TYPE_ACAS_SURVEILLANCE = 1,
    MSG_TYPE_SURVEILLANCE_ALTITUDE = 2,
    MSG_TYPE_SURVEILLANCE_IDENTITY = 3,
    MSG_TYPE_ALL_CALL_REPLY = 4,
    MSG_TYPE_ACAS_COORDINATION = 5,
    MSG_TYPE_ADSB_MESSAGE = 6,
    MSG_TYPE_TISB_MESSAGE = 7,
    MSG_TYPE_COMM_B_ALTITUDE = 8,
    MSG_TYPE_COMM_B_IDENTITY = 9,
} MessageType;

/* ADS-B payload type discriminant */
typedef enum {
    ADSB_TYPE_NONE = 0,
    ADSB_TYPE_IDENTIFICATION = 1,
    ADSB_TYPE_AIRBORNE_POSITION = 2,
    ADSB_TYPE_SURFACE_POSITION = 3,
    ADSB_TYPE_AIRBORNE_VELOCITY = 4,
    ADSB_TYPE_AIRCRAFT_STATUS = 5,
    ADSB_TYPE_TARGET_STATE = 6,
    ADSB_TYPE_OPERATION_STATUS = 7,
} AdsbType;

/* Velocity type discriminant */
typedef enum {
    VELOCITY_TYPE_NONE = 0,
    VELOCITY_TYPE_GROUND_SPEED = 1,
    VELOCITY_TYPE_AIRSPEED = 2,
} VelocityType;

/* Altitude source */
typedef enum {
    ALTITUDE_SOURCE_UNKNOWN = 0,
    ALTITUDE_SOURCE_BAROMETRIC = 1,
    ALTITUDE_SOURCE_GNSS = 2,
} AltitudeSource;

/* Flight status */
typedef enum {
    FLIGHT_STATUS_AIRBORNE_NO_ALERT = 0,
    FLIGHT_STATUS_GROUND_NO_ALERT = 1,
    FLIGHT_STATUS_AIRBORNE_ALERT = 2,
    FLIGHT_STATUS_GROUND_ALERT = 3,
    FLIGHT_STATUS_ALERT_SPI = 4,
    FLIGHT_STATUS_SPI = 5,
    FLIGHT_STATUS_RESERVED = 6,
    FLIGHT_STATUS_NOT_ASSIGNED = 7,
} FlightStatus;

/* Vertical status */
typedef enum {
    VERTICAL_STATUS_AIRBORNE = 0,
    VERTICAL_STATUS_GROUND = 1,
} VerticalStatus;

/**
 * Flat C-compatible decoded message.
 * This struct matches the Rust FlatMessage exactly.
 */
typedef struct {
    /* Common fields */
    int32_t df;
    MessageType msg_type;
    uint32_t icao;

    /* Altitude */
    int32_t has_altitude;
    int32_t altitude_feet;
    AltitudeSource altitude_source;

    /* Squawk */
    int32_t has_squawk;
    uint16_t squawk;
    uint16_t _pad1;

    /* Flight status */
    int32_t has_flight_status;
    FlightStatus flight_status;

    /* Vertical status */
    int32_t has_vertical_status;
    VerticalStatus vertical_status;

    /* Capability */
    int32_t has_capability;
    uint8_t capability;
    uint8_t _pad2[3];

    /* ADS-B specific */
    AdsbType adsb_type;
    uint8_t type_code;
    uint8_t _pad3[3];

    /* Identification */
    char callsign[9];
    uint8_t _pad4[3];
    uint8_t wake_category;
    uint8_t _pad5[3];

    /* Position */
    int32_t has_cpr;
    int32_t cpr_odd;
    uint32_t cpr_lat;
    uint32_t cpr_lon;
    int32_t utc_sync;

    /* Velocity */
    VelocityType velocity_type;
    double speed_knots;
    double heading_degrees;
    uint16_t airspeed_knots;
    int32_t airspeed_is_tas;
    uint16_t _pad6;

    /* Vertical rate */
    int32_t has_vertical_rate;
    int32_t vertical_rate_fpm;
    AltitudeSource vertical_rate_source;

    /* Surface position */
    int32_t has_ground_speed;
    double ground_speed_knots;
    int32_t has_heading;
    int32_t _pad7;

    /* Raw data */
    uint8_t mb_data[7];
    uint8_t mv_data[7];
    uint8_t _pad8[2];
} FlatMessage;

/* FFI function declarations */
extern int adsb_decode_hex_flat(const char* hex, FlatMessage* out);
extern size_t adsb_flat_message_size(void);

/* Helper to get message type name */
const char* msg_type_name(MessageType t) {
    switch (t) {
        case MSG_TYPE_ACAS_SURVEILLANCE: return "ACAS Surveillance (DF0)";
        case MSG_TYPE_SURVEILLANCE_ALTITUDE: return "Surveillance Altitude (DF4)";
        case MSG_TYPE_SURVEILLANCE_IDENTITY: return "Surveillance Identity (DF5)";
        case MSG_TYPE_ALL_CALL_REPLY: return "All-Call Reply (DF11)";
        case MSG_TYPE_ACAS_COORDINATION: return "ACAS Coordination (DF16)";
        case MSG_TYPE_ADSB_MESSAGE: return "ADS-B (DF17)";
        case MSG_TYPE_TISB_MESSAGE: return "TIS-B (DF18)";
        case MSG_TYPE_COMM_B_ALTITUDE: return "Comm-B Altitude (DF20)";
        case MSG_TYPE_COMM_B_IDENTITY: return "Comm-B Identity (DF21)";
        default: return "Unknown";
    }
}

/* Helper to get ADS-B type name */
const char* adsb_type_name(AdsbType t) {
    switch (t) {
        case ADSB_TYPE_IDENTIFICATION: return "Identification";
        case ADSB_TYPE_AIRBORNE_POSITION: return "Airborne Position";
        case ADSB_TYPE_SURFACE_POSITION: return "Surface Position";
        case ADSB_TYPE_AIRBORNE_VELOCITY: return "Airborne Velocity";
        case ADSB_TYPE_AIRCRAFT_STATUS: return "Aircraft Status";
        case ADSB_TYPE_TARGET_STATE: return "Target State";
        case ADSB_TYPE_OPERATION_STATUS: return "Operation Status";
        default: return "None";
    }
}

void decode_and_print(const char* hex) {
    FlatMessage msg;

    if (adsb_decode_hex_flat(hex, &msg) != 0) {
        printf("Failed to decode: %s\n\n", hex);
        return;
    }

    printf("Hex: %s\n", hex);
    printf("  Type: %s\n", msg_type_name(msg.msg_type));
    printf("  ICAO: %06X\n", msg.icao);
    printf("  DF: %d\n", msg.df);

    if (msg.has_altitude) {
        printf("  Altitude: %d ft (%s)\n", msg.altitude_feet,
               msg.altitude_source == ALTITUDE_SOURCE_BAROMETRIC ? "baro" : "gnss");
    }

    if (msg.has_squawk) {
        printf("  Squawk: %04o\n", msg.squawk);
    }

    if (msg.adsb_type != ADSB_TYPE_NONE) {
        printf("  ADS-B Type: %s (TC=%d)\n", adsb_type_name(msg.adsb_type), msg.type_code);
    }

    if (msg.adsb_type == ADSB_TYPE_IDENTIFICATION) {
        printf("  Callsign: %s\n", msg.callsign);
    }

    if (msg.has_cpr) {
        printf("  CPR: %s frame, lat=%u, lon=%u\n",
               msg.cpr_odd ? "odd" : "even", msg.cpr_lat, msg.cpr_lon);
    }

    if (msg.velocity_type == VELOCITY_TYPE_GROUND_SPEED) {
        printf("  Ground Speed: %.1f kts, Heading: %.1f deg\n",
               msg.speed_knots, msg.heading_degrees);
    } else if (msg.velocity_type == VELOCITY_TYPE_AIRSPEED) {
        printf("  Airspeed: %d kts (%s), Heading: %.1f deg\n",
               msg.airspeed_knots,
               msg.airspeed_is_tas ? "TAS" : "IAS",
               msg.heading_degrees);
    }

    if (msg.has_vertical_rate) {
        printf("  Vertical Rate: %d fpm\n", msg.vertical_rate_fpm);
    }

    printf("\n");
}

int main() {
    printf("FlatMessage struct size: %zu bytes (C) vs %zu bytes (Rust)\n\n",
           sizeof(FlatMessage), adsb_flat_message_size());

    /* Test messages */
    decode_and_print("8D4840D6202CC371C32CE0576098");  /* Identification: KLM1023 */
    decode_and_print("8D485020994409940838175B284F");  /* Velocity */
    decode_and_print("8D40621D58C382D690C8AC2863A7");  /* Position (even) */
    decode_and_print("8D40621D58C386435CC412692AD6");  /* Position (odd) */
    decode_and_print("5D4B164513B842");                /* All-call reply (DF11) */

    return 0;
}
