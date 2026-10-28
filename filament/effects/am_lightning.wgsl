// Lightning: AbsentMotion's lightning (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Draws a forked bolt of lightning between two points on the layer.
// --track@R_from_x:Start x,-1,2,0.5,0.01
// --track@R_from_y:Start y,-1,2,0.05,0.01
// --track@to_x:End x,-1,2,0.5,0.01
// --track@to_y:End y,-1,2,0.95,0.01
// --track@width:Width,0.5,50,4.0,0.1
// --track@jag:Detail,0,1,0.15,0.001
// --track@evolution:Evolution,-1000,1000,0.0,0.01
// --track@glow:Glow,0,4,1.0,0.01
// --track@color_x:Color R,0,1,0.502,0.01
// --track@color_y:Color G,0,1,0.6275,0.01
// --track@color_z:Color B,0,1,1,0.01
// --track@color_w:Color A,0,1,1,0.01
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
  R_from: vec2<f32>,
  to: vec2<f32>,
  width: f32,
  jag: f32,
  evolution: f32,
  glow: f32,
  color: vec4<f32>,
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
fn bolt_ff2f2f2ff(coord: vec2<f32>, a: vec2<f32>, b: vec2<f32>, amp: f32, salt: f32) -> f32 {
  {
    let ab: vec2<f32> = b - a;
    let len: f32 = max(length(ab), 1.0);
    let dir: vec2<f32> = ab / len;
    let nrm: vec2<f32> = vec2<f32>(-dir.y, dir.x);
    let t: f32 = clamp(dot(coord - a, dir) / len, 0.0, 1.0);
    let n: f32 = fbm_ff3ff2f(vec3<f32>(t * 6.0 + salt, _uniform0.seed * 3.1 + salt, _uniform0.evolution), 6.0, vec2<f32>(0.0), 0.62) - 0.5;
    let at: vec2<f32> = (a + ab * t) + (((nrm * n) * amp) * len) * sin(3.14159274 * t);
    return length(coord - at);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let a: vec2<f32> = _uniform0.R_from * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let b: vec2<f32> = _uniform0.to * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    var d: f32 = bolt_ff2f2f2ff(coord, a, b, _uniform0.jag * 2.0, 0.0);
    let mid: vec2<f32> = mix(a, b, vec2<f32>(0.45)) + (vec2<f32>(-(b - a).y, (b - a).x) * (fbm_ff3ff2f(vec3<f32>(_uniform0.seed, 7.7, _uniform0.evolution), 3.0, vec2<f32>(0.0), 0.5) - 0.5)) * _uniform0.jag;
    let fork: vec2<f32> = (mid + (b - a) * 0.35) + ((vec2<f32>(-(b - a).y, (b - a).x) * 0.25) * _uniform0.jag) * 2.0;
    d = min(d, bolt_ff2f2f2ff(coord, mid, fork, _uniform0.jag * 2.0, 17.3) + _uniform0.width * 0.4);
    let core: f32 = 1.0 - smoothstep(_uniform0.width * 0.25, _uniform0.width * 0.5 + 0.5, d);
    let halo: f32 = exp(-d / max(_uniform0.width * 3.0, 0.5)) * _uniform0.glow;
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let light: vec3<f32> = vec3<f32>(_uniform0.color.xyz) * halo + vec3<f32>(core);
    let rgb: vec3<f32> = clamp((base + light) - base * light, vec3<f32>(0.0), vec3<f32>(1.0));
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
