// Displacement Map: AbsentMotion's displacementmap (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --picture@backdrop:Behind
// --picture@below:Map layer
// --about:Uses the layer's colors to push the layers behind it around, then hides itself; or, with Map set to Layer Below, pushes this layer by the layer beneath.
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@xchannel:X,0,1,0.0,0.01
// --track@ychannel:Y,0,1,1.0,0.01
// --track@edges:Edges,0,1,1.0,0.01
// --track@map:Map,0,1,0.0,0.01
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
  screen: vec4<f32>,
  offset: vec2<f32>,
  xchannel: f32,
  ychannel: f32,
  edges: f32,
  fromCenter: f32,
  invert: f32,
  map: f32,
  layerRect: vec4<f32>,
  backdropBox: vec4<f32>,
};
@group(0) @binding(6) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
@group(0) @binding(3) var backdropPic_Sampler: sampler;
@group(0) @binding(2) var backdropPic_Texture: texture_2d<f32>;
@group(0) @binding(5) var belowPic_Sampler: sampler;
@group(0) @binding(4) var belowPic_Texture: texture_2d<f32>;
fn layerAt__f2() -> vec2<f32> {
  {
    return select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0));
  }
}
fn srgbOf__h4h4(c: vec4<f32>) -> vec4<f32> {
  {
    if c.w <= 0.0 {
      return vec4<f32>(0.0);
    }
    let u: vec3<f32> = clamp(c.xyz / c.w, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(mix(1.055 * pow(u, vec3<f32>(0.416666657)) - 0.055, u * 12.92, step(u, vec3<f32>(0.0031308))) * c.w, c.w);
  }
}
fn edged_f2f2f(p: vec2<f32>, mode: f32) -> vec2<f32> {
  {
    var q: vec2<f32> = (p - _uniform0.screen.xy) / _uniform0.screen.zw;
    if mode < 0.5 {
      q = clamp(q, 0.5 / _uniform0.screen.zw, 1.0 - 0.5 / _uniform0.screen.zw);
    } else {
      if mode < 1.5 {
        q = 1.0 - abs(1.0 - (q - 2.0 * floor(q / 2.0)));
      } else {
        if mode > 2.5 {
          q = fract(q);
        }
      }
    }
    return _uniform0.screen.xy + q * _uniform0.screen.zw;
  }
}
fn pick_fh4f(c: vec4<f32>, ch: f32) -> f32 {
  {
    var _skTemp1: vec3<f32>;
    if c.w > 0.0 {
      _skTemp1 = vec3<f32>(c.xyz) / f32(c.w);
    } else {
      _skTemp1 = vec3<f32>(0.0);
    }
    let u: vec3<f32> = _skTemp1;
    var _skTemp2: f32;
    if ch < 0.5 {
      _skTemp2 = u.x;
    } else {
      _skTemp2 = select(u.z, u.y, ch < 1.5);
    }
    return _skTemp2;
  }
}
fn shift_f2h4(c: vec4<f32>) -> vec2<f32> {
  {
    var m: vec2<f32> = vec2<f32>(pick_fh4f(c, _uniform0.xchannel), pick_fh4f(c, _uniform0.ychannel));
    if _uniform0.invert > 0.5 {
      m = 1.0 - m;
    }
    if _uniform0.fromCenter > 0.5 {
      m = m * 2.0 - 1.0;
    }
    return m * _uniform0.offset;
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    if _uniform0.map > 0.5 {
      {
        let m: vec4<f32> = srgbOf__h4h4(textureSample(belowPic_Texture, belowPic_Sampler, (coord + layerAt__f2()) / _uniform0.resolution));
        let _0_p: vec2<f32> = edged_f2f2f(coord - shift_f2h4(m) * f32(m.w), _uniform0.edges);
        return textureSample(image_Texture, image_Sampler, (_0_p + layerAt__f2()) / _uniform0.resolution);
      }
    }
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + layerAt__f2()) / _uniform0.resolution);
    if src.w <= 0.0 {
      return vec4<f32>(0.0);
    }
    let _1_p: vec2<f32> = edged_f2f2f(coord - shift_f2h4(src), _uniform0.edges);
    return srgbOf__h4h4(textureSample(backdropPic_Texture, backdropPic_Sampler, _uniform0.backdropBox.xy + ((_1_p + layerAt__f2()) / _uniform0.resolution) * _uniform0.backdropBox.zw)) * src.w;
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy - layerAt__f2());
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
