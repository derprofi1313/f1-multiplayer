#include "swofty/postfx.hlsli"

float4 effect(float2 uv, uint2 pixel)
{
    float4 c = scene_at(int2(pixel));
    float3 rgb = to_linear(c.rgb);
    float strength = option(0) / 100, saturation = option(1) / 100;
    rgb = lerp(luminance(rgb).xxx, rgb, saturation);
    float2 d = uv - 0.5;
    d.x *= float(swofty_size.x) / float(swofty_size.y);
    rgb *= 1 - strength * smoothstep(0.3, 0.9, length(d));
    return float4(from_linear(rgb), c.a);
}
