/*{
  "ID": "motolii.blur",
  "DESCRIPTION": "ガウスぼかし(横→縦の 2 段)。半径が大きい時は 1/4・1/16 に落として掛け、双一次で戻す(ぼけた絵に高い密度は要らない — 費用は半径に依らず comp の定数倍)。出入りの「ぼけて入って締まる」、影、被写界深度の土台",
  "OUTPUT_FLOAT": true,
  "FILTER": "linear",
  "PADDING": { "PARAM": "radius", "SCALE": 3.0 },
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "radius", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.0, "MAX": 128.0, "SUBTYPE": "DISTANCE" }
  ],
  "PASSES": [
    { "TARGET": "blur_h", "FLOAT": true },
    { "TARGET": "blur_v", "FLOAT": true },
    { "TARGET": "quarter", "FLOAT": true, "WIDTH": "$WIDTH/4", "HEIGHT": "$HEIGHT/4" },
    { "TARGET": "quarter_h", "FLOAT": true, "WIDTH": "$WIDTH/4", "HEIGHT": "$HEIGHT/4" },
    { "TARGET": "quarter_v", "FLOAT": true, "WIDTH": "$WIDTH/4", "HEIGHT": "$HEIGHT/4" },
    { "TARGET": "sixteenth", "FLOAT": true, "WIDTH": "$WIDTH/16", "HEIGHT": "$HEIGHT/16" },
    { "TARGET": "sixteenth_h", "FLOAT": true, "WIDTH": "$WIDTH/16", "HEIGHT": "$HEIGHT/16" },
    { "TARGET": "sixteenth_v", "FLOAT": true, "WIDTH": "$WIDTH/16", "HEIGHT": "$HEIGHT/16" },
    { }
  ]
}*/

// image の並び = INPUTS の image → PASSES の TARGET(宣言順)。texture は 2n、sampler は 2n+1。
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var blur_h_tex: texture_2d<f32>;
@group(0) @binding(4) var blur_v_tex: texture_2d<f32>;
@group(0) @binding(6) var quarter_tex: texture_2d<f32>;
@group(0) @binding(7) var quarter_sampler: sampler;
@group(0) @binding(8) var quarter_h_tex: texture_2d<f32>;
@group(0) @binding(10) var quarter_v_tex: texture_2d<f32>;
@group(0) @binding(11) var quarter_v_sampler: sampler;
@group(0) @binding(12) var sixteenth_tex: texture_2d<f32>;
@group(0) @binding(14) var sixteenth_h_tex: texture_2d<f32>;
@group(0) @binding(16) var sixteenth_v_tex: texture_2d<f32>;
@group(0) @binding(17) var sixteenth_v_sampler: sampler;
@group(1) @binding(0) var<uniform> radius: f32;
@group(1) @binding(1) var<uniform> render_size: vec2f;
@group(1) @binding(2) var<uniform> pass_index: f32;

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

// 半径(画素)で段を選ぶ: 0 = 等倍、1 = 1/4、2 = 1/16。段を落とした後の σ が 2 px を下回らない所で切る。
const QUARTER_FROM: f32 = 16.0;
const SIXTEENTH_FROM: f32 = 128.0;
fn level() -> u32 {
  if (radius >= SIXTEENTH_FROM) { return 2u; }
  if (radius >= QUARTER_FROM) { return 1u; }
  return 0u;
}

// ガウス(σ ≒ radius / 2 を段の縮尺で割る)。tap は 3σ まで 1px 刻み(上限 96、越えたら刻みを広げる)。
fn blur(t: texture_2d<f32>, p: vec2i, direction: vec2i, sigma: f32) -> vec4f {
  let size = vec2i(textureDimensions(t));
  let lo = vec2i(0);
  let hi = size - vec2i(1);
  if (sigma < 0.25) {
    return textureLoad(t, clamp(p, lo, hi), 0);
  }
  let reach = ceil(sigma * 3.0);
  let taps = i32(min(reach, 96.0));
  let step = max(1.0, reach / 96.0);
  var sum = vec4f(0.0);
  var weight = 0.0;
  for (var i = -taps; i <= taps; i = i + 1) {
    let offset = f32(i) * step;
    let w = exp(-(offset * offset) / (2.0 * sigma * sigma));
    let q = clamp(p + direction * i32(round(offset)), lo, hi);
    sum = sum + textureLoad(t, q, 0) * w;
    weight = weight + w;
  }
  return sum / weight;
}

// 1/4 へ落とす: 出力画素の中心に 4×4 の箱(双一次 2×2 を 4 回)。
fn downsample4(t: texture_2d<f32>, s: sampler, p: vec2i, out_size: vec2f) -> vec4f {
  let uv = (vec2f(p) + 0.5) / out_size;
  let texel = 1.0 / vec2f(textureDimensions(t));
  return 0.25 * (textureSampleLevel(t, s, uv + vec2f(-1.0, -1.0) * texel, 0.0)
               + textureSampleLevel(t, s, uv + vec2f( 1.0, -1.0) * texel, 0.0)
               + textureSampleLevel(t, s, uv + vec2f(-1.0,  1.0) * texel, 0.0)
               + textureSampleLevel(t, s, uv + vec2f( 1.0,  1.0) * texel, 0.0));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  let p = vec2i(in.position.xy);
  let stage = u32(pass_index);
  let lvl = level();
  let sigma = max(radius * 0.5, 0.5);
  // 等倍の段
  if (stage == 0u) { if (lvl != 0u) { return vec4f(0.0); } if (radius < 0.5) { return textureLoad(source_tex, p, 0); } return blur(source_tex, p, vec2i(1, 0), sigma); }
  if (stage == 1u) { if (lvl != 0u) { return vec4f(0.0); } if (radius < 0.5) { return textureLoad(blur_h_tex, p, 0); } return blur(blur_h_tex, p, vec2i(0, 1), sigma); }
  // 1/4 の段
  if (stage == 2u) { if (lvl == 0u) { return vec4f(0.0); } return downsample4(source_tex, source_sampler, p, render_size); }
  if (stage == 3u) { if (lvl != 1u) { return vec4f(0.0); } return blur(quarter_tex, p, vec2i(1, 0), sigma / 4.0); }
  if (stage == 4u) { if (lvl != 1u) { return vec4f(0.0); } return blur(quarter_h_tex, p, vec2i(0, 1), sigma / 4.0); }
  // 1/16 の段
  if (stage == 5u) { if (lvl != 2u) { return vec4f(0.0); } return downsample4(quarter_tex, quarter_sampler, p, render_size); }
  if (stage == 6u) { if (lvl != 2u) { return vec4f(0.0); } return blur(sixteenth_tex, p, vec2i(1, 0), sigma / 16.0); }
  if (stage == 7u) { if (lvl != 2u) { return vec4f(0.0); } return blur(sixteenth_h_tex, p, vec2i(0, 1), sigma / 16.0); }
  // 出力: 段の絵を双一次で等倍へ戻す(ぼけた絵なので継ぎ目は見えない)
  let uv = (vec2f(p) + 0.5) / render_size;
  if (lvl == 0u) { return textureLoad(blur_v_tex, p, 0); }
  if (lvl == 1u) { return textureSampleLevel(quarter_v_tex, quarter_v_sampler, uv, 0.0); }
  return textureSampleLevel(sixteenth_v_tex, sixteenth_v_sampler, uv, 0.0);
}
