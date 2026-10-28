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
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    let inside: bool = (((coord.x >= 0.0) && (coord.y >= 0.0)) && (coord.x < _uniform0.resolution.x)) && (coord.y < _uniform0.resolution.y);
    var _skTemp1: f32;
    if (_uniform0.screenSpace > 0.5) || inside {
      _skTemp1 = _uniform0.color.w * f32(clamp(_uniform0.alpha, 0.0, 1.0));
    } else {
      _skTemp1 = 0.0;
    }
    let m: f32 = _skTemp1;
    let matte: vec4<f32> = vec4<f32>(_uniform0.color.xyz * m, m);
    if _uniform0.punch > 0.5 {
      return matte * (1.0 - c.w);
    }
    return c + matte * (1.0 - c.w);
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
