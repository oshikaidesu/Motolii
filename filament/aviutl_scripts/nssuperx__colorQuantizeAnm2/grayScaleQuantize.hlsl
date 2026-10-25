Texture2D src : register(t0);
cbuffer constant0 : register(b0) {
    float color_num;
    float threshold_shift;
    float curve;
}

float4 grayScaleQuantize(float4 pos : SV_Position) : SV_TARGET
{
    float4 color = src[uint2(floor(pos.xy))];
    float adjust_color_num = max(1.0, color_num - 1.0);
    float curve_exp = (curve >= 0.0) ? (1.0-curve) : (1.0 / (1.0+curve)) + 0.0001;
    float3 curved = pow(saturate(color.rgb), curve_exp);
    float t = dot(curved, float3(1.0, 1.0, 1.0)) / 3.0;
    t = saturate(t);
    t = round(t * adjust_color_num + threshold_shift) / adjust_color_num;
    color.rgb = saturate(float3(t, t, t));
    return color;
}
