// Star Prism: AbsentMotion's starprism (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Cuts the layer into a star and gives it depth.
// --track@rotX:Rotation X,-3600,3600,-20.0,0.1
// --track@rotY:Rotation Y,-3600,3600,-35.0,0.1
// --track@rotZ:Rotation Z,-3600,3600,0.0,0.1
// --track@points:Points,3,12,5.0,1
// --track@radius:Radius,0.02,5,0.95,0.01
// --track@inner:Inner Radius,0.05,1,0.45,0.01
// --track@depth:Depth,0.01,5,0.25,0.01
// --track@smoothing:Smoothing,0,1,0.15,0.01
// --track@R_position_x:Position x,-5000,5000,0,1
// --track@R_position_y:Position y,-5000,5000,0,1
// --track@scale:Scale,0.05,10,1.0,0.01
// --track@R_perspective:Perspective,0,1,0.5,0.01
// --track@shading:Shading,0,1,0.6,0.01
// --track@light:Light Angle,-3600,3600,135.0,0.1
// --track@shine:Shine,0,1,0.3,0.01
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
  rotX: f32,
  rotY: f32,
  rotZ: f32,
  points: f32,
  radius: f32,
  inner: f32,
  depth: f32,
  smoothing: f32,
  R_position: vec2<f32>,
  scale: f32,
  R_perspective: f32,
  shading: f32,
  light: f32,
  shine: f32,
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
const PI: f32 = 3.14159274;
fn cover_f2f2f2(uv: vec2<f32>, face: vec2<f32>) -> vec2<f32> {
  {
    let r: f32 = (face.x / max(face.y, 0.0001)) / (layerSize__f2().x / max(layerSize__f2().y, 0.0001));
    var _skTemp1: vec2<f32>;
    if r > 1.0 {
      _skTemp1 = vec2<f32>(1.0, 1.0 / r);
    } else {
      _skTemp1 = vec2<f32>(r, 1.0);
    }
    return (uv - 0.5) * _skTemp1 + 0.5;
  }
}
fn map_ff3(p: vec3<f32>) -> f32 {
  {
    let r: f32 = (_uniform0.smoothing * min(_uniform0.radius * _uniform0.inner, _uniform0.depth)) * 0.5;
    let _8_an: f32 = PI / floor(_uniform0.points + 0.5);
    let _skTemp2 = atan2(p.x, -p.y);
    let _skTemp3 = 2.0 * _8_an;
    var _9_b: f32 = (_skTemp2 - _skTemp3 * floor(_skTemp2 / _skTemp3));
    var _skTemp4: f32;
    if _9_b > _8_an {
      _skTemp4 = 2.0 * _8_an - _9_b;
    } else {
      _skTemp4 = _9_b;
    }
    _9_b = _skTemp4;
    let _10_q: vec2<f32> = length(p.xy) * vec2<f32>(cos(_9_b), sin(_9_b));
    let _11_a: vec2<f32> = vec2<f32>(_uniform0.radius, 0.0);
    let _12_e: vec2<f32> = (_uniform0.radius * _uniform0.inner) * vec2<f32>(cos(_8_an), sin(_8_an)) - _11_a;
    let _13_w: vec2<f32> = _10_q - _11_a;
    let _14_d: f32 = length(_13_w - _12_e * clamp(dot(_13_w, _12_e) / dot(_12_e, _12_e), 0.0, 1.0));
    let _15_w: vec2<f32> = vec2<f32>((select(_14_d, -_14_d, (_12_e.x * _13_w.y - _12_e.y * _13_w.x) > 0.0)) + r, (abs(p.z) - _uniform0.depth) + r);
    return (min(max(_15_w.x, _15_w.y), 0.0) + length(max(_15_w, vec2<f32>(0.0)))) - r;
  }
}
fn normal_f3f3f(p: vec3<f32>, e: f32) -> vec3<f32> {
  {
    const k: vec2<f32> = vec2<f32>(1.0, -1.0);
    return normalize(((k.xyy * map_ff3(p + k.xyy * e) + k.yyx * map_ff3(p + k.yyx * e)) + k.yxy * map_ff3(p + k.yxy * e)) + k.xxx * map_ff3(p + k.xxx * e));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let unit: f32 = max((min(layerSize__f2().x, layerSize__f2().y) * 0.5) * _uniform0.scale, 0.5);
    let rb: f32 = length(vec2<f32>(_uniform0.radius, _uniform0.depth));
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - layerSize__f2() * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let reach: f32 = rb + (2.0 * dist) / (unit * max(dist - rb, 0.1));
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + reach * reach;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _26_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _27_cx: f32 = cos(_26_a.x);
    let _28_sx: f32 = sin(_26_a.x);
    let _29_cy: f32 = cos(_26_a.y);
    let _30_sy: f32 = sin(_26_a.y);
    let _31_cz: f32 = cos(_26_a.z);
    let _32_sz: f32 = sin(_26_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_31_cz, -_32_sz, 0.0, _32_sz, _31_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_29_cy, 0.0, _30_sy, 0.0, 1.0, 0.0, -_30_sy, 0.0, _29_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _27_cx, -_28_sx, 0.0, _28_sx, _27_cx);
    let ro: vec3<f32> = ow * m;
    let rd: vec3<f32> = rw * m;
    let far: f32 = -b + sqrt(disc);
    var t: f32 = max(-b - sqrt(disc), 0.0);
    let px: f32 = (unit * dist) / rw.z;
    var best: f32 = 1e+09;
    var tb: f32 = t;
    var hit: bool = false;
    {
      var i: i32 = 0;
      for (; i < 112; i = i + i32(1)) {
        {
          let h: f32 = map_ff3(ro + rd * t);
          let g: f32 = (h * px) / max(t, 0.0001);
          if g < best {
            {
              best = g;
              tb = t;
            }
          }
          if g < 0.2 {
            {
              hit = true;
              break;
            }
          }
          t = t + h * 0.9;
          if t > far {
            break;
          }
        }
      }
    }
    if (!hit) && (best >= 1.0) {
      return vec4<f32>(0.0);
    }
    if !hit {
      t = tb;
    }
    let e: f32 = max((0.5 * t) / px, 0.0005);
    var p: vec3<f32> = ro + rd * t;
    let n: vec3<f32> = normal_f3f3f(p, e);
    if !hit {
      p = p - n * map_ff3(p);
    }
    let _33_uv: vec2<f32> = cover_f2f2f2((vec2<f32>(select(p.x, -p.x, n.z > 0.0), p.y) / _uniform0.radius) * 0.5 + 0.5, vec2<f32>(1.0));
    let _34_p: vec2<f32> = clamp(_33_uv * layerSize__f2(), vec2<f32>(0.5), max(layerSize__f2() - 0.5, vec2<f32>(0.5)));
    let _35_c: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, (_34_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution));
    let _36_n: vec3<f32> = normalize(m * n);
    let _37_a: f32 = radians(_uniform0.light);
    let _38_l: vec3<f32> = normalize(vec3<f32>(cos(_37_a), -sin(_37_a), -1.2));
    let _39_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_36_n, _38_l), 0.0));
    let _40_s: f32 = _uniform0.shine * pow(max(dot(_36_n, normalize(_38_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_35_c.xyz * _39_k + _40_s * _35_c.w, vec3<f32>(_35_c.w)), _35_c.w);
    if hit && (c.w < 0.996) {
      {
        var s: f32 = far;
        var back: bool = false;
        {
          var i: i32 = 0;
          for (; i < 64; i = i + i32(1)) {
            {
              let h: f32 = map_ff3(ro + rd * s);
              if ((h * px) / max(s, 0.0001)) < 0.2 {
                {
                  back = true;
                  break;
                }
              }
              s = s - h * 0.9;
              if s < t {
                break;
              }
            }
          }
        }
        if back {
          {
            let q: vec3<f32> = ro + rd * s;
            let _41_n: vec3<f32> = -normal_f3f3f(q, e);
            let _42_uv: vec2<f32> = cover_f2f2f2((vec2<f32>(select(q.x, -q.x, _41_n.z > 0.0), q.y) / _uniform0.radius) * 0.5 + 0.5, vec2<f32>(1.0));
            let _43_p: vec2<f32> = clamp(_42_uv * layerSize__f2(), vec2<f32>(0.5), max(layerSize__f2() - 0.5, vec2<f32>(0.5)));
            let _44_c: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, (_43_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution));
            let _45_n: vec3<f32> = normalize(m * _41_n);
            let _46_a: f32 = radians(_uniform0.light);
            let _47_l: vec3<f32> = normalize(vec3<f32>(cos(_46_a), -sin(_46_a), -1.2));
            let _48_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_45_n, _47_l), 0.0));
            let _49_s: f32 = _uniform0.shine * pow(max(dot(_45_n, normalize(_47_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_44_c.xyz * _48_k + _49_s * _44_c.w, vec3<f32>(_44_c.w)), _44_c.w) * (1.0 - c.w);
          }
        }
      }
    }
    var _skTemp5: vec4<f32>;
    if hit {
      _skTemp5 = c;
    } else {
      _skTemp5 = c * (1.0 - best);
    }
    return vec4<f32>(_skTemp5);
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
