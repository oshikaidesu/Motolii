// Ribbon: AbsentMotion's ribbon (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Draws a twisting ribbon of changing width across the opaque parts of the layer; animate Phase to make it flow, or punch it out of the layer.
// --track@magnitude:Magnitude,0,1,0.2,0.001
// --track@frequency:Frequency,0.1,20,3.0,0.01
// --track@width:Width,0,5,0.2,0.001
// --track@phase:Phase,0,1000,0.0,0.01
// --track@offset:Offset,-1,1,0.0,0.001
// --track@color_x:Color R,0,1,0.1765,0.01
// --track@color_y:Color G,0,1,0.1176,0.01
// --track@color_z:Color B,0,1,0.9647,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@smoothing:Smoothing,0,1,0.01,0.001
// --track@tint_x:Tint R,0,1,0.5333,0.01
// --track@tint_y:Tint G,0,1,0.8,0.01
// --track@tint_z:Tint B,0,1,1,0.01
// --track@tint_w:Tint A,0,1,1,0.01
// --track@tintAmount:Tint Amount,0,1,0.0,0.001
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
  time: f32,
  magnitude: f32,
  frequency: f32,
  width: f32,
  phase: f32,
  offset: f32,
  color: vec4<f32>,
  punch: f32,
  smoothing: f32,
  tint: vec4<f32>,
  tintAmount: f32,
  angle: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn wobble_ff(x: f32) -> f32 {
  {
    let i: f32 = floor(x);
    let t: f32 = x - i;
    let a: f32 = (fract(sin(i * 127.1 + 311.7) * 43758.5469) * 2.0 - 1.0) * t;
    let b: f32 = (fract(sin((i + 1.0) * 127.1 + 311.7) * 43758.5469) * 2.0 - 1.0) * (t - 1.0);
    return mix(a, b, (t * t) * (3.0 - 2.0 * t)) * 2.2;
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return vec4<f32>(0.0);
    }
    let ra: f32 = radians(_uniform0.angle);
    let ax: vec2<f32> = vec2<f32>(cos(ra), sin(ra));
    let ay: vec2<f32> = vec2<f32>(-ax.y, ax.x);
    let span: f32 = max(abs(dot(_uniform0.resolution, abs(ax))), 1.0);
    let tall: f32 = max(abs(dot(_uniform0.resolution, abs(ay))), 1.0);
    let r: vec2<f32> = vec2<f32>(dot(coord - _uniform0.resolution * 0.5, ax), dot(coord - _uniform0.resolution * 0.5, ay));
    let x: f32 = r.x / span + 0.5;
    let f: f32 = max(_uniform0.frequency, 0.1);
    let centre: f32 = ((clamp(wobble_ff(x * f + _uniform0.phase), -1.0, 1.0) * _uniform0.magnitude + _uniform0.offset) * tall) * 0.5;
    let d: f32 = 0.5 / span;
    let slope: f32 = (((((wobble_ff((x + d) * f + _uniform0.phase) - wobble_ff((x - d) * f + _uniform0.phase)) / (2.0 * d)) * _uniform0.magnitude) * tall) * 0.5) / span;
    let turn: f32 = wobble_ff(((x * f) * 0.5 + _uniform0.phase * 0.7) + 17.3);
    let halfWidth: f32 = ((max(_uniform0.width, 0.0) * tall) * 0.06) * abs(turn) + 0.25;
    let off: f32 = abs(r.y - centre) / sqrt(1.0 + slope * slope);
    let soft: f32 = 0.75 + (max(_uniform0.smoothing, 0.0) * tall) * 0.1;
    let cover: f32 = (1.0 - smoothstep(halfWidth - soft, halfWidth + soft, off)) * f32(src.w);
    if _uniform0.punch > 0.5 {
      return src * f32(1.0 - cover);
    }
    var _skTemp1: vec3<f32>;
    if turn < 0.0 {
      _skTemp1 = vec3<f32>(mix(vec3<f32>(_uniform0.color.xyz), vec3<f32>(_uniform0.tint.xyz), vec3<f32>(clamp(_uniform0.tintAmount, 0.0, 1.0))));
    } else {
      _skTemp1 = _uniform0.color.xyz;
    }
    let rgb: vec3<f32> = _skTemp1;
    let a: f32 = cover * f32(_uniform0.color.w);
    return vec4<f32>(rgb * f32(a), f32(a)) + src * f32(1.0 - a);
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
