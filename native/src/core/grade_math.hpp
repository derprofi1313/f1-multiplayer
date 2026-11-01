#pragma once

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <string>
#include <vector>

// The colour maths of shader looks, free of D3D12 so the tests run it on the host. A look is a
// 3D LUT in the build's packed form (packages/formats/src/cube.rs): "SLUT", version 1, size,
// domain min and max, then size^3 RGB entries with red changing fastest, all little-endian.

namespace grade {

struct Lut {
    uint32_t size = 0;
    float min[3] = {0, 0, 0}, max[3] = {1, 1, 1};
    std::vector<float> table;
};

inline bool unpack(const std::string &data, Lut &out, std::string &why)
{
    auto word = [&](size_t at, void *to) {
        if (at + 4 > data.size())
            return false;
        memcpy(to, data.data() + at, 4);
        return true;
    };
    uint32_t version = 0, size = 0;
    if (data.size() < 4 || memcmp(data.data(), "SLUT", 4) != 0) {
        why = "it does not start with SLUT";
        return false;
    }
    if (!word(4, &version) || version != 1 || !word(8, &size) || size < 2 || size > 129) {
        why = "its header is not version 1 with a size from 2 to 129";
        return false;
    }
    for (int c = 0; c < 3; c++)
        if (!word(12 + c * 4, &out.min[c]) || !word(24 + c * 4, &out.max[c]) || !(out.min[c] < out.max[c])) {
            why = "its domain is not below its maximum in every channel";
            return false;
        }
    size_t count = size_t(size) * size * size * 3;
    if (data.size() != 36 + count * 4) {
        why = "it is " + std::to_string(data.size()) + " bytes, not the " + std::to_string(36 + count * 4) +
              " its size needs";
        return false;
    }
    out.size = size;
    out.table.resize(count);
    memcpy(out.table.data(), data.data() + 36, count * 4);
    return true;
}

inline float srgb_encode(float x)
{
    if (x <= 0.0031308f)
        return 12.92f * x;
    return 1.055f * std::pow(x, 1.0f / 2.4f) - 0.055f;
}

inline float srgb_decode(float x)
{
    if (x <= 0.04045f)
        return x / 12.92f;
    return std::pow((x + 0.055f) / 1.055f, 2.4f);
}

// Trilinear lookup of an encoded colour, clamped to the LUT's domain.
inline void sample(const Lut &l, const float in[3], float out[3])
{
    int lo[3], hi[3];
    float f[3];
    for (int c = 0; c < 3; c++) {
        float t = (std::clamp(in[c], l.min[c], l.max[c]) - l.min[c]) / (l.max[c] - l.min[c]) * float(l.size - 1);
        lo[c] = std::min(int(t), int(l.size) - 2);
        hi[c] = lo[c] + 1;
        f[c] = t - float(lo[c]);
    }
    auto at = [&](int r, int g, int b, int c) { return l.table[((size_t(b) * l.size + g) * l.size + r) * 3 + c]; };
    for (int c = 0; c < 3; c++) {
        float c00 = at(lo[0], lo[1], lo[2], c) * (1 - f[0]) + at(hi[0], lo[1], lo[2], c) * f[0];
        float c10 = at(lo[0], hi[1], lo[2], c) * (1 - f[0]) + at(hi[0], hi[1], lo[2], c) * f[0];
        float c01 = at(lo[0], lo[1], hi[2], c) * (1 - f[0]) + at(hi[0], lo[1], hi[2], c) * f[0];
        float c11 = at(lo[0], hi[1], hi[2], c) * (1 - f[0]) + at(hi[0], hi[1], hi[2], c) * f[0];
        float c0 = c00 * (1 - f[1]) + c10 * f[1], c1 = c01 * (1 - f[1]) + c11 * f[1];
        out[c] = c0 * (1 - f[2]) + c1 * f[2];
    }
}

// Grades one colour of the game's tonemap LUT. It stores display colour as linear light, so the
// look, graded on screenshots, applies to its sRGB encoding. Light above the domain keeps its
// overshoot, so an identity look changes nothing; strength mixes the look with the original.
inline void apply(const Lut &l, float strength, float rgb[3])
{
    float enc[3], looked[3];
    for (int c = 0; c < 3; c++)
        enc[c] = srgb_encode(std::max(rgb[c], 0.0f));
    sample(l, enc, looked);
    for (int c = 0; c < 3; c++) {
        float over = std::max(enc[c] - l.max[c], 0.0f);
        float graded = looked[c] + over;
        rgb[c] = srgb_decode(std::max(enc[c] + (graded - enc[c]) * strength, 0.0f));
    }
}

inline float unpack_half(uint16_t h)
{
    int e = (h >> 10) & 31, m = h & 1023;
    float v = e == 0 ? std::ldexp(float(m), -24) : e == 31 ? 65504.0f : std::ldexp(float(1024 + m), e - 25);
    return (h & 0x8000) ? -v : v;
}

// Non-negative floats only, rounded to nearest; what a display colour needs.
inline uint16_t pack_half(float x)
{
    if (!(x > 0))
        return 0;
    if (x >= 65504.0f)
        return 0x7bff;
    int e = int(std::floor(std::log2(x)));
    if (e < -14)
        return uint16_t(std::min(std::lround(x * 16777216.0f), 1023L));
    long m = std::lround((x / std::ldexp(1.0f, e) - 1) * 1024.0f);
    if (m == 1024) {
        e++;
        m = 0;
    }
    if (e > 15)
        return 0x7bff;
    return uint16_t(((e + 15) << 10) | m);
}

}
