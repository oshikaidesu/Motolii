// YUV 4:2:0 → ガンマ保持RGBA変換。
// 係数・レンジはuniformで受け取り、FrameDesc.color_spaceから選択される
// (落とし穴B-3: 「フィールドを持っている」と「正しく変換している」は別 — 決め打ち禁止)。
//
// クロマは4:2:0のsiting(H.264/MPEG-2慣習: 水平=左cosited、垂直=中間)を考慮した
// 位置でバイリニアサンプルする(レビュー指摘#3: 最近傍+siting無視だと実素材で
// 色が半ピクセルずれ、色エッジがブロック化する)。
// リニア化は後段のブレンドの責務(performance-model.md)。

struct ColorParams {
    y_off: f32,   // limited: 16.0 / full: 0.0
    y_scale: f32, // limited: 1/219 / full: 1/255
    c_scale: f32, // limited: 1/224 / full: 1/255
    _pad: f32,
    crv: f32, // R += crv * Cr
    cbu: f32, // B += cbu * Cb
    cgu: f32, // G += cgu * Cb
    cgv: f32, // G += cgv * Cr
};

@group(0) @binding(0) var y_tex: texture_2d<f32>;
@group(0) @binding(1) var u_tex: texture_2d<f32>;
@group(0) @binding(2) var v_tex: texture_2d<f32>;
@group(0) @binding(3) var<uniform> params: ColorParams;
@group(0) @binding(4) var chroma_samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// フルスクリーン三角形(頂点バッファ不要)
@vertex
fn vs(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let x = f32((vi << 1u) & 2u);
    let y = f32(vi & 2u);
    out.uv = vec2<f32>(x, y);
    out.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let y_dim = vec2<f32>(textureDimensions(y_tex));
    let c_dim = vec2<f32>(textureDimensions(u_tex));

    // 輝度: 出力ピクセルと1:1なので最近傍(textureLoad)で正確に読む
    let luma_px = vec2<i32>(in.uv * y_dim);
    let yv = textureLoad(y_tex, luma_px, 0).r * 255.0;

    // クロマ: 出力ピクセル(i,j)に対するクロマ位置(テクセル単位、中心=+0.5)
    //   水平(左cosited): pos_x = i/2 + 0.5
    //   垂直(中間):      pos_y = j/2 + 0.25
    let i = floor(in.uv.x * y_dim.x);
    let j = floor(in.uv.y * y_dim.y);
    let c_uv = vec2<f32>((i * 0.5 + 0.5) / c_dim.x, (j * 0.5 + 0.25) / c_dim.y);
    let uv_ = textureSampleLevel(u_tex, chroma_samp, c_uv, 0.0).r * 255.0;
    let vv = textureSampleLevel(v_tex, chroma_samp, c_uv, 0.0).r * 255.0;

    let yl = (yv - params.y_off) * params.y_scale;
    let cb = (uv_ - 128.0) * params.c_scale;
    let cr = (vv - 128.0) * params.c_scale;

    let r = yl + params.crv * cr;
    let g = yl + params.cgu * cb + params.cgv * cr;
    let b = yl + params.cbu * cb;

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
