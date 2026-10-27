// Heart: AbsentMotion's heart (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --track@heartSize:Size,0.02,5,1.3,0.01
// --track@depth:Depth,0.01,5,0.3,0.01
// --track@smoothing:Smoothing,0,1,0.7,0.01
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
  heartSize: f32,
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
fn heart2_ff2(_skParam0: vec2<f32>) -> f32 {
  var p = _skParam0;
  {
    p.x = abs(p.x);
    if (p.y + p.x) > 1.0 {
      return length(p - vec2<f32>(0.25, 0.75)) - 0.353553385;
    }
    let a: vec2<f32> = p - vec2<f32>(0.0, 1.0);
    let b: vec2<f32> = p - 0.5 * max(p.x + p.y, 0.0);
    return sqrt(min(dot(a, a), dot(b, b))) * sign(p.x - p.y);
  }
}
fn map_ff3(p: vec3<f32>) -> f32 {
  {
    let s: f32 = _uniform0.heartSize;
    let d: f32 = heart2_ff2(vec2<f32>(p.x, -p.y) / s + vec2<f32>(0.0, 0.55)) * s;
    let _1_r: f32 = (_uniform0.smoothing * _uniform0.depth) * 0.95;
    let _2_w: vec2<f32> = vec2<f32>(d + _1_r, (abs(p.z) - _uniform0.depth) + _1_r);
    return (min(max(_2_w.x, _2_w.y), 0.0) + length(max(_2_w, vec2<f32>(0.0)))) - _1_r;
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
    let rb: f32 = length(vec2<f32>(_uniform0.heartSize * 0.7, _uniform0.depth));
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let reach: f32 = rb + (2.0 * dist) / (unit * max(dist - rb, 0.1));
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + reach * reach;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _9_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _10_cx: f32 = cos(_9_a.x);
    let _11_sx: f32 = sin(_9_a.x);
    let _12_cy: f32 = cos(_9_a.y);
    let _13_sy: f32 = sin(_9_a.y);
    let _14_cz: f32 = cos(_9_a.z);
    let _15_sz: f32 = sin(_9_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_14_cz, -_15_sz, 0.0, _15_sz, _14_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_12_cy, 0.0, _13_sy, 0.0, 1.0, 0.0, -_13_sy, 0.0, _12_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _10_cx, -_11_sx, 0.0, _11_sx, _10_cx);
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
    let _16_c: vec4<f32> = pic_f4f2(cover_f2f2f2((vec2<f32>(select(p.x, -p.x, n.z > 0.0), p.y) / (_uniform0.heartSize * 0.62)) * 0.5 + 0.5, vec2<f32>(1.0)));
    let _17_n: vec3<f32> = normalize(m * n);
    let _18_a: f32 = radians(_uniform0.light);
    let _19_l: vec3<f32> = normalize(vec3<f32>(cos(_18_a), -sin(_18_a), -1.2));
    let _20_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_17_n, _19_l), 0.0));
    let _21_s: f32 = _uniform0.shine * pow(max(dot(_17_n, normalize(_19_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_16_c.xyz * _20_k + _21_s * _16_c.w, vec3<f32>(_16_c.w)), _16_c.w);
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
            let _22_n: vec3<f32> = -normal_f3f3f(q, e);
            let _23_c: vec4<f32> = pic_f4f2(cover_f2f2f2((vec2<f32>(select(q.x, -q.x, _22_n.z > 0.0), q.y) / (_uniform0.heartSize * 0.62)) * 0.5 + 0.5, vec2<f32>(1.0)));
            let _24_n: vec3<f32> = normalize(m * _22_n);
            let _25_a: f32 = radians(_uniform0.light);
            let _26_l: vec3<f32> = normalize(vec3<f32>(cos(_25_a), -sin(_25_a), -1.2));
            let _27_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_24_n, _26_l), 0.0));
            let _28_s: f32 = _uniform0.shine * pow(max(dot(_24_n, normalize(_26_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_23_c.xyz * _27_k + _28_s * _23_c.w, vec3<f32>(_23_c.w)), _23_c.w) * (1.0 - c.w);
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
