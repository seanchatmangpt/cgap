/* AB-5 interop oracle: liboqs ML-KEM-768 (FIPS 203 final).
 * Usage:
 *   oracle keypair                 -> "<ek_hex>|<dk_hex>"
 *   oracle enc <ek_hex>            -> "<ct_hex>|<ss_hex>"
 *   oracle dec <dk_hex> <ct_hex>   -> "<ss_hex>"
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <oqs/oqs.h>

static int hexin(const char *hex, uint8_t *out, size_t n) {
    if (strlen(hex) < 2 * n) return 0;
    for (size_t i = 0; i < n; i++) {
        unsigned v;
        if (sscanf(hex + 2 * i, "%2x", &v) != 1) return 0;
        out[i] = (uint8_t)v;
    }
    return 1;
}

static void hexout(const uint8_t *b, size_t n) {
    for (size_t i = 0; i < n; i++) printf("%02x", b[i]);
}

int main(int argc, char **argv) {
    if (argc < 2) return 2;
    if (strcmp(argv[1], "keypair") == 0 && argc == 2) {
        uint8_t ek[1184], dk[2400];
        if (OQS_KEM_ml_kem_768_keypair(ek, dk) != OQS_SUCCESS) return 3;
        hexout(ek, 1184); printf("|"); hexout(dk, 2400); printf("\n");
        return 0;
    }
    if (strcmp(argv[1], "enc") == 0 && argc == 3) {
        uint8_t ek[1184], ct[1088], ss[32];
        if (!hexin(argv[2], ek, 1184)) return 4;
        if (OQS_KEM_ml_kem_768_encaps(ct, ss, ek) != OQS_SUCCESS) return 3;
        hexout(ct, 1088); printf("|"); hexout(ss, 32); printf("\n");
        return 0;
    }
    if (strcmp(argv[1], "dec") == 0 && argc == 4) {
        uint8_t dk[2400], ct[1088], ss[32];
        if (!hexin(argv[2], dk, 2400)) return 4;
        if (!hexin(argv[3], ct, 1088)) return 4;
        if (OQS_KEM_ml_kem_768_decaps(ss, ct, dk) != OQS_SUCCESS) return 3;
        hexout(ss, 32); printf("\n");
        return 0;
    }
    return 2;
}
