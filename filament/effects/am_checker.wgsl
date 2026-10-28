// Checker: AbsentMotion's checker (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Draws a checkerboard over the opaque parts of the layer, or punches the checkers out of it.
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@count:Count,0.5,200,10.0,0.1
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
// --track@color_x:Color R,0,1,0.1765,0.01
// --track@color_y:Color G,0,1,0.1176,0.01
// --track@color_z:Color B,0,1,0.9647,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@angle:Angle,-3600,3600,0.0,0.5
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
  screen: vec4<f32>,
  time: f32,
  offset: vec2<f32>,
  count: f32,
  stretch: vec2<f32>,
  color: vec4<f32>,
  punch: f32,
  angle: f32,
  screenSpace: f32,
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
    let patternSize: vec2<f32> = select(_uniform0.resolution, _uniform0.screen.zw, vec2<bool>(_uniform0.screenSpace > 0.5));
    var _skTemp1: vec2<f32>;
    if _uniform0.screenSpace > 0.5 {
      _skTemp1 = _uniform0.screen.xy + _uniform0.screen.zw * 0.5;
    } else {
      _skTemp1 = _uniform0.resolution * 0.5;
    }
    let origin: vec2<f32> = _skTemp1;
    let cell: f32 = patternSize.y / (2.475 * max(_uniform0.count, 0.5));
    let a: f32 = radians(_uniform0.angle);
    var p: vec2<f32> = (coord - origin) - _uniform0.offset;
    p = vec2<f32>(cos(a) * p.x + sin(a) * p.y, -sin(a) * p.x + cos(a) * p.y) / (max(abs(_uniform0.stretch), vec2<f32>(0.01)) * cell);
    let e: vec2<f32> = (min(fract(p), 1.0 - fract(p)) * max(abs(_uniform0.stretch), vec2<f32>(0.01))) * cell;
    let _skTemp2 = floor(p.x) + floor(p.y);
    let on: f32 = (_skTemp2 - 2.0 * floor(_skTemp2 / 2.0));
    let aa: f32 = clamp(min(e.x, e.y) + 0.5, 0.0, 1.0);
    let s: f32 = mix(0.5, on, aa);
    if _uniform0.punch > 0.5 {
      return src * f32(1.0 - s);
    }
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let rgb: vec3<f32> = mix(base, vec3<f32>(_uniform0.color.xyz), vec3<f32>(s * f32(_uniform0.color.w)));
    return vec4<f32>(vec3<f32>(rgb) * src.w, src.w);
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
