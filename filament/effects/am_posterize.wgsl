// Posterize: AbsentMotion's posterize (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Cuts each color channel into a number of flat Steps, so smooth gradients turn into bands. Offset moves where the band edges fall.
// --track@levels:Steps,2,255,10.0,1
// --track@offset:Offset,-1,1,0.0,0.001
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
  levels: f32,
  offset: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let col: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if col.w < 0.001 {
      return vec4<f32>(0.0);
    }
    let rgb: vec3<f32> = vec3<f32>(col.xyz / col.w);
    let n: f32 = max(_uniform0.levels, 1.0);
    let q: vec3<f32> = clamp((floor(rgb * n + _uniform0.offset) - _uniform0.offset) / n, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(vec3<f32>(q) * col.w, col.w);
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
