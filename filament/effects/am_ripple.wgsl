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
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_centerPx: vec2<f32> = _uniform0.R_center * _uniform0.resolution;
    let _1_d: vec2<f32> = _stageIn.sk_FragCoord.xy - _0_centerPx;
    let _2_dist: f32 = length(_1_d);
    let _3_phase: f32 = _uniform0.time * _uniform0.speed;
    let _4_wave: f32 = sin(_2_dist / max(_uniform0.wavelength, 0.0001) - _3_phase) * _uniform0.amplitude;
    var _skTemp1: vec2<f32>;
    if _2_dist > 0.0 {
      _skTemp1 = (_1_d / _2_dist) * _4_wave;
    } else {
      _skTemp1 = vec2<f32>(0.0);
    }
    let _5_offset: vec2<f32> = _skTemp1;
    var _6_color: vec4<f32> = textureSample(image_Texture, image_Sampler, (_stageIn.sk_FragCoord.xy + _5_offset) / _uniform0.resolution);
    if _6_color.w > 0.0 {
      {
        _6_color = vec4<f32>((_6_color.xyz / _6_color.w), _6_color.w);
      }
    }
    _6_color = vec4<f32>((_6_color.xyz * _6_color.w), _6_color.w);
    (*_stageOut).sk_FragColor = _6_color;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
