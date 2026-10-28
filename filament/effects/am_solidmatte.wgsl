// Solid Matte: AbsentMotion's solidmatte (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Key
// --about:Puts a solid color behind the layer, filling its transparent parts. Punch Out shows only the color with the layer's shape cut out of it, and Screen Space fills the whole screen instead of the layer's box.
// --track@color_x:Color R,0,1,0.1765,0.01
// --track@color_y:Color G,0,1,0.1176,0.01
// --track@color_z:Color B,0,1,0.9647,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@alpha:Alpha,0,1,1.0,0.01
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
  color: vec4<f32>,
  alpha: f32,
  punch: f32,
  screenSpace: f32,
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
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var _skTemp1: bool;
    var _skTemp2: bool;
    if (coord.x >= 0.0) && (coord.y >= 0.0) {
      _skTemp2 = coord.x < layerSize__f2().x;
    } else {
      _skTemp2 = false;
    }
    if _skTemp2 {
      _skTemp1 = coord.y < layerSize__f2().y;
    } else {
      _skTemp1 = false;
    }
    let inside: bool = _skTemp1;
    var _skTemp3: f32;
    if (_uniform0.screenSpace > 0.5) || inside {
      _skTemp3 = _uniform0.color.w * f32(clamp(_uniform0.alpha, 0.0, 1.0));
    } else {
      _skTemp3 = 0.0;
    }
    let m: f32 = _skTemp3;
    let matte: vec4<f32> = vec4<f32>(_uniform0.color.xyz * m, m);
    if _uniform0.punch > 0.5 {
      return matte * (1.0 - c.w);
    }
    return c + matte * (1.0 - c.w);
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
