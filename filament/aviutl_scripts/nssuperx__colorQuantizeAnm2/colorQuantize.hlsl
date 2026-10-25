Texture2D src : register(t0);
cbuffer constant0 : register(b0) {
    float color_num;
    float threshold_shift;
    float curve;
    float startColorR;
    float startColorG;
    float startColorB;
    float endColorR;
    float endColorG;
    float endColorB;
}

float4 colorQuantize(float4 pos : SV_Position) : SV_TARGET
{
    float3 startColor = float3(startColorR, startColorG, startColorB) / 255.0;
    float3 endColor = float3(endColorR, endColorG, endColorB) / 255.0;
    float4 color = src[uint2(floor(pos.xy))];
    float adjust_color_num = max(1.0, color_num - 1.0);
    float curve_exp = (curve >= 0.0) ? (1.0-curve) : (1.0 / (1.0+curve)) + 0.0001;
    float3 curved = pow(saturate(color.rgb), curve_exp);
    float3 axis = endColor - startColor;
    float t = dot(curved - startColor, axis) / dot(axis, axis);
    t = saturate(t);
    t = round(t * adjust_color_num + threshold_shift) / adjust_color_num;
    float3 quantized = startColor + axis * t;
    color.rgb = saturate(quantized);
    return color;
}
