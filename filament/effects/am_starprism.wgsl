// Star Prism: AbsentMotion's starprism (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Cuts the layer into a star and gives it depth.
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
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
const PI: f32 = 3.14159274;
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
    let r: f32 = (_uniform0.smoothing * min(_uniform0.radius * _uniform0.inner, _uniform0.depth)) * 0.5;
    let _1_an: f32 = PI / floor(_uniform0.points + 0.5);
    let _skTemp2 = atan2(p.x, -p.y);
    let _skTemp3 = 2.0 * _1_an;
    var _2_b: f32 = (_skTemp2 - _skTemp3 * floor(_skTemp2 / _skTemp3));
    var _skTemp4: f32;
    if _2_b > _1_an {
      _skTemp4 = 2.0 * _1_an - _2_b;
    } else {
      _skTemp4 = _2_b;
    }
    _2_b = _skTemp4;
    let _3_q: vec2<f32> = length(p.xy) * vec2<f32>(cos(_2_b), sin(_2_b));
    let _4_a: vec2<f32> = vec2<f32>(_uniform0.radius, 0.0);
    let _5_e: vec2<f32> = (_uniform0.radius * _uniform0.inner) * vec2<f32>(cos(_1_an), sin(_1_an)) - _4_a;
    let _6_w: vec2<f32> = _3_q - _4_a;
    let _7_d: f32 = length(_6_w - _5_e * clamp(dot(_6_w, _5_e) / dot(_5_e, _5_e), 0.0, 1.0));
    let _8_w: vec2<f32> = vec2<f32>((select(_7_d, -_7_d, (_5_e.x * _6_w.y - _5_e.y * _6_w.x) > 0.0)) + r, (abs(p.z) - _uniform0.depth) + r);
    return (min(max(_8_w.x, _8_w.y), 0.0) + length(max(_8_w, vec2<f32>(0.0)))) - r;
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
    let rb: f32 = length(vec2<f32>(_uniform0.radius, _uniform0.depth));
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let reach: f32 = rb + (2.0 * dist) / (unit * max(dist - rb, 0.1));
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + reach * reach;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _15_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _16_cx: f32 = cos(_15_a.x);
    let _17_sx: f32 = sin(_15_a.x);
    let _18_cy: f32 = cos(_15_a.y);
    let _19_sy: f32 = sin(_15_a.y);
    let _20_cz: f32 = cos(_15_a.z);
    let _21_sz: f32 = sin(_15_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_20_cz, -_21_sz, 0.0, _21_sz, _20_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_18_cy, 0.0, _19_sy, 0.0, 1.0, 0.0, -_19_sy, 0.0, _18_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _16_cx, -_17_sx, 0.0, _17_sx, _16_cx);
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
    let _22_c: vec4<f32> = pic_f4f2(cover_f2f2f2((vec2<f32>(select(p.x, -p.x, n.z > 0.0), p.y) / _uniform0.radius) * 0.5 + 0.5, vec2<f32>(1.0)));
    let _23_n: vec3<f32> = normalize(m * n);
    let _24_a: f32 = radians(_uniform0.light);
    let _25_l: vec3<f32> = normalize(vec3<f32>(cos(_24_a), -sin(_24_a), -1.2));
    let _26_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_23_n, _25_l), 0.0));
    let _27_s: f32 = _uniform0.shine * pow(max(dot(_23_n, normalize(_25_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_22_c.xyz * _26_k + _27_s * _22_c.w, vec3<f32>(_22_c.w)), _22_c.w);
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
            let _28_n: vec3<f32> = -normal_f3f3f(q, e);
            let _29_c: vec4<f32> = pic_f4f2(cover_f2f2f2((vec2<f32>(select(q.x, -q.x, _28_n.z > 0.0), q.y) / _uniform0.radius) * 0.5 + 0.5, vec2<f32>(1.0)));
            let _30_n: vec3<f32> = normalize(m * _28_n);
            let _31_a: f32 = radians(_uniform0.light);
            let _32_l: vec3<f32> = normalize(vec3<f32>(cos(_31_a), -sin(_31_a), -1.2));
            let _33_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_30_n, _32_l), 0.0));
            let _34_s: f32 = _uniform0.shine * pow(max(dot(_30_n, normalize(_32_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_29_c.xyz * _33_k + _34_s * _29_c.w, vec3<f32>(_29_c.w)), _29_c.w) * (1.0 - c.w);
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
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
