// Fade In/Out: AbsentMotion's fade (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Fades the layer in at its start and out again at its end, over the times you set.
// --track@fadeIn:In,0,10,0.5,0.01
// --track@fadeOut:Out,0,10,0.5,0.01
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
  fadeIn: f32,
  fadeOut: f32,
  duration: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _0_a: f32 = 1.0;
    if _uniform0.fadeIn > 0.0 {
      _0_a = _0_a * clamp(_uniform0.time / _uniform0.fadeIn, 0.0, 1.0);
    }
    if _uniform0.fadeOut > 0.0 {
      _0_a = _0_a * clamp((_uniform0.duration - _uniform0.time) / _uniform0.fadeOut, 0.0, 1.0);
    }
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, ((_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution) * f32(_0_a);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
