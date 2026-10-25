Texture2D src : register(t0);
cbuffer constant0 : register(b0) {
    float color_bits;
}

float4 bitQuantize(float4 pos : SV_Position) : SV_TARGET
{
    float4 color = src[uint2(floor(pos.xy))];
    int levels = 1 << uint1(color_bits);
    color.rgb = saturate(floor(color.rgb * levels) / (levels - 1));
    return color;
}
