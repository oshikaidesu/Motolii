// Spot Color: AbsentMotion's spotcolor (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Keeps one color and takes the saturation out of every other color in the layer.
// --track@color_x:Color R,0,1,1,0.01
// --track@color_y:Color G,0,1,0,0.01
// --track@color_z:Color B,0,1,0,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@threshold:Threshold,0,2,0.35,0.001
// --track@feather:Feather,0,2,0.25,0.001
// --track@saturation:Saturation,0,1,0.0,0.001
// --track@choke:Choke,0.5,20,5.0,0.01
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
  color: vec4<f32>,
  threshold: f32,
  feather: f32,
  saturation: f32,
  choke: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let rgb: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let d: f32 = distance(rgb, vec3<f32>(_uniform0.color.xyz)) * _uniform0.choke;
    let keep: f32 = 1.0 - smoothstep(_uniform0.threshold, (_uniform0.threshold + _uniform0.feather) + 0.0001, d);
    let grey: f32 = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let other: vec3<f32> = mix(vec3<f32>(grey), rgb, vec3<f32>(_uniform0.saturation));
    return vec4<f32>(vec3<f32>(mix(other, rgb, vec3<f32>(keep))) * src.w, src.w);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
