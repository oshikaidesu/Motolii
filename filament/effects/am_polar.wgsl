// Polar Coordinates: AbsentMotion's polar (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --track@amount:Interpolation,0,1,1.0,0.001
// --track@direction:Type,0,1,0.0,0.01
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
  amount: f32,
  direction: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec2<f32> = _uniform0.resolution * 0.5;
    let r: f32 = 0.5 * min(_uniform0.resolution.x, _uniform0.resolution.y);
    var q: vec2<f32>;
    if _uniform0.direction < 0.5 {
      {
        let d: vec2<f32> = coord - c;
        var _skTemp1: f32;
        if dot(d, d) > 0.0 {
          _skTemp1 = atan2(d.x, -d.y);
        } else {
          _skTemp1 = 0.0;
        }
        let t: f32 = _skTemp1 * 0.159154937 + 0.5;
        q = vec2<f32>(t * _uniform0.resolution.x, (length(d) / r) * _uniform0.resolution.y);
      }
    } else {
      {
        let t: f32 = (coord.x / _uniform0.resolution.x - 0.5) * 6.28318548;
        let k: f32 = (coord.y / _uniform0.resolution.y) * r;
        q = c + vec2<f32>(sin(t), -cos(t)) * k;
      }
    }
    let at: vec2<f32> = mix(coord, q, vec2<f32>(clamp(_uniform0.amount, 0.0, 1.0)));
    if (((at.x < 0.0) || (at.y < 0.0)) || (at.x > _uniform0.resolution.x)) || (at.y > _uniform0.resolution.y) {
      return vec4<f32>(0.0);
    }
    return textureSample(image_Texture, image_Sampler, at / _uniform0.resolution);
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
