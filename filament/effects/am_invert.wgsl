// Invert: AbsentMotion's invert (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Inverts the color channels of the image with a blend amount and optional alpha inversion.
// --track@amount:Amount,0.0,1.0,1.0,0.01
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
  invert_alpha: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_c: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    let _1_a: f32 = _0_c.w;
    var _2_rgb: vec3<f32> = _0_c.xyz;
    if _1_a > 0.001 {
      {
        _2_rgb = _2_rgb / _1_a;
      }
    }
    let _3_inv: vec3<f32> = 1.0 - _2_rgb;
    var _4_out_rgb: vec3<f32> = vec3<f32>(mix(vec3<f32>(_2_rgb), vec3<f32>(_3_inv), vec3<f32>(_uniform0.amount)));
    var _5_out_a: f32 = _0_c.w;
    if _uniform0.invert_alpha == 1.0 {
      {
        _5_out_a = 1.0 - _1_a;
      }
    }
    _4_out_rgb = _4_out_rgb * _5_out_a;
    (*_stageOut).sk_FragColor = vec4<f32>(_4_out_rgb, _5_out_a);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
