// Pyramid: AbsentMotion's pyramid (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Puts the layer on the faces of a square pyramid.
// --track@rotX:Rotation X,-3600,3600,-15.0,0.1
// --track@rotY:Rotation Y,-3600,3600,-35.0,0.1
// --track@rotZ:Rotation Z,-3600,3600,0.0,0.1
// --track@base:Base,0.02,5,0.75,0.01
// --track@height:Height,0.02,5,1.3,0.01
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
  base: f32,
  height: f32,
  R_position: vec2<f32>,
  scale: f32,
  R_perspective: f32,
  shading: f32,
  light: f32,
  shine: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn pic_f4f2(uv: vec2<f32>) -> vec4<f32> {
  {
    return vec4<f32>(textureSample(image_Texture, image_Sampler, clamp(uv * _uniform0.resolution, vec2<f32>(0.5), max(_uniform0.resolution - 0.5, vec2<f32>(0.5))) / _uniform0.resolution));
  }
}
fn cover_f2f2f2(uv: vec2<f32>, face: vec2<f32>) -> vec2<f32> {
  {
    let r: f32 = (face.x / max(face.y, 0.0001)) / (_uniform0.resolution.x / max(_uniform0.resolution.y, 0.0001));
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
    let s: f32 = 2.0 * _uniform0.base;
    var _1_p: vec3<f32> = vec3<f32>(p.x, _uniform0.height * 0.5 - p.y, p.z) / s;
    let _2_h: f32 = _uniform0.height / s;
    let _3_m2: f32 = _2_h * _2_h + 0.25;
    _1_p = vec3<f32>((abs(_1_p.xz)), _1_p.y).xzy;
    _1_p = vec3<f32>((select(_1_p.xz, _1_p.zx, vec2<bool>(_1_p.z > _1_p.x))), _1_p.y).xzy;
    _1_p = vec3<f32>((_1_p.xz - 0.5), _1_p.y).xzy;
    let _4_q: vec3<f32> = vec3<f32>(_1_p.z, _2_h * _1_p.y - 0.5 * _1_p.x, _2_h * _1_p.x + 0.5 * _1_p.y);
    let _5_s: f32 = max(-_4_q.x, 0.0);
    let _6_t: f32 = clamp((_4_q.y - 0.5 * _1_p.z) / (_3_m2 + 0.25), 0.0, 1.0);
    let _7_a: f32 = (_3_m2 * (_4_q.x + _5_s)) * (_4_q.x + _5_s) + _4_q.y * _4_q.y;
    let _8_b: f32 = (_3_m2 * (_4_q.x + 0.5 * _6_t)) * (_4_q.x + 0.5 * _6_t) + (_4_q.y - _3_m2 * _6_t) * (_4_q.y - _3_m2 * _6_t);
    var _skTemp2: f32;
    if min(_4_q.y, -_4_q.x * _3_m2 - _4_q.y * 0.5) > 0.0 {
      _skTemp2 = 0.0;
    } else {
      _skTemp2 = min(_7_a, _8_b);
    }
    let _9_d2: f32 = _skTemp2;
    return (sqrt((_9_d2 + _4_q.z * _4_q.z) / _3_m2) * sign(max(_4_q.z, -_1_p.y))) * s;
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
    let unit: f32 = max((min(_uniform0.resolution.x, _uniform0.resolution.y) * 0.5) * _uniform0.scale, 0.5);
    let rb: f32 = length(vec3<f32>(_uniform0.base, _uniform0.height * 0.5, _uniform0.base));
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let reach: f32 = rb + (2.0 * dist) / (unit * max(dist - rb, 0.1));
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + reach * reach;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _24_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _25_cx: f32 = cos(_24_a.x);
    let _26_sx: f32 = sin(_24_a.x);
    let _27_cy: f32 = cos(_24_a.y);
    let _28_sy: f32 = sin(_24_a.y);
    let _29_cz: f32 = cos(_24_a.z);
    let _30_sz: f32 = sin(_24_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_29_cz, -_30_sz, 0.0, _30_sz, _29_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_27_cy, 0.0, _28_sy, 0.0, 1.0, 0.0, -_28_sy, 0.0, _27_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _25_cx, -_26_sx, 0.0, _26_sx, _25_cx);
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
      for (; i < 96; i = i + i32(1)) {
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
    let _31_h: vec3<f32> = vec3<f32>(_uniform0.base, _uniform0.height * 0.5, _uniform0.base);
    let _32_a: vec3<f32> = abs(n);
    var _33_uv: vec2<f32>;
    var _34_face: vec2<f32>;
    if (_32_a.z >= _32_a.x) && (_32_a.z >= _32_a.y) {
      {
        _33_uv = vec2<f32>(select(-p.x, p.x, n.z < 0.0), p.y) / _31_h.xy;
        _34_face = _31_h.xy;
      }
    } else {
      if _32_a.x >= _32_a.y {
        {
          _33_uv = vec2<f32>(select(-p.z, p.z, n.x > 0.0), p.y) / _31_h.zy;
          _34_face = _31_h.zy;
        }
      } else {
        {
          _33_uv = vec2<f32>(p.x, select(p.z, -p.z, n.y < 0.0)) / _31_h.xz;
          _34_face = _31_h.xz;
        }
      }
    }
    let _35_c: vec4<f32> = pic_f4f2(cover_f2f2f2(_33_uv * 0.5 + 0.5, _34_face));
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
            let _42_h: vec3<f32> = vec3<f32>(_uniform0.base, _uniform0.height * 0.5, _uniform0.base);
            let _43_a: vec3<f32> = abs(_41_n);
            var _44_uv: vec2<f32>;
            var _45_face: vec2<f32>;
            if (_43_a.z >= _43_a.x) && (_43_a.z >= _43_a.y) {
              {
                _44_uv = vec2<f32>(select(-q.x, q.x, _41_n.z < 0.0), q.y) / _42_h.xy;
                _45_face = _42_h.xy;
              }
            } else {
              if _43_a.x >= _43_a.y {
                {
                  _44_uv = vec2<f32>(select(-q.z, q.z, _41_n.x > 0.0), q.y) / _42_h.zy;
                  _45_face = _42_h.zy;
                }
              } else {
                {
                  _44_uv = vec2<f32>(q.x, select(q.z, -q.z, _41_n.y < 0.0)) / _42_h.xz;
                  _45_face = _42_h.xz;
                }
              }
            }
            let _46_c: vec4<f32> = pic_f4f2(cover_f2f2f2(_44_uv * 0.5 + 0.5, _45_face));
            let _47_n: vec3<f32> = normalize(m * _41_n);
            let _48_a: f32 = radians(_uniform0.light);
            let _49_l: vec3<f32> = normalize(vec3<f32>(cos(_48_a), -sin(_48_a), -1.2));
            let _50_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_47_n, _49_l), 0.0));
            let _51_s: f32 = _uniform0.shine * pow(max(dot(_47_n, normalize(_49_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_46_c.xyz * _50_k + _51_s * _46_c.w, vec3<f32>(_46_c.w)), _46_c.w) * (1.0 - c.w);
          }
        }
      }
    }
    var _skTemp3: vec4<f32>;
    if hit {
      _skTemp3 = c;
    } else {
      _skTemp3 = c * (1.0 - best);
    }
    return vec4<f32>(_skTemp3);
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
