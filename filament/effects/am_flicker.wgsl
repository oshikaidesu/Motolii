// Flicker: AbsentMotion's flicker (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Flickers the opacity of the layer at random.
// --track@speed:Frequency,0.0,60.0,7.0,0.01
// --track@intensity:Strength,0.0,1.0,0.75,0.01
// --track@seed:Seed,0.0,100.0,0.0,0.1
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
  speed: f32,
  intensity: f32,
  seed: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn hash21_ff2(p: vec2<f32>) -> f32 {
  {
    var q: vec3<f32> = fract(p.xyx * 0.1031);
    q = q + dot(q, q.yzx + 33.33);
    return fract((q.x + q.y) * q.z);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if ((c.w <= 0.001) || (_uniform0.speed <= 0.0)) || (_uniform0.intensity <= 0.0) {
      return c;
    }
    let f: f32 = _uniform0.time * _uniform0.speed;
    let n: f32 = floor(f);
    const ramp: f32 = 0.15;
    let t: f32 = clamp(((f - n) - (1.0 - ramp)) / ramp, 0.0, 1.0);
    let _skTemp1 = n + 1.0;
    let r: f32 = mix(hash21_ff2(vec2<f32>((n - 4096.0 * floor(n / 4096.0)), _uniform0.seed)), hash21_ff2(vec2<f32>((_skTemp1 - 4096.0 * floor(_skTemp1 / 4096.0)), _uniform0.seed)), t);
    let keep: f32 = clamp(1.0 - _uniform0.intensity * r, 0.0, 1.0);
    return c * f32(keep);
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
