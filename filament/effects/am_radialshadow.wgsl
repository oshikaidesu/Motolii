// Radial Shadow: AbsentMotion's radialshadow (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --track@light_x:Light x,-2,3,0.3,0.01
// --track@light_y:Light y,-2,3,0.2,0.01
// --track@reach:Length,0,5,0.25,0.001
// --track@color_x:Color R,0,1,0,0.01
// --track@color_y:Color G,0,1,0,0.01
// --track@color_z:Color B,0,1,0,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@alpha:Alpha,0,1,0.1,0.001
// --track@fade:Fade,0,1,0.0,0.001
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
  light: vec2<f32>,
  reach: f32,
  color: vec4<f32>,
  alpha: f32,
  fade: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_src: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    let _1_l: vec2<f32> = _uniform0.light * _uniform0.resolution;
    var _2_shade: f32 = 0.0;
    {
      var _3_i: i32 = 1;
      for (; _3_i <= 64; _3_i = _3_i + i32(1)) {
        {
          let _4_t: f32 = f32(_3_i) * 0.015625;
          let _5_k: f32 = 1.0 / (1.0 + max(_uniform0.reach, 0.0) * _4_t);
          let _6_hit: f32 = f32(textureSample(image_Texture, image_Sampler, (_1_l + (_stageIn.sk_FragCoord.xy - _1_l) * _5_k) / _uniform0.resolution).w) * (1.0 - _uniform0.fade * _4_t);
          _2_shade = max(_2_shade, _6_hit);
        }
      }
    }
    let _7_sa: f32 = (_2_shade * clamp(_uniform0.alpha, 0.0, 1.0)) * f32(_uniform0.color.w);
    let _8_s: vec4<f32> = vec4<f32>(_uniform0.color.xyz * f32(_7_sa), f32(_7_sa));
    (*_stageOut).sk_FragColor = _0_src + _8_s * (1.0 - _0_src.w);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
