/*{
  "EXPOSE": false,
  "DESCRIPTION": "層と背景を W3C Compositing and Blending Level 1 の規則で混ぜる。式は借り物 — reference/vello-blend.wgsl(vello_shaders 0.10.0、原文のまま)を前置きして呼ぶだけ。Motolii は式を持たない",
  "INPUTS": [
    { "NAME": "backdrop", "TYPE": "image" },
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "mode", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 65535.0 }
  ]
}*/

// 束縛はマニフェストが決める(effects/vism.rs)。image は宣言順に texture と
// sampler を2つずつ取り、param はその後ろへ並ぶ。
@group(0) @binding(0) var backdrop_tex: texture_2d<f32>;
@group(0) @binding(2) var source_tex: texture_2d<f32>;
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

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  // 画素は1対1。フィルタを掛けたくないので textureLoad(sampler は使わない)。
  let p = vec2<i32>(in.position.xy);
  let cb = textureLoad(backdrop_tex, p, 0);
  let cs = textureLoad(source_tex, p, 0);
  // mode は (mix << 8) | compose。上流の番号体系をそのまま使う。
  return blend_mix_compose(cb, cs, u32(mode));
}
