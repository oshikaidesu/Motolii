// Raster Extrude: AbsentMotion's rasterextrude (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Pushes the layer's picture out into a solid block; the sides take the interior color.
// --track@depth:Depth,0.005,5,0.25,0.01
// --track@interior_x:Interior Color R,0,1,0.2392,0.01
// --track@interior_y:Interior Color G,0,1,0.2549,0.01
// --track@interior_z:Interior Color B,0,1,0.3216,0.01
// --track@interior_w:Interior Color A,0,1,1,0.01
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
  depth: f32,
  interior: vec4<f32>,
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
fn alpha_ff2f2(q: vec2<f32>, h: vec2<f32>) -> f32 {
  {
    return vec4<f32>(textureSample(image_Texture, image_Sampler, clamp(((q / h) * 0.5 + 0.5) * _uniform0.resolution, vec2<f32>(0.5), max(_uniform0.resolution - 0.5, vec2<f32>(0.5))) / _uniform0.resolution)).w;
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let side: f32 = min(_uniform0.resolution.x, _uniform0.resolution.y);
    let unit: f32 = max((side * 0.5) * _uniform0.scale, 0.5);
    let h: vec3<f32> = vec3<f32>(_uniform0.resolution / side, _uniform0.depth);
    let rb: f32 = length(h);
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), rb + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.R_position) / unit, dist));
    let ow: vec3<f32> = vec3<f32>(0.0, 0.0, -dist);
    let b: f32 = dot(ow, rw);
    let disc: f32 = (b * b - dot(ow, ow)) + rb * rb;
    if disc <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _0_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _1_cx: f32 = cos(_0_a.x);
    let _2_sx: f32 = sin(_0_a.x);
    let _3_cy: f32 = cos(_0_a.y);
    let _4_sy: f32 = sin(_0_a.y);
    let _5_cz: f32 = cos(_0_a.z);
    let _6_sz: f32 = sin(_0_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_5_cz, -_6_sz, 0.0, _6_sz, _5_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_3_cy, 0.0, _4_sy, 0.0, 1.0, 0.0, -_4_sy, 0.0, _3_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _1_cx, -_2_sx, 0.0, _2_sx, _1_cx);
    let ro: vec3<f32> = ow * m;
    let rd: vec3<f32> = rw * m;
    let inv: vec3<f32> = (1.0 / max(abs(rd), vec3<f32>(1e-06))) * sign(rd + 1e-07);
    let t1: vec3<f32> = (-h - ro) * inv;
    let t2: vec3<f32> = (h - ro) * inv;
    let lo: vec3<f32> = min(t1, t2);
    let hi: vec3<f32> = max(t1, t2);
    let t0: f32 = max(max(lo.x, lo.y), max(lo.z, 0.0));
    let tEnd: f32 = min(hi.x, min(hi.y, hi.z));
    if t0 >= tEnd {
      return vec4<f32>(0.0);
    }
    let dt: f32 = (tEnd - t0) * 0.015625;
    var t: f32 = t0;
    var peak: f32 = 0.0;
    var tp: f32 = t0;
    var hit: bool = false;
    {
      var i: i32 = 0;
      for (; i <= 64; i = i + i32(1)) {
        {
          t = t0 + dt * f32(i);
          let a: f32 = alpha_ff2f2((ro + rd * t).xy, h.xy);
          if a > peak {
            {
              peak = a;
              tp = t;
            }
          }
          if a > 0.5 {
            {
              hit = true;
              break;
            }
          }
        }
      }
    }
    if (!hit) && (peak < 0.02) {
      return vec4<f32>(0.0);
    }
    if hit {
      {
        var lo2: f32 = max(t - dt, t0);
        var hi2: f32 = t;
        {
          var i: i32 = 0;
          for (; i < 6; i = i + i32(1)) {
            {
              let mid: f32 = 0.5 * (lo2 + hi2);
              if alpha_ff2f2((ro + rd * mid).xy, h.xy) > 0.5 {
                hi2 = mid;
              } else {
                lo2 = mid;
              }
            }
          }
        }
        t = hi2;
      }
    } else {
      {
        t = tp;
      }
    }
    let p: vec3<f32> = ro + rd * t;
    let e: f32 = 1.5 / unit;
    var c: vec4<f32>;
    var n: vec3<f32>;
    if abs(p.z) > (h.z - max((dt * abs(rd.z)) * 1.5, 0.0005)) {
      {
        let s: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, clamp(((p.xy / h.xy) * 0.5 + 0.5) * _uniform0.resolution, vec2<f32>(0.5), max(_uniform0.resolution - 0.5, vec2<f32>(0.5))) / _uniform0.resolution));
        c = vec4<f32>(s.xyz / max(s.w, 0.0001), 1.0);
        n = vec3<f32>(0.0, 0.0, sign(p.z));
      }
    } else {
      {
        let g: vec2<f32> = vec2<f32>(alpha_ff2f2(p.xy + vec2<f32>(e, 0.0), h.xy) - alpha_ff2f2(p.xy - vec2<f32>(e, 0.0), h.xy), alpha_ff2f2(p.xy + vec2<f32>(0.0, e), h.xy) - alpha_ff2f2(p.xy - vec2<f32>(0.0, e), h.xy));
        var _skTemp1: vec3<f32>;
        if length(g) > 0.0001 {
          _skTemp1 = vec3<f32>(-normalize(g), 0.0);
        } else {
          _skTemp1 = vec3<f32>(0.0, 0.0, -1.0);
        }
        n = _skTemp1;
        c = vec4<f32>(vec3<f32>(_uniform0.interior.xyz * _uniform0.interior.w), f32(_uniform0.interior.w));
      }
    }
    let _7_n: vec3<f32> = normalize(m * n);
    let _8_a: f32 = radians(_uniform0.light);
    let _9_l: vec3<f32> = normalize(vec3<f32>(cos(_8_a), -sin(_8_a), -1.2));
    let _10_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_7_n, _9_l), 0.0));
    let _11_s: f32 = _uniform0.shine * pow(max(dot(_7_n, normalize(_9_l + -rw)), 0.0), 40.0);
    c = vec4<f32>(min(c.xyz * _10_k + _11_s * c.w, vec3<f32>(c.w)), c.w);
    var _skTemp2: vec4<f32>;
    if hit {
      _skTemp2 = c;
    } else {
      _skTemp2 = c * min(peak * 2.0, 1.0);
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
