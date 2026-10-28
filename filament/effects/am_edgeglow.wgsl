// Edge Glow: AbsentMotion's edgeglow (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --pass@h:0,1,s1,s1
// --pass@blurPicture:0,1,s1,s1
// --pass@:0,0,s1,s1
// --about:Finds the edges in the layer and makes them glow like neon over a darkened picture.
// --track@threshold:Threshold,0,1,0.1,0.001
// --track@blur:Blur,0,300,12.0,0.1
// --track@spread:Spread,0,3,1.5,0.01
// --track@color_x:Color R,0,1,1,0.01
// --track@color_y:Color G,0,1,0.2353,0.01
// --track@color_z:Color B,0,1,0.9412,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@blend:Blend,0,2,0.0,0.001
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
  threshold: f32,
  blur: f32,
  spread: f32,
  color: vec4<f32>,
  blend: f32,
  invert: f32,
  layerRect: vec4<f32>,
  passindex: f32,
};
@group(0) @binding(6) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
@group(0) @binding(3) var h_Sampler: sampler;
@group(0) @binding(2) var h_Texture: texture_2d<f32>;
@group(0) @binding(5) var blurPicture_Sampler: sampler;
@group(0) @binding(4) var blurPicture_Texture: texture_2d<f32>;
fn layerAt__f2() -> vec2<f32> {
  {
    return select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0));
  }
}
fn imageAt_h4f2(p: vec2<f32>) -> vec4<f32> {
  {
    return textureSample(image_Texture, image_Sampler, (p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
fn blurredAt_h4f2(p: vec2<f32>) -> vec4<f32> {
  {
    return textureSample(blurPicture_Texture, blurPicture_Sampler, (p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
fn gauss__h4f2f2i(p: vec2<f32>, by: vec2<f32>, R_from: i32) -> vec4<f32> {
  {
    let sigma: f32 = max(_uniform0.blur, 0.0) * 0.333333343;
    if sigma < 0.5 {
      var _skTemp1: vec4<f32>;
      if R_from == 0 {
        _skTemp1 = textureSample(image_Texture, image_Sampler, p / _uniform0.resolution);
      } else {
        _skTemp1 = textureSample(h_Texture, h_Sampler, p / _uniform0.resolution);
      }
      return _skTemp1;
    }
    var _0_all: vec4<f32> = vec4<f32>(0.0);
    var weight: f32 = 0.0;
    {
      var i: i32 = -96;
      for (; i <= 96; i = i + i32(1)) {
        {
          let x: f32 = f32(i);
          if abs(x) > (3.0 * sigma) {
            continue;
          }
          let w: f32 = exp((-x * x) / ((2.0 * sigma) * sigma));
          var _skTemp2: vec4<f32>;
          if R_from == 0 {
            _skTemp2 = textureSample(image_Texture, image_Sampler, (p + by * x) / _uniform0.resolution);
          } else {
            _skTemp2 = textureSample(h_Texture, h_Sampler, (p + by * x) / _uniform0.resolution);
          }
          let c: vec4<f32> = _skTemp2;
          _0_all = _0_all + vec4<f32>(vec4<f32>(c) * w);
          weight = weight + w;
        }
      }
    }
    return vec4<f32>(vec4<f32>(_0_all) / weight);
  }
}
fn luma_ff3(c: vec3<f32>) -> f32 {
  {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = imageAt_h4f2(coord);
    let _0_l: f32 = luma_ff3(vec3<f32>(imageAt_h4f2(coord + vec2<f32>(-1.0, 0.0)).xyz));
    let _1_r: f32 = luma_ff3(vec3<f32>(imageAt_h4f2(coord + vec2<f32>(1.0, 0.0)).xyz));
    let _2_u: f32 = luma_ff3(vec3<f32>(imageAt_h4f2(coord + vec2<f32>(0.0, -1.0)).xyz));
    let _3_d: f32 = luma_ff3(vec3<f32>(imageAt_h4f2(coord + vec2<f32>(0.0, 1.0)).xyz));
    let _4_al: f32 = f32(imageAt_h4f2(coord + vec2<f32>(-1.0, 0.0)).w);
    let _5_ar: f32 = f32(imageAt_h4f2(coord + vec2<f32>(1.0, 0.0)).w);
    let _6_au: f32 = f32(imageAt_h4f2(coord + vec2<f32>(0.0, -1.0)).w);
    let _7_ad: f32 = f32(imageAt_h4f2(coord + vec2<f32>(0.0, 1.0)).w);
    let sharp: f32 = length(vec2<f32>(_1_r - _0_l, _3_d - _2_u)) + length(vec2<f32>(_5_ar - _4_al, _7_ad - _6_au));
    let _8_step_: f32 = max(_uniform0.blur * 0.25, 1.0);
    let _9_l: f32 = luma_ff3(vec3<f32>(blurredAt_h4f2(coord + vec2<f32>(-_8_step_, 0.0)).xyz));
    let _10_r: f32 = luma_ff3(vec3<f32>(blurredAt_h4f2(coord + vec2<f32>(_8_step_, 0.0)).xyz));
    let _11_u: f32 = luma_ff3(vec3<f32>(blurredAt_h4f2(coord + vec2<f32>(0.0, -_8_step_)).xyz));
    let _12_d: f32 = luma_ff3(vec3<f32>(blurredAt_h4f2(coord + vec2<f32>(0.0, _8_step_)).xyz));
    let _13_al: f32 = f32(blurredAt_h4f2(coord + vec2<f32>(-_8_step_, 0.0)).w);
    let _14_ar: f32 = f32(blurredAt_h4f2(coord + vec2<f32>(_8_step_, 0.0)).w);
    let _15_au: f32 = f32(blurredAt_h4f2(coord + vec2<f32>(0.0, -_8_step_)).w);
    let _16_ad: f32 = f32(blurredAt_h4f2(coord + vec2<f32>(0.0, _8_step_)).w);
    let soft: f32 = ((length(vec2<f32>(_10_r - _9_l, _12_d - _11_u)) + length(vec2<f32>(_14_ar - _13_al, _16_ad - _15_au))) * max(_uniform0.blur * 0.25, 1.0)) * 0.5;
    var e: f32 = max(sharp - _uniform0.threshold, 0.0) * 2.0 + max(soft - _uniform0.threshold * 0.5, 0.0) * _uniform0.spread;
    if _uniform0.invert > 0.5 {
      e = 1.0 - clamp(e, 0.0, 1.0);
    }
    e = clamp(e, 0.0, 1.0);
    let a: f32 = f32(src.w);
    var _skTemp3: vec3<f32>;
    if a > 0.0 {
      _skTemp3 = vec3<f32>(src.xyz) / a;
    } else {
      _skTemp3 = vec3<f32>(0.0);
    }
    let base: vec3<f32> = _skTemp3;
    let under: vec3<f32> = (base * clamp(_uniform0.blend, 0.0, 2.0)) * 0.5;
    let glow: vec3<f32> = (vec3<f32>(_uniform0.color.xyz) * e) * f32(_uniform0.color.w);
    let rgb: vec3<f32> = clamp(under + glow, vec3<f32>(0.0), vec3<f32>(1.0)) * a + glow * (1.0 - a);
    return vec4<f32>(vec3<f32>(rgb), f32(a + e * (1.0 - a)));
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _skTemp4: vec4<f32>;
    if _uniform0.passindex < 0.5 {
      _skTemp4 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(1.0, 0.0), 0);
    } else {
      var _skTemp5: vec4<f32>;
      if _uniform0.passindex < 1.5 {
        _skTemp5 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(0.0, 1.0), 1);
      } else {
        _skTemp5 = effect__h4f2(_stageIn.sk_FragCoord.xy - layerAt__f2());
      }
      _skTemp4 = _skTemp5;
    }
    (*_stageOut).sk_FragColor = _skTemp4;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
