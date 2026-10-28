// Inner Pinch/Bulge: AbsentMotion's innerbulge (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Bulges the middle of the layer out or pinches it in, while the edges of the layer stay where they are.
// --track@strength:Strength,-1,1,0.5,0.001
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
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let h: vec2<f32> = _uniform0.resolution * 0.5;
    let p: vec2<f32> = (coord - h) / h;
    if (abs(p.x) >= 1.0) || (abs(p.y) >= 1.0) {
      {
        return textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
      }
    }
    let e: vec2<f32> = 1.0 - p * p;
    let w: f32 = e.x * e.y;
    return textureSample(image_Texture, image_Sampler, (h + (p * (1.0 - (_uniform0.strength * 0.6) * w)) * h) / _uniform0.resolution);
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
