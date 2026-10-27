// Box: AbsentMotion's box (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --track@width:Width,0.02,5,0.75,0.01
// --track@height:Height,0.02,5,0.75,0.01
// --track@depth:Depth,0.01,5,0.4,0.01
// --track@smoothing:Smoothing,0,1,0.15,0.01
// --track@R_position_x:Position x,-5000,5000,0,1
// --track@R_position_y:Position y,-5000,5000,0,1
// --track@scale:Scale,0.05,10,1.0,0.01
// --track@R_perspective:Perspective,0,1,0.5,0.01
// --track@shading:Shading,0,1,0.6,0.01
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
  width: f32,
  height: f32,
  depth: f32,
  smoothing: f32,
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
fn dims_f3() -> vec3<f32> {
  {
    let side: f32 = min(_uniform0.resolution.x, _uniform0.resolution.y);
    return vec3<f32>((_uniform0.width * _uniform0.resolution.x) / side, (_uniform0.height * _uniform0.resolution.y) / side, _uniform0.depth);
  }
}
fn map_ff3(p: vec3<f32>) -> f32 {
  {
    let h: vec3<f32> = dims_f3();
    let _1_r: f32 = (_uniform0.smoothing * min(h.x, min(h.y, h.z))) * 0.5;
    let _2_q: vec3<f32> = abs(p) - (h - _1_r);
    return (length(max(_2_q, vec3<f32>(0.0))) + min(max(_2_q.x, max(_2_q.y, _2_q.z)), 0.0)) - _1_r;
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
    let rb: f32 = length(dims_f3());
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let reach: f32 = rb + (2.0 * dist) / (unit * max(dist - rb, 0.1));
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + reach * reach;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _17_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _18_cx: f32 = cos(_17_a.x);
    let _19_sx: f32 = sin(_17_a.x);
    let _20_cy: f32 = cos(_17_a.y);
    let _21_sy: f32 = sin(_17_a.y);
    let _22_cz: f32 = cos(_17_a.z);
    let _23_sz: f32 = sin(_17_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_22_cz, -_23_sz, 0.0, _23_sz, _22_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_20_cy, 0.0, _21_sy, 0.0, 1.0, 0.0, -_21_sy, 0.0, _20_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _18_cx, -_19_sx, 0.0, _19_sx, _18_cx);
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
    let _24_h: vec3<f32> = dims_f3();
    let _25_a: vec3<f32> = abs(n);
    var _26_uv: vec2<f32>;
    var _27_face: vec2<f32>;
    if (_25_a.z >= _25_a.x) && (_25_a.z >= _25_a.y) {
      {
        _26_uv = vec2<f32>(select(-p.x, p.x, n.z < 0.0), p.y) / _24_h.xy;
        _27_face = _24_h.xy;
      }
    } else {
      if _25_a.x >= _25_a.y {
        {
          _26_uv = vec2<f32>(select(-p.z, p.z, n.x > 0.0), p.y) / _24_h.zy;
          _27_face = _24_h.zy;
        }
      } else {
        {
          _26_uv = vec2<f32>(p.x, select(p.z, -p.z, n.y < 0.0)) / _24_h.xz;
          _27_face = _24_h.xz;
        }
      }
    }
    let _28_c: vec4<f32> = pic_f4f2(cover_f2f2f2(_26_uv * 0.5 + 0.5, _27_face));
    let _29_n: vec3<f32> = normalize(m * n);
    let _30_a: f32 = radians(_uniform0.light);
    let _31_l: vec3<f32> = normalize(vec3<f32>(cos(_30_a), -sin(_30_a), -1.2));
    let _32_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_29_n, _31_l), 0.0));
    let _33_s: f32 = _uniform0.shine * pow(max(dot(_29_n, normalize(_31_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_28_c.xyz * _32_k + _33_s * _28_c.w, vec3<f32>(_28_c.w)), _28_c.w);
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
            let _34_n: vec3<f32> = -normal_f3f3f(q, e);
            let _35_h: vec3<f32> = dims_f3();
            let _36_a: vec3<f32> = abs(_34_n);
            var _37_uv: vec2<f32>;
            var _38_face: vec2<f32>;
            if (_36_a.z >= _36_a.x) && (_36_a.z >= _36_a.y) {
              {
                _37_uv = vec2<f32>(select(-q.x, q.x, _34_n.z < 0.0), q.y) / _35_h.xy;
                _38_face = _35_h.xy;
              }
            } else {
              if _36_a.x >= _36_a.y {
                {
                  _37_uv = vec2<f32>(select(-q.z, q.z, _34_n.x > 0.0), q.y) / _35_h.zy;
                  _38_face = _35_h.zy;
                }
              } else {
                {
                  _37_uv = vec2<f32>(q.x, select(q.z, -q.z, _34_n.y < 0.0)) / _35_h.xz;
                  _38_face = _35_h.xz;
                }
              }
            }
            let _39_c: vec4<f32> = pic_f4f2(cover_f2f2f2(_37_uv * 0.5 + 0.5, _38_face));
            let _40_n: vec3<f32> = normalize(m * _34_n);
            let _41_a: f32 = radians(_uniform0.light);
            let _42_l: vec3<f32> = normalize(vec3<f32>(cos(_41_a), -sin(_41_a), -1.2));
            let _43_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_40_n, _42_l), 0.0));
            let _44_s: f32 = _uniform0.shine * pow(max(dot(_40_n, normalize(_42_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_39_c.xyz * _43_k + _44_s * _39_c.w, vec3<f32>(_39_c.w)), _39_c.w) * (1.0 - c.w);
          }
        }
      }
    }
    var _skTemp2: vec4<f32>;
    if hit {
      _skTemp2 = c;
    } else {
      _skTemp2 = c * (1.0 - best);
    }
    return vec4<f32>(_skTemp2);
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
