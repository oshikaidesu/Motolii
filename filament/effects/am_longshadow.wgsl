// Long Shadow: AbsentMotion's longshadow (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Casts long, parallel shadows from the opaque parts of the layer.
// --track@angle:Angle,-3600,3600,45.0,0.5
// --track@reach:Length,0,4000,300.0,1
// --track@color_x:Color R,0,1,0,0.01
// --track@color_y:Color G,0,1,0,0.01
// --track@color_z:Color B,0,1,0,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@alpha:Alpha,0,1,0.6,0.001
// --track@fade:Fade,0,1,0.5,0.001
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
    let _1_a: f32 = radians(_uniform0.angle);
    let _2_d: vec2<f32> = vec2<f32>(cos(_1_a), sin(_1_a)) * _uniform0.reach;
    var _3_shade: f32 = 0.0;
    {
      var _4_i: i32 = 1;
      for (; _4_i <= 64; _4_i = _4_i + i32(1)) {
        {
          let _5_t: f32 = f32(_4_i) * 0.015625;
          let _6_hit: f32 = f32(textureSample(image_Texture, image_Sampler, (_stageIn.sk_FragCoord.xy - _2_d * _5_t) / _uniform0.resolution).w) * (1.0 - _uniform0.fade * _5_t);
          _3_shade = max(_3_shade, _6_hit);
        }
      }
    }
    let _7_sa: f32 = (_3_shade * clamp(_uniform0.alpha, 0.0, 1.0)) * f32(_uniform0.color.w);
    let _8_s: vec4<f32> = vec4<f32>(_uniform0.color.xyz * f32(_7_sa), f32(_7_sa));
    (*_stageOut).sk_FragColor = _0_src + _8_s * (1.0 - _0_src.w);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
