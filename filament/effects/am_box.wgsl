// Box: AbsentMotion's box (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Wraps the layer onto a box whose width, height and depth you choose.
// --track@rotX:Rotation X,-3600,3600,-20.0,0.1
// --track@rotY:Rotation Y,-3600,3600,-35.0,0.1
// --track@rotZ:Rotation Z,-3600,3600,0.0,0.1
// --track@width:Width,0.02,5,0.75,0.01
// --track@height:Height,0.02,5,0.75,0.01
// --track@depth:Depth,0.01,5,0.4,0.01
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
  width: f32,
  height: f32,
  depth: f32,
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
fn dims_f3() -> vec3<f32> {
  {
    let side: f32 = min(layerSize__f2().x, layerSize__f2().y);
    return vec3<f32>((_uniform0.width * layerSize__f2().x) / side, (_uniform0.height * layerSize__f2().y) / side, _uniform0.depth);
  }
}
fn map_ff3(p: vec3<f32>) -> f32 {
  {
    let h: vec3<f32> = dims_f3();
    let _8_r: f32 = (_uniform0.smoothing * min(h.x, min(h.y, h.z))) * 0.5;
    let _9_q: vec3<f32> = abs(p) - (h - _8_r);
    return (length(max(_9_q, vec3<f32>(0.0))) + min(max(_9_q.x, max(_9_q.y, _9_q.z)), 0.0)) - _8_r;
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
    let _28_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _29_cx: f32 = cos(_28_a.x);
    let _30_sx: f32 = sin(_28_a.x);
    let _31_cy: f32 = cos(_28_a.y);
    let _32_sy: f32 = sin(_28_a.y);
    let _33_cz: f32 = cos(_28_a.z);
    let _34_sz: f32 = sin(_28_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_33_cz, -_34_sz, 0.0, _34_sz, _33_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_31_cy, 0.0, _32_sy, 0.0, 1.0, 0.0, -_32_sy, 0.0, _31_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _29_cx, -_30_sx, 0.0, _30_sx, _29_cx);
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
    let _35_h: vec3<f32> = dims_f3();
    let _36_a: vec3<f32> = abs(n);
    var _37_uv: vec2<f32>;
    var _38_face: vec2<f32>;
    if (_36_a.z >= _36_a.x) && (_36_a.z >= _36_a.y) {
      {
        _37_uv = vec2<f32>(select(-p.x, p.x, n.z < 0.0), p.y) / _35_h.xy;
        _38_face = _35_h.xy;
      }
    } else {
      if _36_a.x >= _36_a.y {
        {
          _37_uv = vec2<f32>(select(-p.z, p.z, n.x > 0.0), p.y) / _35_h.zy;
          _38_face = _35_h.zy;
        }
      } else {
        {
          _37_uv = vec2<f32>(p.x, select(p.z, -p.z, n.y < 0.0)) / _35_h.xz;
          _38_face = _35_h.xz;
        }
      }
    }
    let _39_uv: vec2<f32> = cover_f2f2f2(_37_uv * 0.5 + 0.5, _38_face);
    let _40_p: vec2<f32> = clamp(_39_uv * layerSize__f2(), vec2<f32>(0.5), max(layerSize__f2() - 0.5, vec2<f32>(0.5)));
    let _41_c: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, (_40_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution));
    let _42_n: vec3<f32> = normalize(m * n);
    let _43_a: f32 = radians(_uniform0.light);
    let _44_l: vec3<f32> = normalize(vec3<f32>(cos(_43_a), -sin(_43_a), -1.2));
    let _45_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_42_n, _44_l), 0.0));
    let _46_s: f32 = _uniform0.shine * pow(max(dot(_42_n, normalize(_44_l + -rw)), 0.0), 40.0);
    var c: vec4<f32> = vec4<f32>(min(_41_c.xyz * _45_k + _46_s * _41_c.w, vec3<f32>(_41_c.w)), _41_c.w);
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
            let _47_n: vec3<f32> = -normal_f3f3f(q, e);
            let _48_h: vec3<f32> = dims_f3();
            let _49_a: vec3<f32> = abs(_47_n);
            var _50_uv: vec2<f32>;
            var _51_face: vec2<f32>;
            if (_49_a.z >= _49_a.x) && (_49_a.z >= _49_a.y) {
              {
                _50_uv = vec2<f32>(select(-q.x, q.x, _47_n.z < 0.0), q.y) / _48_h.xy;
                _51_face = _48_h.xy;
              }
            } else {
              if _49_a.x >= _49_a.y {
                {
                  _50_uv = vec2<f32>(select(-q.z, q.z, _47_n.x > 0.0), q.y) / _48_h.zy;
                  _51_face = _48_h.zy;
                }
              } else {
                {
                  _50_uv = vec2<f32>(q.x, select(q.z, -q.z, _47_n.y < 0.0)) / _48_h.xz;
                  _51_face = _48_h.xz;
                }
              }
            }
            let _52_uv: vec2<f32> = cover_f2f2f2(_50_uv * 0.5 + 0.5, _51_face);
            let _53_p: vec2<f32> = clamp(_52_uv * layerSize__f2(), vec2<f32>(0.5), max(layerSize__f2() - 0.5, vec2<f32>(0.5)));
            let _54_c: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, (_53_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution));
            let _55_n: vec3<f32> = normalize(m * _47_n);
            let _56_a: f32 = radians(_uniform0.light);
            let _57_l: vec3<f32> = normalize(vec3<f32>(cos(_56_a), -sin(_56_a), -1.2));
            let _58_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_55_n, _57_l), 0.0));
            let _59_s: f32 = _uniform0.shine * pow(max(dot(_55_n, normalize(_57_l + -rw)), 0.0), 40.0);
            c = c + vec4<f32>(min(_54_c.xyz * _58_k + _59_s * _54_c.w, vec3<f32>(_54_c.w)), _54_c.w) * (1.0 - c.w);
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
