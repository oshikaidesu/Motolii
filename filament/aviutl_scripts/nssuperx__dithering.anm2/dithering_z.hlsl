Texture2D src : register(t0);
cbuffer constant0 : register(b0) {
    float start_z;
    float end_z;
    float obj_z;
    float fineness;
}

static const float4x4 BayerMatrix = {
    { 0.0 / 16.0,  8.0 / 16.0,  2.0 / 16.0, 10.0 / 16.0 },
    {12.0 / 16.0,  4.0 / 16.0, 14.0 / 16.0,  6.0 / 16.0 },
    { 3.0 / 16.0, 11.0 / 16.0,  1.0 / 16.0,  9.0 / 16.0 },
    {15.0 / 16.0,  7.0 / 16.0, 13.0 / 16.0,  5.0 / 16.0 }
};

float4 dithering_z(float4 pos : SV_Position) : SV_TARGET
{
    uint2 pixelCoord = uint2(pos.xy*fineness*0.01) % 4;
    float threshold = BayerMatrix[pixelCoord.y][pixelCoord.x];
    float fade = (obj_z - start_z) / (end_z - start_z);
    // ディザリング判定
    return (fade > threshold) ? float4(0.0, 0.0, 0.0, 0.0) : src[uint2(floor(pos.xy))];
}
