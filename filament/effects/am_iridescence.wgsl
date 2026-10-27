// Iridescence: AbsentMotion's iridescence (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --track@scale:Scale,0.01,50,1.0,0.01
// --track@evolution:Evolution,-50,50,0.0,0.01
// --track@shift:Hue Shift,-10,10,0.0,0.001
// --track@saturation:Saturation,0,1,0.6,0.001
// --track@alpha:Alpha,0,1,0.8,0.001
// --track@blend:Blending Mode,0,1,0.0,0.01
// --track@seed:Seed,0,5,0.0,0.01
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
  scale: f32,
  evolution: f32,
  shift: f32,
  saturation: f32,
  alpha: f32,
  blend: f32,
  seed: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn hash13_ff3(_skParam0: vec3<f32>) -> f32 {
  var p = _skParam0;
  {
    p = fract(p * 0.1031);
    p = p + dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
  }
}
fn fbm_ff3ff2f(_skParam0: vec3<f32>, octaves: f32, drift: vec2<f32>, gain: f32) -> f32 {
  var p = _skParam0;
  {
    var sum: f32 = 0.0;
    var amp: f32 = 0.5;
    var norm: f32 = 0.0;
    {
      var i: i32 = 0;
      for (; i < 9; i = i + i32(1)) {
        {
          if f32(i) >= octaves {
            break;
          }
          let _0_i: vec3<f32> = floor(p);
          var _1_f: vec3<f32> = fract(p);
          _1_f = ((_1_f * _1_f) * _1_f) * (_1_f * (_1_f * 6.0 - 15.0) + 10.0);
          let _2_a: f32 = mix(hash13_ff3(_0_i), hash13_ff3(_0_i + vec3<f32>(1.0, 0.0, 0.0)), _1_f.x);
          let _3_b: f32 = mix(hash13_ff3(_0_i + vec3<f32>(0.0, 1.0, 0.0)), hash13_ff3(_0_i + vec3<f32>(1.0, 1.0, 0.0)), _1_f.x);
          let _4_c: f32 = mix(hash13_ff3(_0_i + vec3<f32>(0.0, 0.0, 1.0)), hash13_ff3(_0_i + vec3<f32>(1.0, 0.0, 1.0)), _1_f.x);
          let _5_d: f32 = mix(hash13_ff3(_0_i + vec3<f32>(0.0, 1.0, 1.0)), hash13_ff3(_0_i + vec3<f32>(1.0)), _1_f.x);
          sum = sum + amp * mix(mix(_2_a, _3_b, _1_f.y), mix(_4_c, _5_d, _1_f.y), _1_f.z);
          norm = norm + amp;
          amp = amp * gain;
          p = vec3<f32>((p.xy * 2.03 + drift) + vec2<f32>(17.13, 9.71), p.z * 1.7 + 3.1);
        }
      }
    }
    return sum / max(norm, 0.0001);
  }
}
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
    let unit: f32 = (0.4 * _uniform0.resolution.y) * max(_uniform0.scale, 0.01);
    let p: vec2<f32> = (coord - _uniform0.resolution * 0.5) / unit + _uniform0.seed * vec2<f32>(11.3, 5.9);
    let w: f32 = fbm_ff3ff2f(vec3<f32>(p, _uniform0.evolution), 5.0, vec2<f32>(0.0), 0.5);
    let v: f32 = fbm_ff3ff2f(vec3<f32>((p + 3.0 * w) + vec2<f32>(2.7, 8.1), _uniform0.evolution * 0.7), 5.0, vec2<f32>(0.0), 0.5);
    let _skTemp1 = fract(v * 2.5 + _uniform0.shift) * 6.0 + vec3<f32>(0.0, 4.0, 2.0);
    let film: vec3<f32> = mix(vec3<f32>(1.0), clamp(abs((_skTemp1 - 6.0 * floor(_skTemp1 / 6.0)) - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(clamp(_uniform0.saturation, 0.0, 1.0)));
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let rgb: vec3<f32> = mix(base, clamp(blendWith_f3ff3f3(_uniform0.blend, base, film), vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(clamp(_uniform0.alpha, 0.0, 1.0)));
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
