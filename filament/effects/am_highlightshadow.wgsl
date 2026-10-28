// Highlights & Shadows: AbsentMotion's highlightshadow (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Recovers or boosts bright highlights and lifts or deepens dark shadows, going by the brightness of each pixel.
// --track@highlights:Highlights,0.0,2.0,1.0,0.01
// --track@shadows:Shadows,0.0,2.0,1.0,0.01
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
  highlights: f32,
  shadows: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.001 {
      return vec4<f32>(0.0);
    }
    let col: vec3<f32> = c.xyz / c.w;
    let l: f32 = saturate(dot(col, vec3<f32>(0.2126, 0.7152, 0.0722)));
    let hi: f32 = (1.89 * pow(l, 3.5)) * (1.0 - l);
    let lo: f32 = (1.89 * pow(1.0 - l, 3.5)) * l;
    let outc: vec3<f32> = vec3<f32>(clamp((vec3<f32>(col) + (_uniform0.highlights - 1.0) * f32(hi)) + (_uniform0.shadows - 1.0) * f32(lo), vec3<f32>(0.0), vec3<f32>(1.0)));
    return vec4<f32>(outc * c.w, c.w);
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
