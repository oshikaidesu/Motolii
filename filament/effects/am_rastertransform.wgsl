// Raster Transform: AbsentMotion's rastertransform (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Move
// --about:Scales, turns and moves the finished picture of the layer, after the effects above it, so their result can be transformed too. Fill keeps some of the original underneath and Mask To Layer keeps the result inside the original shape.
// --track@scale:Scale,0.01,50,1.0,0.01
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@alpha:Alpha,0,1,1.0,0.01
// --track@fill:Fill,0,1,0.0,0.01
// --track@sampling:Sampling,0,1,1.0,0.01
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
  offset: vec2<f32>,
  mask: f32,
  alpha: f32,
  fill: f32,
  sampling: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_c: vec2<f32> = _uniform0.resolution * 0.5;
    let _1_a: f32 = radians(_uniform0.angle);
    let _2_v: vec2<f32> = (_stageIn.sk_FragCoord.xy - _0_c) - _uniform0.offset;
    var _3_p: vec2<f32> = _0_c + vec2<f32>(cos(_1_a) * _2_v.x + sin(_1_a) * _2_v.y, cos(_1_a) * _2_v.y - sin(_1_a) * _2_v.x) / max(_uniform0.scale, 0.01);
    if _uniform0.sampling < 0.5 {
      _3_p = floor(_3_p) + 0.5;
    }
    let _4_own: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    let _5_moved: vec4<f32> = textureSample(image_Texture, image_Sampler, _3_p / _uniform0.resolution) * f32(clamp(_uniform0.alpha, 0.0, 1.0));
    var _6_color: vec4<f32> = _5_moved + (_4_own * f32(clamp(_uniform0.fill, 0.0, 1.0))) * (1.0 - _5_moved.w);
    if _uniform0.mask > 0.5 {
      _6_color = _6_color * _4_own.w;
    }
    (*_stageOut).sk_FragColor = _6_color;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
