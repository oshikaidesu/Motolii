// Kaleidoscope Tile: ISF-Files' Kaleidoscope Tile.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Distort
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --track@size:size,0,2,0.5,0.001
// --track@sides:sides,1,32,6,0.001
// --track@rotation:rotation,0,1,0.125,0.001
// --track@angle:angle,0,1,0,0.001
// --track@slide1_x:slide1 x,0,1,0,0.001
// --track@slide1_y:slide1 y,0,1,0,0.001
// --track@slide2_x:slide2 x,0,1,0,0.001
// --track@slide2_y:slide2 y,0,1,0,0.001
// --track@shift_x:shift x,0,1,0,0.001
// --track@shift_y:shift y,0,1,0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    size: f32,
    sides: f32,
    rotation: f32,
    angle: f32,
    slide1_: vec2<f32>,
    slide2_: vec2<f32>,
    shift: vec2<f32>,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const tau: f32 = 6.2831855f;

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn pattern() -> vec2<f32> {
    var s: f32;
    var c: f32;
    var tex: vec2<f32>;
    var scale: f32;
    var point: vec2<f32>;
    var center: vec2<f32> = vec2<f32>(0.5f, 0.5f);
    var r: f32;
    var a: f32;
    var zoom: f32;

    let _e27 = global.rotation;
    s = sin((tau * _e27));
    let _e31 = global.rotation;
    c = cos((tau * _e31));
    let _e35 = isf_FragNormCoord_1;
    let _e36 = global.resolution;
    tex = (_e35 * _e36);
    let _e40 = global.size;
    scale = (1f / max(_e40, 0.001f));
    let _e45 = c;
    let _e46 = tex;
    let _e49 = s;
    let _e50 = tex;
    let _e54 = s;
    let _e55 = tex;
    let _e58 = c;
    let _e59 = tex;
    let _e64 = scale;
    point = (vec2<f32>(((_e45 * _e46.x) - (_e49 * _e50.y)), ((_e54 * _e55.x) + (_e58 * _e59.y))) * _e64);
    let _e67 = point;
    let _e68 = global.shift;
    let _e69 = global.resolution;
    let _e72 = global.resolution;
    point = ((_e67 - (_e68 * _e69)) / _e72);
    let _e74 = point;
    let _e76 = vec2(1f);
    point = (_e74 - (floor((_e74 / _e76)) * _e76));
    let _e81 = point;
    if (_e81.x < 0.5f) {
        {
            let _e86 = point;
            let _e88 = global.slide1_;
            let _e90 = (_e86.y + _e88.y);
            point.y = (_e90 - (floor((_e90 / 1f)) * 1f));
        }
    } else {
        {
            let _e97 = point;
            let _e99 = global.slide2_;
            let _e101 = (_e97.y + _e99.y);
            point.y = (_e101 - (floor((_e101 / 1f)) * 1f));
        }
    }
    let _e107 = point;
    if (_e107.y < 0.5f) {
        {
            let _e112 = point;
            let _e114 = global.slide1_;
            let _e116 = (_e112.x + _e114.x);
            point.x = (_e116 - (floor((_e116 / 1f)) * 1f));
        }
    } else {
        {
            let _e123 = point;
            let _e125 = global.slide2_;
            let _e127 = (_e123.x + _e125.x);
            point.x = (_e127 - (floor((_e127 / 1f)) * 1f));
        }
    }
    let _e136 = point;
    point = (vec2(1f) - abs((vec2(1f) - (2f * _e136))));
    let _e147 = center;
    let _e148 = point;
    r = distance(_e147, _e148);
    let _e151 = point;
    let _e153 = center;
    let _e156 = point;
    let _e158 = center;
    a = atan2((_e151.y - _e153.y), (_e156.x - _e158.x));
    let _e163 = a;
    let _e164 = global.sides;
    let _e165 = (tau / _e164);
    a = (_e163 - (floor((_e163 / _e165)) * _e165));
    let _e170 = a;
    let _e171 = global.sides;
    a = abs((_e170 - ((tau / _e171) / 2f)));
    let _e177 = a;
    let _e178 = global.angle;
    s = sin((_e177 + (tau * _e178)));
    let _e182 = a;
    let _e183 = global.angle;
    c = cos((_e182 + (tau * _e183)));
    let _e187 = global.resolution;
    let _e189 = global.resolution;
    zoom = (_e187.x / _e189.y);
    let _e194 = r;
    let _e195 = c;
    let _e197 = zoom;
    point.x = (((_e194 * _e195) / _e197) + 0.5f);
    let _e202 = r;
    let _e203 = s;
    let _e205 = zoom;
    point.y = (((_e202 * _e203) / _e205) + 0.5f);
    let _e209 = point;
    return _e209;
}

fn main_1() {
    var pat: vec2<f32>;

    let _e27 = pattern();
    pat = _e27;
    let _e29 = pat;
    let _e31 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e29, 0f);
    isf_FragColor = _e31;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e35 = isf_FragColor;
    return FragmentOutput(_e35);
}
