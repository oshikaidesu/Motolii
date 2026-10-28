// Pinch/Bulge: AbsentMotion's bulge (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Bulges the layer out like a lens (positive Strength) or pinches it in toward Center (negative) inside a circle of Radius, a share of the layer size.
// --track@R_center_x:Center x,0,1,0.5,0.01
// --track@R_center_y:Center y,0,1,0.5,0.01
// --track@amount:Strength,-1,1,-0.9,0.01
// --track@radius:Radius,0,8,0.3,0.001
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
  R_center: vec2<f32>,
  amount: f32,
  radius: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec2<f32> = _uniform0.R_center * _uniform0.resolution;
    let d: vec2<f32> = coord - c;
    let r: f32 = max(_uniform0.radius * min(_uniform0.resolution.x, _uniform0.resolution.y), 0.0001);
    let a: f32 = clamp(_uniform0.amount, -1.0, 1.0);
    let len: f32 = length(d);
    if (a < 0.0) && (len < r) {
      {
        let u: f32 = len / r;
        return textureSample(image_Texture, image_Sampler, (c + (d / max(len, 0.0001)) * (r * pow(u, 1.0 + (0.77 * a) * (1.0 - u)))) / _uniform0.resolution);
      }
    }
    let t: f32 = clamp(1.0 - len / r, 0.0, 1.0);
    var k: f32 = max(1.0 - (a * t) * t, 0.0);
    k = k * k;
    return textureSample(image_Texture, image_Sampler, (c + (d * k) * k) / _uniform0.resolution);
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
