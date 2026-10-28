// Three-axis Cross: AbsentMotion's cross (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Puts the layer on three bars that cross at the middle.
// --track@reach:Length,0.02,5,0.9,0.01
// --track@thickness:Thickness,0.01,2,0.22,0.01
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
  reach: f32,
  thickness: f32,
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
fn turn_f33() -> mat3x3<f32> {
  {
    let a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let cx: f32 = cos(a.x);
    let sx: f32 = sin(a.x);
    let cy: f32 = cos(a.y);
    let sy: f32 = sin(a.y);
    let cz: f32 = cos(a.z);
    let sz: f32 = sin(a.z);
    return (mat3x3<f32>(cz, -sz, 0.0, sz, cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(cy, 0.0, sy, 0.0, 1.0, 0.0, -sy, 0.0, cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, cx, -sx, 0.0, sx, cx);
  }
}
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
fn bound_f() -> f32 {
  {
    return length(vec3<f32>(_uniform0.reach, _uniform0.thickness, _uniform0.thickness));
  }
}
fn map_ff3(p: vec3<f32>) -> f32 {
  {
    let r: f32 = (_uniform0.smoothing * _uniform0.thickness) * 0.5;
    let _1_q: vec3<f32> = abs(p) - (vec3<f32>(_uniform0.reach, _uniform0.thickness, _uniform0.thickness) - r);
    let _2_q: vec3<f32> = abs(p) - (vec3<f32>(_uniform0.thickness, _uniform0.reach, _uniform0.thickness) - r);
    let _3_q: vec3<f32> = abs(p) - (vec3<f32>(_uniform0.thickness, _uniform0.thickness, _uniform0.reach) - r);
    return min((length(max(_1_q, vec3<f32>(0.0))) + min(max(_1_q.x, max(_1_q.y, _1_q.z)), 0.0)) - r, min((length(max(_2_q, vec3<f32>(0.0))) + min(max(_2_q.x, max(_2_q.y, _2_q.z)), 0.0)) - r, (length(max(_3_q, vec3<f32>(0.0))) + min(max(_3_q.x, max(_3_q.y, _3_q.z)), 0.0)) - r));
  }
}
fn normal_f3f3f(p: vec3<f32>, e: f32) -> vec3<f32> {
  {
    const k: vec2<f32> = vec2<f32>(1.0, -1.0);
    return normalize(((k.xyy * map_ff3(p + k.xyy * e) + k.yyx * map_ff3(p + k.yyx * e)) + k.yxy * map_ff3(p + k.yxy * e)) + k.xxx * map_ff3(p + k.xxx * e));
  }
}
fn paint_f4f3f3f33f3(p: vec3<f32>, n: vec3<f32>, m: mat3x3<f32>, rw: vec3<f32>) -> vec4<f32> {
  {
    let _7_a: vec3<f32> = abs(n);
    var _8_uv: vec2<f32>;
    var _9_face: vec2<f32>;
    if (_7_a.z >= _7_a.x) && (_7_a.z >= _7_a.y) {
      {
        _8_uv = vec2<f32>(select(-p.x, p.x, n.z < 0.0), p.y) / vec2<f32>(_uniform0.reach);
        _9_face = vec2<f32>(_uniform0.reach);
      }
    } else {
      if _7_a.x >= _7_a.y {
        {
          _8_uv = vec2<f32>(select(-p.z, p.z, n.x > 0.0), p.y) / vec2<f32>(_uniform0.reach);
          _9_face = vec2<f32>(_uniform0.reach);
        }
      } else {
        {
          _8_uv = vec2<f32>(p.x, select(p.z, -p.z, n.y < 0.0)) / vec2<f32>(_uniform0.reach);
          _9_face = vec2<f32>(_uniform0.reach);
        }
      }
    }
    let _10_c: vec4<f32> = pic_f4f2(cover_f2f2f2(_8_uv * 0.5 + 0.5, _9_face));
    let _11_n: vec3<f32> = normalize(m * n);
    let _12_a: f32 = radians(_uniform0.light);
    let _13_l: vec3<f32> = normalize(vec3<f32>(cos(_12_a), -sin(_12_a), -1.2));
    let _14_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_11_n, _13_l), 0.0));
    let _15_s: f32 = _uniform0.shine * pow(max(dot(_11_n, normalize(_13_l + -rw)), 0.0), 40.0);
    return vec4<f32>(min(_10_c.xyz * _14_k + _15_s * _10_c.w, vec3<f32>(_10_c.w)), _10_c.w);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let unit: f32 = max((min(_uniform0.resolution.x, _uniform0.resolution.y) * 0.5) * _uniform0.scale, 0.5);
    let rb: f32 = bound_f();
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let reach: f32 = rb + (2.0 * dist) / (unit * max(dist - rb, 0.1));
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + reach * reach;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let m: mat3x3<f32> = turn_f33();
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
    var c: vec4<f32> = paint_f4f3f3f33f3(p, n, m, rw);
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
            c = c + paint_f4f3f3f33f3(q, -normal_f3f3f(q, e), m, rw) * (1.0 - c.w);
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
