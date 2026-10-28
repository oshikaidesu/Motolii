// Zoom Blur: AbsentMotion's radialblur (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Blur
// --about:Applies a radial zoom blur by averaging samples along rays from a center point with adjustable strength.
// --track@R_center_x:Center x,0,1,0.5,0.01
// --track@R_center_y:Center y,0,1,0.5,0.01
// --track@amount:Strength,0.0,1.9,0.15,0.01
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
  amount: f32,
  R_center: vec2<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_c: vec2<f32> = _uniform0.R_center * _uniform0.resolution;
    let _1_d: vec2<f32> = _stageIn.sk_FragCoord.xy - _0_c;
    let _2_s: f32 = clamp(_uniform0.amount, 0.0, 1.9);
    var _3_sum: vec4<f32> = vec4<f32>(0.0);
    {
      var _4_i: i32 = 0;
      for (; _4_i < 48; _4_i = _4_i + i32(1)) {
        {
          _3_sum = _3_sum + textureSample(image_Texture, image_Sampler, (_0_c + _1_d * (1.0 + _2_s * (f32(_4_i) * 0.0212765951 - 0.5))) / _uniform0.resolution);
        }
      }
    }
    (*_stageOut).sk_FragColor = _3_sum * 0.020833334;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
