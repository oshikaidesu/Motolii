// Swirl: AbsentMotion's twirl (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Twists the layer around Center like a whirlpool, strongest in the middle and fading out at Radius (a share of the layer size). Negative Strength twists the other way.
// --track@R_center_x:Center x,0,1,0.5,0.01
// --track@R_center_y:Center y,0,1,0.5,0.01
// --track@strength:Strength,-0.5,0.5,0.1,0.001
// --track@radius:Radius,0,3,0.3,0.01
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
  strength: f32,
  radius: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_c: vec2<f32> = _uniform0.R_center * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _1_d: vec2<f32> = (_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) - _0_c;
    let _2_r: f32 = max(_uniform0.radius * min((select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).x, (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y), 0.0001);
    let _3_t: f32 = clamp(1.0 - length(_1_d) / _2_r, 0.0, 1.0);
    let _4_a: f32 = (-48.0 * _uniform0.strength) * _3_t;
    let _5_cs: f32 = cos(_4_a);
    let _6_sn: f32 = sin(_4_a);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, ((_0_c + vec2<f32>(_5_cs * _1_d.x - _6_sn * _1_d.y, _6_sn * _1_d.x + _5_cs * _1_d.y)) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
