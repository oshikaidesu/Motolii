// Hollow Box: AbsentMotion's hollowbox (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --track@width:Width,0.02,5,0.7,0.01
// --track@height:Height,0.02,5,0.7,0.01
// --track@depth:Depth,0.02,5,0.7,0.01
// --track@thickness:Thickness,0.005,1,0.06,0.01
// --track@smoothing:Smoothing,0,1,0.15,0.01
// --track@interior_x:Interior Color R,0,1,0.2392,0.01
// --track@interior_y:Interior Color G,0,1,0.2549,0.01
// --track@interior_z:Interior Color B,0,1,0.3216,0.01
// --track@interior_w:Interior Color A,0,1,1,0.01
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
  thickness: f32,
  smoothing: f32,
  interior: vec4<f32>,
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
fn sdBox_ff3f3(p: vec3<f32>, h: vec3<f32>) -> f32 {
  {
    let q: vec3<f32> = abs(p) - h;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
  }
}
fn dims_f3() -> vec3<f32> {
  {
    let side: f32 = min(_uniform0.resolution.x, _uniform0.resolution.y);
    return vec3<f32>((_uniform0.width * _uniform0.resolution.x) / side, (_uniform0.height * _uniform0.resolution.y) / side, _uniform0.depth);
  }
}
fn hole_ff3(p: vec3<f32>) -> f32 {
  {
    let h: vec3<f32> = dims_f3();
    let w: f32 = min(_uniform0.thickness, min(h.x, h.z) * 0.9);
    let _1_q: vec3<f32> = abs(p) - vec3<f32>(h.x - w, h.y + 1.0, h.z - w);
    return length(max(_1_q, vec3<f32>(0.0))) + min(max(_1_q.x, max(_1_q.y, _1_q.z)), 0.0);
  }
}
fn map_ff3(p: vec3<f32>) -> f32 {
  {
    let h: vec3<f32> = dims_f3();
    let _2_r: f32 = (_uniform0.smoothing * min(_uniform0.thickness, min(h.x, min(h.y, h.z)))) * 0.5;
    let _3_q: vec3<f32> = abs(p) - (h - _2_r);
    let _4_h: vec3<f32> = dims_f3();
    let _5_w: f32 = min(_uniform0.thickness, min(_4_h.x, _4_h.z) * 0.9);
    let _6_q: vec3<f32> = abs(p) - vec3<f32>(_4_h.x - _5_w, _4_h.y + 1.0, _4_h.z - _5_w);
    return max((length(max(_3_q, vec3<f32>(0.0))) + min(max(_3_q.x, max(_3_q.y, _3_q.z)), 0.0)) - _2_r, -(length(max(_6_q, vec3<f32>(0.0))) + min(max(_6_q.x, max(_6_q.y, _6_q.z)), 0.0)));
  }
}
fn surface_f4f3f3(p: vec3<f32>, n: vec3<f32>) -> vec4<f32> {
  {
    let h: vec3<f32> = dims_f3();
    var _skTemp2: bool;
    if abs(n.y) > 0.7 {
      _skTemp2 = true;
    } else {
      _skTemp2 = abs(hole_ff3(p)) < abs(sdBox_ff3f3(p, h));
    }
    if _skTemp2 {
      {
        return vec4<f32>(vec3<f32>(_uniform0.interior.xyz * _uniform0.interior.w), f32(_uniform0.interior.w));
      }
    }
    let _7_a: vec3<f32> = abs(n);
    var _8_uv: vec2<f32>;
    var _9_face: vec2<f32>;
    if (_7_a.z >= _7_a.x) && (_7_a.z >= _7_a.y) {
      {
        _8_uv = vec2<f32>(select(-p.x, p.x, n.z < 0.0), p.y) / h.xy;
        _9_face = h.xy;
      }
    } else {
      if _7_a.x >= _7_a.y {
        {
          _8_uv = vec2<f32>(select(-p.z, p.z, n.x > 0.0), p.y) / h.zy;
          _9_face = h.zy;
        }
      } else {
        {
          _8_uv = vec2<f32>(p.x, select(p.z, -p.z, n.y < 0.0)) / h.xz;
          _9_face = h.xz;
        }
      }
    }
    return pic_f4f2(cover_f2f2f2(_8_uv * 0.5 + 0.5, _9_face));
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
    let _16_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _17_cx: f32 = cos(_16_a.x);
    let _18_sx: f32 = sin(_16_a.x);
    let _19_cy: f32 = cos(_16_a.y);
    let _20_sy: f32 = sin(_16_a.y);
    let _21_cz: f32 = cos(_16_a.z);
    let _22_sz: f32 = sin(_16_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_21_cz, -_22_sz, 0.0, _22_sz, _21_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_19_cy, 0.0, _20_sy, 0.0, 1.0, 0.0, -_20_sy, 0.0, _19_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _17_cx, -_18_sx, 0.0, _18_sx, _17_cx);
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
    let _23_c: vec4<f32> = surface_f4f3f3(p, n);
    let _24_n: vec3<f32> = normalize(m * n);
    let _25_a: f32 = radians(_uniform0.light);
    let _26_l: vec3<f32> = normalize(vec3<f32>(cos(_25_a), -sin(_25_a), -1.2));
    let _27_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_24_n, _26_l), 0.0));
    let _28_s: f32 = _uniform0.shine * pow(max(dot(_24_n, normalize(_26_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_23_c.xyz * _27_k + _28_s * _23_c.w, vec3<f32>(_23_c.w)), _23_c.w);
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
            let _29_n: vec3<f32> = -normal_f3f3f(q, e);
            let _30_c: vec4<f32> = surface_f4f3f3(q, _29_n);
            let _31_n: vec3<f32> = normalize(m * _29_n);
            let _32_a: f32 = radians(_uniform0.light);
            let _33_l: vec3<f32> = normalize(vec3<f32>(cos(_32_a), -sin(_32_a), -1.2));
            let _34_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_31_n, _33_l), 0.0));
            let _35_s: f32 = _uniform0.shine * pow(max(dot(_31_n, normalize(_33_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_30_c.xyz * _34_k + _35_s * _30_c.w, vec3<f32>(_30_c.w)), _30_c.w) * (1.0 - c.w);
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
