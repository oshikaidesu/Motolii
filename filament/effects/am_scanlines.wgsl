// Scanlines: AbsentMotion's scanlines (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --track@spacing:Line Spacing,2.0,20.0,6.0,1.0
// --track@darkness:Darkness,0.0,1.0,0.4,0.01
// --track@roll_speed:Roll Speed,0.0,2.0,0.3,0.01
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
  spacing: f32,
  darkness: f32,
  roll_speed: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let y: f32 = coord.y + (_uniform0.time * _uniform0.roll_speed) * 60.0;
    let line: f32 = fract(y / max(_uniform0.spacing, 0.001));
    let scan: f32 = 1.0 - _uniform0.darkness * f32(line < 0.5);
    var c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.0 {
      return c;
    }
    var rgb: vec3<f32> = c.xyz / c.w;
    rgb = rgb * f32(scan);
    c = vec4<f32>((rgb * c.w), c.w);
    return c;
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
