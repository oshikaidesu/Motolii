/*{
  "ID": "motolii.contact_shadow",
  "LABEL": "Contact Shadow",
  "DESCRIPTION": "A soft ground shadow under the layer's own silhouette, as a product shot on a seamless sweep has: every pixel is darkened by how close the layer is directly above it (an overhead soft box), then blurred twice (the 13-tap / tent of Glow). Drawn under the layer; no light to place",
  "FILTER": "linear",
  "PADDING": { "PARAM": "reach", "SCALE": 1.0 },
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0, "SUBTYPE": "FACTOR", "HERO": true },
    { "NAME": "reach", "LABEL": "Reach", "TYPE": "float", "DEFAULT": 40.0, "MIN": 4.0, "MAX": 512.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "light_through", "LABEL": "Light Through", "TYPE": "float", "DEFAULT": -1.0, "MIN": -1.0, "MAX": 1.0, "SUBTYPE": "FACTOR", "ADVANCED": true }
  ],
  "PASSES": [
    { "TARGET": "near", "FLOAT": true, "WIDTH": "$WIDTH/4",  "HEIGHT": "$HEIGHT/4" },
    { "TARGET": "soft", "FLOAT": true, "WIDTH": "$WIDTH/8",  "HEIGHT": "$HEIGHT/8" },
    { "TARGET": "wide", "FLOAT": true, "WIDTH": "$WIDTH/16", "HEIGHT": "$HEIGHT/16" },
    { }
  ]
}*/

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_smp: sampler;
@group(0) @binding(2) var near_tex: texture_2d<f32>;
@group(0) @binding(3) var near_smp: sampler;
@group(0) @binding(4) var soft_tex: texture_2d<f32>;
@group(0) @binding(5) var soft_smp: sampler;
@group(0) @binding(6) var wide_tex: texture_2d<f32>;
@group(0) @binding(7) var wide_smp: sampler;
@group(1) @binding(0) var<uniform> strength: f32;
@group(1) @binding(1) var<uniform> reach: f32;
@group(1) @binding(2) var<uniform> light_through: f32;
@group(1) @binding(3) var<uniform> render_size: vec2f;
@group(1) @binding(4) var<uniform> pass_index: f32;

struct VsOut { @builtin(position) position: vec4f, };

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VsOut {
  let positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  var out: VsOut;
  out.position = vec4f(positions[index], 0.0, 1.0);
  return out;
}

// How much of the layer is directly above `uv` (image +y is down), nearer counting more.
fn occlusion(uv: vec2f) -> f32 {
  let step = reach / 16.0 / render_size.y;
  var o = 0.0;
  for (var k = 1; k <= 16; k++) {
    let a = textureSampleLevel(source_tex, source_smp, uv - vec2f(0.0, f32(k) * step), 0.0).a;
    let w = 1.0 - f32(k - 1) / 16.0;
    o = max(o, a * w * w * w);
  }
  return o;
}

// [COD] slide 153 13-tap and slide 162 tent, as in glow.wgsl.
fn down13(t: texture_2d<f32>, s: sampler, uv: vec2f) -> f32 {
  let ps = 1.0 / vec2f(textureDimensions(t));
  var c = 0.0;
  c += (textureSampleLevel(t, s, uv + vec2f(-2.0, 2.0) * ps, 0.0).r + textureSampleLevel(t, s, uv + vec2f(2.0, 2.0) * ps, 0.0).r
      + textureSampleLevel(t, s, uv + vec2f(-2.0, -2.0) * ps, 0.0).r + textureSampleLevel(t, s, uv + vec2f(2.0, -2.0) * ps, 0.0).r) * 0.03125;
  c += (textureSampleLevel(t, s, uv + vec2f(0.0, 2.0) * ps, 0.0).r + textureSampleLevel(t, s, uv + vec2f(-2.0, 0.0) * ps, 0.0).r
      + textureSampleLevel(t, s, uv + vec2f(2.0, 0.0) * ps, 0.0).r + textureSampleLevel(t, s, uv + vec2f(0.0, -2.0) * ps, 0.0).r) * 0.0625;
  c += (textureSampleLevel(t, s, uv, 0.0).r + textureSampleLevel(t, s, uv + vec2f(-1.0, 1.0) * ps, 0.0).r + textureSampleLevel(t, s, uv + vec2f(1.0, 1.0) * ps, 0.0).r
      + textureSampleLevel(t, s, uv + vec2f(-1.0, -1.0) * ps, 0.0).r + textureSampleLevel(t, s, uv + vec2f(1.0, -1.0) * ps, 0.0).r) * 0.125;
  return c;
}

fn tent(t: texture_2d<f32>, s: sampler, uv: vec2f) -> f32 {
  let ps = 1.0 / vec2f(textureDimensions(t));
  let x = ps.x; let y = ps.y;
  return (textureSampleLevel(t, s, uv + vec2f(-x, y), 0.0).r + textureSampleLevel(t, s, uv + vec2f(x, y), 0.0).r
        + textureSampleLevel(t, s, uv + vec2f(-x, -y), 0.0).r + textureSampleLevel(t, s, uv + vec2f(x, -y), 0.0).r) * 0.0625
       + (textureSampleLevel(t, s, uv + vec2f(0.0, y), 0.0).r + textureSampleLevel(t, s, uv + vec2f(-x, 0.0), 0.0).r
        + textureSampleLevel(t, s, uv + vec2f(x, 0.0), 0.0).r + textureSampleLevel(t, s, uv + vec2f(0.0, -y), 0.0).r) * 0.125
       + textureSampleLevel(t, s, uv, 0.0).r * 0.25;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  let uv = in.position.xy / render_size;
  let stage = u32(pass_index);
  if (stage == 0u) { return vec4f(occlusion(uv)); }
  if (stage == 1u) { return vec4f(down13(near_tex, near_smp, uv)); }
  if (stage == 2u) { return vec4f(down13(soft_tex, soft_smp, uv)); }
  // A tight core where the layer nearly touches, and a wide penumbra; drawn under the layer (premultiplied).
  let shadow = clamp(0.7 * tent(near_tex, near_smp, uv) + 0.35 * tent(soft_tex, soft_smp, uv) + 0.15 * tent(wide_tex, wide_smp, uv), 0.0, 1.0) * strength;
  // A clear solid lets most light through: its shadow is as thin as it is clear (auto: the host reads
  // the layer's own surface — Glass's transmission × (1 − metallic)).
  let thinned = shadow * (1.0 - 0.8 * clamp(light_through, 0.0, 1.0));
  let src = textureLoad(source_tex, vec2i(in.position.xy), 0);
  return vec4f(src.rgb, src.a + thinned * (1.0 - src.a));
}
