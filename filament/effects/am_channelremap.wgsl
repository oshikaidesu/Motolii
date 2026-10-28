// Channel Remap (RGB): AbsentMotion's channelremap (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Rebuilds each color channel from another channel of the layer, for false color or for moving luminance into alpha.
// --track@r:R,0,1,0.0,0.01
// --track@g:G,0,1,1.0,0.01
// --track@b:B,0,1,2.0,0.01
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
  r: f32,
  g: f32,
  b: f32,
  a: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn pick_fff4(k: f32, c: vec4<f32>) -> f32 {
  {
    if k < 0.5 {
      return c.x;
    }
    if k < 1.5 {
      return c.y;
    }
    if k < 2.5 {
      return c.z;
    }
    if k < 3.5 {
      return c.w;
    }
    if k < 4.5 {
      return dot(c.xyz, vec3<f32>(0.2126, 0.7152, 0.0722));
    }
    if k < 5.5 {
      return 0.0;
    }
    return 1.0;
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_src: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    var _1_c: vec4<f32> = vec4<f32>(_0_src);
    if _1_c.w > 0.0 {
      _1_c = vec4<f32>((_1_c.xyz / _1_c.w), _1_c.w);
    }
    let _2_o: vec4<f32> = clamp(vec4<f32>(pick_fff4(_uniform0.r, _1_c), pick_fff4(_uniform0.g, _1_c), pick_fff4(_uniform0.b, _1_c), pick_fff4(_uniform0.a, _1_c)), vec4<f32>(0.0), vec4<f32>(1.0));
    (*_stageOut).sk_FragColor = vec4<f32>(vec3<f32>(_2_o.xyz) * f32(_2_o.w), f32(_2_o.w));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
