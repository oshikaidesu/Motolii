// Time Displacement: keijiro/SlitScanCam's shader (Unlicense, public domain, github.com/keijiro/SlitScanCam) in WGSL: each point of the picture as it was so many frames ago, further back across the picture; or eight past frames, each its own hue
// --label:Time
// --history@60:0.5
// --track@axis:Across or down,0,1,0,0.01
// --track@reach:How far back,0,1,1,0.01
// --check@rainbow:Rainbow,0

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
@group(0) @binding(2) var past: texture_2d_array<f32>;
@group(0) @binding(3) var pastNear: sampler;
struct Knobs { history_head: f32, history_count: f32, axis: f32, reach: f32, rainbow: f32 }
@group(0) @binding(4) var<uniform> knobs: Knobs;

fn history(uv: vec2f, offset: i32) -> vec3f {
  let n = i32(knobs.history_count);
  let i = (i32(knobs.history_head) + n - (offset % n)) % n;
  return textureSampleLevel(past, pastNear, uv, i, 0.0).rgb;
}

@fragment fn fs_main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
  let n = knobs.history_count;
  if (knobs.rainbow > 0.5) {
    var acc = vec3f(0.0);
    let step = max(1, i32(n / 8.0));
    for (var i = 0; i < 8; i++) {
      var c = history(uv, i * step);
      let h = f32(i) / 8.0 * 6.0 - 2.0;
      c *= clamp(vec3f(abs(h - 1.0) - 1.0, 2.0 - abs(h), 2.0 - abs(h - 2.0)), vec3f(0.0), vec3f(1.0));
      acc += c / 4.0;
    }
    return vec4f(acc, 1.0);
  }
  let delay = mix(uv.x, 1.0 - uv.y, knobs.axis) * (n - 2.0) * knobs.reach;
  let o = i32(delay);
  return vec4f(mix(history(uv, o), history(uv, o + 1), fract(delay)), 1.0);
}
