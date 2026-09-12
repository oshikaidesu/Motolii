/*{
  "ID": "motolii.glow",
  "DESCRIPTION": "明部を抜いて広げ、元へ足す。段は ISF の PASSES で宣言する(1フレーム内の中間ターゲットだけ。PERSISTENT は採らない)",
  "OUTPUT_FLOAT": true,
  "SPILL": "screen",
  "PADDING": { "PARAM": "radius", "SCALE": 2.0 },
  "THUMBNAIL": { "threshold": 0.8, "intensity": 1.5, "radius": 24.0 },
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "threshold", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "intensity", "TYPE": "float", "DEFAULT": 0.75, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "radius", "TYPE": "float", "DEFAULT": 1.0, "MIN": 1.0, "MAX": 64.0, "SUBTYPE": "DISTANCE" }
  ],
  "PASSES": [
    { "TARGET": "bright", "FLOAT": true },
    { "TARGET": "blur_h", "FLOAT": true },
    { "TARGET": "blur_v", "FLOAT": true },
    { }
  ]
}*/

// image の並び = INPUTS の image → PASSES の TARGET(宣言順)。
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(2) var bright_tex: texture_2d<f32>;
@group(0) @binding(4) var blur_h_tex: texture_2d<f32>;
@group(0) @binding(6) var blur_v_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> threshold: f32;
@group(1) @binding(1) var<uniform> intensity: f32;
@group(1) @binding(2) var<uniform> radius: f32;
@group(1) @binding(3) var<uniform> render_size: vec2f;
@group(1) @binding(4) var<uniform> pass_index: f32;

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

fn bright(p: vec2i) -> vec4f {
  let value = textureLoad(source_tex, p, 0);
  // 輝度は線形の Rec.709(合成が線形光で走っているのに合わせる)
  let luminance = dot(value.rgb, vec3f(0.2126, 0.7152, 0.0722));
  let contribution = max(luminance - threshold, 0.0) / max(luminance, 0.000001);
  return vec4f(value.rgb * contribution, value.a * contribution);
}

fn blur(t: texture_2d<f32>, p: vec2i, direction: vec2i) -> vec4f {
  let size = vec2i(textureDimensions(t));
  let lo = vec2i(0);
  let hi = size - vec2i(1);
  let step = max(i32(round(radius)), 1);
  let d1 = direction * step;
  let d2 = direction * step * 2;
  return textureLoad(t, clamp(p - d2, lo, hi), 0) * 0.0625
    + textureLoad(t, clamp(p - d1, lo, hi), 0) * 0.25
    + textureLoad(t, clamp(p, lo, hi), 0) * 0.375
    + textureLoad(t, clamp(p + d1, lo, hi), 0) * 0.25
    + textureLoad(t, clamp(p + d2, lo, hi), 0) * 0.0625;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  let p = vec2i(in.position.xy);
  let stage = u32(pass_index);
  if (stage == 0u) {
    return bright(p);
  }
  if (stage == 1u) {
    return blur(bright_tex, p, vec2i(1, 0));
  }
  if (stage == 2u) {
    return blur(blur_h_tex, p, vec2i(0, 1));
  }
  let src = textureLoad(source_tex, p, 0);
  let glow = textureLoad(blur_v_tex, p, 0) * intensity;
  return clamp(
    vec4f(src.rgb + glow.rgb, src.a + glow.a * (1.0 - src.a)),
    vec4f(0.0),
    vec4f(1.0)
  );
}
