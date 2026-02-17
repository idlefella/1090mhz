/*
 * Java example for adsb-decoder via JNA (Java Native Access).
 *
 * Build the shared library first:
 *   cargo build --release -p adsb-decoder-jni
 *
 * Compile and run (requires jna.jar on the classpath):
 *   javac -cp jna.jar examples/java/AdsbExample.java
 *   java -cp jna.jar:examples/java -Djna.library.path=target/release AdsbExample
 */

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

public class AdsbExample {

    /** JNA interface mapping the C ABI functions. */
    public interface AdsbLib extends Library {
        Pointer adsb_decode_hex(String hex);
        int     adsb_message_df(Pointer msg);
        int     adsb_message_icao(Pointer msg);
        void    adsb_message_free(Pointer msg);
    }

    public static void main(String[] args) {
        AdsbLib lib = Native.load("adsb_decoder_jni", AdsbLib.class);

        // DF17 identification message: KLM1023
        decode(lib, "8D4840D6202CC371C32CE0576098");
        System.out.println();

        // DF11 all-call reply
        decode(lib, "5D4B164513B842");
    }

    static void decode(AdsbLib lib, String hex) {
        Pointer msg = lib.adsb_decode_hex(hex);
        if (msg == null) {
            System.err.println("Failed to decode: " + hex);
            return;
        }

        int df   = lib.adsb_message_df(msg);
        int icao = lib.adsb_message_icao(msg);

        System.out.printf("Hex:  %s%n", hex);
        System.out.printf("DF:   %d%n", df);
        System.out.printf("ICAO: %06X%n", icao);

        lib.adsb_message_free(msg);
    }
}
