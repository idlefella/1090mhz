/*
 * C example for adsb-decoder: read hex messages from stdin, decode positions.
 *
 * Reads Mode-S/ADS-B hex messages line by line from stdin and outputs
 * decoded messages as JSON. For airborne position messages, tracks even/odd
 * CPR pairs per aircraft and decodes global lat/lon when both are available.
 *
 * Supports common input formats:
 *   - Raw hex: 8D4840D6202CC371C32CE0576098
 *   - AVR format: *8D4840D6202CC371C32CE0576098;
 *   - With leading @ timestamp: @000012345678...hex...;
 *
 * Build:
 *   cargo build --release -p adsb-decoder-ffi
 *   gcc -o stdin_json_position examples/c/stdin_json_position.c \
 *       -L target/release -ladsb_decoder_ffi -lm
 *
 * Run (from repository root):
 *   echo "8D40621D58C382D690C8AC2863A7" | LD_LIBRARY_PATH=target/release ./stdin_json_position
 *
 * Or with rpath for standalone use:
 *   gcc -o stdin_json_position examples/c/stdin_json_position.c \
 *       -L target/release -ladsb_decoder_ffi -lm \
 *       -Wl,-rpath,$PWD/target/release
 *
 * Pipe from dump1090 (AVR format on port 30002):
 *   nc localhost 30002 | LD_LIBRARY_PATH=target/release ./stdin_json_position
 */

#include <ctype.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* Maximum number of aircraft to track simultaneously. */
#define MAX_AIRCRAFT 256

/* Maximum age in seconds before discarding a CPR position. */
#define CPR_MAX_AGE_SEC 10

/* Opaque handle returned by the library. */
typedef struct DecodedMessage DecodedMessage;

/* Global position result from CPR decoding. */
typedef struct {
    int valid;
    double latitude;
    double longitude;
} GlobalPosition;

/* C ABI functions exported by libadsb_decoder_ffi. */
DecodedMessage *adsb_decode_hex(const char *hex);
char           *adsb_message_json(const DecodedMessage *msg);
uint32_t        adsb_message_icao(const DecodedMessage *msg);
int32_t         adsb_message_df(const DecodedMessage *msg);
int32_t         adsb_message_has_cpr(const DecodedMessage *msg);
int32_t         adsb_message_cpr_odd(const DecodedMessage *msg);
uint32_t        adsb_message_cpr_lat(const DecodedMessage *msg);
uint32_t        adsb_message_cpr_lon(const DecodedMessage *msg);
GlobalPosition  adsb_cpr_decode_global(uint32_t even_lat, uint32_t even_lon,
                                        uint32_t odd_lat, uint32_t odd_lon,
                                        int32_t most_recent_odd);
void            adsb_message_free(DecodedMessage *msg);
void            adsb_string_free(char *s);

/* Aircraft CPR state tracking. */
typedef struct {
    uint32_t icao;
    /* Even position */
    uint32_t even_lat;
    uint32_t even_lon;
    time_t   even_time;
    int      has_even;
    /* Odd position */
    uint32_t odd_lat;
    uint32_t odd_lon;
    time_t   odd_time;
    int      has_odd;
} AircraftState;

/* Global aircraft state table. */
static AircraftState aircraft[MAX_AIRCRAFT];
static int num_aircraft = 0;

/* Find or create aircraft state entry. */
static AircraftState *get_aircraft(uint32_t icao) {
    /* Search for existing entry */
    for (int i = 0; i < num_aircraft; i++) {
        if (aircraft[i].icao == icao) {
            return &aircraft[i];
        }
    }
    /* Create new entry if space available */
    if (num_aircraft < MAX_AIRCRAFT) {
        AircraftState *ac = &aircraft[num_aircraft++];
        memset(ac, 0, sizeof(*ac));
        ac->icao = icao;
        return ac;
    }
    /* Table full, reuse oldest entry */
    return &aircraft[0];
}

/* Strip leading/trailing whitespace, AVR framing (*...;), and @ timestamps. */
static char *strip_hex(char *line) {
    /* Skip leading whitespace */
    while (*line && isspace((unsigned char)*line)) {
        line++;
    }

    /* Handle @ timestamp prefix: @XXXXXXXXXXXX...hex...; */
    if (*line == '@') {
        line++;
        /* Skip 12 hex digits of timestamp */
        for (int i = 0; i < 12 && *line && isxdigit((unsigned char)*line); i++) {
            line++;
        }
    }
    /* Handle AVR * prefix */
    else if (*line == '*') {
        line++;
    }

    /* Find the end of hex digits */
    char *end = line;
    while (*end && (isxdigit((unsigned char)*end))) {
        end++;
    }
    *end = '\0';

    return line;
}

/* Output JSON with optional position. */
static void output_json(const char *json, uint32_t icao,
                        int has_position, double lat, double lon) {
    if (!has_position) {
        printf("%s\n", json);
    } else {
        /* Insert position before the closing brace */
        size_t len = strlen(json);
        if (len > 0 && json[len-1] == '}') {
            printf("%.*s,\"position\":{\"latitude\":%.6f,\"longitude\":%.6f}}\n",
                   (int)(len-1), json, lat, lon);
        } else {
            printf("%s\n", json);
        }
    }
    fflush(stdout);
}

int main(void) {
    char line[256];
    int decoded = 0;
    int failed = 0;
    int positions_decoded = 0;

    while (fgets(line, sizeof(line), stdin)) {
        /* Strip framing and whitespace */
        char *hex = strip_hex(line);

        /* Skip empty lines */
        size_t len = strlen(hex);
        if (len == 0) {
            continue;
        }

        /* Skip lines that are too short (minimum 14 hex chars = 7 bytes) */
        if (len < 14) {
            continue;
        }

        /* Decode the message */
        DecodedMessage *msg = adsb_decode_hex(hex);
        if (!msg) {
            failed++;
            continue;
        }

        uint32_t icao = adsb_message_icao(msg);
        int has_position = 0;
        double lat = 0.0, lon = 0.0;

        /* Check if this is a position message */
        if (adsb_message_has_cpr(msg)) {
            AircraftState *ac = get_aircraft(icao);
            time_t now = time(NULL);
            int is_odd = adsb_message_cpr_odd(msg);
            uint32_t cpr_lat = adsb_message_cpr_lat(msg);
            uint32_t cpr_lon = adsb_message_cpr_lon(msg);

            if (is_odd) {
                ac->odd_lat = cpr_lat;
                ac->odd_lon = cpr_lon;
                ac->odd_time = now;
                ac->has_odd = 1;
            } else {
                ac->even_lat = cpr_lat;
                ac->even_lon = cpr_lon;
                ac->even_time = now;
                ac->has_even = 1;
            }

            /* Try to decode global position if we have both even and odd */
            if (ac->has_even && ac->has_odd) {
                /* Check that both positions are recent enough */
                time_t even_age = now - ac->even_time;
                time_t odd_age = now - ac->odd_time;

                if (even_age <= CPR_MAX_AGE_SEC && odd_age <= CPR_MAX_AGE_SEC) {
                    /* Determine which message is more recent */
                    int most_recent_odd = (ac->odd_time >= ac->even_time);

                    GlobalPosition gpos = adsb_cpr_decode_global(
                        ac->even_lat, ac->even_lon,
                        ac->odd_lat, ac->odd_lon,
                        most_recent_odd
                    );

                    if (gpos.valid) {
                        has_position = 1;
                        lat = gpos.latitude;
                        lon = gpos.longitude;
                        positions_decoded++;
                    }
                }
            }
        }

        /* Get JSON representation and output */
        char *json = adsb_message_json(msg);
        if (json) {
            output_json(json, icao, has_position, lat, lon);
            adsb_string_free(json);
            decoded++;
        }

        adsb_message_free(msg);
    }

    fprintf(stderr, "Decoded: %d, Positions: %d, Failed: %d\n",
            decoded, positions_decoded, failed);
    return 0;
}
