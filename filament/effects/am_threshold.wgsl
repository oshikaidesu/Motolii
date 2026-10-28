// Threshold: AbsentMotion's threshold (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Pure black and white, split at a brightness.
// --track@level:Threshold,0,1,0.4,0.01
// --track@softness:Feather,0,1,0.0,0.01
// --track@blend:Blending Mode,0,1,0.0,0.01
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
  level: f32,
  softness: f32,
  invert: f32,
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
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _1_color: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    let _2_c: vec4<f32> = vec4<f32>(_1_color.xyz / max(_1_color.w, 0.0001), _1_color.w);
    let _3_lum: f32 = dot(vec3<f32>(_2_c.xyz), vec3<f32>(0.2126, 0.7152, 0.0722));
    var _skTemp1: f32;
    if _uniform0.softness > 0.0 {
      _skTemp1 = smoothstep(_uniform0.level - _uniform0.softness, (_uniform0.level + _uniform0.softness) + 0.0001, _3_lum);
    } else {
      _skTemp1 = step(_uniform0.level, _3_lum);
    }
    var _4_v: f32 = _skTemp1;
    if _uniform0.invert > 0.5 {
      _4_v = 1.0 - _4_v;
    }
    let _5_rgb: vec3<f32> = clamp(blendWith_f3ff3f3(_uniform0.blend, vec3<f32>(_2_c.xyz), vec3<f32>(_4_v)), vec3<f32>(0.0), vec3<f32>(1.0));
    (*_stageOut).sk_FragColor = vec4<f32>(vec3<f32>(_5_rgb) * _2_c.w, _2_c.w);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
