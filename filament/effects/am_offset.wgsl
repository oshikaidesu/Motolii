// Offset: AbsentMotion's offset (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Move
// --about:Shifts the layer's picture, wrapping it round at the edges of the layer's box.
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@feather:Feather,0,1,0.0,0.001
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
  offset: vec2<f32>,
  feather: f32,
  mask: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let box: vec2<f32> = max(_uniform0.resolution, vec2<f32>(1.0));
    if (((coord.x < 0.0) || (coord.y < 0.0)) || (coord.x >= box.x)) || (coord.y >= box.y) {
      {
        return textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
      }
    }
    let _skTemp1 = coord - _uniform0.offset;
    var color: vec4<f32> = textureSample(image_Texture, image_Sampler, (_skTemp1 - box * floor(_skTemp1 / box)) / _uniform0.resolution);
    if _uniform0.feather > 0.0 {
      {
        let _skTemp2 = _uniform0.offset;
        let seam: vec2<f32> = (_skTemp2 - box * floor(_skTemp2 / box));
        var d: vec2<f32> = abs(coord - seam);
        d = min(d, box - d);
        var w: vec2<f32> = smoothstep(vec2<f32>(0.0), max((_uniform0.feather * box) * 0.5, vec2<f32>(0.001)), d);
        if min(seam.x, box.x - seam.x) < 0.5 {
          w.x = 1.0;
        }
        if min(seam.y, box.y - seam.y) < 0.5 {
          w.y = 1.0;
        }
        color = mix(textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution), color, vec4<f32>(f32(w.x * w.y)));
      }
    }
    if _uniform0.mask > 0.5 {
      {
        color = color * textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution).w;
      }
    }
    return color;
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
