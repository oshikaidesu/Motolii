// Edge Detect: AbsentMotion's edgedetect (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Bright lines where the picture changes, dark elsewhere.
// --track@strength:Strength,0,20,2.0,0.01
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
  strength: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn l_hf2(p: vec2<f32>) -> f32 {
  {
    return dot(textureSample(image_Texture, image_Sampler, p / _uniform0.resolution).xyz, vec3<f32>(0.2126, 0.7152, 0.0722));
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_gx: f32 = ((((l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(1.0, -1.0)) + 2.0 * l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(1.0, 0.0))) + l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(1.0))) - l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(-1.0))) - 2.0 * l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(-1.0, 0.0))) - l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(-1.0, 1.0));
    let _1_gy: f32 = ((((l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(-1.0, 1.0)) + 2.0 * l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(0.0, 1.0))) + l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(1.0))) - l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(-1.0))) - 2.0 * l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(0.0, -1.0))) - l_hf2(_stageIn.sk_FragCoord.xy + vec2<f32>(1.0, -1.0));
    let _2_a: f32 = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution).w;
    (*_stageOut).sk_FragColor = vec4<f32>(vec3<f32>(saturate(length(vec2<f32>(_0_gx, _1_gy)) * f32(_uniform0.strength))) * _2_a, _2_a);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
