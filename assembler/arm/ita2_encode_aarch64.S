/* ita2_encode_aarch64.S
 *
 * Build:
 *   as -o ita2_encode_aarch64.o ita2_encode_aarch64.S
 *   ld -o ita2_encode_aarch64 ita2_encode_aarch64.o
 *
 * Run:
 *   ./ita2_encode_aarch64 input.txt output.ita2bin
 */

    .arch armv8-a
    .text
    .global _start

/* Linux AArch64 syscall numbers */
    .equ SYS_read,     63
    .equ SYS_write,    64
    .equ SYS_openat,   56
    .equ SYS_close,    57
    .equ SYS_exit,     93

    .equ AT_FDCWD,   -100
    .equ O_RDONLY,    0
    .equ O_WRONLY,    1
    .equ O_CREAT,     64
    .equ O_TRUNC,     512
    .equ MODE_644,    420      /* 0644 */

/* ITA2 shift symbols */
    .equ ITA2_FIGS,   27       /* 11011b */
    .equ ITA2_LTRS,   31       /* 11111b */

_start:
    /* Stack layout:
       sp+0   = argc
       sp+8   = argv[0]
       sp+16  = argv[1]  (input)
       sp+24  = argv[2]  (output)
     */

    ldr x0, [sp]              /* argc */
    cmp x0, #3
    b.ge args_ok
    /* Not enough args → exit(2) */
    mov x0, #2
    mov x8, #SYS_exit
    svc #0

args_ok:
    /* argv[1] = input path */
    ldr x1, [sp, #16]

    /* fd_in = openat(AT_FDCWD, argv[1], O_RDONLY, 0) */
    mov x0, #AT_FDCWD
    mov x2, #O_RDONLY
    mov x3, #0
    mov x8, #SYS_openat
    svc #0
    cmp x0, #0
    b.ge opened_input
    mov x0, #1
    mov x8, #SYS_exit
    svc #0

opened_input:
    mov x19, x0               /* x19 = input fd */

    /* argv[2] = output path */
    ldr x1, [sp, #24]

    /* fd_out = openat(AT_FDCWD, argv[2], O_WRONLY|O_CREAT|O_TRUNC, 0644) */
    mov x0, #AT_FDCWD
    mov x2, #(O_WRONLY | O_CREAT | O_TRUNC)
    mov x3, #MODE_644
    mov x8, #SYS_openat
    svc #0
    cmp x0, #0
    b.ge opened_output
    mov x0, #1
    mov x8, #SYS_exit
    svc #0

opened_output:
    mov x18, x0               /* x18 = output fd */

    /* bit_buffer (x20) = 0
       bit_count  (x21) = 0
       state      (x22) = 0 (LTRS)
     */
    mov x20, #0
    mov x21, #0
    mov x22, #0               /* 0=LTRS, 1=FIGS */

read_loop:
    /* read one byte from input */
    mov x0, x19
    adrp x1, inbyte
    add  x1, x1, :lo12:inbyte
    mov x2, #1
    mov x8, #SYS_read
    svc #0
    cmp x0, #1
    b.ne flush_and_exit      /* EOF or error */

    /* load ASCII byte */
    adrp x3, inbyte
    add  x3, x3, :lo12:inbyte
    ldrb w4, [x3]             /* w4 = ASCII char */

    /* map ASCII → ITA2 symbol */
    mov w0, w4
    bl  ascii_to_ita2        /* returns symbol in w0, shift_needed in w1 */

    /* If shift_needed != 0, emit shift symbol first */
    cbz w1, no_shift
    /* w1 = 1 means FIGS, 2 means LTRS */
    cmp w1, #1
    b.ne emit_ltrs

emit_figs:
    mov w5, #ITA2_FIGS
    bl  emit_symbol
    mov x22, #1               /* state = FIGS */
    b no_shift

emit_ltrs:
    mov w5, #ITA2_LTRS
    bl  emit_symbol
    mov x22, #0               /* state = LTRS */

no_shift:
    /* Emit the actual character symbol */
    mov w5, w0
    bl  emit_symbol
    b read_loop

flush_and_exit:
    /* If leftover bits exist, pad and flush one last byte */
    cmp x21, #0
    beq close_files

    /* Pad with zeros to 8 bits */
    mov x6, #8
    sub x6, x6, x21
    lsl x20, x20, x6
    mov x21, #8

    bl flush_byte

close_files:
    /* close input */
    mov x0, x19
    mov x8, #SYS_close
    svc #0

    /* close output */
    mov x0, x18
    mov x8, #SYS_close
    svc #0

    /* exit(0) */
    mov x0, #0
    mov x8, #SYS_exit
    svc #0

/* -------------------------------------------------------------
 * emit_symbol
 *   Input: w5 = 5-bit symbol to emit
 *   Uses:  x20 = bit_buffer, x21 = bit_count
 * ------------------------------------------------------------- */
emit_symbol:
    /* Shift buffer left 5 bits, OR in new symbol */
    lsl x20, x20, #5
    orr x20, x20, x5
    add x21, x21, #5

emit_loop:
    cmp x21, #8
    b.lt emit_done

    /* Extract top 8 bits and write */
    mov x6, x21
    sub x6, x6, #8
    lsr x7, x20, x6
    and x7, x7, #0xff

    /* write byte */
    adrp x9, outbyte
    add  x9, x9, :lo12:outbyte
    strb w7, [x9]

    mov x0, x18
    mov x1, x9
    mov x2, #1
    mov x8, #SYS_write
    svc #0

    sub x21, x21, #8
    b emit_loop

emit_done:
    ret

/* -------------------------------------------------------------
 * flush_byte
 *   Writes the top 8 bits from buffer
 * ------------------------------------------------------------- */
flush_byte:
    lsr x7, x20, #0
    and x7, x7, #0xff

    adrp x9, outbyte
    add  x9, x9, :lo12:outbyte
    strb w7, [x9]

    mov x0, x18
    mov x1, x9
    mov x2, #1
    mov x8, #SYS_write
    svc #0
    ret

/* -------------------------------------------------------------
 * ascii_to_ita2
 *   Input:  w0 = ASCII char
 *   Output: w0 = ITA2 symbol (0..31)
 *           w1 = shift_needed (0 = none, 1 = FIGS, 2 = LTRS)
 * ------------------------------------------------------------- */
ascii_to_ita2:
    /* Default: no shift */
    mov w1, #0

    /* Uppercase letters */
    cmp w0, #'A'
    b.lt check_digits
    cmp w0, #'Z'
    b.gt check_digits

    /* If currently FIGS, request LTRS shift */
    cmp x22, #1
    b.ne letter_ok
    mov w1, #2                /* need LTRS */

letter_ok:
    /* table lookup: ita2_ltrs_index[char - 'A'] */
    sub w2, w0, #'A'
    adrp x3, ita2_ltrs_index
    add  x3, x3, :lo12:ita2_ltrs_index
    ldrb w0, [x3, x2]
    ret

check_digits:
    cmp w0, #'0'
    b.lt check_space
    cmp w0, #'9'
    b.gt check_space

    /* If currently LTRS, request FIGS shift */
    cmp x22, #0
    b.ne digit_ok
    mov w1, #1                /* need FIGS */

digit_ok:
    sub w2, w0, #'0'
    adrp x3, ita2_figs_index
    add  x3, x3, :lo12:ita2_figs_index
    ldrb w0, [x3, x2]
    ret

check_space:
    cmp w0, #' '
    b.ne unknown_char

    /* SPACE is same in both modes */
    mov w0, #4                /* 00100b */
    ret

unknown_char:
    /* Map unknowns to SPACE for now */
    mov w0, #4
    ret

    .data
    .balign 16
inbyte:
    .byte 0
outbyte:
    .byte 0

/* -------------------------------------------------------------
 * Index tables
 *   Maps ASCII → ITA2 symbol number
 * ------------------------------------------------------------- */

    .balign 16
ita2_ltrs_index:
    /* A..Z */
    .byte 3   /* A */
    .byte 25  /* B */
    .byte 14  /* C */
    .byte 9   /* D */
    .byte 1   /* E */
    .byte 13  /* F */
    .byte 26  /* G */
    .byte 20  /* H */
    .byte 6   /* I */
    .byte 11  /* J */
    .byte 15  /* K */
    .byte 18  /* L */
    .byte 28  /* M */
    .byte 12  /* N */
    .byte 24  /* O */
    .byte 22  /* P */
    .byte 23  /* Q */
    .byte 10  /* R */
    .byte 5   /* S */
    .byte 16  /* T */
    .byte 7   /* U */
    .byte 30  /* V */
    .byte 19  /* W */
    .byte 29  /* X */
    .byte 21  /* Y */
    .byte 17  /* Z */

    .balign 16
ita2_figs_index:
    /* 0..9 */
    .byte 22  /* 0 */
    .byte 23  /* 1 */
    .byte 19  /* 2 */
    .byte 1   /* 3 */
    .byte 10  /* 4 */
    .byte 16  /* 5 */
    .byte 21  /* 6 */
    .byte 7   /* 7 */
    .byte 6   /* 8 */
    .byte 24  /* 9 */
