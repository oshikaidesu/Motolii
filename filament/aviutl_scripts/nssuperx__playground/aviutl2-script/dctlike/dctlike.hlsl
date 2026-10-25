cbuffer constant0 : register(b0) {
    float freq_x;
    float freq_y;
    float height;
    float width;
}

float4 dctlike(float4 pos : SV_Position) : SV_TARGET
{
    float N = 1000.0;
    float2 uv = pos.xy / float2(width, height);
    
    // DCTっぽい
    float val = cos(((2.0 * uv.x * N + 1.0) * freq_x * 3.1415926535) / (2.0 * N)) *
                cos(((2.0 * uv.y * N + 1.0) * freq_y * 3.1415926535) / (2.0 * N));

    float color = (val + 1.0) / 2.0;

    return float4(color, color, color, 1.0);
}
