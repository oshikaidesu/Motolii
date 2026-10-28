// Block Noise: AbsentMotion's blocknoise (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --about:Flickers random blocks of noise over the layer. Multiply modulates the existing colors; Add adds noise to them. Freeze holds the pattern still.
// --track@blockSize:Size,1,999,20.0,0.1
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
// --track@strength:Strength,0,1,0.25,0.001
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@overshoot:Overshoot,0.5,2,2.0,0.001
// --track@seed:Seed,0,5,0.0,0.01
// --track@blending:Blending Mode,0,1,1.0,0.01
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
  layer: vec2<f32>,
  time: f32,
  blockSize: f32,
  stretch: vec2<f32>,
  strength: f32,
  mono: f32,
  freeze: f32,
  extremes: f32,
  offset: vec2<f32>,
  overshoot: f32,
  seed: f32,
  blending: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn hash13_ff3(_skParam0: vec3<f32>) -> f32 {
  var p = _skParam0;
  {
    p = fract(p * 0.1031);
    p = p + dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let dims: vec2<f32> = ((max(abs(_uniform0.stretch), vec2<f32>(0.01)) * max(_uniform0.blockSize, 1.0)) * _uniform0.layer) * 0.006666667;
    let n: vec2<f32> = floor(((coord - _uniform0.resolution * 0.5) - _uniform0.offset) / dims);
    var _skTemp1: f32;
    if _uniform0.freeze > 0.5 {
      _skTemp1 = 0.0;
    } else {
      _skTemp1 = floor(_uniform0.time * 24.0);
    }
    let frame: f32 = _skTemp1;
    let k: vec3<f32> = vec3<f32>(n, frame + _uniform0.seed * 101.0);
    var v: vec3<f32> = vec3<f32>(hash13_ff3(k), hash13_ff3(k + 19.7), hash13_ff3(k + 41.3));
    if _uniform0.mono > 0.5 {
      v = v.xxx;
    }
    if _uniform0.extremes > 0.5 {
      v = step(vec3<f32>(0.5), v);
    }
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let amount: f32 = clamp(_uniform0.strength, 0.0, 1.0);
    var _skTemp2: vec3<f32>;
    if _uniform0.blending < 0.5 {
      _skTemp2 = base + ((v - 0.5) * _uniform0.overshoot) * amount;
    } else {
      _skTemp2 = base * mix(vec3<f32>(1.0), v * _uniform0.overshoot, vec3<f32>(amount));
    }
    var rgb: vec3<f32> = _skTemp2;
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(vec3<f32>(rgb) * src.w, src.w);
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
