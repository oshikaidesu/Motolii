// Mirror: AbsentMotion's mirror (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Repeat
// --about:Reflects one half of the image across a mirror line defined by an angle and a center point.
// --track@angle:Angle,0,360,90.0,1
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
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _1_theta: f32 = radians(_uniform0.angle);
    let _3_normal: vec2<f32> = vec2<f32>(-sin(_1_theta), cos(_1_theta));
    let _4_c: vec2<f32> = _uniform0.R_center * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _5_v: vec2<f32> = _0_coord - _4_c;
    let _6_side: f32 = dot(_5_v, _3_normal);
    var _7_sc: vec2<f32> = _0_coord;
    if _6_side < 0.0 {
      {
        _7_sc = _4_c + reflect(_5_v, _3_normal);
      }
    }
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_7_sc + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
