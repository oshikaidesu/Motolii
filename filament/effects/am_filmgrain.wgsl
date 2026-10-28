// Film Grain: AbsentMotion's filmgrain (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --about:Animated monochrome film grain with adjustable amount, grain size, and animation speed.
// --track@amount:Amount,0.0,1.0,0.5,0.01
// --track@grainSize:Grain Size,1.0,8.0,4.0,1.0
// --track@speed:Speed,0.0,5.0,1.0,0.1
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
  amount: f32,
  grainSize: f32,
  speed: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn hash_ff3(_skParam0: vec3<f32>) -> f32 {
  var p = _skParam0;
  {
    p = fract(p * 0.1031);
    p = p + dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    let a: f32 = f32(src.w);
    if a < 1e-06 {
      return src;
    }
    var rgb: vec3<f32> = vec3<f32>(vec3<f32>(src.xyz) / a);
    let _skTemp1 = floor(((_uniform0.time + 0.000999) * max(_uniform0.speed, 0.0)) * 30.0);
    let frame: f32 = (_skTemp1 - 4096.0 * floor(_skTemp1 / 4096.0));
    let cell: f32 = exp2((clamp(_uniform0.grainSize, 1.0, 8.0) - 4.0) * 0.5);
    let p: vec2<f32> = coord / max(cell, 1.0) - 0.5;
    let i: vec2<f32> = floor(p);
    var f: vec2<f32> = fract(p);
    f = (f * f) * (3.0 - 2.0 * f);
    var noise: f32 = mix(mix(hash_ff3(vec3<f32>(i, frame)), hash_ff3(vec3<f32>(i + vec2<f32>(1.0, 0.0), frame)), f.x), mix(hash_ff3(vec3<f32>(i + vec2<f32>(0.0, 1.0), frame)), hash_ff3(vec3<f32>(i + 1.0, frame)), f.x), f.y);
    noise = 0.5 + (noise - 0.5) * min(cell, 1.0);
    let grain: f32 = (noise * 2.0 - 1.0) * _uniform0.amount;
    rgb = rgb + f32(grain);
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(vec3<f32>(vec3<f32>(rgb) * a), f32(a));
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
