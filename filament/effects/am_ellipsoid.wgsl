// Ellipsoid: AbsentMotion's ellipsoid (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Wraps the layer around a sphere that you can squeeze and flatten.
// --track@rotX:Rotation X,-3600,3600,-15.0,0.1
// --track@rotY:Rotation Y,-3600,3600,0.0,0.1
// --track@rotZ:Rotation Z,-3600,3600,0.0,0.1
// --track@radius:Radius,0.02,5,0.8,0.01
// --track@squeezeRatio:Squeeze,0.1,1.9,1.0,0.01
// --track@flattenRatio:Flatten,0.05,1,1.0,0.01
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
  squeezeRatio: f32,
  flattenRatio: f32,
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
fn map_ff3(p: vec3<f32>) -> f32 {
  {
    let r: vec3<f32> = _uniform0.radius * vec3<f32>(1.0, _uniform0.squeezeRatio, _uniform0.flattenRatio);
    let k0: f32 = length(p / r);
    let k1: f32 = length(p / (r * r));
    return (k0 * (k0 - 1.0)) / max(k1, 0.0001);
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
    let rb: f32 = _uniform0.radius * max(1.0, _uniform0.squeezeRatio);
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let reach: f32 = rb + (2.0 * dist) / (unit * max(dist - rb, 0.1));
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + reach * reach;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _8_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _9_cx: f32 = cos(_8_a.x);
    let _10_sx: f32 = sin(_8_a.x);
    let _11_cy: f32 = cos(_8_a.y);
    let _12_sy: f32 = sin(_8_a.y);
    let _13_cz: f32 = cos(_8_a.z);
    let _14_sz: f32 = sin(_8_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_13_cz, -_14_sz, 0.0, _14_sz, _13_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_11_cy, 0.0, _12_sy, 0.0, 1.0, 0.0, -_12_sy, 0.0, _11_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _9_cx, -_10_sx, 0.0, _10_sx, _9_cx);
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
    let _15_q: vec3<f32> = normalize(p / (_uniform0.radius * vec3<f32>(1.0, _uniform0.squeezeRatio, _uniform0.flattenRatio)));
    let _16_c: vec4<f32> = pic_f4f2(vec2<f32>(atan2(_15_q.x, -_15_q.z) * 0.159154937 + 0.5, acos(clamp(-_15_q.y, -1.0, 1.0)) * 0.318309873));
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
            let _23_q: vec3<f32> = normalize(q / (_uniform0.radius * vec3<f32>(1.0, _uniform0.squeezeRatio, _uniform0.flattenRatio)));
            let _24_c: vec4<f32> = pic_f4f2(vec2<f32>(atan2(_23_q.x, -_23_q.z) * 0.159154937 + 0.5, acos(clamp(-_23_q.y, -1.0, 1.0)) * 0.318309873));
            let _25_n: vec3<f32> = normalize(m * _22_n);
            let _26_a: f32 = radians(_uniform0.light);
            let _27_l: vec3<f32> = normalize(vec3<f32>(cos(_26_a), -sin(_26_a), -1.2));
            let _28_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_25_n, _27_l), 0.0));
            let _29_s: f32 = _uniform0.shine * pow(max(dot(_25_n, normalize(_27_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_24_c.xyz * _28_k + _29_s * _24_c.w, vec3<f32>(_24_c.w)), _24_c.w) * (1.0 - c.w);
          }
        }
      }
    }
    var _skTemp1: vec4<f32>;
    if hit {
      _skTemp1 = c;
    } else {
      _skTemp1 = c * (1.0 - best);
    }
    return vec4<f32>(_skTemp1);
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
