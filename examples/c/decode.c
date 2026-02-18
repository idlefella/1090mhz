/*
 * C example for adsb-decoder via the C ABI (libadsb_decoder_ffi).
 *
 * Build and run:
 *   cargo build --release -p adsb-decoder-ffi
 *   gcc -o decode examples/c/decode.c -L target/release -ladsb_decoder_ffi \
 *       -Wl,-rpath,'$ORIGIN/../../target/release'
 *   ./decode
 *
 * Or with an explicit rpath to the repo root:
 *   gcc -o /tmp/decode examples/c/decode.c -L target/release -ladsb_decoder_ffi \
 *       -Wl,-rpath,$PWD/target/release
 *   /tmp/decode
 */

#include <stdint.h>
#include <stdio.h>

/* Opaque handle returned by the library. */
typedef struct DecodedMessage DecodedMessage;

/* C ABI functions exported by libadsb_decoder_ffi. */
DecodedMessage *adsb_decode_hex(const char *hex);
int32_t         adsb_message_df(const DecodedMessage *msg);
uint32_t        adsb_message_icao(const DecodedMessage *msg);
void            adsb_message_free(DecodedMessage *msg);

int main(void) {
    /* DF17 identification message: KLM1023 */
    const char *hex = "8D4840D6202CC371C32CE0576098";

    DecodedMessage *msg = adsb_decode_hex(hex);
    if (!msg) {
        fprintf(stderr, "Failed to decode: %s\n", hex);
        return 1;
    }

    int32_t  df   = adsb_message_df(msg);
    uint32_t icao = adsb_message_icao(msg);

    printf("Hex:  %s\n", hex);
    printf("DF:   %d\n", df);
    printf("ICAO: %06X\n", icao);

    adsb_message_free(msg);

    /* DF11 all-call reply */
    const char *hex2 = "5D4B164513B842";
    msg = adsb_decode_hex(hex2);
    if (msg) {
        printf("\nHex:  %s\n", hex2);
        printf("DF:   %d\n", adsb_message_df(msg));
        printf("ICAO: %06X\n", adsb_message_icao(msg));
        adsb_message_free(msg);
    }

    return 0;
}
