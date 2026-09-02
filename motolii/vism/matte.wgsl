/*{
  "EXPOSE": false,
  "DESCRIPTION": "トラックマット。どの層で切るか(alpha / 反転alpha / luma / 反転luma)は編集の意味なので Motolii が持つ。輝度の式は借り物 — reference/vello-blend.wgsl の svg_lum(SVG luminanceToAlpha と同じ係数)を呼ぶ",
  "INPUTS": [
    { "NAME": "layer", "TYPE": "image" },
    { "NAME": "matte", "TYPE": "image" },
    { "NAME": "mode", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 3.0 }
  ]
}*/

@group(0) @binding(0) var layer_tex: texture_2d<f32>;
@group(0) @binding(2) var matte_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> mode: f32;

struct VsOut {
  @builtin(position) position: vec4f,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VsOut {
  let positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  var out: VsOut;
  out.position = vec4f(positions[index], 0.0, 1.0);
  return out;
}

/// どれで切るか = 編集の意味(AE のトラックマット4種)。
fn coverage(m: u32, matte: vec4f) -> f32 {
  if (m == 0u) { return matte.a; }
  if (m == 1u) { return 1.0 - matte.a; }
  if (m == 2u) { return svg_lum(matte.rgb); }
  return 1.0 - svg_lum(matte.rgb);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  let p = vec2<i32>(in.position.xy);
  let layer = textureLoad(layer_tex, p, 0);
  let matte = textureLoad(matte_tex, p, 0);
  let c = clamp(coverage(u32(mode), matte), 0.0, 1.0);
  return clamp(layer * c, vec4f(0.0), vec4f(1.0));
}
