#pragma once
#include <emmintrin.h>
#include <stddef.h>
#include <stdint.h>

static const uint8_t BYTE_RANK[256] = {
    255, 248, 243, 229, 239, 244, 182, 183, 230, 159, 121, 107, 198, 221, 112, 249,
    231, 215,  80,  93, 170, 200,  94,  82, 204,  76,  60,  50, 118, 104,  49, 187,
    219,  78,  35,  39, 250, 129,  41,  38, 224, 193,  52, 110,  92,  57, 151,  36,
    212, 209,  62, 125,  99,  75,  25,  43, 211, 214,  47, 174, 101, 177,  28,  61,
    220, 241, 141, 167, 235, 216, 181, 168, 254, 238, 150, 128, 245, 213, 155, 134,
    223, 133,  83, 126, 189, 135, 169, 195, 203, 142,  29, 119, 175, 109, 176, 172,
    166,  24,  20,  59,  87, 115, 226,  15, 156, 117,   9,  44, 122,  16,  45,  96,
    171,  33,  98,  77, 242, 218, 105, 111, 208, 100,  79,  53, 160,  51,  95, 138,
    210, 201,  81, 234, 232, 240, 186, 145, 192, 252,  56, 253, 163, 246, 149, 137,
    178,  26,  63,  11, 148,  40,  85,  22, 127,  21,   4,  10,  73,   3,  65,  19,
    130,  66,   1,   7,  30,   2,  14,   6, 120,   5,  72,  17,  55,   0,  13,  67,
    144,  68,   8,  18,  97,  12, 188,  84, 185, 146, 164,  69, 102,  34, 123,  64,
    236, 227, 179, 197, 217, 233, 202, 225, 196, 205, 165, 108, 247,  86, 131, 103,
    190, 153, 191, 113,  54,  70,  71,  91, 161, 147, 114, 132,  23,  27,  31,  42,
    180, 136, 154,  88,  74,  37,  46,  48, 237, 222, 140, 207, 152, 106,  58,  90,
    206, 173, 158, 139,  32,  89, 162, 143, 228, 199, 184, 116, 124, 157, 194, 251,
};

static inline int pick_anchors(const int *pat, int n, int *a1, int *a2)
{
    *a1 = *a2 = -1;
    for (int i = 0; i < n; i++) {
        if (pat[i] < 0)
            continue;
        if (*a1 < 0 || BYTE_RANK[pat[i]] < BYTE_RANK[pat[*a1]])
            *a2 = *a1, *a1 = i;
        else if (*a2 < 0 || BYTE_RANK[pat[i]] < BYTE_RANK[pat[*a2]])
            *a2 = i;
    }
    if (*a1 < 0)
        return 0;
    if (*a2 < 0)
        *a2 = *a1;
    return 1;
}

static inline int pattern_hit(const uint8_t *c, const int *pat, int n)
{
    int i = 0;
    while (i < n && (pat[i] < 0 || c[i] == pat[i]))
        i++;
    return i == n;
}

static inline int scan_range(const uint8_t *lo, const uint8_t *hi, const int *pat, int n, int a1, int a2, uintptr_t *first)
{
    int hits = 0;
    if (hi - lo < n)
        return 0;
    size_t span = (size_t)(hi - lo) - n + 1, i = 0;
    const __m128i v1 = _mm_set1_epi8((char)pat[a1]), v2 = _mm_set1_epi8((char)pat[a2]);
    for (; i + 16 <= span; i += 16) {
        __m128i x1 = _mm_loadu_si128((const __m128i *)(lo + i + a1));
        __m128i x2 = _mm_loadu_si128((const __m128i *)(lo + i + a2));
        unsigned m = (unsigned)_mm_movemask_epi8(_mm_and_si128(_mm_cmpeq_epi8(x1, v1), _mm_cmpeq_epi8(x2, v2)));
        while (m) {
            const uint8_t *c = lo + i + __builtin_ctz(m);
            m &= m - 1;
            if (pattern_hit(c, pat, n) && hits++ == 0)
                *first = (uintptr_t)c;
        }
    }
    for (; i < span; i++)
        if (pattern_hit(lo + i, pat, n) && hits++ == 0)
            *first = (uintptr_t)(lo + i);
    return hits;
}
