// Turbulent Displace: AbsentMotion's turbulentdisplace (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Pushes the layer around with smooth fractal noise, for fire, water, heat haze or rough natural edges.
// --track@parallax_x:Parallax x,-5000,5000,0,1
// --track@parallax_y:Parallax y,-5000,5000,0,1
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@strength:Strength,0,2.5,0.25,0.001
// --track@evolution:Evolution,-50,50,0.0,0.01
// --track@direction:Direction,0,1,0.0,0.01
// --track@scale:Scale,0.01,50,1.0,0.01
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
// --track@seed:Seed,0,5,0.0,0.01
// --track@octaves:Octaves,1,9,6.0,1
// --track@bias:Bias,-1,1,0.0,0.001
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
  parallax: vec2<f32>,
  offset: vec2<f32>,
  strength: f32,
  evolution: f32,
  direction: f32,
  scale: f32,
  stretch: vec2<f32>,
  seed: f32,
  octaves: f32,
  bias: f32,
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
fn vnoise_ff3(p: vec3<f32>) -> f32 {
  {
    let i: vec3<f32> = floor(p);
    var f: vec3<f32> = fract(p);
    f = ((f * f) * f) * (f * (f * 6.0 - 15.0) + 10.0);
    let a: f32 = mix(hash13_ff3(i), hash13_ff3(i + vec3<f32>(1.0, 0.0, 0.0)), f.x);
    let b: f32 = mix(hash13_ff3(i + vec3<f32>(0.0, 1.0, 0.0)), hash13_ff3(i + vec3<f32>(1.0, 1.0, 0.0)), f.x);
    let c: f32 = mix(hash13_ff3(i + vec3<f32>(0.0, 0.0, 1.0)), hash13_ff3(i + vec3<f32>(1.0, 0.0, 1.0)), f.x);
    let d: f32 = mix(hash13_ff3(i + vec3<f32>(0.0, 1.0, 1.0)), hash13_ff3(i + vec3<f32>(1.0)), f.x);
    return mix(mix(a, b, f.y), mix(c, d, f.y), f.z);
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
          sum = sum + amp * vnoise_ff3(p);
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
    let _0_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _1_unit: f32 = (0.4 * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y) * max(_uniform0.scale, 0.01);
    let _2_rel: vec2<f32> = _0_coord - (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5;
    let _3_p: vec2<f32> = (_2_rel - _uniform0.offset) / (_1_unit * max(abs(_uniform0.stretch), vec2<f32>(0.01)));
    let _4_q: vec3<f32> = vec3<f32>(_3_p + _uniform0.seed * vec2<f32>(37.1, 17.3), _uniform0.evolution);
    let _5_drift: vec2<f32> = _uniform0.parallax / _1_unit;
    let _6_nx: f32 = (fbm_ff3ff2f(_4_q, _uniform0.octaves, _5_drift, 0.62) - 0.5) + _uniform0.bias * 0.5;
    let _7_ny: f32 = (fbm_ff3ff2f(_4_q + vec3<f32>(31.7, 47.3, 11.0), _uniform0.octaves, _5_drift, 0.62) - 0.5) + _uniform0.bias * 0.5;
    let _8_amount: f32 = ((4.0 * _uniform0.strength) * 0.4) * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y;
    var _9_d: vec2<f32> = vec2<f32>(_6_nx, _7_ny) * _8_amount;
    if (_uniform0.direction > 0.5) && (_uniform0.direction < 1.5) {
      _9_d.y = 0.0;
    }
    if (_uniform0.direction > 1.5) && (_uniform0.direction < 2.5) {
      _9_d.x = 0.0;
    }
    if _uniform0.direction > 2.5 {
      {
        var _skTemp1: vec2<f32>;
        if length(_2_rel) > 0.001 {
          _skTemp1 = normalize(_2_rel);
        } else {
          _skTemp1 = vec2<f32>(1.0, 0.0);
        }
        let _10_r: vec2<f32> = _skTemp1;
        var _skTemp2: vec2<f32>;
        if _uniform0.direction < 3.5 {
          _skTemp2 = (_10_r * _6_nx) * _8_amount;
        } else {
          _skTemp2 = (vec2<f32>(-_10_r.y, _10_r.x) * _6_nx) * _8_amount;
        }
        _9_d = _skTemp2;
      }
    }
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, ((_0_coord + _9_d) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
