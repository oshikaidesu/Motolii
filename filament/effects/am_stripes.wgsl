// Stripes: AbsentMotion's stripes (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Draws a pattern of stripes over the opaque parts of the layer, or punches the stripes out of it.
// --track@count:Count,0.5,200,10.0,0.1
// --track@width:Width,0,1,0.5,0.01
// --track@angle:Angle,-3600,3600,45.0,0.5
// --track@phase:Phase,-1000,1000,0.0,0.01
// --track@color_x:Color R,0,1,0.1765,0.01
// --track@color_y:Color G,0,1,0.1176,0.01
// --track@color_z:Color B,0,1,0.9647,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@smoothing:Smoothing,0,0.5,0.01,0.001
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
  count: f32,
  width: f32,
  angle: f32,
  phase: f32,
  color: vec4<f32>,
  punch: f32,
  smoothing: f32,
  screenSpace: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn layerSize__f2() -> vec2<f32> {
  {
    return select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    var _skTemp1: vec2<f32>;
    if _uniform0.screenSpace > 0.5 {
      _skTemp1 = _uniform0.screen.zw;
    } else {
      _skTemp1 = layerSize__f2();
    }
    let patternSize: vec2<f32> = _skTemp1;
    var _skTemp2: vec2<f32>;
    if _uniform0.screenSpace > 0.5 {
      _skTemp2 = _uniform0.screen.xy + _uniform0.screen.zw * 0.5;
    } else {
      _skTemp2 = layerSize__f2() * 0.5;
    }
    let origin: vec2<f32> = _skTemp2;
    let a: f32 = radians(_uniform0.angle);
    let period: f32 = patternSize.y / (2.475 * max(_uniform0.count, 0.5));
    let u: f32 = dot(coord - origin, vec2<f32>(-sin(a), cos(a))) / period + _uniform0.phase;
    let e: f32 = abs(fract(u) - 0.5);
    let edge: f32 = max(_uniform0.smoothing * 0.5, 0.5 / period);
    var s: f32 = 1.0 - smoothstep(_uniform0.width * 0.5 - edge, _uniform0.width * 0.5 + edge, e);
    s = s * step(0.0001, _uniform0.width);
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
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0))));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
