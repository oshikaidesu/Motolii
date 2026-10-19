// Datamosh: newomp4/phosphor's luma_half, motion and mosh shaders (MIT, Copyright (c) 2026 Owen, github.com/newomp4/phosphor) as one effect's passes in WGSL: the picture drawn last moved along each block's motion and not drawn afresh, as a P-frame decoded against the wrong reference; afresh every so many frames
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --pass@mv:0,1,s0.0625,s0.0625
// --pass@mosh:1,1,s1,s1
// --pass@luma:1,1,s0.5,s0.5
// --pass@src:1,0,s1,s1
// --pass@:0,0,s1,s1
// --track@keyframe:Afresh every (frames),0,600,120,1
// --track@residual:Residual,0,1,0.1,0.01
// --track@range:Search,1,8,3,1

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
@group(0) @binding(2) var mv: texture_2d<f32>;
@group(0) @binding(3) var mvNear: sampler;
@group(0) @binding(4) var mosh: texture_2d<f32>;
@group(0) @binding(5) var moshNear: sampler;
@group(0) @binding(6) var luma: texture_2d<f32>;
@group(0) @binding(7) var lumaNear: sampler;
@group(0) @binding(8) var src: texture_2d<f32>;
@group(0) @binding(9) var srcNear: sampler;
struct Knobs { passindex: f32, frame: f32, keyframe: f32, residual: f32, range: f32 }
@group(0) @binding(10) var<uniform> knobs: Knobs;

const BLOCK = 8;   // half-res pixels: a full-res macroblock of 16

fn lumaNow(p: vec2i) -> f32 {   // (luma_half: the picture now, at half size)
  let sz = vec2i(textureDimensions(picture)) - 1;
  let q = p * 2;
  let s = textureLoad(picture, min(q, sz), 0).rgb + textureLoad(picture, min(q + vec2i(1, 0), sz), 0).rgb
        + textureLoad(picture, min(q + vec2i(0, 1), sz), 0).rgb + textureLoad(picture, min(q + vec2i(1, 1), sz), 0).rgb;
  return dot(s * 0.25, vec3f(0.299, 0.587, 0.114));
}

@fragment fn fs_main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
  let which = i32(knobs.passindex);
  let p = vec2i(frag.xy);
  if (which == 0) {   // (motion: full search of each block of the luma now in the last frame's luma)
    let p0 = p * BLOCK;
    let sz = vec2i(textureDimensions(luma)) - 1;
    let range = i32(knobs.range);
    var now: array<f32, 64>;   // (the block's luma now, made once, not again for every place tried)
    for (var j = 0; j < BLOCK; j++) {
      for (var i = 0; i < BLOCK; i++) { now[j * BLOCK + i] = lumaNow(min(p0 + vec2i(i, j), sz)); }
    }
    var best = 1e30;
    var bv = vec2i(0);
    for (var dy = -range; dy <= range; dy++) {
      for (var dx = -range; dx <= range; dx++) {
        var sad = 0.0;
        for (var j = 0; j < BLOCK; j++) {
          for (var i = 0; i < BLOCK; i++) {
            let b = clamp(min(p0 + vec2i(i, j), sz) + vec2i(dx, dy), vec2i(0), sz);
            sad += abs(now[j * BLOCK + i] - textureLoad(luma, b, 0).r);
          }
        }
        let cost = sad + 0.01 * f32(abs(dx) + abs(dy));
        if (cost < best) { best = cost; bv = vec2i(dx, dy); }
      }
    }
    return vec4f(vec2f(bv) * 2.0, best, 1.0);
  }
  if (which == 1) {   // (mosh: the last output moved by the motion, a share of the true difference added)
    let cur = textureLoad(picture, p, 0).rgb;
    let k = i32(knobs.keyframe);
    if (i32(knobs.frame) < 2 || (k > 0 && i32(knobs.frame) % k == 0)) { return vec4f(cur, 1.0); }
    let sz = vec2i(textureDimensions(picture)) - 1;
    let m = textureLoad(mv, p / (BLOCK * 2), 0).xy;
    let q = clamp(p + vec2i(m), vec2i(0), sz);
    let warped = textureLoad(mosh, q, 0).rgb;
    let pred = textureLoad(src, q, 0).rgb;
    return vec4f(clamp(warped + knobs.residual * (cur - pred), vec3f(0.0), vec3f(1.5)), 1.0);
  }
  if (which == 2) { return vec4f(lumaNow(p), 0.0, 0.0, 1.0); }   // (the luma now, the next frame's last)
  if (which == 3) { return vec4f(textureLoad(picture, p, 0).rgb, 1.0); }   // (the picture now, the next frame's last)
  return vec4f(textureLoad(mosh, p, 0).rgb, 1.0);
}
