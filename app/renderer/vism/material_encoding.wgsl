/*{
  "ID": "motolii.material_encoding", "EXPOSE": false, "OUTPUT_FLOAT": true,
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "to_linear", "TYPE": "float", "DEFAULT": 0.0 },
    { "NAME": "source_encoded", "TYPE": "float", "DEFAULT": 1.0 },
    { "NAME": "source_premultiplied", "TYPE": "float", "DEFAULT": 0.0 }
  ]
}*/
// 効果の列の色の規約は**乗算済み**(累算器・rerun・Skia・Nuke と同じ)。ぼかしや縮小が透明の隣で
// 黒く沈まないための普通の方法。出口は 2 つ: 乗算済み線形(warp・置く時)か、乗算済み sRGB(pass)。
// 入口は 3 通り: 素材そのもの(非乗算 sRGB、層の法)、列の途中(乗算済み sRGB)、累算器(乗算済み線形)。
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> to_linear: f32;
@group(1) @binding(1) var<uniform> source_encoded: f32;
@group(1) @binding(2) var<uniform> source_premultiplied: f32;
@group(1) @binding(3) var<uniform> render_size: vec2f;
@group(1) @binding(4) var<uniform> pass_index: f32;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
  let p = array<vec2f,3>(vec2f(-1.0,-1.0),vec2f(3.0,-1.0),vec2f(-1.0,3.0));
  return vec4f(p[i],0.0,1.0);
}

// IEC sRGB transfer, as used by rerun's utils/srgb.wgsl.
fn decode_srgb(c: vec3f) -> vec3f {
  return select(pow(max((c + 0.055) / 1.055, vec3f(0.0)),vec3f(2.4)), c / 12.92, c <= vec3f(0.04045));
}
fn encode_srgb(c: vec3f) -> vec3f {
  return select(1.055 * pow(max(c,vec3f(0.0)),vec3f(1.0/2.4)) - 0.055, c * 12.92, c <= vec3f(0.0031308));
}
@fragment
fn fs_main(@builtin(position) p: vec4f) -> @location(0) vec4f {
  let color = textureLoad(source_tex, vec2i(p.xy), 0);
  var lin = select(color.rgb, decode_srgb(color.rgb), source_encoded > 0.5);
  if source_premultiplied < 0.5 { lin = lin * color.a; }
  if to_linear > 0.5 { return vec4f(lin, color.a); }
  return vec4f(encode_srgb(lin), color.a);
}
