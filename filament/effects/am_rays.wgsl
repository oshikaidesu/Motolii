// Rays: AbsentMotion's rays (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Shines rays of light out from the bright parts of the layer, away from a center point.
// --track@R_center_x:Center x,-1,2,0.5,0.01
// --track@R_center_y:Center y,-1,2,0.5,0.01
// --track@reach:Length,0,4,0.15,0.001
// --track@threshold:Threshold,0,1,0.6,0.001
// --track@thresholdAdd_x:Threshold Add R,0,1,0,0.01
// --track@thresholdAdd_y:Threshold Add G,0,1,0,0.01
// --track@thresholdAdd_z:Threshold Add B,0,1,0,0.01
// --track@thresholdAdd_w:Threshold Add A,0,1,1,0.01
// --track@intensity:Intensity,0,5,1.0,0.01
// --track@color_x:Color R,0,1,0.1765,0.01
// --track@color_y:Color G,0,1,0.1176,0.01
// --track@color_z:Color B,0,1,0.9647,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@blend:Blend,0,1,0.0,0.01
// --track@style:Style,0,1,1.0,0.01
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
  R_center: vec2<f32>,
  reach: f32,
  threshold: f32,
  intensity: f32,
  blend: f32,
  style: f32,
  thresholdAdd: vec4<f32>,
  color: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn luma_ff3(c: vec3<f32>) -> f32 {
  {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
  }
}
fn effect__h4f2(p: vec2<f32>) -> vec4<f32> {
  {
    if _uniform0.style < 0.5 {
      {
        let _0_src: vec4<f32> = textureSample(image_Texture, image_Sampler, p / _uniform0.resolution);
        let _1_c: vec2<f32> = _uniform0.R_center * _uniform0.resolution;
        let _2_length_: f32 = max(_uniform0.reach, 0.0);
        var _3_sum: f32 = 0.0;
        {
          var _4_i: i32 = 0;
          for (; _4_i < 48; _4_i = _4_i + i32(1)) {
            {
              let _5_t: f32 = f32(_4_i) * 0.0212765951;
              let _6_s: vec4<f32> = textureSample(image_Texture, image_Sampler, (_1_c + (p - _1_c) * (1.0 - (_2_length_ * _5_t) / (1.0 + _2_length_))) / _uniform0.resolution);
              var _skTemp1: f32;
              if _6_s.w > 0.0 {
                _skTemp1 = luma_ff3(vec3<f32>(_6_s.xyz) / f32(_6_s.w)) * f32(_6_s.w);
              } else {
                _skTemp1 = 0.0;
              }
              let _7_l: f32 = _skTemp1;
              _3_sum = _3_sum + (max(_7_l - _uniform0.threshold, 0.0) / max(1.0 - _uniform0.threshold, 0.001)) * (1.0 - _5_t);
            }
          }
        }
        let _8_g: f32 = clamp(((_3_sum * 0.0416666679) * _uniform0.intensity) * f32(_uniform0.color.w), 0.0, 1.0);
        let _9_a: f32 = f32(_0_src.w);
        var _skTemp2: vec3<f32>;
        if _9_a > 0.0 {
          _skTemp2 = vec3<f32>(_0_src.xyz) / _9_a;
        } else {
          _skTemp2 = vec3<f32>(0.0);
        }
        let _10_base: vec3<f32> = _skTemp2;
        let _11_light: vec3<f32> = vec3<f32>(_uniform0.color.xyz) * _8_g;
        let _12_outA: f32 = max(_9_a, _8_g);
        var _skTemp3: vec3<f32>;
        if _12_outA > 0.0 {
          _skTemp3 = ((_10_base * _9_a + _11_light) - (_10_base * _9_a) * _11_light) / _12_outA;
        } else {
          _skTemp3 = vec3<f32>(0.0);
        }
        let _13_rgb: vec3<f32> = _skTemp3;
        return vec4<f32>(vec3<f32>(clamp(_13_rgb, vec3<f32>(0.0), vec3<f32>(1.0))) * f32(_12_outA), f32(_12_outA));
      }
    }
    let src: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, p / _uniform0.resolution));
    let len: f32 = clamp(_uniform0.reach, 0.0, 1000.0);
    let gain: f32 = (clamp(_uniform0.intensity, 0.0, 100.0) * clamp(f32(_uniform0.color.w), 0.0, 1.0)) * clamp(len * 100.0, 0.0, 1.0);
    if gain <= 0.0 {
      return vec4<f32>(src);
    }
    let c: vec2<f32> = clamp(_uniform0.R_center, vec2<f32>(-8.0), vec2<f32>(9.0)) * _uniform0.resolution;
    let d: vec2<f32> = p - c;
    let f: f32 = len / (len + 0.1);
    let n: f32 = clamp(ceil((f * length(d)) * 0.333333343), 1.0, 48.0);
    let j: f32 = fract(dot(p, vec2<f32>(0.7548777, 0.5698403)));
    let thr: f32 = clamp(_uniform0.threshold + dot(vec3<f32>(_uniform0.thresholdAdd.xyz), vec3<f32>(0.2126, 0.7152, 0.0722)), 0.04, 2.0);
    let knee: f32 = thr - 0.04;
    let stride: vec2<f32> = d * (f / n);
    var q: vec2<f32> = p - stride * j;
    var w: f32 = n - j;
    var acc: vec4<f32> = vec4<f32>(0.0);
    {
      var i: i32 = 0;
      for (; i < 48; i = i + i32(1)) {
        {
          if f32(i) >= n {
            break;
          }
          let s: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, q / _uniform0.resolution));
          var _skTemp4: f32;
          if s.w > 0.0 {
            _skTemp4 = smoothstep(knee, thr, luma_ff3(s.xyz) / max(s.w, 1e-06));
          } else {
            _skTemp4 = 0.0;
          }
          let g: f32 = _skTemp4;
          acc = acc + (w * g) * s;
          q = q - stride;
          w = w - 1.0;
        }
      }
    }
    acc = vec4<f32>((mix(acc.xyz, vec3<f32>(_uniform0.color.xyz) * dot(acc.xyz, vec3<f32>(0.2126, 0.7152, 0.0722)), vec3<f32>(clamp(_uniform0.blend, 0.0, 1.0)))), acc.w);
    var o: vec4<f32> = src + acc * ((gain * 0.5) / max((n * (n + 1.0)) * 0.5 - n * j, 0.0001));
    o.w = clamp(o.w, 0.0, 1.0);
    o = vec4<f32>((clamp(o.xyz, vec3<f32>(0.0), vec3<f32>(o.w))), o.w);
    return vec4<f32>(o);
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
