// Datamosh: AbsentMotion's datamosh (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Between the start and end times, in seconds from the beginning of the layer, the picture stops refreshing and is dragged around by the motion that follows, as if a key frame were missing. Video layers are smeared by their real motion, other layers by moving blocks.
// --track@start:Start,0,3600,0.0,0.01
// --track@end:End,0,3600,2.0,0.01
// --track@intensity:Intensity,0,4,1.0,0.01
// --track@block:Block Size,1,128,16.0,1
// --track@decay:Decay,0,1,0.05,0.001
// --track@glitch:Glitch,0,1,0.1,0.001
// --track@bloom:Bloom,0,0.95,0.3,0.001
diagnostic(off, derivative_uniformity);
diagnostic(off, chromium.unreachable_code);
struct FSIn {
  @builtin(position) sk_FragCoord: vec4<f32>,
};
struct FSOut {
  @location(0) sk_FragColor: vec4<f32>,
};
struct Knobs {
  resolution: vec2<f32>,
  time: f32,
  start: f32,
  end: f32,
  duration: f32,
  intensity: f32,
  block: f32,
  decay: f32,
  glitch: f32,
  bloom: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn hash_ff2(p: vec2<f32>) -> f32 {
  {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5469);
  }
}
fn effect__h4f2(xy: vec2<f32>) -> vec4<f32> {
  {
    let plain: vec4<f32> = textureSample(image_Texture, image_Sampler, xy / _uniform0.resolution);
    let R_from: f32 = clamp(_uniform0.start, 0.0, _uniform0.duration);
    let into: f32 = _uniform0.time - R_from;
    if ((into < 0.0) || (_uniform0.time >= clamp(_uniform0.end, R_from, _uniform0.duration))) || (plain.w <= 0.0) {
      {
        return plain;
      }
    }
    if (((xy.x < 0.0) || (xy.y < 0.0)) || (xy.x > _uniform0.resolution.x)) || (xy.y > _uniform0.resolution.y) {
      {
        return plain;
      }
    }
    let steps: f32 = floor(into * 15.0) + 1.0;
    let b: f32 = max(_uniform0.block, 2.0);
    let cell: vec2<f32> = floor(xy / b);
    let region: vec2<f32> = floor(cell * 0.25);
    var dir: vec2<f32> = vec2<f32>(hash_ff2(region + 3.7), hash_ff2(region + 9.1)) * 2.0 - 1.0;
    dir = dir + (vec2<f32>(hash_ff2(cell + 1.1), hash_ff2(cell + 2.2)) - 0.5) * 0.6;
    let moving: f32 = step(0.3, hash_ff2(region + 5.5));
    let keep: f32 = pow(1.0 - clamp(_uniform0.decay, 0.0, 1.0), steps);
    let grow: f32 = steps * (1.0 + (_uniform0.bloom * steps) * 0.05);
    let shift: vec2<f32> = (((((dir * moving) * _uniform0.intensity) * b) * 0.15) * grow) * keep;
    const lo: vec2<f32> = vec2<f32>(0.5);
    let hi: vec2<f32> = max(_uniform0.resolution - 0.5, lo);
    let moved: vec4<f32> = textureSample(image_Texture, image_Sampler, clamp(xy - shift, lo, hi) / _uniform0.resolution);
    let trail: vec4<f32> = (textureSample(image_Texture, image_Sampler, clamp(xy - shift * 0.66, lo, hi) / _uniform0.resolution) + textureSample(image_Texture, image_Sampler, clamp(xy - shift * 0.33, lo, hi) / _uniform0.resolution)) * 0.5;
    var c: vec4<f32> = mix(moved, trail, vec4<f32>(0.35));
    let g: f32 = hash_ff2(cell + vec2<f32>(steps * 7.13, steps * 3.71));
    if g < (_uniform0.glitch * 0.3) {
      {
        let r: vec4<f32> = textureSample(image_Texture, image_Sampler, clamp((xy - shift) + vec2<f32>(b * 0.5, 0.0), lo, hi) / _uniform0.resolution);
        c = vec4<f32>(min(r.x, max(c.w, r.w)), c.y, c.z * 0.8, max(c.w, r.w));
      }
    }
    let a: f32 = min(plain.w, c.w);
    return vec4<f32>(vec4<f32>(c.xyz / max(c.w, 0.0001), c.w).xyz * a, a);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
