// Hollow Box: AbsentMotion's hollowbox (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:A box with open top and bottom: the layer covers the outside and the inside takes the interior color.
// --track@rotX:Rotation X,-3600,3600,-28.0,0.1
// --track@rotY:Rotation Y,-3600,3600,-35.0,0.1
// --track@rotZ:Rotation Z,-3600,3600,0.0,0.1
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
fn sdBox_ff3f3(p: vec3<f32>, h: vec3<f32>) -> f32 {
  {
    let q: vec3<f32> = abs(p) - h;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
  }
}
fn dims_f3() -> vec3<f32> {
  {
    let side: f32 = min(layerSize__f2().x, layerSize__f2().y);
    return vec3<f32>((_uniform0.width * layerSize__f2().x) / side, (_uniform0.height * layerSize__f2().y) / side, _uniform0.depth);
  }
}
fn hole_ff3(p: vec3<f32>) -> f32 {
  {
    let h: vec3<f32> = dims_f3();
    let w: f32 = min(_uniform0.thickness, min(h.x, h.z) * 0.9);
    let _8_q: vec3<f32> = abs(p) - vec3<f32>(h.x - w, h.y + 1.0, h.z - w);
    return length(max(_8_q, vec3<f32>(0.0))) + min(max(_8_q.x, max(_8_q.y, _8_q.z)), 0.0);
  }
}
fn map_ff3(p: vec3<f32>) -> f32 {
  {
    let h: vec3<f32> = dims_f3();
    let _9_r: f32 = (_uniform0.smoothing * min(_uniform0.thickness, min(h.x, min(h.y, h.z)))) * 0.5;
    let _10_q: vec3<f32> = abs(p) - (h - _9_r);
    let _11_h: vec3<f32> = dims_f3();
    let _12_w: f32 = min(_uniform0.thickness, min(_11_h.x, _11_h.z) * 0.9);
    let _13_q: vec3<f32> = abs(p) - vec3<f32>(_11_h.x - _12_w, _11_h.y + 1.0, _11_h.z - _12_w);
    return max((length(max(_10_q, vec3<f32>(0.0))) + min(max(_10_q.x, max(_10_q.y, _10_q.z)), 0.0)) - _9_r, -(length(max(_13_q, vec3<f32>(0.0))) + min(max(_13_q.x, max(_13_q.y, _13_q.z)), 0.0)));
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
    let _14_a: vec3<f32> = abs(n);
    var _15_uv: vec2<f32>;
    var _16_face: vec2<f32>;
    if (_14_a.z >= _14_a.x) && (_14_a.z >= _14_a.y) {
      {
        _15_uv = vec2<f32>(select(-p.x, p.x, n.z < 0.0), p.y) / h.xy;
        _16_face = h.xy;
      }
    } else {
      if _14_a.x >= _14_a.y {
        {
          _15_uv = vec2<f32>(select(-p.z, p.z, n.x > 0.0), p.y) / h.zy;
          _16_face = h.zy;
        }
      } else {
        {
          _15_uv = vec2<f32>(p.x, select(p.z, -p.z, n.y < 0.0)) / h.xz;
          _16_face = h.xz;
        }
      }
    }
    let _17_uv: vec2<f32> = cover_f2f2f2(_15_uv * 0.5 + 0.5, _16_face);
    let _18_p: vec2<f32> = clamp(_17_uv * layerSize__f2(), vec2<f32>(0.5), max(layerSize__f2() - 0.5, vec2<f32>(0.5)));
    return vec4<f32>(textureSample(image_Texture, image_Sampler, (_18_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution));
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
    let rb: f32 = length(dims_f3());
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - layerSize__f2() * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let reach: f32 = rb + (2.0 * dist) / (unit * max(dist - rb, 0.1));
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + reach * reach;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _25_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _26_cx: f32 = cos(_25_a.x);
    let _27_sx: f32 = sin(_25_a.x);
    let _28_cy: f32 = cos(_25_a.y);
    let _29_sy: f32 = sin(_25_a.y);
    let _30_cz: f32 = cos(_25_a.z);
    let _31_sz: f32 = sin(_25_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_30_cz, -_31_sz, 0.0, _31_sz, _30_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_28_cy, 0.0, _29_sy, 0.0, 1.0, 0.0, -_29_sy, 0.0, _28_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _26_cx, -_27_sx, 0.0, _27_sx, _26_cx);
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
    let _32_c: vec4<f32> = surface_f4f3f3(p, n);
    let _33_n: vec3<f32> = normalize(m * n);
    let _34_a: f32 = radians(_uniform0.light);
    let _35_l: vec3<f32> = normalize(vec3<f32>(cos(_34_a), -sin(_34_a), -1.2));
    let _36_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_33_n, _35_l), 0.0));
    let _37_s: f32 = _uniform0.shine * pow(max(dot(_33_n, normalize(_35_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_32_c.xyz * _36_k + _37_s * _32_c.w, vec3<f32>(_32_c.w)), _32_c.w);
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
            let _38_n: vec3<f32> = -normal_f3f3f(q, e);
            let _39_c: vec4<f32> = surface_f4f3f3(q, _38_n);
            let _40_n: vec3<f32> = normalize(m * _38_n);
            let _41_a: f32 = radians(_uniform0.light);
            let _42_l: vec3<f32> = normalize(vec3<f32>(cos(_41_a), -sin(_41_a), -1.2));
            let _43_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_40_n, _42_l), 0.0));
            let _44_s: f32 = _uniform0.shine * pow(max(dot(_40_n, normalize(_42_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_39_c.xyz * _43_k + _44_s * _39_c.w, vec3<f32>(_39_c.w)), _39_c.w) * (1.0 - c.w);
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
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0))));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
