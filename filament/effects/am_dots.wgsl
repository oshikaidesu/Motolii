// Dots: AbsentMotion's dots (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Draws a random but even pattern of soft dots over the layer, or uses the layer only as a mask for the dots.
// --track@scale:Scale,0.01,100,1.0,0.01
// --track@phase:Phase,-99,99,0.0,0.001
// --track@radius:Radius,0,2,0.2,0.001
// --track@feather:Feather,0,1,0.1,0.001
// --track@color1_x:Color 1 R,0,1,1,0.01
// --track@color1_y:Color 1 G,0,1,1,0.01
// --track@color1_z:Color 1 B,0,1,1,0.01
// --track@color1_w:Color 1 A,0,1,1,0.01
// --track@color2_x:Color 2 R,0,1,0,0.01
// --track@color2_y:Color 2 G,0,1,0,0.01
// --track@color2_z:Color 2 B,0,1,0,0.01
// --track@color2_w:Color 2 A,0,1,1,0.01
// --track@density:Density,1,13,12.0,1
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@fill:Fill,0,1,1.0,0.001
// --track@alpha:Alpha,0,1,1.0,0.001
// --track@blend:Blending Mode,0,1,0.0,0.01
// --track@seed:Seed,0,5,0.0,0.01
// --track@slack:Slack,0,1,1.0,0.001
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
  radius: f32,
  feather: f32,
  color1: vec4<f32>,
  color2: vec4<f32>,
  density: f32,
  offset: vec2<f32>,
  fill: f32,
  alpha: f32,
  blend: f32,
  seed: f32,
  slack: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn hash12_ff2(p: vec2<f32>) -> f32 {
  {
    var q: vec3<f32> = fract(p.xyx * 0.1031);
    q = q + dot(q, q.yzx + 33.33);
    return fract((q.x + q.y) * q.z);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let cell: f32 = ((select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y * max(_uniform0.scale, 0.01)) * 0.07692308;
    let x: vec2<f32> = ((coord - (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5) - _uniform0.offset) / cell;
    let n: vec2<f32> = floor(x);
    var cover: f32 = 0.0;
    var dot_: vec3<f32> = vec3<f32>(0.0);
    let r: f32 = max(_uniform0.radius, 0.0) * 0.5;
    let soft: f32 = max(_uniform0.feather * r, 0.5 / cell);
    {
      var j: i32 = -1;
      for (; j <= 1; j = j + i32(1)) {
        {
          {
            var i: i32 = -1;
            for (; i <= 1; i = i + i32(1)) {
              {
                let g: vec2<f32> = n + vec2<f32>(f32(i), f32(j));
                {
                  var k: i32 = 0;
                  for (; k < 3; k = k + i32(1)) {
                    {
                      var _0_q: vec3<f32> = fract(((g + _uniform0.seed * 9.7) + f32(k) * 41.3).xyx * vec3<f32>(0.1031, 0.103, 0.0973));
                      _0_q = _0_q + dot(_0_q, _0_q.yzx + 33.33);
                      let h: vec2<f32> = fract((_0_q.xx + _0_q.yz) * _0_q.zy);
                      var _skTemp1: f32;
                      if k == 0 {
                        _skTemp1 = _uniform0.density * 0.09090909;
                      } else {
                        _skTemp1 = (_uniform0.density - 10.0) - f32(k);
                      }
                      let chance: f32 = _skTemp1;
                      if hash12_ff2((g * 1.7 + f32(k) * 13.1) + _uniform0.seed) >= chance {
                        continue;
                      }
                      let t: f32 = 6.28318548 * (_uniform0.phase * (0.6 + 0.8 * h.y) + h.x);
                      let at: vec2<f32> = ((g + 0.5) + ((h - 0.5) * _uniform0.slack) * 0.6) + (0.2 * _uniform0.slack) * vec2<f32>(cos(t), sin(t));
                      let d: f32 = length(x - at);
                      let c: f32 = 1.0 - smoothstep(r - soft, r + soft, d);
                      if c > cover {
                        {
                          cover = c;
                          dot_ = mix(vec3<f32>(_uniform0.color1.xyz), vec3<f32>(_uniform0.color2.xyz), vec3<f32>(hash12_ff2((g * 3.1 + f32(k) * 5.3) + _uniform0.seed * 2.1)));
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    let a: f32 = f32(src.w);
    let base: vec3<f32> = vec3<f32>(src.xyz) / a;
    var _skTemp2: vec3<f32>;
    if _uniform0.blend < 0.5 {
      _skTemp2 = dot_;
    } else {
      var _skTemp3: vec3<f32>;
      if _uniform0.blend < 1.5 {
        _skTemp3 = base * dot_;
      } else {
        _skTemp3 = (base + dot_) - base * dot_;
      }
      _skTemp2 = _skTemp3;
    }
    let shade: vec3<f32> = _skTemp2;
    let k: f32 = cover * clamp(_uniform0.alpha, 0.0, 1.0);
    let layerA: f32 = clamp(_uniform0.fill, 0.0, 1.0);
    let outA: f32 = k + layerA * (1.0 - k);
    var _skTemp4: vec3<f32>;
    if outA > 0.0 {
      _skTemp4 = (shade * k + (base * layerA) * (1.0 - k)) / outA;
    } else {
      _skTemp4 = vec3<f32>(0.0);
    }
    let rgb: vec3<f32> = _skTemp4;
    return vec4<f32>(vec3<f32>(rgb) * f32(outA * a), f32(outA * a));
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
