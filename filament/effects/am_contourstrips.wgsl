// Contour Strips: AbsentMotion's contourstrips (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Draws stacked paper strips whose height follows the layer's brightness or opacity, like a ridge-line landscape.
// --track@count:Count,0.5,200,10.0,0.1
// --track@height:Height,0,15,1.0,0.01
// --track@angle:Angle,-3600,3600,0.0,0.5
// --track@thickness:Thickness,0,1,0.005,0.001
// --track@phase:Phase,-1000,1000,0.0,0.01
// --track@edgeColor_x:Edge Color R,0,1,0.1333,0.01
// --track@edgeColor_y:Edge Color G,0,1,0.2,0.01
// --track@edgeColor_z:Edge Color B,0,1,0.3333,0.01
// --track@edgeColor_w:Edge Color A,0,1,1,0.01
// --track@color1_x:Color 1 R,0,1,1,0.01
// --track@color1_y:Color 1 G,0,1,0.6,0.01
// --track@color1_z:Color 1 B,0,1,0.2,0.01
// --track@color1_w:Color 1 A,0,1,1,0.01
// --track@color2_x:Color 2 R,0,1,0.6667,0.01
// --track@color2_y:Color 2 G,0,1,0.2,0.01
// --track@color2_z:Color 2 B,0,1,0.6,0.01
// --track@color2_w:Color 2 A,0,1,1,0.01
// --track@light_x:Light Color R,0,1,1,0.01
// --track@light_y:Light Color G,0,1,1,0.01
// --track@light_z:Light Color B,0,1,1,0.01
// --track@light_w:Light Color A,0,1,1,0.01
// --track@lightAngle:Light Angle,0,3600,0.0,0.5
// --track@map:Height Map,0,1,1.0,0.01
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
  count: f32,
  height: f32,
  angle: f32,
  thickness: f32,
  phase: f32,
  invert: f32,
  edgeColor: vec4<f32>,
  color1: vec4<f32>,
  color2: vec4<f32>,
  light: vec4<f32>,
  lightAngle: f32,
  map: f32,
  mask: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn imageAt_h4f2(p: vec2<f32>) -> vec4<f32> {
  {
    return textureSample(image_Texture, image_Sampler, (p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
fn heightAt_ff2(p: vec2<f32>) -> f32 {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, (p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var _skTemp1: f32;
    if _uniform0.map > 0.5 {
      var _skTemp2: f32;
      if c.w > 0.0 {
        _skTemp2 = dot(vec3<f32>(c.xyz) / f32(c.w), vec3<f32>(0.2126, 0.7152, 0.0722)) * f32(c.w);
      } else {
        _skTemp2 = 0.0;
      }
      _skTemp1 = _skTemp2;
    } else {
      _skTemp1 = f32(c.w);
    }
    let h: f32 = _skTemp1;
    var _skTemp3: f32;
    if _uniform0.invert > 0.5 {
      _skTemp3 = 1.0 - h;
    } else {
      _skTemp3 = h;
    }
    return _skTemp3;
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let ra: f32 = radians(_uniform0.angle);
    let ax: vec2<f32> = vec2<f32>(cos(ra), sin(ra));
    let ay: vec2<f32> = vec2<f32>(-ax.y, ax.x);
    let mid: vec2<f32> = (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5;
    let r: vec2<f32> = vec2<f32>(dot(coord - mid, ax), dot(coord - mid, ay));
    let extent: f32 = abs(dot(select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)), abs(ay)));
    let gap: f32 = extent / max(_uniform0.count, 0.5);
    let tall: f32 = max(_uniform0.height, 0.0) * gap;
    let thick: f32 = (clamp(_uniform0.thickness, 0.0, 1.0) * gap) * 0.5;
    let first: f32 = floor(((r.y + tall) + thick) / gap - _uniform0.phase);
    {
      var k: i32 = 0;
      for (; k < 48; k = k + i32(1)) {
        {
          let i: f32 = first - f32(k);
          let base: f32 = (i + _uniform0.phase) * gap;
          if base < r.y {
            break;
          }
          let at: vec2<f32> = (mid + r.x * ax) + base * ay;
          let h: f32 = heightAt_ff2(at) * tall;
          let top: f32 = base - h;
          if r.y >= (top - thick) {
            {
              var rgb: vec3<f32>;
              if r.y < top {
                {
                  let slope: f32 = ((heightAt_ff2(at + ax * 2.0) - heightAt_ff2(at - ax * 2.0)) * tall) * 0.25;
                  let la: f32 = radians(_uniform0.lightAngle);
                  let lit: f32 = 0.5 + 0.5 * dot(normalize(vec2<f32>(slope, -1.0)), vec2<f32>(cos(la), -sin(la)));
                  rgb = (vec3<f32>(_uniform0.edgeColor.xyz) * vec3<f32>(_uniform0.light.xyz)) * mix(0.35, 1.0, lit);
                }
              } else {
                {
                  var _skTemp4: f32;
                  if h > 0.0 {
                    _skTemp4 = clamp((r.y - top) / h, 0.0, 1.0);
                  } else {
                    _skTemp4 = 1.0;
                  }
                  let t: f32 = _skTemp4;
                  rgb = mix(vec3<f32>(_uniform0.color1.xyz), vec3<f32>(_uniform0.color2.xyz), vec3<f32>(t));
                }
              }
              var _skTemp5: f32;
              if _uniform0.mask > 0.5 {
                _skTemp5 = f32(imageAt_h4f2(coord).w);
              } else {
                _skTemp5 = 1.0;
              }
              let a: f32 = _skTemp5;
              return vec4<f32>(vec3<f32>(rgb * a), f32(a));
            }
          }
        }
      }
    }
    return vec4<f32>(0.0);
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
