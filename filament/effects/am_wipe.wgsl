// Wipe: AbsentMotion's wipe (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Key
// --about:Hides the layer from two opposite sides. Animate Start or End for a wipe transition.
// --track@start:Start,-0.5,1.5,0.0,0.001
// --track@end:End,-0.5,1.5,1.0,0.001
// --track@angle:Angle,-3600,3600,0.0,0.5
// --track@feather:Feather,0,100,0.0,0.1
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
  start: f32,
  end: f32,
  angle: f32,
  feather: f32,
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
    let _3_ext: f32 = abs((select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).x * _2_d.x) + abs((select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y * _2_d.y);
    let _4_x: f32 = dot(_0_coord - (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5, _2_d) + _3_ext * 0.5;
    let _5_w: f32 = max(_uniform0.feather, 1.0);
    let _6_m: f32 = clamp((_4_x - _uniform0.start * _3_ext) / _5_w + 0.5, 0.0, 1.0) * clamp((_uniform0.end * _3_ext - _4_x) / _5_w + 0.5, 0.0, 1.0);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_0_coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution) * f32(_6_m);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
