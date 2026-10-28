// Torus: AbsentMotion's torus (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Wraps the layer around a ring.
// --track@rotX:Rotation X,-3600,3600,-35.0,0.1
// --track@rotY:Rotation Y,-3600,3600,-20.0,0.1
// --track@rotZ:Rotation Z,-3600,3600,0.0,0.1
// --track@radius:Radius,0.02,5,0.62,0.01
// --track@thickness:Thickness,0.01,3,0.28,0.01
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
  thickness: f32,
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
    return length(vec2<f32>(length(p.xy) - _uniform0.radius, p.z)) - _uniform0.thickness;
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
    let rb: f32 = _uniform0.radius + _uniform0.thickness;
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
    let _14_c: vec4<f32> = pic_f4f2(vec2<f32>(atan2(p.x, -p.y) * 0.159154937 + 0.5, atan2(p.z, length(p.xy) - _uniform0.radius) * 0.159154937 + 0.5));
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
            let _20_n: vec3<f32> = -normal_f3f3f(q, e);
            let _21_c: vec4<f32> = pic_f4f2(vec2<f32>(atan2(q.x, -q.y) * 0.159154937 + 0.5, atan2(q.z, length(q.xy) - _uniform0.radius) * 0.159154937 + 0.5));
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
