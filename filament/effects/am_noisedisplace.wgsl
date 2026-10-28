// Noise Displace: AbsentMotion's noisedisplace (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Displaces pixels using smooth value noise that animates over time.
// --track@amount:Amount,0,100,30.0,1
// --track@scale:Noise Scale,5,400,100.0,1
// --track@speed:Animation Speed,0,5,1.0,0.1
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
  scale: f32,
  speed: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn noise_ff2(p: vec2<f32>) -> f32 {
  {
    let i: vec2<f32> = floor(p);
    let f: vec2<f32> = fract(p);
    let u: vec2<f32> = (f * f) * (3.0 - 2.0 * f);
    let a: f32 = fract(sin(dot(i, vec2<f32>(12.9898, 78.233))) * 43758.5469);
    let b: f32 = fract(sin(dot(i + vec2<f32>(1.0, 0.0), vec2<f32>(12.9898, 78.233))) * 43758.5469);
    let c: f32 = fract(sin(dot(i + vec2<f32>(0.0, 1.0), vec2<f32>(12.9898, 78.233))) * 43758.5469);
    let d: f32 = fract(sin(dot(i + vec2<f32>(1.0), vec2<f32>(12.9898, 78.233))) * 43758.5469);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _1_uv: vec2<f32> = _0_coord / max(_uniform0.scale, 0.0001);
    let _2_t: f32 = _uniform0.time * _uniform0.speed;
    let _3_dx: f32 = noise_ff2(_1_uv + vec2<f32>(_2_t, 0.0)) - 0.5;
    let _4_dy: f32 = noise_ff2(_1_uv + vec2<f32>(37.0, _2_t + 17.0)) - 0.5;
    let _5_offset: vec2<f32> = vec2<f32>(_3_dx, _4_dy) * _uniform0.amount;
    let _6_col: vec4<f32> = textureSample(image_Texture, image_Sampler, ((_0_coord + _5_offset) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    (*_stageOut).sk_FragColor = _6_col;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
