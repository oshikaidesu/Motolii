// Chroma Key: AbsentMotion's chromakey (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Key
// --about:Removes one color, such as a green screen.
// --track@key_x:Key Color R,0,1,0,0.01
// --track@key_y:Key Color G,0,1,1,0.01
// --track@key_z:Key Color B,0,1,0,0.01
// --track@key_w:Key Color A,0,1,1,0.01
// --track@tolerance:Tolerance,0,1,0.25,0.01
// --track@softness:Softness,0,1,0.1,0.01
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
  key: vec4<f32>,
  tolerance: f32,
  softness: f32,
  invert: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn chroma_f2h3(c: vec3<f32>) -> vec2<f32> {
  {
    return vec2<f32>(f32((-0.1687 * c.x - 0.3313 * c.y) + 0.5 * c.z), f32((0.5 * c.x - 0.4187 * c.y) - 0.0813 * c.z));
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_c: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    let _1_d: f32 = distance(chroma_f2h3(vec4<f32>(_0_c.xyz / max(_0_c.w, 0.0001), _0_c.w).xyz), chroma_f2h3(_uniform0.key.xyz));
    let _2_keep: f32 = smoothstep(_uniform0.tolerance, (_uniform0.tolerance + _uniform0.softness) + 0.0001, _1_d);
    var _skTemp1: f32;
    if _uniform0.invert > 0.5 {
      _skTemp1 = 1.0 - _2_keep;
    } else {
      _skTemp1 = _2_keep;
    }
    (*_stageOut).sk_FragColor = _0_c * f32(_skTemp1);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
