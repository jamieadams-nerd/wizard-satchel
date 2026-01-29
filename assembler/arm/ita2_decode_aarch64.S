/* ita2_decode_aarch64.S
 *
 * Build (native ARM64 Linux):
 *   as -o ita2_decode_aarch64.o ita2_decode_aarch64.S
 *   ld -o ita2_decode_aarch64 ita2_decode_aarch64.o
 *
 * Build (on my iMac)
 *   clang -target aarch64-unknown-linux-gnu  -c ita2_decode_aarch64.S -o ita2_decode_aarch64.o
 *
 * Run:
 *   ./ita2_decode_aarch64 demo.ita2bin
 */

    .arch armv8-a
    .text
    .global _start

/* Linux AArch64 syscall numbers */
    .equ SYS_read,    63
    .equ SYS_write,   64
    .equ SYS_openat,  56
    .equ SYS_close,   57
    .equ SYS_exit,    93

    .equ AT_FDCWD,   -100
    .equ O_RDONLY,    0

/* ITA2 shift symbols (common) */
    .equ ITA2_FIGS,   27      /* 11011b */
    .equ ITA2_LTRS,   31      /* 11111b */

_start:
    /* On Linux, initial stack:
       sp -> argc
             argv[0]
             argv[1]
             ...
     */

    ldr x0, [sp]              /* argc */
    cmp x0, #2
    b.ge 1f
    /* argc < 2: exit(2) */
    mov x0, #2
    mov x8, #SYS_exit
    svc #0

1:
    /* argv[1] pointer is at sp + 16 (argc at +0, argv0 at +8, argv1 at +16) */
    ldr x1, [sp, #16]         /* x1 = path */

    /* fd = openat(AT_FDCWD, path, O_RDONLY, 0) */
    mov x0, #AT_FDCWD
    mov x2, #O_RDONLY
    mov x3, #0
    mov x8, #SYS_openat
    svc #0
    /* x0 = fd or -errno */
    cmp x0, #0
    b.ge 2f
    /* open failed: exit(1) */
    mov x0, #1
    mov x8, #SYS_exit
    svc #0

2:
    mov x19, x0               /* x19 = fd */

    /* bit_buffer = 0 (x20), bit_count = 0 (x21), state = 0(LTRS) in x22 */
    mov x20, #0               /* bit_buffer (up to 64 bits is plenty) */
    mov x21, #0               /* bit_count */
    mov x22, #0               /* 0=LTRS, 1=FIGS */

    /* read loop */
read_more:
    /* n = read(fd, &inbyte, 1) */
    mov x0, x19
    adr x1, inbyte 
    /* adrp x1, inbyte
    add  x1, x1, :lo12:inbyte
    */

    mov x2, #1
    mov x8, #SYS_read
    svc #0
    /* x0 = bytes read */
    cmp x0, #1
    b.ne done                 /* 0=EOF, <0=error treated as done */

    /* load byte */
    adr x3, inbyte
    /*
    adrp x3, inbyte
    add  x3, x3, :lo12:inbyte
    */

    ldrb w4, [x3]             /* w4 = byte */

    /* bit_buffer = (bit_buffer << 8) | byte */
    lsl x20, x20, #8
    orr x20, x20, x4
    add x21, x21, #8          /* bit_count += 8 */

extract_symbols:
    /* while (bit_count >= 5) */
    cmp x21, #5
    b.lt read_more

    /* shift = bit_count - 5 */
    sub x5, x21, #5

    /* symbol = (bit_buffer >> shift) & 0x1F */
    lsr x6, x20, x5
    and x6, x6, #0x1f         /* x6 = symbol (0..31) */

    /* bit_count -= 5 */
    sub x21, x21, #5

    /* Handle shift codes */
    cmp x6, #ITA2_LTRS
    b.ne 3f
    mov x22, #0               /* LTRS state */
    b extract_symbols

3:
    cmp x6, #ITA2_FIGS
    b.ne 4f
    mov x22, #1               /* FIGS state */
    b extract_symbols

4:
    /* Lookup glyph from table based on state */
    /* table = (state==0)? ita2_ltrs : ita2_figs */
    cmp x22, #0
    b.ne use_figs

use_ltrs:
    adr x7, ita2_ltrs
    /*
    adrp x7, ita2_ltrs
    add  x7, x7, :lo12:ita2_ltrs
    */

    b lookup

use_figs:
    adr x7, ita2_figs
/*
    adrp x7, ita2_figs
    add  x7, x7, :lo12:ita2_figs
    */

lookup:
    /* glyph = table[symbol] */
    ldrb w8, [x7, x6]

    /* If glyph == 0, ignore (e.g., NUL) */
    cbz w8, extract_symbols

    /* Write glyph to stdout */
    adr x9, outch
    /*
    adrp x9, outch
    add  x9, x9, :lo12:outch
    */
    strb w8, [x9]

    mov x0, #1                /* stdout */
    mov x1, x9
    mov x2, #1
    mov x8, #SYS_write
    svc #0

    b extract_symbols

done:
    /* close(fd) */
    mov x0, x19
    mov x8, #SYS_close
    svc #0

    /* exit(0) */
    mov x0, #0
    mov x8, #SYS_exit
    svc #0

    .data
    .balign 16
inbyte:
    .byte 0
outch:
    .byte 0

/* --------------------------------------------------------------------
 * ITA2 tables (32 bytes each)
 * Index = 5-bit symbol 0..31
 * Value = ASCII byte to emit (0 = ignore/handled)
 *
 * IMPORTANT: Replace with YOUR canonical mapping from your JSON spec.
 * -------------------------------------------------------------------- */

    .balign 16
ita2_ltrs:
    .byte 0      /*  0 NUL */
    .byte 'E'    /*  1 */
    .byte 10     /*  2 LF */
    .byte 'A'    /*  3 */
    .byte ' '    /*  4 SPACE */
    .byte 'S'    /*  5 */
    .byte 'I'    /*  6 */
    .byte 'U'    /*  7 */
    .byte 13     /*  8 CR */
    .byte 'D'    /*  9 */
    .byte 'R'    /* 10 */
    .byte 'J'    /* 11 */
    .byte 'N'    /* 12 */
    .byte 'F'    /* 13 */
    .byte 'C'    /* 14 */
    .byte 'K'    /* 15 */
    .byte 'T'    /* 16 */
    .byte 'Z'    /* 17 */
    .byte 'L'    /* 18 */
    .byte 'W'    /* 19 */
    .byte 'H'    /* 20 */
    .byte 'Y'    /* 21 */
    .byte 'P'    /* 22 */
    .byte 'Q'    /* 23 */
    .byte 'O'    /* 24 */
    .byte 'B'    /* 25 */
    .byte 'G'    /* 26 */
    .byte 0      /* 27 FIGS (handled) */
    .byte 'M'    /* 28 */
    .byte 'X'    /* 29 */
    .byte 'V'    /* 30 */
    .byte 0      /* 31 LTRS (handled) */

    .balign 16
ita2_figs:
    .byte 0      /*  0 NUL */
    .byte '3'    /*  1 */
    .byte 10     /*  2 LF */
    .byte '-'    /*  3 */
    .byte ' '    /*  4 SPACE */
    .byte '\''   /*  5 (often apostrophe) */
    .byte '8'    /*  6 */
    .byte '7'    /*  7 */
    .byte 13     /*  8 CR */
    .byte '$'    /*  9 */
    .byte '4'    /* 10 */
    .byte 0      /* 11 (BEL or other in some variants) */
    .byte ','    /* 12 */
    .byte '!'    /* 13 */
    .byte ':'    /* 14 */
    .byte '('    /* 15 */
    .byte '5'    /* 16 */
    .byte '"'    /* 17 */
    .byte ')'    /* 18 */
    .byte '2'    /* 19 */
    .byte '#'    /* 20 (or other; variant) */
    .byte '6'    /* 21 */
    .byte '0'    /* 22 */
    .byte '1'    /* 23 */
    .byte '9'    /* 24 */
    .byte '?'    /* 25 */
    .byte '&'    /* 26 */
    .byte 0      /* 27 FIGS (handled) */
    .byte '.'    /* 28 */
    .byte '/'    /* 29 */
    .byte ';'    /* 30 */
    .byte 0      /* 31 LTRS (handled) */

