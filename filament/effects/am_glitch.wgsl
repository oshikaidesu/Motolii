// Glitch: AbsentMotion's glitch (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Random horizontal block displacement and color channel offsets, changing a few times per second.
// --track@intensity:Intensity,0.0,1.0,0.5,0.01
// --track@blockHeight:Block Height,4.0,80.0,20.0,1.0
// --track@seed:Seed,0.0,1000.0,0.0,1.0
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
  intensity: f32,
  blockHeight: f32,
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
fn hash_ff2(p: vec2<f32>) -> f32 {
  {
    var q: vec3<f32> = fract(vec3<f32>(p, _uniform0.seed) * 0.1031);
    q = q + dot(q, q.zyx + 31.32);
    return fract((q.x + q.y) * q.z);
  }
}
fn noise_ff2(p: vec2<f32>) -> f32 {
  {
    let i: vec2<f32> = floor(p);
    let f: vec2<f32> = fract(p);
    let a: f32 = hash_ff2(i);
    let b: f32 = hash_ff2(i + vec2<f32>(1.0, 0.0));
    let c: f32 = hash_ff2(i + vec2<f32>(0.0, 1.0));
    let d: f32 = hash_ff2(i + vec2<f32>(1.0));
    let u: vec2<f32> = (f * f) * (3.0 - 2.0 * f);
    return (mix(a, b, u.x) + ((c - a) * u.y) * (1.0 - u.x)) + ((d - b) * u.x) * u.y;
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let _skTemp1 = floor(_uniform0.time * 4.0);
    let t: f32 = (_skTemp1 - 1024.0 * floor(_skTemp1 / 1024.0)) * 0.25;
    let block: f32 = floor(coord.y / max(_uniform0.blockHeight, 0.0001));
    let glitchNoise: f32 = noise_ff2(vec2<f32>(block, t * 100.0));
    let offset: f32 = ((glitchNoise - 0.5) * _uniform0.intensity) * 20.0;
    let rNoise: f32 = noise_ff2(vec2<f32>(coord.x * 0.1 + t * 50.0, block * 10.0));
    let gNoise: f32 = noise_ff2(vec2<f32>((coord.x * 0.1 + t * 50.0) + 100.0, block * 10.0));
    let bNoise: f32 = noise_ff2(vec2<f32>((coord.x * 0.1 + t * 50.0) + 200.0, block * 10.0));
    let rOffset: f32 = ((rNoise - 0.5) * _uniform0.intensity) * 10.0;
    let gOffset: f32 = ((gNoise - 0.5) * _uniform0.intensity) * 10.0;
    let bOffset: f32 = ((bNoise - 0.5) * _uniform0.intensity) * 10.0;
    var rCoord: vec2<f32> = coord + vec2<f32>(offset + rOffset, 0.0);
    var gCoord: vec2<f32> = coord + vec2<f32>(offset + gOffset, 0.0);
    var bCoord: vec2<f32> = coord + vec2<f32>(offset + bOffset, 0.0);
    var _skTemp2: bool;
    if all((coord >= vec2<f32>(0.0))) {
      _skTemp2 = all((coord < layerSize__f2()));
    } else {
      _skTemp2 = false;
    }
    if _skTemp2 {
      {
        rCoord = clamp(rCoord, vec2<f32>(0.0), (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) - 1.0);
        gCoord = clamp(gCoord, vec2<f32>(0.0), (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) - 1.0);
        bCoord = clamp(bCoord, vec2<f32>(0.0), (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) - 1.0);
      }
    }
    let rCol: vec4<f32> = textureSample(image_Texture, image_Sampler, (rCoord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let gCol: vec4<f32> = textureSample(image_Texture, image_Sampler, (gCoord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let bCol: vec4<f32> = textureSample(image_Texture, image_Sampler, (bCoord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let alpha: f32 = max(max(rCol.w, gCol.w), bCol.w);
    if alpha < 0.001 {
      {
        return vec4<f32>(0.0);
      }
    }
    let r: f32 = f32(vec4<f32>(rCol.xyz / max(rCol.w, 0.0001), rCol.w).x);
    let g: f32 = f32(vec4<f32>(gCol.xyz / max(gCol.w, 0.0001), gCol.w).y);
    let b: f32 = f32(vec4<f32>(bCol.xyz / max(bCol.w, 0.0001), bCol.w).z);
    return vec4<f32>(f32(r * f32(alpha)), f32(g * f32(alpha)), f32(b * f32(alpha)), alpha);
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
