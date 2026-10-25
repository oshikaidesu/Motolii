Texture2D src : register(t0);
cbuffer constant0 : register(b0) {
    float color_bits_r;
    float color_bits_g;
    float color_bits_b;
}

float4 bitRGBQuantize(float4 pos : SV_Position) : SV_TARGET
{
    float4 color = src[uint2(floor(pos.xy))];
    int levels_r = 1 << uint1(color_bits_r);
    int levels_g = 1 << uint1(color_bits_g);
    int levels_b = 1 << uint1(color_bits_b);
    float3 levels = float3(levels_r, levels_g, levels_b);
    color.rgb = floor(color.rgb * levels) / (levels - 1);
    color.rgb = saturate(color.rgb);
    return color;
}
