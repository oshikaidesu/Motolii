Texture2D src : register(t0);
cbuffer constant0 : register(b0) {
    float start_dist;
    float end_dist;
    float obj_x;
    float obj_y;
    float obj_z;
    float cam_x;
    float cam_y;
    float cam_z;
    float fineness;
}

static const float4x4 BayerMatrix = {
    { 0.0 / 16.0,  8.0 / 16.0,  2.0 / 16.0, 10.0 / 16.0 },
    {12.0 / 16.0,  4.0 / 16.0, 14.0 / 16.0,  6.0 / 16.0 },
    { 3.0 / 16.0, 11.0 / 16.0,  1.0 / 16.0,  9.0 / 16.0 },
    {15.0 / 16.0,  7.0 / 16.0, 13.0 / 16.0,  5.0 / 16.0 }
};

float4 dithering_camera(float4 pos : SV_Position) : SV_TARGET
{
    float3 objPos = float3(obj_x, obj_y, obj_z);
    float3 camPos = float3(cam_x, cam_y, cam_z);
    uint2 pixelCoord = uint2(pos.xy*fineness*0.01) % 4;
    float threshold = BayerMatrix[pixelCoord.y][pixelCoord.x];
    float pos_dist = distance(objPos, camPos);
    float fade = pos_dist / abs(start_dist - end_dist);
    // start_dist >= end_dist なら近づいたら透明になり、そうでないなら離れたら透明になる
    bool show = start_dist >= end_dist ? (fade > threshold && pos_dist > end_dist) : (pos_dist < start_dist || fade <= threshold && pos_dist < end_dist);
    return show ? src[uint2(floor(pos.xy))] : float4(0.0, 0.0, 0.0, 0.0);
}
