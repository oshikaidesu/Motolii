// Star Polyhedron: AbsentMotion's starpolyhedron (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Puts the layer on a star made of two crossed pyramids.
// --track@rotX:Rotation X,-3600,3600,-25.0,0.1
// --track@rotY:Rotation Y,-3600,3600,-35.0,0.1
// --track@rotZ:Rotation Z,-3600,3600,0.0,0.1
// --track@radius:Radius,0.02,5,0.95,0.01
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
  radius: f32,
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
    let k: f32 = _uniform0.radius * 0.577350259;
    let q: vec3<f32> = p / k;
    return min((max(abs(q.x + q.y) - q.z, abs(q.x - q.y) + q.z) - 1.0) * 0.577350259, (max(abs((-q).x + (-q).y) - (-q).z, abs((-q).x - (-q).y) + (-q).z) - 1.0) * 0.577350259) * k;
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
    let _13_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _14_cx: f32 = cos(_13_a.x);
    let _15_sx: f32 = sin(_13_a.x);
    let _16_cy: f32 = cos(_13_a.y);
    let _17_sy: f32 = sin(_13_a.y);
    let _18_cz: f32 = cos(_13_a.z);
    let _19_sz: f32 = sin(_13_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_18_cz, -_19_sz, 0.0, _19_sz, _18_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_16_cy, 0.0, _17_sy, 0.0, 1.0, 0.0, -_17_sy, 0.0, _16_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _14_cx, -_15_sx, 0.0, _15_sx, _14_cx);
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
    let _20_f: vec3<f32> = normalize(sign(n));
    let _21_w: vec3<f32> = vec3<f32>(-_20_f.z, 0.0, _20_f.x);
    var _skTemp2: vec3<f32>;
    if dot(_21_w, _21_w) > 0.0 {
      _skTemp2 = normalize(_21_w);
    } else {
      _skTemp2 = vec3<f32>(1.0, 0.0, 0.0);
    }
    let _22_u: vec3<f32> = _skTemp2;
    let _23_c: vec4<f32> = pic_f4f2(cover_f2f2f2((vec2<f32>(dot(p, _22_u), dot(p, cross(_22_u, _20_f))) / (_uniform0.radius * 0.8)) * 0.5 + 0.5, vec2<f32>(1.0)));
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
            let _30_f: vec3<f32> = normalize(sign(_29_n));
            let _31_w: vec3<f32> = vec3<f32>(-_30_f.z, 0.0, _30_f.x);
            var _skTemp3: vec3<f32>;
            if dot(_31_w, _31_w) > 0.0 {
              _skTemp3 = normalize(_31_w);
            } else {
              _skTemp3 = vec3<f32>(1.0, 0.0, 0.0);
            }
            let _32_u: vec3<f32> = _skTemp3;
            let _33_c: vec4<f32> = pic_f4f2(cover_f2f2f2((vec2<f32>(dot(q, _32_u), dot(q, cross(_32_u, _30_f))) / (_uniform0.radius * 0.8)) * 0.5 + 0.5, vec2<f32>(1.0)));
            let _34_n: vec3<f32> = normalize(m * _29_n);
            let _35_a: f32 = radians(_uniform0.light);
            let _36_l: vec3<f32> = normalize(vec3<f32>(cos(_35_a), -sin(_35_a), -1.2));
            let _37_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_34_n, _36_l), 0.0));
            let _38_s: f32 = _uniform0.shine * pow(max(dot(_34_n, normalize(_36_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_33_c.xyz * _37_k + _38_s * _33_c.w, vec3<f32>(_33_c.w)), _33_c.w) * (1.0 - c.w);
          }
        }
      }
    }
    var _skTemp4: vec4<f32>;
    if hit {
      _skTemp4 = c;
    } else {
      _skTemp4 = c * (1.0 - best);
    }
    return vec4<f32>(_skTemp4);
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
