// Stretch Axis: AbsentMotion's stretchaxis (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Move
// --about:Stretches the layer in any direction.
// --track@scale:Scale,0.01,50,1.0,0.01
// --track@angle:Angle,-3600,3600,0.0,0.1
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
  scale: f32,
  angle: f32,
  mask: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _1_a: f32 = radians(_uniform0.angle);
    let _2_d: vec2<f32> = vec2<f32>(cos(_1_a), sin(_1_a));
    let _3_v: vec2<f32> = _0_coord - (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5;
    let _4_along: f32 = dot(_3_v, _2_d);
    var _5_color: vec4<f32> = textureSample(image_Texture, image_Sampler, ((_0_coord + _2_d * (_4_along / max(_uniform0.scale, 0.01) - _4_along)) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if _uniform0.mask > 0.5 {
      {
        _5_color = _5_color * textureSample(image_Texture, image_Sampler, (_0_coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution).w;
      }
    }
    (*_stageOut).sk_FragColor = _5_color;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
