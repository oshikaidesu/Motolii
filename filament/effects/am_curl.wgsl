// Curl: AbsentMotion's curl (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Rolls one end of the layer around a circle while the other end stays flat; animate Offset to roll or unroll it. Best on wide, thin layers.
// --track@offset:Offset,-0.5,1.5,0.5,0.001
// --track@radius:Radius,0,50,1.0,0.01
// --track@curlFrom:Curl From,0,1,1.0,0.01
// --track@direction:Direction,0,1,0.0,0.01
// --track@stretch:Stretch,0.01,50,1.0,0.01
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
  offset: f32,
  radius: f32,
  curlFrom: f32,
  direction: f32,
  stretch: f32,
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
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    var p: vec2<f32> = coord;
    if _uniform0.curlFrom < 0.5 {
      p.x = layerSize__f2().x - p.x;
    }
    if _uniform0.direction > 0.5 {
      p.y = layerSize__f2().y - p.y;
    }
    var _skTemp1: f32;
    if _uniform0.curlFrom < 0.5 {
      _skTemp1 = layerSize__f2().x * (1.0 - _uniform0.offset);
    } else {
      _skTemp1 = layerSize__f2().x * _uniform0.offset;
    }
    let x0: f32 = _skTemp1;
    let yc: f32 = layerSize__f2().y * 0.5;
    let r: f32 = (max(_uniform0.radius, 0.001) * layerSize__f2().y) * 0.5;
    let k: f32 = max(_uniform0.stretch, 0.01) / r;
    let reach: f32 = layerSize__f2().x - x0;
    let c: vec2<f32> = vec2<f32>(x0, yc - r);
    let d: vec2<f32> = p - c;
    let off: f32 = length(d) - r;
    var phi: f32 = atan2(d.x, d.y);
    if phi < 0.0 {
      phi = phi + 6.28318548;
    }
    var src: vec2<f32> = vec2<f32>(-1.0);
    if (abs(off) <= (yc + 0.5)) && (reach > 0.0) {
      {
        {
          var m: i32 = 7;
          for (; m >= 0; m = m - i32(1)) {
            {
              let along: f32 = (phi + 6.28318548 * f32(m)) / k;
              if along < reach {
                {
                  src = vec2<f32>(x0 + along, yc + off);
                  break;
                }
              }
            }
          }
        }
      }
    }
    if (src.x < 0.0) && (p.x <= x0) {
      src = p;
    }
    if src.x < 0.0 {
      return vec4<f32>(0.0);
    }
    if _uniform0.curlFrom < 0.5 {
      src.x = layerSize__f2().x - src.x;
    }
    if _uniform0.direction > 0.5 {
      src.y = layerSize__f2().y - src.y;
    }
    return textureSample(image_Texture, image_Sampler, (src + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
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
