// Palette Map: AbsentMotion's palettemap (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Maps each pixel to the nearest color in a palette of up to eight colors. Alpha blends the mapped colors with the original layer.
// --track@colors:Colors,2,8,5.0,1
// --track@color1_x:Color 1 R,0,1,0.1137,0.01
// --track@color1_y:Color 1 G,0,1,0.1686,0.01
// --track@color1_z:Color 1 B,0,1,0.3255,0.01
// --track@color1_w:Color 1 A,0,1,1,0.01
// --track@color2_x:Color 2 R,0,1,0.4941,0.01
// --track@color2_y:Color 2 G,0,1,0.1451,0.01
// --track@color2_z:Color 2 B,0,1,0.3255,0.01
// --track@color2_w:Color 2 A,0,1,1,0.01
// --track@color3_x:Color 3 R,0,1,1,0.01
// --track@color3_y:Color 3 G,0,1,0,0.01
// --track@color3_z:Color 3 B,0,1,0.302,0.01
// --track@color3_w:Color 3 A,0,1,1,0.01
// --track@color4_x:Color 4 R,0,1,1,0.01
// --track@color4_y:Color 4 G,0,1,0.6392,0.01
// --track@color4_z:Color 4 B,0,1,0,0.01
// --track@color4_w:Color 4 A,0,1,1,0.01
// --track@color5_x:Color 5 R,0,1,1,0.01
// --track@color5_y:Color 5 G,0,1,0.9451,0.01
// --track@color5_z:Color 5 B,0,1,0.9098,0.01
// --track@color5_w:Color 5 A,0,1,1,0.01
// --track@color6_x:Color 6 R,0,1,1,0.01
// --track@color6_y:Color 6 G,0,1,0.3333,0.01
// --track@color6_z:Color 6 B,0,1,1,0.01
// --track@color6_w:Color 6 A,0,1,1,0.01
// --track@color7_x:Color 7 R,0,1,1,0.01
// --track@color7_y:Color 7 G,0,1,1,0.01
// --track@color7_z:Color 7 B,0,1,0.3333,0.01
// --track@color7_w:Color 7 A,0,1,1,0.01
// --track@color8_x:Color 8 R,0,1,1,0.01
// --track@color8_y:Color 8 G,0,1,1,0.01
// --track@color8_z:Color 8 B,0,1,1,0.01
// --track@color8_w:Color 8 A,0,1,1,0.01
// --track@dither:Dither,0,1,0.0,0.001
// --track@alpha:Alpha,0,1,1.0,0.001
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
  colors: f32,
  color1: vec4<f32>,
  color2: vec4<f32>,
  color3: vec4<f32>,
  color4: vec4<f32>,
  color5: vec4<f32>,
  color6: vec4<f32>,
  color7: vec4<f32>,
  color8: vec4<f32>,
  dither: f32,
  alpha: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    var _0_q: vec3<f32> = fract(floor(coord).xyx * 0.1031);
    _0_q = _0_q + dot(_0_q, _0_q.yzx + 33.33);
    let rgb: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w) + ((fract((_0_q.x + _0_q.y) * _0_q.z) - 0.5) * _uniform0.dither) * 0.3;
    var best: vec3<f32> = vec3<f32>(_uniform0.color1.xyz);
    var bestD: f32 = distance(rgb, best);
    var d: f32 = distance(rgb, vec3<f32>(_uniform0.color2.xyz));
    if d < bestD {
      {
        bestD = d;
        best = vec3<f32>(_uniform0.color2.xyz);
      }
    }
    if _uniform0.colors > 2.5 {
      {
        d = distance(rgb, vec3<f32>(_uniform0.color3.xyz));
        if d < bestD {
          {
            bestD = d;
            best = vec3<f32>(_uniform0.color3.xyz);
          }
        }
      }
    }
    if _uniform0.colors > 3.5 {
      {
        d = distance(rgb, vec3<f32>(_uniform0.color4.xyz));
        if d < bestD {
          {
            bestD = d;
            best = vec3<f32>(_uniform0.color4.xyz);
          }
        }
      }
    }
    if _uniform0.colors > 4.5 {
      {
        d = distance(rgb, vec3<f32>(_uniform0.color5.xyz));
        if d < bestD {
          {
            bestD = d;
            best = vec3<f32>(_uniform0.color5.xyz);
          }
        }
      }
    }
    if _uniform0.colors > 5.5 {
      {
        d = distance(rgb, vec3<f32>(_uniform0.color6.xyz));
        if d < bestD {
          {
            bestD = d;
            best = vec3<f32>(_uniform0.color6.xyz);
          }
        }
      }
    }
    if _uniform0.colors > 6.5 {
      {
        d = distance(rgb, vec3<f32>(_uniform0.color7.xyz));
        if d < bestD {
          {
            bestD = d;
            best = vec3<f32>(_uniform0.color7.xyz);
          }
        }
      }
    }
    if _uniform0.colors > 7.5 {
      {
        d = distance(rgb, vec3<f32>(_uniform0.color8.xyz));
        if d < bestD {
          {
            bestD = d;
            best = vec3<f32>(_uniform0.color8.xyz);
          }
        }
      }
    }
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    return vec4<f32>(vec3<f32>(mix(base, best, vec3<f32>(clamp(_uniform0.alpha, 0.0, 1.0)))) * src.w, src.w);
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
