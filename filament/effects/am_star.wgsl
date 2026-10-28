// Star: AbsentMotion's star (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Draws one glowing star on the layer.
// --track@R_position_x:Position x,-1,2,0.5,0.01
// --track@R_position_y:Position y,-1,2,0.5,0.01
// --track@radius:Radius,0,2.5,0.3,0.001
// --track@brightness:Brightness,0,5,3.0,0.001
// --track@shine:Shine,-1,1,-0.1,0.001
// --track@color_x:Color R,0,1,0.1765,0.01
// --track@color_y:Color G,0,1,0.1176,0.01
// --track@color_z:Color B,0,1,0.9647,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@points:Points,2,16,5.0,1
// --track@alpha:Alpha,0,1,1.0,0.001
// --track@blend:Blending Mode,0,1,2.0,0.01
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
  R_position: vec2<f32>,
  radius: f32,
  brightness: f32,
  shine: f32,
  color: vec4<f32>,
  points: f32,
  alpha: f32,
  blend: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn blendWith_f3ff3f3(m: f32, b: vec3<f32>, s: vec3<f32>) -> vec3<f32> {
  {
    if m < 0.5 {
      return s;
    }
    if m < 1.5 {
      return b * s;
    }
    if m < 2.5 {
      return (b + s) - b * s;
    }
    if m < 3.5 {
      return mix((2.0 * b) * s, 1.0 - (2.0 * (1.0 - b)) * (1.0 - s), step(vec3<f32>(0.5), b));
    }
    if m < 4.5 {
      {
        let d: vec3<f32> = mix(sqrt(b), ((16.0 * b - 12.0) * b + 4.0) * b, step(b, vec3<f32>(0.25)));
        return mix(b - ((1.0 - 2.0 * s) * b) * (1.0 - b), b + (2.0 * s - 1.0) * (d - b), step(vec3<f32>(0.5), s));
      }
    }
    if m < 5.5 {
      return mix((2.0 * b) * s, 1.0 - (2.0 * (1.0 - b)) * (1.0 - s), step(vec3<f32>(0.5), s));
    }
    if m < 6.5 {
      return min(b + s, vec3<f32>(1.0));
    }
    if m < 7.5 {
      return min(b, s);
    }
    if m < 8.5 {
      return max(b, s);
    }
    if m < 9.5 {
      return abs(b - s);
    }
    if m < 10.5 {
      return min(b / max(1.0 - s, vec3<f32>(0.0001)), vec3<f32>(1.0));
    }
    return 1.0 - min((1.0 - b) / max(s, vec3<f32>(0.0001)), vec3<f32>(1.0));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let p: vec2<f32> = coord - _uniform0.R_position * _uniform0.resolution;
    let r: f32 = length(p) / max((_uniform0.radius * 0.5) * min(_uniform0.resolution.x, _uniform0.resolution.y), 0.0001);
    var _skTemp1: f32;
    if dot(p, p) > 0.0 {
      _skTemp1 = atan2(p.y, p.x);
    } else {
      _skTemp1 = 0.0;
    }
    let t: f32 = _skTemp1 + 1.57079625;
    let spike: f32 = pow(0.5 + 0.5 * cos(floor(_uniform0.points + 0.5) * t), 2.0);
    let rr: f32 = r / mix(0.12, 1.0, spike);
    let g: f32 = (_uniform0.brightness * 0.333333343) * (1.8 * exp(-3.0 * rr) + 0.45 * exp((-2.0 * r) / max(1.0 + _uniform0.shine, 0.05)));
    let glow: vec3<f32> = vec3<f32>(_uniform0.color.xyz) * min(g, 1.0) + vec3<f32>(max(g - 1.0, 0.0));
    let amount: f32 = clamp(_uniform0.alpha, 0.0, 1.0) * f32(_uniform0.color.w);
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let cover: f32 = clamp(g, 0.0, 1.0);
    var _skTemp2: vec3<f32>;
    if (_uniform0.blend > 5.5) && (_uniform0.blend < 6.5) {
      _skTemp2 = min(base + glow, vec3<f32>(1.0));
    } else {
      _skTemp2 = mix(base, clamp(blendWith_f3ff3f3(_uniform0.blend, base, clamp(glow, vec3<f32>(0.0), vec3<f32>(1.0))), vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(cover));
    }
    let mixed: vec3<f32> = _skTemp2;
    return vec4<f32>(vec3<f32>(mix(base, mixed, vec3<f32>(amount))) * src.w, src.w);
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
