// Meta Image: ISF-Files' Meta Image.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by Toneburst), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@cell_size:cell_size,0.001,1.0,0.125,0.001
// --track@zoom_tile:zoom_tile,0.0,2.0,1.0,0.001
// --track@mixAmt:mix,0.0,1.0,0.5,0.001
// --track@mode:mode,0,1,0,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    cell_size: f32,
    zoom_tile: f32,
    mixAmt: f32,
    mode: i32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;

fn main_1() {
    var xy: vec2<f32>;
    var CellWidth: f32;
    var CellHeight: f32;
    var x1_: f32;
    var x2_: f32;
    var y1_: f32;
    var y2_: f32;
    var avgX: vec4<f32>;
    var avgY: vec4<f32>;
    var avgC: vec4<f32>;
    var avgClr: vec4<f32>;
    var cellPosX: f32;
    var cellPosY: f32;
    var loc: vec2<f32>;
    var modifiedCenter: vec2<f32> = vec2(0.5f);
    var littlePix: vec4<f32>;

    let _e19 = gl_FragCoord_1;
    let _e21 = global.resolution;
    xy = (_e19.xy / _e21.xy);
    let _e25 = global.cell_size;
    CellWidth = _e25;
    let _e27 = global.cell_size;
    CellHeight = _e27;
    let _e29 = xy;
    let _e31 = CellWidth;
    let _e34 = CellWidth;
    x1_ = (floor((_e29.x / _e31)) * _e34);
    let _e37 = xy;
    let _e39 = CellWidth;
    let _e42 = CellWidth;
    x2_ = clamp((ceil((_e37.x / _e39)) * _e42), 0f, 1f);
    let _e48 = xy;
    let _e50 = CellHeight;
    let _e53 = CellHeight;
    y1_ = (floor((_e48.y / _e50)) * _e53);
    let _e56 = xy;
    let _e58 = CellHeight;
    let _e61 = CellHeight;
    y2_ = clamp((ceil((_e56.y / _e58)) * _e61), 0f, 1f);
    let _e67 = x1_;
    let _e68 = y1_;
    let _e71 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e67, _e68), 0f);
    let _e72 = x2_;
    let _e73 = y1_;
    let _e76 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e72, _e73), 0f);
    avgX = ((_e71 + _e76) / vec4(2f));
    let _e82 = x1_;
    let _e83 = y1_;
    let _e86 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e82, _e83), 0f);
    let _e87 = x1_;
    let _e88 = y2_;
    let _e91 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e87, _e88), 0f);
    avgY = ((_e86 + _e91) / vec4(2f));
    let _e97 = x1_;
    let _e98 = CellWidth;
    let _e102 = y2_;
    let _e103 = CellHeight;
    let _e109 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>((_e97 + (_e98 / 2f)), (_e102 + (_e103 / 2f))), 0f);
    avgC = _e109;
    let _e111 = avgX;
    let _e112 = avgY;
    let _e114 = avgC;
    avgClr = (((_e111 + _e112) + _e114) / vec4(3f));
    let _e120 = xy;
    let _e122 = x1_;
    let _e124 = CellWidth;
    cellPosX = ((_e120.x - _e122) / _e124);
    let _e127 = xy;
    let _e129 = y1_;
    let _e131 = CellHeight;
    cellPosY = ((_e127.y - _e129) / _e131);
    let _e134 = cellPosX;
    let _e135 = cellPosY;
    loc = vec2<f32>(_e134, _e135);
    let _e142 = loc;
    let _e144 = modifiedCenter;
    let _e148 = global.zoom_tile;
    let _e151 = modifiedCenter;
    loc.x = (((_e142.x - _e144.x) * (1f / _e148)) + _e151.x);
    let _e155 = loc;
    let _e157 = modifiedCenter;
    let _e161 = global.zoom_tile;
    let _e164 = modifiedCenter;
    loc.y = (((_e155.y - _e157.y) * (1f / _e161)) + _e164.y);
    let _e168 = loc;
    let _e172 = loc;
    let _e177 = loc;
    let _e182 = loc;
    if ((((_e168.x < 0f) || (_e172.y < 0f)) || (_e177.x > 1f)) || (_e182.y > 1f)) {
        {
            littlePix = vec4(0f);
        }
    } else {
        {
            let _e189 = loc;
            let _e191 = textureSampleLevel(isf_picture, isf_sampler, _e189, 0f);
            littlePix = _e191;
        }
    }
    let _e192 = global.mode;
    if (_e192 == 0i) {
        {
            let _e195 = littlePix;
            let _e196 = avgClr;
            isf_FragColor = vec4<f32>((_e195 * _e196));
            return;
        }
    } else {
        {
            let _e199 = littlePix;
            let _e200 = avgClr;
            let _e201 = global.mixAmt;
            isf_FragColor = vec4<f32>(mix(_e199, _e200, vec4(_e201)));
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e27 = isf_FragColor;
    return FragmentOutput(_e27);
}
