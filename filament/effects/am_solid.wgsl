// Solid Color: AbsentMotion's solid (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Lays one color over the layer, keeping its shape, with a choice of blend modes.
// --track@color_x:Color R,0,1,0.1765,0.01
// --track@color_y:Color G,0,1,0.1176,0.01
// --track@color_z:Color B,0,1,0.9647,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@amount:Alpha,0,1,1.0,0.01
// --track@blend:Blending Mode,0,1,0.0,0.01
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
  amount: f32,
  blend: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn blendWith_f3ff3f3(m: f32, b: vec3<f32>, s: vec3<f32>) -> vec3<f32> {
  {
    if m < 0.5 {
      return s;
    }
    if m < 1.5 {
      return b * s;
    }
    if m < 2.5 {
      return (b + s) - b * s;
    }
    if m < 3.5 {
      return mix((2.0 * b) * s, 1.0 - (2.0 * (1.0 - b)) * (1.0 - s), step(vec3<f32>(0.5), b));
    }
    if m < 4.5 {
      {
        let d: vec3<f32> = mix(sqrt(b), ((16.0 * b - 12.0) * b + 4.0) * b, step(b, vec3<f32>(0.25)));
        return mix(b - ((1.0 - 2.0 * s) * b) * (1.0 - b), b + (2.0 * s - 1.0) * (d - b), step(vec3<f32>(0.5), s));
      }
    }
    if m < 5.5 {
      return mix((2.0 * b) * s, 1.0 - (2.0 * (1.0 - b)) * (1.0 - s), step(vec3<f32>(0.5), s));
    }
    if m < 6.5 {
      return min(b + s, vec3<f32>(1.0));
    }
    if m < 7.5 {
      return min(b, s);
    }
    if m < 8.5 {
      return max(b, s);
    }
    if m < 9.5 {
      return abs(b - s);
    }
    if m < 10.5 {
      return min(b / max(1.0 - s, vec3<f32>(0.0001)), vec3<f32>(1.0));
    }
    return 1.0 - min((1.0 - b) / max(s, vec3<f32>(0.0001)), vec3<f32>(1.0));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if c.w <= 0.0 {
      return c;
    }
    let base: vec3<f32> = vec3<f32>(c.xyz) / f32(c.w);
    let rgb: vec3<f32> = mix(base, clamp(blendWith_f3ff3f3(_uniform0.blend, base, vec3<f32>(_uniform0.color.xyz)), vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(clamp(_uniform0.amount, 0.0, 1.0) * f32(_uniform0.color.w)));
    return vec4<f32>(vec3<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0))) * c.w, c.w);
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
