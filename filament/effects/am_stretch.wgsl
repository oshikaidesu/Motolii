// Stretch Segment: AbsentMotion's stretch (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --track@stretch:Stretch,0,5000,10.0,1
// --track@offset:Offset,-5000,5000,0.0,1
// --track@spread:Spread,0,500,0.0,1
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
  angle: f32,
  stretch: f32,
  offset: f32,
  spread: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_r: f32 = radians(_uniform0.angle);
    let _1_dir: vec2<f32> = vec2<f32>(cos(_0_r), sin(_0_r));
    let _2_d: f32 = dot(_stageIn.sk_FragCoord.xy - 0.5 * _uniform0.resolution, _1_dir) - _uniform0.offset;
    let _3_gap: f32 = 0.5 * max(_uniform0.stretch, 0.0);
    var _skTemp1: vec2<f32>;
    if abs(_2_d) <= _3_gap {
      _skTemp1 = (_stageIn.sk_FragCoord.xy - _1_dir * _2_d) + (_1_dir * (_2_d / max(_3_gap, 0.001))) * _uniform0.spread;
    } else {
      _skTemp1 = _stageIn.sk_FragCoord.xy - (_1_dir * sign(_2_d)) * _3_gap;
    }
    let _4_from: vec2<f32> = _skTemp1;
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, _4_from / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
