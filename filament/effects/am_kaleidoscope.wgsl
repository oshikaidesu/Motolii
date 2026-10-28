// Kaleidoscope: AbsentMotion's kaleidoscope (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Repeat
// --about:Mirrored slices around a point.
// --track@R_center_x:Center x,-1,2,0.5,0.01
// --track@R_center_y:Center y,-1,2,0.5,0.01
// --track@segments:Segments,1,64,6.0,1
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
  segments: f32,
  angle: f32,
  mirror: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec2<f32> = _uniform0.R_center * _uniform0.resolution;
    let d: vec2<f32> = coord - c;
    let radius: f32 = length(d);
    if radius < 1e-05 {
      {
        return textureSample(image_Texture, image_Sampler, c / _uniform0.resolution);
      }
    }
    let slice: f32 = 6.28318548 / max(floor(_uniform0.segments), 1.0);
    let _skTemp1 = atan2(d.y, d.x) - radians(_uniform0.angle);
    var a: f32 = (_skTemp1 - slice * floor(_skTemp1 / slice));
    var _skTemp2: f32;
    if _uniform0.mirror > 0.5 {
      _skTemp2 = min(a, slice - a);
    } else {
      _skTemp2 = a;
    }
    a = _skTemp2 + radians(_uniform0.angle);
    return textureSample(image_Texture, image_Sampler, (c + radius * vec2<f32>(cos(a), sin(a))) / _uniform0.resolution);
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
