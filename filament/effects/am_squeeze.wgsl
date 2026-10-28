// Squeeze: AbsentMotion's squeeze (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Pinches the middle of the layer inward with positive Strength and bulges it outward with negative Strength. Legacy preserves the shape of older projects.
// --track@amount:Strength,-2,2,0.0,0.01
// --track@shape:Shape,0,1,0.0,0.01
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
  shape: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let h: vec2<f32> = (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5;
    var p: vec2<f32> = (coord - h) / h;
    let strength: f32 = _uniform0.amount;
    if (_uniform0.shape > 0.5) && (strength < 0.0) {
      p = p.yx;
    }
    var _skTemp1: f32;
    if _uniform0.shape > 0.5 {
      _skTemp1 = abs(strength);
    } else {
      _skTemp1 = strength;
    }
    let a: f32 = _skTemp1 * 0.5;
    let squeeze: f32 = 1.0 - a * (1.0 - p.y * p.y);
    if squeeze <= 0.0001 {
      return vec4<f32>(0.0);
    }
    var q: vec2<f32> = vec2<f32>(p.x / squeeze, p.y / (1.0 + 0.3 * abs(strength)));
    if (_uniform0.shape > 0.5) && (strength < 0.0) {
      q = q.yx;
    }
    return textureSample(image_Texture, image_Sampler, ((h + q * h) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0))));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
