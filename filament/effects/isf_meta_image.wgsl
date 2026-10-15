// Meta Image: ISF-Files' Meta Image.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by Toneburst), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@cell_size:cell_size,0.001,1.0,0.125,0.001
// --track@zoom_tile:zoom_tile,0.0,2.0,1.0,0.001
// --track@mixAmt:mix,0.0,1.0,0.5,0.001
// --track@mode:mode,0,1,0,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    cell_size: f32,
    zoom_tile: f32,
    mixAmt: f32,
    mode: i32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
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

    let _e21 = gl_FragCoord_1;
    let _e23 = global.resolution;
    xy = (_e21.xy / _e23.xy);
    let _e27 = global.cell_size;
    CellWidth = _e27;
    let _e29 = global.cell_size;
    CellHeight = _e29;
    let _e31 = xy;
    let _e33 = CellWidth;
    let _e36 = CellWidth;
    x1_ = (floor((_e31.x / _e33)) * _e36);
    let _e39 = xy;
    let _e41 = CellWidth;
    let _e44 = CellWidth;
    x2_ = clamp((ceil((_e39.x / _e41)) * _e44), 0f, 1f);
    let _e50 = xy;
    let _e52 = CellHeight;
    let _e55 = CellHeight;
    y1_ = (floor((_e50.y / _e52)) * _e55);
    let _e58 = xy;
    let _e60 = CellHeight;
    let _e63 = CellHeight;
    y2_ = clamp((ceil((_e58.y / _e60)) * _e63), 0f, 1f);
    let _e69 = x1_;
    let _e70 = y1_;
    let _e73 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e69, _e70), 0f);
    let _e74 = x2_;
    let _e75 = y1_;
    let _e78 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e74, _e75), 0f);
    avgX = ((_e73 + _e78) / vec4(2f));
    let _e84 = x1_;
    let _e85 = y1_;
    let _e88 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e84, _e85), 0f);
    let _e89 = x1_;
    let _e90 = y2_;
    let _e93 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e89, _e90), 0f);
    avgY = ((_e88 + _e93) / vec4(2f));
    let _e99 = x1_;
    let _e100 = CellWidth;
    let _e104 = y2_;
    let _e105 = CellHeight;
    let _e111 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>((_e99 + (_e100 / 2f)), (_e104 + (_e105 / 2f))), 0f);
    avgC = _e111;
    let _e113 = avgX;
    let _e114 = avgY;
    let _e116 = avgC;
    avgClr = (((_e113 + _e114) + _e116) / vec4(3f));
    let _e122 = xy;
    let _e124 = x1_;
    let _e126 = CellWidth;
    cellPosX = ((_e122.x - _e124) / _e126);
    let _e129 = xy;
    let _e131 = y1_;
    let _e133 = CellHeight;
    cellPosY = ((_e129.y - _e131) / _e133);
    let _e136 = cellPosX;
    let _e137 = cellPosY;
    loc = vec2<f32>(_e136, _e137);
    let _e144 = loc;
    let _e146 = modifiedCenter;
    let _e150 = global.zoom_tile;
    let _e153 = modifiedCenter;
    loc.x = (((_e144.x - _e146.x) * (1f / _e150)) + _e153.x);
    let _e157 = loc;
    let _e159 = modifiedCenter;
    let _e163 = global.zoom_tile;
    let _e166 = modifiedCenter;
    loc.y = (((_e157.y - _e159.y) * (1f / _e163)) + _e166.y);
    let _e170 = loc;
    let _e174 = loc;
    let _e179 = loc;
    let _e184 = loc;
    if ((((_e170.x < 0f) || (_e174.y < 0f)) || (_e179.x > 1f)) || (_e184.y > 1f)) {
        {
            littlePix = vec4(0f);
        }
    } else {
        {
            let _e191 = loc;
            let _e193 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e191, 0f);
            littlePix = _e193;
        }
    }
    let _e194 = global.mode;
    if (_e194 == 0i) {
        {
            let _e197 = littlePix;
            let _e198 = avgClr;
            isf_FragColor = vec4<f32>((_e197 * _e198));
            return;
        }
    } else {
        {
            let _e201 = littlePix;
            let _e202 = avgClr;
            let _e203 = global.mixAmt;
            isf_FragColor = vec4<f32>(mix(_e201, _e202, vec4(_e203)));
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e29 = isf_FragColor;
    return FragmentOutput(_e29);
}
