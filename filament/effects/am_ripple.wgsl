// Ripple: AbsentMotion's ripple (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Concentric ripples radiating from a center point with configurable amplitude, wavelength, and speed.
// --track@R_center_x:Center x,0,1,0.5,0.01
// --track@R_center_y:Center y,0,1,0.5,0.01
// --track@amplitude:Amplitude,0.0,100.0,20.0,0.1
// --track@wavelength:Wavelength,1.0,200.0,30.0,0.1
// --track@speed:Speed,0.0,10.0,2.0,0.1
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
  R_center: vec2<f32>,
  amplitude: f32,
  wavelength: f32,
  speed: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _1_centerPx: vec2<f32> = _uniform0.R_center * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _2_d: vec2<f32> = _0_coord - _1_centerPx;
    let _3_dist: f32 = length(_2_d);
    let _4_phase: f32 = _uniform0.time * _uniform0.speed;
    let _5_wave: f32 = sin(_3_dist / max(_uniform0.wavelength, 0.0001) - _4_phase) * _uniform0.amplitude;
    var _skTemp1: vec2<f32>;
    if _3_dist > 0.0 {
      _skTemp1 = (_2_d / _3_dist) * _5_wave;
    } else {
      _skTemp1 = vec2<f32>(0.0);
    }
    let _6_offset: vec2<f32> = _skTemp1;
    var _7_color: vec4<f32> = textureSample(image_Texture, image_Sampler, ((_0_coord + _6_offset) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if _7_color.w > 0.0 {
      {
        _7_color = vec4<f32>((_7_color.xyz / _7_color.w), _7_color.w);
      }
    }
    _7_color = vec4<f32>((_7_color.xyz * _7_color.w), _7_color.w);
    (*_stageOut).sk_FragColor = _7_color;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
