/*
 * C example for adsb-decoder: read hex messages from stdin, output JSON.
 *
 * Reads Mode-S/ADS-B hex messages line by line from stdin and outputs
 * decoded messages as JSON, one per line.
 *
 * Supports common input formats:
 *   - Raw hex: 8D4840D6202CC371C32CE0576098
 *   - AVR format: *8D4840D6202CC371C32CE0576098;
 *   - With leading @ timestamp: @000012345678...hex...;
 *
 * Build:
 *   cargo build --release -p adsb-decoder-ffi
 *   gcc -o stdin_json examples/c/stdin_json.c -L target/release -ladsb_decoder_ffi
 *
 * Run (from repository root):
 *   echo "8D4840D6202CC371C32CE0576098" | LD_LIBRARY_PATH=target/release ./stdin_json
 *
 * Or with rpath for standalone use:
 *   gcc -o stdin_json examples/c/stdin_json.c -L target/release -ladsb_decoder_ffi \
 *       -Wl,-rpath,$PWD/target/release
 *
 * Pipe from dump1090 (AVR format on port 30002):
 *   nc localhost 30002 | LD_LIBRARY_PATH=target/release ./stdin_json
 */

#include <ctype.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Opaque handle returned by the library. */
typedef struct DecodedMessage DecodedMessage;

/* C ABI functions exported by libadsb_decoder_ffi. */
DecodedMessage *adsb_decode_hex(const char *hex);
char           *adsb_message_json(const DecodedMessage *msg);
void            adsb_message_free(DecodedMessage *msg);
void            adsb_string_free(char *s);

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

int main(void) {
    char line[256];
    int decoded = 0;
    int failed = 0;

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

        /* Get JSON representation */
        char *json = adsb_message_json(msg);
        if (json) {
            printf("%s\n", json);
            fflush(stdout);
            adsb_string_free(json);
            decoded++;
        }

        adsb_message_free(msg);
    }

    fprintf(stderr, "Decoded: %d, Failed: %d\n", decoded, failed);
    return 0;
}
