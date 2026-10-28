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
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_c: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    let _1_d: f32 = distance(_stageIn.sk_FragCoord.xy, _uniform0.R_center * _uniform0.resolution) / max(_uniform0.radius * max(_uniform0.resolution.x, _uniform0.resolution.y), 0.0001);
    let _2_light: f32 = _uniform0.brightness * (exp((-_1_d * _1_d) * 6.0) + 0.35 * exp((-(_1_d - 1.0) * (_1_d - 1.0)) * 60.0));
    let _3_rgb: vec3<f32> = _0_c.xyz + _uniform0.color.xyz * f32(_2_light);
    (*_stageOut).sk_FragColor = vec4<f32>(_3_rgb, max(_0_c.w, max(_3_rgb.x, max(_3_rgb.y, _3_rgb.z))));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
