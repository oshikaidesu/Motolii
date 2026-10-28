// Channel Remap (HSV): AbsentMotion's channelremaphsv (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Rebuilds hue, saturation, value and alpha from other channels of the layer.
// --track@h:H,0,1,0.0,0.01
// --track@s:S,0,1,1.0,0.01
// --track@v:V,0,1,2.0,0.01
// --track@a:A,0,1,3.0,0.01
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
  h: f32,
  s: f32,
  v: f32,
  a: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn pick_fff4f(k: f32, hsva: vec4<f32>, luma: f32) -> f32 {
  {
    if k < 0.5 {
      return hsva.x;
    }
    if k < 1.5 {
      return hsva.y;
    }
    if k < 2.5 {
      return hsva.z;
    }
    if k < 3.5 {
      return hsva.w;
    }
    if k < 4.5 {
      return luma;
    }
    if k < 5.5 {
      return 0.0;
    }
    return 1.0;
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _5_src: vec4<f32> = textureSample(image_Texture, image_Sampler, ((_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var _6_c: vec4<f32> = vec4<f32>(_5_src);
    if _6_c.w > 0.0 {
      _6_c = vec4<f32>((_6_c.xyz / _6_c.w), _6_c.w);
    }
    let _7_hi: f32 = max(_6_c.x, max(_6_c.y, _6_c.z));
    let _8_lo: f32 = min(_6_c.x, min(_6_c.y, _6_c.z));
    let _9_d: f32 = _7_hi - _8_lo;
    var _10_hue: f32 = 0.0;
    if _9_d > 1e-05 {
      {
        if _7_hi == _6_c.x {
          let _skTemp1 = (_6_c.y - _6_c.z) / _9_d;
          _10_hue = (_skTemp1 - 6.0 * floor(_skTemp1 / 6.0));
        } else {
          if _7_hi == _6_c.y {
            _10_hue = (_6_c.z - _6_c.x) / _9_d + 2.0;
          } else {
            _10_hue = (_6_c.x - _6_c.y) / _9_d + 4.0;
          }
        }
      }
    }
    var _skTemp2: f32;
    if _7_hi > 0.0 {
      _skTemp2 = _9_d / _7_hi;
    } else {
      _skTemp2 = 0.0;
    }
    let _11_hsva: vec4<f32> = vec4<f32>(_10_hue * 0.166666672, _skTemp2, _7_hi, _6_c.w);
    let _12_luma: f32 = dot(_6_c.xyz, vec3<f32>(0.2126, 0.7152, 0.0722));
    let _13_o: vec4<f32> = clamp(vec4<f32>(pick_fff4f(_uniform0.h, _11_hsva, _12_luma), pick_fff4f(_uniform0.s, _11_hsva, _12_luma), pick_fff4f(_uniform0.v, _11_hsva, _12_luma), pick_fff4f(_uniform0.a, _11_hsva, _12_luma)), vec4<f32>(0.0), vec4<f32>(1.0));
    let _skTemp3 = _13_o.x * 6.0 + vec3<f32>(0.0, 4.0, 2.0);
    let _14_k: vec3<f32> = clamp(abs((_skTemp3 - 6.0 * floor(_skTemp3 / 6.0)) - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
    (*_stageOut).sk_FragColor = vec4<f32>(vec3<f32>(_13_o.z * mix(vec3<f32>(1.0), _14_k, vec3<f32>(_13_o.y))) * f32(_13_o.w), f32(_13_o.w));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
