// Light Flare: AbsentMotion's lensflare (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:A bright light with a halo, laid over the picture.
// --track@R_center_x:Center x,-1,2,0.3,0.01
// --track@R_center_y:Center y,-1,2,0.3,0.01
// --track@radius:Radius,0.01,2,0.25,0.01
// --track@brightness:Brightness,0,4,1.0,0.01
// --track@color_x:Color R,0,1,1,0.01
// --track@color_y:Color G,0,1,0.8784,0.01
// --track@color_z:Color B,0,1,0.6902,0.01
// --track@color_w:Color A,0,1,1,0.01
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
  radius: f32,
  brightness: f32,
  color: vec4<f32>,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _1_c: vec4<f32> = textureSample(image_Texture, image_Sampler, (_0_coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let _2_d: f32 = distance(_0_coord, _uniform0.R_center * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / max(_uniform0.radius * max((select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).x, (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y), 0.0001);
    let _3_light: f32 = _uniform0.brightness * (exp((-_2_d * _2_d) * 6.0) + 0.35 * exp((-(_2_d - 1.0) * (_2_d - 1.0)) * 60.0));
    let _4_rgb: vec3<f32> = _1_c.xyz + _uniform0.color.xyz * f32(_3_light);
    (*_stageOut).sk_FragColor = vec4<f32>(_4_rgb, max(_1_c.w, max(_4_rgb.x, max(_4_rgb.y, _4_rgb.z))));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
