// Wipe: AbsentMotion's wipe (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Key
// --track@start:Start,-0.5,1.5,0.0,0.001
// --track@end:End,-0.5,1.5,1.0,0.001
// --track@feather:Feather,0,100,0.0,0.1
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
  angle: f32,
  feather: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_a: f32 = radians(_uniform0.angle);
    let _1_d: vec2<f32> = vec2<f32>(cos(_0_a), sin(_0_a));
    let _2_ext: f32 = abs(_uniform0.resolution.x * _1_d.x) + abs(_uniform0.resolution.y * _1_d.y);
    let _3_x: f32 = dot(_stageIn.sk_FragCoord.xy - _uniform0.resolution * 0.5, _1_d) + _2_ext * 0.5;
    let _4_w: f32 = max(_uniform0.feather, 1.0);
    let _5_m: f32 = clamp((_3_x - _uniform0.start * _2_ext) / _4_w + 0.5, 0.0, 1.0) * clamp((_uniform0.end * _2_ext - _3_x) / _4_w + 0.5, 0.0, 1.0);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution) * f32(_5_m);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
