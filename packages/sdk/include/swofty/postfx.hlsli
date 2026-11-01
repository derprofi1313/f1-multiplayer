// Swofty post-process pass contract (postfx v2). The launcher compiles a pass as a cs_6_0 compute
// shader with this header on the include path, and the core runs it on the game's picture after the
// tonemap and before the HUD, inside the game's own frame, so the HUD draws over the result.
//
// A pass defines `float4 effect(float2 uv, uint2 pixel)` and returns the colour for that pixel of its
// output: the picture, or one of the addon's own textures. A pass with "entry" in the manifest names
// another function of the same shape. A pass with "raw": true defines its own [numthreads] compute
// entry point and writes swofty_output itself.

#ifndef SWOFTY_POSTFX_HLSLI
#define SWOFTY_POSTFX_HLSLI

#define SWOFTY_POSTFX_SDR 0
#define SWOFTY_POSTFX_HDR10 1
#define SWOFTY_POSTFX_SCRGB 2

#define SWOFTY_VIEW_UNKNOWN 0
#define SWOFTY_VIEW_COCKPIT 1
#define SWOFTY_VIEW_TCAM 2
#define SWOFTY_VIEW_TCAM_OFFSET 3
#define SWOFTY_VIEW_CHASE_NEAR 4
#define SWOFTY_VIEW_CHASE_FAR 5
#define SWOFTY_VIEW_OTHER 6

#define SWOFTY_VIEW_PAUSED 1
#define SWOFTY_VIEW_REPLAY 2
#define SWOFTY_VIEW_PHOTO 4
#define SWOFTY_VIEW_MENU 8
#define SWOFTY_VIEW_CUTSCENE 16
#define SWOFTY_VIEW_SPECTATING 32

#define SWOFTY_INPUTS 8

cbuffer SwoftyPostFx : register(b0)
{
    uint2 swofty_size;
    float2 swofty_texel;
    float swofty_time;
    uint swofty_frame;
    uint swofty_encoding;
    uint swofty_pass;
    float4 swofty_options[4];
    uint2 swofty_depth_size;
    uint swofty_has_depth;
    float swofty_frametime;
    uint2 swofty_scene_size;
    uint swofty_random;
    uint swofty_view;
    uint swofty_view_flags;
    uint swofty_view_valid;
    uint swofty_has_previous;
    uint swofty_reserved;
};

Texture2D<float4> swofty_scene : register(t0);
Texture2D<float> swofty_depth : register(t1);
Texture2D<float4> swofty_inputs[SWOFTY_INPUTS] : register(t2);
RWTexture2D<float4> swofty_output : register(u0);
SamplerState swofty_linear : register(s0);
SamplerState swofty_point : register(s1);
SamplerState swofty_linear_wrap : register(s2);
SamplerState swofty_point_wrap : register(s3);

float option(uint i)
{
    return swofty_options[i >> 2][i & 3];
}

// The picture this pass works on: the game's picture after the passes before this one.
float4 scene(float2 uv)
{
    return swofty_scene.SampleLevel(swofty_linear, uv, 0);
}

float4 scene_at(int2 pixel)
{
    return swofty_scene.Load(int3(clamp(pixel, int2(0, 0), int2(swofty_scene_size) - 1), 0));
}

// The pass's "inputs", in manifest order: the addon's own textures, "original" (the picture before
// any pass this frame) or "previous" (the last frame's picture after every pass).
float4 input(uint i, float2 uv)
{
    return swofty_inputs[i].SampleLevel(swofty_linear, uv, 0);
}

float4 input_at(uint i, int2 pixel)
{
    uint w, h;
    swofty_inputs[i].GetDimensions(w, h);
    return swofty_inputs[i].Load(int3(clamp(pixel, int2(0, 0), int2(w, h) - 1), 0));
}

uint2 input_size(uint i)
{
    uint w, h;
    swofty_inputs[i].GetDimensions(w, h);
    return uint2(w, h);
}

// The game's depth buffer at uv, as stored: 1 at the near plane, falling towards 0 in the distance.
// 0 everywhere when the depth buffer is not available (has_depth() is false).
float scene_depth(float2 uv)
{
    return swofty_depth.SampleLevel(swofty_point, uv, 0);
}

bool has_depth()
{
    return swofty_has_depth != 0;
}

// Whether "previous" holds last frame's picture; false on the first frame and after a camera cut,
// a resize or a frame without passes.
bool has_previous()
{
    return swofty_has_previous != 0;
}

// The game's view this frame: SWOFTY_VIEW_COCKPIT and the others, or SWOFTY_VIEW_UNKNOWN while the
// core cannot tell. view_is(SWOFTY_VIEW_PAUSED) tests a flag.
uint view_kind()
{
    return swofty_view_valid ? swofty_view : SWOFTY_VIEW_UNKNOWN;
}

bool view_is(uint flag)
{
    return swofty_view_valid && (swofty_view_flags & flag) != 0;
}

// A hash of pixel, frame and salt, from 0 to 1, for grain and dithering.
float random(uint2 pixel, uint salt)
{
    uint h = pixel.x * 1973u + pixel.y * 9277u + swofty_random * 26699u + salt * 7919u;
    h = (h ^ 61u) ^ (h >> 16);
    h *= 9u;
    h ^= h >> 4;
    h *= 0x27d4eb2du;
    h ^= h >> 15;
    return float(h) / 4294967295.0;
}

float3 srgb_to_linear(float3 c)
{
    return select(c <= 0.04045, c / 12.92, pow((c + 0.055) / 1.055, 2.4));
}

float3 linear_to_srgb(float3 c)
{
    c = max(c, 0);
    return select(c <= 0.0031308, c * 12.92, 1.055 * pow(c, 1.0 / 2.4) - 0.055);
}

float3 pq_to_linear(float3 c)
{
    const float m1 = 0.1593017578125, m2 = 78.84375, c1 = 0.8359375, c2 = 18.8515625, c3 = 18.6875;
    float3 p = pow(max(c, 0), 1.0 / m2);
    return pow(max(p - c1, 0) / (c2 - c3 * p), 1.0 / m1) * 10000.0 / 80.0;
}

float3 linear_to_pq(float3 c)
{
    const float m1 = 0.1593017578125, m2 = 78.84375, c1 = 0.8359375, c2 = 18.8515625, c3 = 18.6875;
    float3 y = pow(max(c * 80.0 / 10000.0, 0), m1);
    return pow((c1 + c2 * y) / (1 + c3 * y), m2);
}

// The picture in linear light: 1 is SDR white in SDR and 80 nits in HDR.
float3 to_linear(float3 c)
{
    if (swofty_encoding == SWOFTY_POSTFX_HDR10)
        return pq_to_linear(c);
    if (swofty_encoding == SWOFTY_POSTFX_SCRGB)
        return c;
    return srgb_to_linear(c);
}

float3 from_linear(float3 c)
{
    if (swofty_encoding == SWOFTY_POSTFX_HDR10)
        return linear_to_pq(c);
    if (swofty_encoding == SWOFTY_POSTFX_SCRGB)
        return c;
    return linear_to_srgb(c);
}

float luminance(float3 linear_rgb)
{
    return dot(linear_rgb, float3(0.2126, 0.7152, 0.0722));
}

#ifndef SWOFTY_RAW
#ifndef SWOFTY_EFFECT
#define SWOFTY_EFFECT effect
#endif

float4 SWOFTY_EFFECT(float2 uv, uint2 pixel);

[numthreads(8, 8, 1)] void main(uint3 id : SV_DispatchThreadID)
{
    if (any(id.xy >= swofty_size))
        return;
    float2 uv = (float2(id.xy) + 0.5) * swofty_texel;
    float4 c = SWOFTY_EFFECT(uv, id.xy);
    // A pass that returns NaN or infinity keeps the pixel it was given: the picture where it writes the
    // picture, black in its own textures.
    if (any(!isfinite(c)))
        c = all(swofty_size == swofty_scene_size) ? scene_at(int2(id.xy)) : float4(0, 0, 0, 0);
    swofty_output[id.xy] = c;
}
#endif

#endif
