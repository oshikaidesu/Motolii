// Voronoi Cells: AbsentMotion's voronoicells (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Fills the opaque parts of the layer with a pattern of cells around randomly placed points.
// --track@scale:Scale,0.01,100,1.0,0.01
// --track@phase:Phase,-99,99,0.0,0.001
// --track@lineWidth:Line Width,0,1,0.0,0.001
// --track@color1_x:Color 1 R,0,1,1,0.01
// --track@color1_y:Color 1 G,0,1,1,0.01
// --track@color1_z:Color 1 B,0,1,1,0.01
// --track@color1_w:Color 1 A,0,1,1,0.01
// --track@color2_x:Color 2 R,0,1,0,0.01
// --track@color2_y:Color 2 G,0,1,0,0.01
// --track@color2_z:Color 2 B,0,1,0,0.01
// --track@color2_w:Color 2 A,0,1,1,0.01
// --track@lineColor_x:Line Color R,0,1,0,0.01
// --track@lineColor_y:Line Color G,0,1,0,0.01
// --track@lineColor_z:Line Color B,0,1,1,0.01
// --track@lineColor_w:Line Color A,0,1,1,0.01
// --track@seed:Seed,0,5,0.0,0.01
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
  scale: f32,
  phase: f32,
  lineWidth: f32,
  color1: vec4<f32>,
  color2: vec4<f32>,
  lineColor: vec4<f32>,
  seed: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn hash22_f2f2(p: vec2<f32>) -> vec2<f32> {
  {
    var q: vec3<f32> = fract(p.xyx * vec3<f32>(0.1031, 0.103, 0.0973));
    q = q + dot(q, q.yzx + 33.33);
    return fract((q.xx + q.yz) * q.zy);
  }
}
fn site_f2f2(n: vec2<f32>) -> vec2<f32> {
  {
    let h: vec2<f32> = hash22_f2f2(n + _uniform0.seed * 19.19);
    let w: vec2<f32> = hash22_f2f2((n + 71.3) + _uniform0.seed * 3.7);
    return 0.5 + 0.42 * vec2<f32>(sin(6.28318548 * (_uniform0.phase * (0.5 + w.x) + h.x)), cos(6.28318548 * (_uniform0.phase * (0.5 + w.y) + h.y)));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let cell: f32 = ((select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y * max(_uniform0.scale, 0.01)) * 0.07692308;
    let x: vec2<f32> = (coord - (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5) / cell;
    let n: vec2<f32> = floor(x);
    let f: vec2<f32> = x - n;
    var best: vec2<f32> = vec2<f32>(0.0);
    var bestCell: vec2<f32> = vec2<f32>(0.0);
    var d1: f32 = 8.0;
    {
      var j: i32 = -1;
      for (; j <= 1; j = j + i32(1)) {
        {
          {
            var i: i32 = -1;
            for (; i <= 1; i = i + i32(1)) {
              {
                let g: vec2<f32> = vec2<f32>(f32(i), f32(j));
                let r: vec2<f32> = (g + site_f2f2(n + g)) - f;
                let d: f32 = dot(r, r);
                if d < d1 {
                  {
                    d1 = d;
                    best = r;
                    bestCell = n + g;
                  }
                }
              }
            }
          }
        }
      }
    }
    var edge: f32 = 8.0;
    {
      var j: i32 = -2;
      for (; j <= 2; j = j + i32(1)) {
        {
          {
            var i: i32 = -2;
            for (; i <= 2; i = i + i32(1)) {
              {
                let g: vec2<f32> = vec2<f32>(f32(i), f32(j));
                let r: vec2<f32> = (g + site_f2f2(n + g)) - f;
                let diff: vec2<f32> = r - best;
                if dot(diff, diff) > 1e-05 {
                  edge = min(edge, dot(0.5 * (best + r), normalize(diff)));
                }
              }
            }
          }
        }
      }
    }
    var _0_q: vec3<f32> = fract(((bestCell + _uniform0.seed * 5.3) + 0.37).xyx * 0.1031);
    _0_q = _0_q + dot(_0_q, _0_q.yzx + 33.33);
    let fill: vec3<f32> = mix(vec3<f32>(_uniform0.color1.xyz), vec3<f32>(_uniform0.color2.xyz), vec3<f32>(fract((_0_q.x + _0_q.y) * _0_q.z)));
    var _skTemp1: f32;
    if _uniform0.lineWidth > 0.0 {
      _skTemp1 = 1.0 - smoothstep(_uniform0.lineWidth * 0.5 - 0.5 / cell, _uniform0.lineWidth * 0.5 + 0.5 / cell, edge);
    } else {
      _skTemp1 = 0.0;
    }
    let line: f32 = _skTemp1;
    let rgb: vec3<f32> = mix(fill, vec3<f32>(_uniform0.lineColor.xyz), vec3<f32>(line * f32(_uniform0.lineColor.w)));
    return vec4<f32>(vec3<f32>(rgb) * src.w, src.w);
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
