// Levels: AbsentMotion's levels (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Adjusts the levels of the image with input black/white points, gamma, and output black/white points.
// --track@inBlack:Input Black,0.0,1.0,0.1,0.01
// --track@inWhite:Input White,0.0,1.0,0.85,0.01
// --track@gamma:Gamma,0.2,5.0,1.0,0.1
// --track@outBlack:Output Black,0.0,1.0,0.0,0.01
// --track@outWhite:Output White,0.0,1.0,1.0,0.01
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
  inBlack: f32,
  inWhite: f32,
  gamma: f32,
  outBlack: f32,
  outWhite: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _0_effect_: vec4<f32>;
    let _1_color: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    let _2_alpha: f32 = f32(_1_color.w);
    if _2_alpha < 1e-06 {
      {
        _0_effect_ = vec4<f32>(0.0);
      }
    } else {
      {
        let _3_rgb: vec3<f32> = _1_color.xyz / vec3<f32>(f32(_2_alpha));
        var _4_denom: f32 = _uniform0.inWhite - _uniform0.inBlack;
        _4_denom = max(_4_denom, 1e-05);
        var _5_normalized: vec3<f32> = (_3_rgb - vec3<f32>(f32(_uniform0.inBlack))) / vec3<f32>(f32(_4_denom));
        _5_normalized = clamp(_5_normalized, vec3<f32>(0.0), vec3<f32>(1.0));
        var _6_gammaed: vec3<f32> = pow(_5_normalized, vec3<f32>(f32(_uniform0.gamma)));
        _6_gammaed = _6_gammaed * vec3<f32>(f32(_uniform0.outWhite - _uniform0.outBlack)) + vec3<f32>(f32(_uniform0.outBlack));
        _6_gammaed = clamp(_6_gammaed, vec3<f32>(0.0), vec3<f32>(1.0));
        _0_effect_ = vec4<f32>(_6_gammaed * vec3<f32>(f32(_2_alpha)), f32(_2_alpha));
      }
    }
    (*_stageOut).sk_FragColor = _0_effect_;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
