// VHS: AbsentMotion's vhs (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Looks like an old video tape: color bleed, smeared lines, scanlines, tape noise and rolling tracking bands.
// --track@chroma:Color Bleed,0,40,4.0,0.1
// --track@smear:Smear,0,20,2.0,0.1
// --track@scanlines:Scanlines,0,1,0.3,0.01
// --track@noise:Noise,0,1,0.3,0.01
// --track@tracking:Tracking,0,1,0.5,0.01
// --track@wobble:Wobble,0,20,1.5,0.1
// --track@saturation:Saturation,0,2,0.7,0.01
// --track@seed:Seed,0,1000,1.0,1
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
  chroma: f32,
  smear: f32,
  scanlines: f32,
  noise: f32,
  tracking: f32,
  wobble: f32,
  saturation: f32,
  headSwitch: f32,
  seed: f32,
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
fn hash_ff2(_skParam0: vec2<f32>) -> f32 {
  var p = _skParam0;
  {
    p = fract(p * vec2<f32>(123.34, 456.21));
    p = p + dot(p, p + 45.32);
    return fract(p.x * p.y);
  }
}
fn wave_fff(x: f32, s: f32) -> f32 {
  {
    let i: f32 = floor(x);
    let f: f32 = fract(x);
    return mix(hash_ff2(vec2<f32>(i, s)), hash_ff2(vec2<f32>(i + 1.0, s)), (f * f) * (3.0 - 2.0 * f));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let _skTemp1 = floor((_uniform0.time + 0.000999) * 30.0);
    let frame: f32 = (_skTemp1 - 997.0 * floor(_skTemp1 / 997.0));
    let _skTemp2 = _uniform0.seed;
    let s: f32 = (_skTemp2 - 1000.0 * floor(_skTemp2 / 1000.0)) * 0.731 + 0.17;
    let y: f32 = coord.y;
    let _skTemp3 = floor(y);
    let line: f32 = (_skTemp3 - 2048.0 * floor(_skTemp3 / 2048.0));
    var shift: f32 = ((hash_ff2(vec2<f32>(line * 0.37 + s, frame)) - 0.5) * _uniform0.wobble) * 0.6 + ((wave_fff(y * 0.015 + _uniform0.time * 1.7, s + 3.0) - 0.5) * _uniform0.wobble) * 2.0;
    let roll: f32 = (fract(_uniform0.time * 0.09 + hash_ff2(vec2<f32>(s, 7.0))) * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y) * 1.4 - (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y * 0.2;
    let reach: f32 = (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y * 0.05 + 4.0;
    let band: f32 = ((1.0 - smoothstep(0.0, reach, abs(y - roll))) * _uniform0.tracking) * smoothstep(0.3, 0.6, wave_fff(_uniform0.time * 0.4, s + 11.0));
    shift = shift + (band * (hash_ff2(vec2<f32>(line, frame + 1.0)) - 0.3)) * 28.0;
    var _skTemp4: f32;
    if _uniform0.headSwitch > 0.5 {
      _skTemp4 = 1.0 - smoothstep(0.0, layerSize__f2().y * 0.035 + 3.0, layerSize__f2().y - y);
    } else {
      _skTemp4 = 0.0;
    }
    let head: f32 = _skTemp4;
    shift = shift + head * (10.0 + 16.0 * hash_ff2(vec2<f32>(floor(y * 0.5), frame + 2.0)));
    let p: vec2<f32> = vec2<f32>(coord.x - shift, y);
    let base: vec4<f32> = textureSample(image_Texture, image_Sampler, (p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var smeared: vec4<f32> = base;
    {
      var i: i32 = 1;
      for (; i <= 4; i = i + i32(1)) {
        {
          smeared = smeared + textureSample(image_Texture, image_Sampler, ((p - vec2<f32>((_uniform0.smear * f32(i)) * 0.5, 0.0)) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
        }
      }
    }
    smeared = smeared * 0.2;
    var bleed: vec4<f32> = vec4<f32>(0.0);
    {
      var i: i32 = 0;
      for (; i < 4; i = i + i32(1)) {
        {
          bleed = bleed + textureSample(image_Texture, image_Sampler, ((p - vec2<f32>(_uniform0.chroma * (0.6 + 0.4 * f32(i)), 0.0)) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
        }
      }
    }
    bleed = bleed * 0.25;
    let alpha: f32 = max(base.w, max(smeared.w, bleed.w));
    if alpha <= 0.0 {
      return vec4<f32>(0.0);
    }
    var _skTemp5: vec3<f32>;
    if base.w > 0.0 {
      _skTemp5 = vec3<f32>(base.xyz) / f32(base.w);
    } else {
      _skTemp5 = vec3<f32>(0.0);
    }
    var _skTemp6: vec3<f32>;
    if smeared.w > 0.0 {
      _skTemp6 = vec3<f32>(smeared.xyz) / f32(smeared.w);
    } else {
      _skTemp6 = vec3<f32>(0.0);
    }
    let _0_c: vec3<f32> = mix(_skTemp5, _skTemp6, vec3<f32>(0.7));
    var luma: f32 = vec3<f32>(dot(_0_c, vec3<f32>(0.299, 0.587, 0.114)), dot(_0_c, vec3<f32>(0.596, -0.274, -0.322)), dot(_0_c, vec3<f32>(0.211, -0.523, 0.312))).x;
    var _skTemp7: vec3<f32>;
    if bleed.w > 0.0 {
      _skTemp7 = vec3<f32>(bleed.xyz) / f32(bleed.w);
    } else {
      _skTemp7 = vec3<f32>(0.0);
    }
    let _1_c: vec3<f32> = _skTemp7;
    let iq: vec2<f32> = vec3<f32>(dot(_1_c, vec3<f32>(0.299, 0.587, 0.114)), dot(_1_c, vec3<f32>(0.596, -0.274, -0.322)), dot(_1_c, vec3<f32>(0.211, -0.523, 0.312))).yz * _uniform0.saturation;
    let grain: f32 = hash_ff2(vec2<f32>(coord.x * 0.71 + frame * 3.1, y * 1.3 + s)) - 0.5;
    luma = luma + (grain * _uniform0.noise) * 0.22;
    let streak: f32 = step((1.0 - _uniform0.noise * 0.004) - band * 0.2, hash_ff2(vec2<f32>(floor(coord.x * 0.0416666679) + line * 0.13, frame + s)));
    luma = luma + (streak * 0.6) * (0.5 + 0.5 * hash_ff2(vec2<f32>(line, frame + 5.0)));
    luma = luma + ((band + head) * (hash_ff2(vec2<f32>(coord.x * 0.37, line + frame)) - 0.35)) * 0.35;
    luma = luma * (1.0 - (_uniform0.scanlines * 0.5) * (0.5 + 0.5 * sin(6.28318548 * fract(y * 0.5))));
    let _2_c: vec3<f32> = vec3<f32>(luma, iq);
    let rgb: vec3<f32> = clamp(vec3<f32>(dot(_2_c, vec3<f32>(1.0, 0.956, 0.621)), dot(_2_c, vec3<f32>(1.0, -0.272, -0.647)), dot(_2_c, vec3<f32>(1.0, -1.106, 1.703))), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(vec3<f32>(rgb) * alpha, alpha);
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
