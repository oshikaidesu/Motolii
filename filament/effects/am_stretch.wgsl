// Stretch Segment: AbsentMotion's stretch (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Cuts the layer along a line and pulls the two halves apart, stretching the pixels on the line into the gap.
// --track@angle:Angle,-360,360,0.0,1
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
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _1_r: f32 = radians(_uniform0.angle);
    let _2_dir: vec2<f32> = vec2<f32>(cos(_1_r), sin(_1_r));
    let _3_d: f32 = dot(_0_coord - 0.5 * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))), _2_dir) - _uniform0.offset;
    let _4_gap: f32 = 0.5 * max(_uniform0.stretch, 0.0);
    var _skTemp1: vec2<f32>;
    if abs(_3_d) <= _4_gap {
      _skTemp1 = (_0_coord - _2_dir * _3_d) + (_2_dir * (_3_d / max(_4_gap, 0.001))) * _uniform0.spread;
    } else {
      _skTemp1 = _0_coord - (_2_dir * sign(_3_d)) * _4_gap;
    }
    let _5_from: vec2<f32> = _skTemp1;
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_5_from + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
