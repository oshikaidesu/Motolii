// Cylinder: AbsentMotion's cylinder (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Wraps the layer around a cylinder and onto its round ends.
// --track@rotX:Rotation X,-3600,3600,-20.0,0.1
// --track@rotY:Rotation Y,-3600,3600,-35.0,0.1
// --track@rotZ:Rotation Z,-3600,3600,0.0,0.1
// --track@radius:Radius,0.02,5,0.6,0.01
// --track@height:Height,0.02,5,0.85,0.01
// --track@smoothing:Smoothing,0,1,0.15,0.01
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
  height: f32,
  smoothing: f32,
  R_position: vec2<f32>,
  scale: f32,
  R_perspective: f32,
  shading: f32,
  light: f32,
  shine: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn layerSize__f2() -> vec2<f32> {
  {
    return select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0));
  }
}
fn pic_f4f2(uv: vec2<f32>) -> vec4<f32> {
  {
    let _0_p: vec2<f32> = clamp(uv * layerSize__f2(), vec2<f32>(0.5), max(layerSize__f2() - 0.5, vec2<f32>(0.5)));
    return vec4<f32>(textureSample(image_Texture, image_Sampler, (_0_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution));
  }
}
fn cover_f2f2f2(uv: vec2<f32>, face: vec2<f32>) -> vec2<f32> {
  {
    let r: f32 = (face.x / max(face.y, 0.0001)) / (layerSize__f2().x / max(layerSize__f2().y, 0.0001));
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
    let r: f32 = (_uniform0.smoothing * min(_uniform0.radius, _uniform0.height)) * 0.5;
    let d: vec2<f32> = vec2<f32>((length(p.xz) - _uniform0.radius) + r, (abs(p.y) - _uniform0.height) + r);
    return (min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0)))) - r;
  }
}
fn surface_f4f3f3(p: vec3<f32>, n: vec3<f32>) -> vec4<f32> {
  {
    if abs(n.y) > 0.7 {
      return pic_f4f2(cover_f2f2f2((vec2<f32>(p.x, select(p.z, -p.z, n.y < 0.0)) / _uniform0.radius) * 0.5 + 0.5, vec2<f32>(1.0)));
    }
    return pic_f4f2(vec2<f32>(atan2(p.x, -p.z) * 0.159154937 + 0.5, (p.y / _uniform0.height) * 0.5 + 0.5));
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
    let unit: f32 = max((min(layerSize__f2().x, layerSize__f2().y) * 0.5) * _uniform0.scale, 0.5);
    let rb: f32 = length(vec2<f32>(_uniform0.radius, _uniform0.height));
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - layerSize__f2() * 0.5) - _uniform0.R_position) / unit, dist));
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
    let _15_c: vec4<f32> = surface_f4f3f3(p, n);
    let _16_n: vec3<f32> = normalize(m * n);
    let _17_a: f32 = radians(_uniform0.light);
    let _18_l: vec3<f32> = normalize(vec3<f32>(cos(_17_a), -sin(_17_a), -1.2));
    let _19_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_16_n, _18_l), 0.0));
    let _20_s: f32 = _uniform0.shine * pow(max(dot(_16_n, normalize(_18_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_15_c.xyz * _19_k + _20_s * _15_c.w, vec3<f32>(_15_c.w)), _15_c.w);
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
            let _21_n: vec3<f32> = -normal_f3f3f(q, e);
            let _22_c: vec4<f32> = surface_f4f3f3(q, _21_n);
            let _23_n: vec3<f32> = normalize(m * _21_n);
            let _24_a: f32 = radians(_uniform0.light);
            let _25_l: vec3<f32> = normalize(vec3<f32>(cos(_24_a), -sin(_24_a), -1.2));
            let _26_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_23_n, _25_l), 0.0));
            let _27_s: f32 = _uniform0.shine * pow(max(dot(_23_n, normalize(_25_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_22_c.xyz * _26_k + _27_s * _22_c.w, vec3<f32>(_22_c.w)), _22_c.w) * (1.0 - c.w);
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
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0))));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
