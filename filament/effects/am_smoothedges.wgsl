// Smooth Edges: AbsentMotion's smoothedges (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Key
// --pass@h:0,1,s1,s1
// --pass@blurPicture:0,1,s1,s1
// --pass@:0,0,s1,s1
// --about:Rounds and smooths the edges of the opaque parts of the layer without changing the colors inside. Mask To Layer only lets the edges move inward.
// --track@blur:Strength,0,300,12.0,0.1
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
  blur: f32,
  mask: f32,
  passindex: f32,
};
@group(0) @binding(6) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
@group(0) @binding(3) var h_Sampler: sampler;
@group(0) @binding(2) var h_Texture: texture_2d<f32>;
@group(0) @binding(5) var blurPicture_Sampler: sampler;
@group(0) @binding(4) var blurPicture_Texture: texture_2d<f32>;
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
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if _uniform0.blur <= 0.0 {
      return c;
    }
    let b: vec4<f32> = textureSample(blurPicture_Texture, blurPicture_Sampler, coord / _uniform0.resolution);
    let e: f32 = clamp(1.2 / max(_uniform0.blur, 1.0), 0.01, 0.5);
    var a: f32 = smoothstep(0.5 - e, 0.5 + e, f32(b.w));
    if _uniform0.mask > 0.5 {
      a = min(a, f32(c.w));
    }
    var _skTemp3: vec3<f32>;
    if c.w > 0.004 {
      _skTemp3 = vec3<f32>(c.xyz) / f32(c.w);
    } else {
      var _skTemp4: vec3<f32>;
      if b.w > 0.004 {
        _skTemp4 = vec3<f32>(b.xyz) / f32(b.w);
      } else {
        _skTemp4 = vec3<f32>(0.0);
      }
      _skTemp3 = _skTemp4;
    }
    let rgb: vec3<f32> = _skTemp3;
    return vec4<f32>(vec3<f32>(rgb * a), f32(a));
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _skTemp5: vec4<f32>;
    if _uniform0.passindex < 0.5 {
      _skTemp5 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(1.0, 0.0), 0);
    } else {
      var _skTemp6: vec4<f32>;
      if _uniform0.passindex < 1.5 {
        _skTemp6 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(0.0, 1.0), 1);
      } else {
        _skTemp6 = effect__h4f2(_stageIn.sk_FragCoord.xy);
      }
      _skTemp5 = _skTemp6;
    }
    (*_stageOut).sk_FragColor = _skTemp5;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
