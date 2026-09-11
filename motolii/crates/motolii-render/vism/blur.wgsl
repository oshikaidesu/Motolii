/*{
  "ID": "motolii.blur",
  "DESCRIPTION": "ガウスぼかし(横→縦の 2 段)。出入りの「ぼけて入って締まる」、影、被写界深度の土台",
  "OUTPUT_FLOAT": true,
  "PADDING": { "PARAM": "radius", "SCALE": 3.0 },
  "THUMBNAIL": { "radius": 24.0 },
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "radius", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.0, "MAX": 128.0 }
  ],
  "PASSES": [
    { "TARGET": "blur_h", "FLOAT": true },
    { }
  ]
}*/

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(2) var blur_h_tex: texture_2d<f32>;
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

// ガウス(σ ≒ radius / 2)。tap は 3σ まで 1px 刻み(上限 96、越えたら刻みを広げる)。半径 0 は素通し。
fn blur(t: texture_2d<f32>, p: vec2i, direction: vec2i) -> vec4f {
  let size = vec2i(textureDimensions(t));
  let lo = vec2i(0);
  let hi = size - vec2i(1);
  if (radius < 0.5) {
    return textureLoad(t, clamp(p, lo, hi), 0);
  }
  let sigma = max(radius * 0.5, 0.5);
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

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  let p = vec2i(in.position.xy);
  let stage = u32(pass_index);
  if (stage == 0u) {
    return blur(source_tex, p, vec2i(1, 0));
  }
  return blur(blur_h_tex, p, vec2i(0, 1));
}
