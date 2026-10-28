// Mirror: AbsentMotion's mirror (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Repeat
// --about:Reflects one half of the image across a mirror line defined by an angle and a center point.
// --track@R_center_x:Center x,0,1,0.5,0.01
// --track@R_center_y:Center y,0,1,0.5,0.01
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
  R_center: vec2<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_theta: f32 = radians(_uniform0.angle);
    let _2_normal: vec2<f32> = vec2<f32>(-sin(_0_theta), cos(_0_theta));
    let _3_c: vec2<f32> = _uniform0.R_center * _uniform0.resolution;
    let _4_v: vec2<f32> = _stageIn.sk_FragCoord.xy - _3_c;
    let _5_side: f32 = dot(_4_v, _2_normal);
    var _6_sc: vec2<f32> = _stageIn.sk_FragCoord.xy;
    if _5_side < 0.0 {
      {
        _6_sc = _3_c + reflect(_4_v, _2_normal);
      }
    }
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, _6_sc / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
