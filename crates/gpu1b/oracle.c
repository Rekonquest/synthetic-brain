/* GPU-1b oracle: GCC's __float128 (libgcc soft-float), an existing independent implementation.
 * stdin: records of 33 bytes = op, a (16 bytes LE), b (16 bytes LE). stdout: 16 bytes per record.
 * op 0 add, 1 sub, 2 mul, 3 compare (byte 0 = a<b, byte 1 = a==b), 4 narrow a to double (8 bytes),
 * 5 widen the double in a's low 8 bytes. */
#include <stdio.h>
#include <string.h>
int main(void) {
    unsigned char in[33], out[16];
    while (fread(in, 1, 33, stdin) == 33) {
        __float128 a, b, r;
        memcpy(&a, in + 1, 16);
        memcpy(&b, in + 17, 16);
        memset(out, 0, 16);
        switch (in[0]) {
        case 0: r = a + b; memcpy(out, &r, 16); break;
        case 1: r = a - b; memcpy(out, &r, 16); break;
        case 2: r = a * b; memcpy(out, &r, 16); break;
        case 3: out[0] = a < b; out[1] = a == b; break;
        case 4: { double d = (double)a; memcpy(out, &d, 8); } break;
        case 5: { double d; memcpy(&d, in + 1, 8); r = (__float128)d; memcpy(out, &r, 16); } break;
        default: return 2;
        }
        if (fwrite(out, 1, 16, stdout) != 16) return 1;
    }
    return 0;
}
