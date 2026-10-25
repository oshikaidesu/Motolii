cbuffer constant0 : register(b0) {
    float4 rectColor;
    float4 lineColor;
    float4 radius; // LT, RT, RB, LB
    float4 lineOuterRadius;
    float2 halfSize; // width, height
    float2 halfRectSize;
    float lineW;
}

struct PSInput {
    float4 pos : SV_Position;
};

// 角丸四角形の符号付き距離関数(SDF)
// p: 点座標, b: 四角形の半分のサイズ, r: 各角の半径(LT, RT, RB, LB)
float sdRoundRect(float2 p, float2 b, float4 r) {
    r.xy = (p.x < 0.0) ? r.xw : r.yz;
    float rad = (p.y < 0.0) ? r.x : r.y;

    float2 q = abs(p) - b + rad;
    return clamp(q.x, q.y, 0.0) + length(max(q, 0.0)) - rad;
}

// アルファブレンド
// src: 前景色, dst: 背景色
float4 blend(float4 src, float4 dst) {
    return src + (1 - src.a) * dst;
}

// ピクセルシェーダーのメイン関数
float4 psmain(PSInput input) : SV_Target {
    // 画像の左上を原点とする座標系から、中心を原点とする座標系に変換
    float2 p = input.pos.xy - halfSize;

    // 角丸四角形
    float d = sdRoundRect(p, halfRectSize, radius);
    float alpha = saturate(0.5 - d) * rectColor.a;
    float4 rectCol = float4(rectColor.rgb * alpha, alpha);

    // ライン(ストローク)
    float d2 = sdRoundRect(p, halfSize, lineOuterRadius);
    float d3 = d2 + lineW;
    float alpha2 = (abs(d2) < abs(d3)) ? saturate(0.5 - d2) : saturate(0.5 + d3);
    alpha2 *= (lineW > 0.0) ? lineColor.a : 0.0;
    float4 lineCol = float4(lineColor.rgb * alpha2, alpha2);

    // 角丸四角形の上にラインを合成
    float4 color = blend(lineCol, rectCol);
    return color;
}
