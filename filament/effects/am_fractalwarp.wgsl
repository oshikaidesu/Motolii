// Fractal Warp: AbsentMotion's fractalwarp (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Warps the layer with folded fractal noise, for fire, water and heat shimmer.
// --track@strength:Strength,0,2,0.15,0.001
// --track@scale:Scale,0.01,50,1.0,0.01
// --track@evolution:Evolution,-50,50,0.0,0.01
// --track@complexity:Complexity,1,9,4.0,1
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
  strength: f32,
  scale: f32,
  evolution: f32,
  complexity: f32,
  seed: f32,
  layerRect: vec4<f32>,
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
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _6_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _7_unit: f32 = (0.4 * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y) * max(_uniform0.scale, 0.01);
    let _8_p: vec2<f32> = (_6_coord - (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5) / _7_unit + _uniform0.seed * vec2<f32>(13.1, 7.7);
    let _9_q: vec2<f32> = vec2<f32>(fbm_ff3ff2f(vec3<f32>(_8_p, _uniform0.evolution), _uniform0.complexity, vec2<f32>(0.0), 0.5), fbm_ff3ff2f(vec3<f32>(_8_p + vec2<f32>(5.2, 1.3), _uniform0.evolution), _uniform0.complexity, vec2<f32>(0.0), 0.5));
    let _10_w: vec2<f32> = vec2<f32>(fbm_ff3ff2f(vec3<f32>((_8_p + 4.0 * _9_q) + vec2<f32>(1.7, 9.2), _uniform0.evolution * 1.3), _uniform0.complexity, vec2<f32>(0.0), 0.5), fbm_ff3ff2f(vec3<f32>((_8_p + 4.0 * _9_q) + vec2<f32>(8.3, 2.8), _uniform0.evolution * 1.3), _uniform0.complexity, vec2<f32>(0.0), 0.5));
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, ((_6_coord + (((_10_w - 0.5) * 2.0) * _uniform0.strength) * _7_unit) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
