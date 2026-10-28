// Replace Color: AbsentMotion's replacecolor (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Replaces one color with another. Alpha controls the replacement strength; Lock Luminance keeps the light and shade of the original.
// --track@source_x:Replace Color R,0,1,1,0.01
// --track@source_y:Replace Color G,0,1,0,0.01
// --track@source_z:Replace Color B,0,1,0,0.01
// --track@source_w:Replace Color A,0,1,1,0.01
// --track@R_target_x:New Color R,0,1,0,0.01
// --track@R_target_y:New Color G,0,1,0,0.01
// --track@R_target_z:New Color B,0,1,1,0.01
// --track@R_target_w:New Color A,0,1,1,0.01
// --track@threshold:Threshold,0,4,0.25,0.001
// --track@feather:Feather,0,2,0.25,0.001
// --track@alpha:Alpha,0,1,1.0,0.001
// --track@choke:Choke,0.5,20,2.0,0.01
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
  source: vec4<f32>,
  R_target: vec4<f32>,
  threshold: f32,
  feather: f32,
  choke: f32,
  alpha: f32,
  lockLuminance: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn luma_ff3(c: vec3<f32>) -> f32 {
  {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let rgb: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let d: f32 = distance(rgb, vec3<f32>(_uniform0.source.xyz)) * _uniform0.choke;
    let keep: f32 = 1.0 - smoothstep(_uniform0.threshold, (_uniform0.threshold + _uniform0.feather) + 0.0001, d);
    var _skTemp1: f32;
    if _uniform0.lockLuminance > 0.5 {
      _skTemp1 = luma_ff3(rgb) - luma_ff3(vec3<f32>(_uniform0.source.xyz));
    } else {
      _skTemp1 = 0.0;
    }
    let light: f32 = _skTemp1;
    let swapped: vec3<f32> = clamp(vec3<f32>(_uniform0.R_target.xyz) + light, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(vec3<f32>(mix(rgb, swapped, vec3<f32>((keep * f32(_uniform0.R_target.w)) * clamp(_uniform0.alpha, 0.0, 1.0)))) * src.w, src.w);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0))));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
