// Spin Blur: AbsentMotion's spinblur (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Blur
// --track@R_center_x:Center x,-1,2,0.5,0.01
// --track@R_center_y:Center y,-1,2,0.5,0.01
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
  sweep: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec2<f32> = _uniform0.R_center * _uniform0.resolution;
    let span: f32 = radians(_uniform0.sweep);
    if span <= 0.0 {
      {
        return textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
      }
    }
    let d: vec2<f32> = coord - c;
    var sum: vec4<f32> = vec4<f32>(0.0);
    {
      var i: i32 = 0;
      for (; i < 48; i = i + i32(1)) {
        {
          let a: f32 = span * (f32(i) * 0.0212765951 - 0.5);
          sum = sum + textureSample(image_Texture, image_Sampler, (c + vec2<f32>(cos(a) * d.x - sin(a) * d.y, sin(a) * d.x + cos(a) * d.y)) / _uniform0.resolution);
        }
      }
    }
    return sum * 0.020833334;
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
