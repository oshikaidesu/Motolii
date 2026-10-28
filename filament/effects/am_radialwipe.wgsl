// Radial Wipe: AbsentMotion's radialwipe (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Key
// --about:Hides a wedge of the layer swept round Center from Start Angle, like a clock hand. Progress is the angle swept, Count repeats the wedge round the circle and Both sweeps both ways. Animate Progress for a transition.
// --track@progress:Progress,0,360,45.0,1
// --track@start:Start Angle,-3600,3600,0.0,0.5
// --track@R_center_x:Center x,-1,2,0.5,0.01
// --track@R_center_y:Center y,-1,2,0.5,0.01
// --track@feather:Feather,0,180,0.0,0.1
// --track@direction:Direction,0,1,0.0,0.01
// --track@count:Count,1,16,1.0,1
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
  progress: f32,
  start: f32,
  R_center: vec2<f32>,
  feather: f32,
  direction: f32,
  count: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let d: vec2<f32> = coord - _uniform0.R_center * _uniform0.resolution;
    let n: f32 = max(floor(_uniform0.count + 0.5), 1.0);
    let _skTemp1 = degrees(atan2(d.x, -d.y)) - _uniform0.start;
    var a: f32 = (_skTemp1 - 360.0 * floor(_skTemp1 / 360.0));
    if (_uniform0.direction > 0.5) && (_uniform0.direction < 1.5) {
      a = 360.0 - a;
    }
    let _skTemp2 = 360.0 / n;
    a = (a - _skTemp2 * floor(a / _skTemp2)) * n;
    let sweep: f32 = clamp(_uniform0.progress, 0.0, 360.0);
    if sweep <= 0.0 {
      {
        return textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
      }
    }
    if _uniform0.direction > 1.5 {
      {
        a = min(a, 360.0 - a) * 2.0;
      }
    }
    let soft: f32 = max(_uniform0.feather, 0.0001);
    var hide: f32 = 1.0 - smoothstep(max(sweep - soft, 0.0), sweep, a);
    if sweep >= 360.0 {
      hide = 1.0;
    }
    return textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution) * f32(1.0 - hide);
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
