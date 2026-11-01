#pragma once
#include <cmath>
#include <cstdint>
#include <cstring>
#include <string>
#include <vector>

namespace hud::png {

inline uint32_t crc32(const uint8_t *p, size_t n, uint32_t c = 0xffffffffu)
{
    static uint32_t table[256];
    static bool made;
    if (!made) {
        for (uint32_t i = 0; i < 256; i++) {
            uint32_t v = i;
            for (int k = 0; k < 8; k++)
                v = v & 1 ? 0xedb88320u ^ (v >> 1) : v >> 1;
            table[i] = v;
        }
        made = true;
    }
    for (size_t i = 0; i < n; i++)
        c = table[(c ^ p[i]) & 0xff] ^ (c >> 8);
    return c;
}

struct Bits {
    std::vector<uint8_t> out;
    uint32_t acc = 0;
    int n = 0;

    void put(uint32_t v, int bits)
    {
        acc |= v << n;
        n += bits;
        while (n >= 8) {
            out.push_back(uint8_t(acc));
            acc >>= 8;
            n -= 8;
        }
    }

    void put_rev(uint32_t code, int bits)
    {
        uint32_t r = 0;
        for (int i = 0; i < bits; i++)
            r |= ((code >> i) & 1) << (bits - 1 - i);
        put(r, bits);
    }

    void flush()
    {
        if (n)
            out.push_back(uint8_t(acc));
        acc = 0;
        n = 0;
    }
};

inline void lit(Bits &b, int v)
{
    if (v < 144)
        b.put_rev(0x30 + v, 8);
    else if (v < 256)
        b.put_rev(0x190 + v - 144, 9);
    else if (v < 280)
        b.put_rev(v - 256, 7);
    else
        b.put_rev(0xc0 + v - 280, 8);
}

inline void match(Bits &b, int len, int dist)
{
    static const int lbase[] = {3,  4,  5,  6,  7,  8,  9,  10, 11,  13,  15,  17,  19,  23, 27,
                                31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258};
    static const int lextra[] = {0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0};
    static const int dbase[] = {1,   2,   3,   4,   5,   7,    9,    13,   17,   25,   33,   49,   65,    97,    129,
                                193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577};
    static const int dextra[] = {0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13};
    int li = 28;
    while (lbase[li] > len)
        li--;
    lit(b, 257 + li);
    if (lextra[li])
        b.put(uint32_t(len - lbase[li]), lextra[li]);
    int di = 29;
    while (dbase[di] > dist)
        di--;
    b.put_rev(uint32_t(di), 5);
    if (dextra[di])
        b.put(uint32_t(dist - dbase[di]), dextra[di]);
}

inline std::vector<uint8_t> deflate(const uint8_t *p, size_t n)
{
    constexpr int HBITS = 15, WINDOW = 32768, MAXLEN = 258;
    std::vector<int32_t> head(size_t(1) << HBITS, -1);
    Bits b;
    b.out.reserve(n / 2 + 64);
    b.put(1, 1);
    b.put(1, 2);
    size_t i = 0;
    while (i < n) {
        int best = 0, dist = 0;
        if (i + 3 <= n) {
            uint32_t h = ((uint32_t(p[i]) << 16 | uint32_t(p[i + 1]) << 8 | p[i + 2]) * 2654435761u) >> (32 - HBITS);
            int32_t cand = head[h];
            head[h] = int32_t(i);
            if (cand >= 0 && i - size_t(cand) <= WINDOW) {
                size_t lim = std::min<size_t>(MAXLEN, n - i);
                size_t k = 0;
                while (k < lim && p[size_t(cand) + k] == p[i + k])
                    k++;
                if (k >= 3)
                    best = int(k), dist = int(i - size_t(cand));
            }
        }
        if (best) {
            match(b, best, dist);
            for (size_t j = i + 1; j < i + size_t(best) && j + 3 <= n; j++) {
                uint32_t h = ((uint32_t(p[j]) << 16 | uint32_t(p[j + 1]) << 8 | p[j + 2]) * 2654435761u) >> (32 - HBITS);
                head[h] = int32_t(j);
            }
            i += size_t(best);
        } else {
            lit(b, p[i]);
            i++;
        }
    }
    lit(b, 256);
    b.flush();
    return std::move(b.out);
}

inline uint32_t adler32(const uint8_t *p, size_t n)
{
    uint32_t a = 1, c = 0;
    for (size_t i = 0; i < n; i++) {
        a = (a + p[i]) % 65521;
        c = (c + a) % 65521;
    }
    return c << 16 | a;
}

inline void be32(std::string &s, uint32_t v)
{
    s += char(v >> 24), s += char(v >> 16), s += char(v >> 8), s += char(v);
}

inline void chunk(std::string &out, const char *type, const uint8_t *data, size_t n)
{
    be32(out, uint32_t(n));
    size_t at = out.size();
    out.append(type, 4);
    out.append(reinterpret_cast<const char *>(data), n);
    be32(out, crc32(reinterpret_cast<const uint8_t *>(out.data() + at), n + 4) ^ 0xffffffffu);
}

inline std::string encode_rgb(const uint8_t *rgb, uint32_t w, uint32_t h)
{
    std::vector<uint8_t> raw;
    raw.reserve(size_t(h) * (size_t(w) * 3 + 1));
    for (uint32_t y = 0; y < h; y++) {
        const uint8_t *row = rgb + size_t(y) * w * 3;
        raw.push_back(1);
        for (uint32_t x = 0; x < w * 3; x++)
            raw.push_back(uint8_t(row[x] - (x >= 3 ? row[x - 3] : 0)));
    }
    std::vector<uint8_t> z = {0x78, 0x01};
    std::vector<uint8_t> d = deflate(raw.data(), raw.size());
    z.insert(z.end(), d.begin(), d.end());
    uint32_t ad = adler32(raw.data(), raw.size());
    z.push_back(uint8_t(ad >> 24)), z.push_back(uint8_t(ad >> 16)), z.push_back(uint8_t(ad >> 8)), z.push_back(uint8_t(ad));
    std::string out("\x89PNG\r\n\x1a\n", 8);
    uint8_t ihdr[13] = {uint8_t(w >> 24), uint8_t(w >> 16), uint8_t(w >> 8), uint8_t(w),
                        uint8_t(h >> 24), uint8_t(h >> 16), uint8_t(h >> 8), uint8_t(h),
                        8,                2,                0,               0,
                        0};
    chunk(out, "IHDR", ihdr, 13);
    chunk(out, "IDAT", z.data(), z.size());
    chunk(out, "IEND", nullptr, 0);
    return out;
}

inline float half(uint16_t h)
{
    uint32_t s = (h >> 15) & 1, e = (h >> 10) & 0x1f, m = h & 0x3ff;
    float v = e == 0 ? std::ldexp(float(m), -24) : e == 31 ? 65504.0f : std::ldexp(float(m | 0x400), int(e) - 25);
    return s ? -v : v;
}

inline uint8_t srgb8(float linear)
{
    float c = linear <= 0 ? 0 : linear >= 1 ? 1 : linear;
    c = c <= 0.0031308f ? c * 12.92f : 1.055f * std::pow(c, 1 / 2.4f) - 0.055f;
    return uint8_t(c * 255.0f + 0.5f);
}

enum class Fmt { rgba8, bgra8, rgb10a2, rgba16f, unknown };

inline Fmt fmt_of(int dxgi)
{
    switch (dxgi) {
    case 27:
    case 28:
    case 29:
        return Fmt::rgba8;
    case 87:
    case 90:
    case 91:
        return Fmt::bgra8;
    case 24:
        return Fmt::rgb10a2;
    case 10:
        return Fmt::rgba16f;
    default:
        return Fmt::unknown;
    }
}

inline bool to_rgb(Fmt f, const uint8_t *src, uint32_t pitch, uint32_t w, uint32_t h, std::vector<uint8_t> &rgb)
{
    if (f == Fmt::unknown)
        return false;
    rgb.resize(size_t(w) * h * 3);
    for (uint32_t y = 0; y < h; y++) {
        const uint8_t *row = src + size_t(y) * pitch;
        uint8_t *o = rgb.data() + size_t(y) * w * 3;
        for (uint32_t x = 0; x < w; x++, o += 3) {
            switch (f) {
            case Fmt::rgba8:
                o[0] = row[x * 4], o[1] = row[x * 4 + 1], o[2] = row[x * 4 + 2];
                break;
            case Fmt::bgra8:
                o[0] = row[x * 4 + 2], o[1] = row[x * 4 + 1], o[2] = row[x * 4];
                break;
            case Fmt::rgb10a2: {
                uint32_t v;
                memcpy(&v, row + x * 4, 4);
                o[0] = uint8_t((v & 1023) >> 2), o[1] = uint8_t(((v >> 10) & 1023) >> 2), o[2] = uint8_t(((v >> 20) & 1023) >> 2);
                break;
            }
            case Fmt::rgba16f: {
                uint16_t c[3];
                memcpy(c, row + x * 8, 6);
                o[0] = srgb8(half(c[0])), o[1] = srgb8(half(c[1])), o[2] = srgb8(half(c[2]));
                break;
            }
            default:
                break;
            }
        }
    }
    return true;
}

}
