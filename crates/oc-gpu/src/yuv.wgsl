// Rec.709 limited-range YUV 4:2:0 → sRGB-encoded RGBA。
// リニア化は後段のブレンドの責務(performance-model.md)。ここはガンマ保持で
// ffmpegのrgbaパス(oc-media)と同じ出力にし、パイプライン全体をsRGB RGBAで一貫させる。

@group(0) @binding(0) var y_tex: texture_2d<f32>;
@group(0) @binding(1) var u_tex: texture_2d<f32>;
@group(0) @binding(2) var v_tex: texture_2d<f32>;

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
    let y_coord = vec2<i32>(in.uv * y_dim);
    let c_coord = vec2<i32>(in.uv * c_dim);

    let yv = textureLoad(y_tex, y_coord, 0).r;
    let uv = textureLoad(u_tex, c_coord, 0).r;
    let vv = textureLoad(v_tex, c_coord, 0).r;

    // limited range 正規化: Y'∈[16,235], C∈[16,240] (8bit値/255で渡ってくる)
    let yl = (yv * 255.0 - 16.0) / 219.0;
    let cb = (uv * 255.0 - 128.0) / 224.0;
    let cr = (vv * 255.0 - 128.0) / 224.0;

    // BT.709 係数
    let r = yl + 1.5748 * cr;
    let g = yl - 0.1873 * cb - 0.4681 * cr;
    let b = yl + 1.8556 * cb;

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
