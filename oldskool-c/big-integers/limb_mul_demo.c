// limb_mul_demo.c
// Demonstrate 64x64 -> 128 limb multiply on AArch64:
//  (1) using hardware mul+umulh
//  (2) using a portable 32-bit decomposition (no umulh)
// Includes correctness check + simple timing loop (not a benchmark war).
//
//

#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <inttypes.h>
#include <time.h>

static inline uint64_t ns_now(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
}

// (1) Hardware: use AArch64 mul (low) + umulh (high)
static inline void mul_u64_u128_hw(uint64_t a, uint64_t b, uint64_t *lo, uint64_t *hi) {
#if defined(__aarch64__)
    uint64_t l, h;
    __asm__ volatile(
        "mul   %0, %2, %3\n\t"
        "umulh %1, %2, %3\n\t"
        : "=&r"(l), "=&r"(h)
        : "r"(a), "r"(b)
        : "cc"
    );
    *lo = l;
    *hi = h;
#else
# error "This demo expects AArch64 (__aarch64__)."
#endif
}

// (2) Fallback: 32-bit decomposition (no umulh).
// Computes exact 128-bit product using only 32x32->64 multiplies and adds.
static inline void mul_u64_u128_32x32(uint64_t a, uint64_t b, uint64_t *lo, uint64_t *hi) {
    uint64_t a0 = (uint32_t)a;
    uint64_t a1 = a >> 32;
    uint64_t b0 = (uint32_t)b;
    uint64_t b1 = b >> 32;

    uint64_t p0 = a0 * b0;  // 64-bit
    uint64_t p1 = a0 * b1;  // 64-bit
    uint64_t p2 = a1 * b0;  // 64-bit
    uint64_t p3 = a1 * b1;  // 64-bit

    // Combine:
    // product = p0 + ((p1 + p2) << 32) + (p3 << 64)
    // Use a carry-safe assembly of the middle.
    uint64_t mid = (p0 >> 32) + (uint32_t)p1 + (uint32_t)p2;

    uint64_t lo_out = (p0 & 0xffffffffull) | (mid << 32);
    uint64_t hi_out = p3 + (p1 >> 32) + (p2 >> 32) + (mid >> 32);

    *lo = lo_out;
    *hi = hi_out;
}

static void print_u128(const char *label, uint64_t hi, uint64_t lo) {
    printf("%s 0x%016" PRIx64 "%016" PRIx64 "\n", label, hi, lo);
}

int main(void) {
    // Pick values that exercise carries.
    uint64_t a = 0xfedcba9876543210ull;
    uint64_t b = 0x0123456789abcdefull;

    uint64_t lo1, hi1, lo2, hi2;

    mul_u64_u128_hw(a, b, &lo1, &hi1);
    mul_u64_u128_32x32(a, b, &lo2, &hi2);

    print_u128("hw : ", hi1, lo1);
    print_u128("32x32:", hi2, lo2);

    if (lo1 != lo2 || hi1 != hi2) {
        printf("ERROR: mismatch\n");
        return 1;
    }
    printf("ok: results match\n");

    // Timing loops (keep it simple and honest).
    // Use volatile sinks to discourage full optimization away.
    volatile uint64_t sink = 0;
    const uint64_t iters = 200000000ull;

    uint64_t t0 = ns_now();
    for (uint64_t i = 0; i < iters; i++) {
        uint64_t lo, hi;
        mul_u64_u128_hw(a + i, b, &lo, &hi);
        sink ^= lo ^ hi;
    }
    uint64_t t1 = ns_now();

    uint64_t t2 = ns_now();
    for (uint64_t i = 0; i < iters; i++) {
        uint64_t lo, hi;
        mul_u64_u128_32x32(a + i, b, &lo, &hi);
        sink ^= lo ^ hi;
    }
    uint64_t t3 = ns_now();

    double hw_ns   = (double)(t1 - t0) / (double)iters;
    double ref_ns  = (double)(t3 - t2) / (double)iters;

    printf("timing: hw    %.3f ns/iter\n", hw_ns);
    printf("timing: 32x32 %.3f ns/iter\n", ref_ns);
    if (hw_ns > 0.0) {
        printf("ratio: 32x32/hw = %.2fx\n", ref_ns / hw_ns);
    }
    printf("sink=%" PRIu64 "\n", sink);

    return 0;
}

