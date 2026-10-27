// Dissolve: AbsentMotion's dissolve (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --track@progress:Strength,0,1,0.75,0.01
// --track@seed:Seed,0,5,0.0,0.01
// --track@grain:Size,1,200,1.0,1
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
  progress: f32,
  grain: f32,
  seed: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _1_p: vec2<f32> = floor(_stageIn.sk_FragCoord.xy / max(_uniform0.grain, 1.0)) + _uniform0.seed * vec2<f32>(17.13, 41.71);
    _1_p = fract(_1_p * vec2<f32>(123.34, 456.21));
    _1_p = _1_p + dot(_1_p, _1_p + 45.32);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution) * f32(step(_uniform0.progress, fract(_1_p.x * _1_p.y)));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
