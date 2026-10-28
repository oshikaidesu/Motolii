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
    let unit: f32 = max((min(layerSize__f2().x, layerSize__f2().y) * 0.5) * _uniform0.scale, 0.5);
    let rb: f32 = _uniform0.radius * max(1.0, _uniform0.squeezeRatio);
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - layerSize__f2() * 0.5) - _uniform0.R_position) / unit, dist));
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
    let _16_q: vec3<f32> = normalize(p / (_uniform0.radius * vec3<f32>(1.0, _uniform0.squeezeRatio, _uniform0.flattenRatio)));
    let _17_c: vec4<f32> = pic_f4f2(vec2<f32>(atan2(_16_q.x, -_16_q.z) * 0.159154937 + 0.5, acos(clamp(-_16_q.y, -1.0, 1.0)) * 0.318309873));
    let _18_n: vec3<f32> = normalize(m * n);
    let _19_a: f32 = radians(_uniform0.light);
    let _20_l: vec3<f32> = normalize(vec3<f32>(cos(_19_a), -sin(_19_a), -1.2));
    let _21_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_18_n, _20_l), 0.0));
    let _22_s: f32 = _uniform0.shine * pow(max(dot(_18_n, normalize(_20_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_17_c.xyz * _21_k + _22_s * _17_c.w, vec3<f32>(_17_c.w)), _17_c.w);
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
            let _23_n: vec3<f32> = -normal_f3f3f(q, e);
            let _24_q: vec3<f32> = normalize(q / (_uniform0.radius * vec3<f32>(1.0, _uniform0.squeezeRatio, _uniform0.flattenRatio)));
            let _25_c: vec4<f32> = pic_f4f2(vec2<f32>(atan2(_24_q.x, -_24_q.z) * 0.159154937 + 0.5, acos(clamp(-_24_q.y, -1.0, 1.0)) * 0.318309873));
            let _26_n: vec3<f32> = normalize(m * _23_n);
            let _27_a: f32 = radians(_uniform0.light);
            let _28_l: vec3<f32> = normalize(vec3<f32>(cos(_27_a), -sin(_27_a), -1.2));
            let _29_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_26_n, _28_l), 0.0));
            let _30_s: f32 = _uniform0.shine * pow(max(dot(_26_n, normalize(_28_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_25_c.xyz * _29_k + _30_s * _25_c.w, vec3<f32>(_25_c.w)), _25_c.w) * (1.0 - c.w);
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
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0))));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
