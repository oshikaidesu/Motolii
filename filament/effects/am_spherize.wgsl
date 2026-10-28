// Spherize: AbsentMotion's spherize (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Wraps the layer onto a sphere using a front projection or an equirectangular texture.
// --track@radius:Radius,0.02,5,0.9,0.01
// --track@coverage:Coverage,0.2,1,0.5,0.01
// --track@projection:Projection,0,1,0.0,0.01
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
  radius: f32,
  coverage: f32,
  projection: f32,
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
fn surface_f4f3f3(p: vec3<f32>, n: vec3<f32>) -> vec4<f32> {
  {
    let q: vec3<f32> = normalize(p);
    if _uniform0.projection > 0.5 {
      {
        var _skTemp1: f32;
        if length(q.xz) > 1e-05 {
          _skTemp1 = atan2(q.x, -q.z);
        } else {
          _skTemp1 = 0.0;
        }
        let longitude: f32 = _skTemp1;
        return pic_f4f2(vec2<f32>(longitude * 0.159154937 + 0.5, acos(clamp(-q.y, -1.0, 1.0)) * 0.318309873));
      }
    }
    let uv: vec2<f32> = 0.5 + (((q.xy / max(length(q.xy), 1e-05)) * acos(clamp(-q.z, -1.0, 1.0))) / (PI * _uniform0.coverage)) * 0.5;
    if (((uv.x < 0.0) || (uv.x > 1.0)) || (uv.y < 0.0)) || (uv.y > 1.0) {
      return vec4<f32>(0.0);
    }
    return pic_f4f2(uv);
  }
}
fn normal_f3f3f(p: vec3<f32>, e: f32) -> vec3<f32> {
  {
    const k: vec2<f32> = vec2<f32>(1.0, -1.0);
    return normalize(((k.xyy * (length(p + k.xyy * e) - _uniform0.radius) + k.yyx * (length(p + k.yyx * e) - _uniform0.radius)) + k.yxy * (length(p + k.yxy * e) - _uniform0.radius)) + k.xxx * (length(p + k.xxx * e) - _uniform0.radius));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let unit: f32 = max((min(_uniform0.resolution.x, _uniform0.resolution.y) * 0.5) * _uniform0.scale, 0.5);
    let rb: f32 = _uniform0.radius;
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let reach: f32 = rb + (2.0 * dist) / (unit * max(dist - rb, 0.1));
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + reach * reach;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _7_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _8_cx: f32 = cos(_7_a.x);
    let _9_sx: f32 = sin(_7_a.x);
    let _10_cy: f32 = cos(_7_a.y);
    let _11_sy: f32 = sin(_7_a.y);
    let _12_cz: f32 = cos(_7_a.z);
    let _13_sz: f32 = sin(_7_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_12_cz, -_13_sz, 0.0, _13_sz, _12_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_10_cy, 0.0, _11_sy, 0.0, 1.0, 0.0, -_11_sy, 0.0, _10_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _8_cx, -_9_sx, 0.0, _9_sx, _8_cx);
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
      for (; i < 64; i = i + i32(1)) {
        {
          let h: f32 = length(ro + rd * t) - _uniform0.radius;
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
      {
        p = p - n * (length(p) - _uniform0.radius);
      }
    }
    let _14_c: vec4<f32> = surface_f4f3f3(p, n);
    let _15_n: vec3<f32> = normalize(m * n);
    let _16_a: f32 = radians(_uniform0.light);
    let _17_l: vec3<f32> = normalize(vec3<f32>(cos(_16_a), -sin(_16_a), -1.2));
    let _18_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_15_n, _17_l), 0.0));
    let _19_s: f32 = _uniform0.shine * pow(max(dot(_15_n, normalize(_17_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_14_c.xyz * _18_k + _19_s * _14_c.w, vec3<f32>(_14_c.w)), _14_c.w);
    if hit && (c.w < 0.996) {
      {
        var s: f32 = far;
        var back: bool = false;
        {
          var i: i32 = 0;
          for (; i < 64; i = i + i32(1)) {
            {
              let h: f32 = length(ro + rd * s) - _uniform0.radius;
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
            let _20_n: vec3<f32> = -normal_f3f3f(q, e);
            let _21_c: vec4<f32> = surface_f4f3f3(q, _20_n);
            let _22_n: vec3<f32> = normalize(m * _20_n);
            let _23_a: f32 = radians(_uniform0.light);
            let _24_l: vec3<f32> = normalize(vec3<f32>(cos(_23_a), -sin(_23_a), -1.2));
            let _25_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_22_n, _24_l), 0.0));
            let _26_s: f32 = _uniform0.shine * pow(max(dot(_22_n, normalize(_24_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_21_c.xyz * _25_k + _26_s * _21_c.w, vec3<f32>(_21_c.w)), _21_c.w) * (1.0 - c.w);
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
