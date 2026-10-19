// Multi-Pixellate: ISF-Files' Multi-Pixellate.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --alpha (on a thing: its cells are its shape, each as clear as what it gathered)
// --track@cell_size:cell_size,0.001,0.5,0.125,0.001
// --track@min_cell_size:min_cell_size,0.001,0.5,0.05000000074505806,0.001
// --track@rSeed:rSeed,0,1,0.25,0.001
// --track@shape:shape,0,1,0,1
// --track@round_to_divisions:round_to_divisions,0,1,2,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    cell_size: f32,
    min_cell_size: f32,
    rSeed: f32,
    shape: i32,
    round_to_divisions: i32,
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

fn distance_(center: vec2<f32>, pt: vec2<f32>) -> f32 {
    var center_1: vec2<f32>;
    var pt_1: vec2<f32>;
    var tmp: f32;

    center_1 = center;
    pt_1 = pt;
    let _e26 = center_1;
    let _e28 = pt_1;
    let _e33 = center_1;
    let _e35 = pt_1;
    tmp = (pow((_e26.x - _e28.x), 2f) + pow((_e33.y - _e35.y), 2f));
    let _e42 = tmp;
    return pow(_e42, 0.5f);
}

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e24 = co_1;
    return fract((sin(dot(_e24.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn main_1() {
    var xy: vec2<f32>;
    var max_cell_size: f32;
    var CellWidth: f32;
    var CellHeight: f32;
    var x1_: f32;
    var y1_: f32;
    var newCellSize: f32;
    var local: f32;
    var maxCount: f32;
    var val: f32;
    var x2_: f32;
    var y2_: f32;
    var avgX: vec4<f32>;
    var avgY: vec4<f32>;
    var avgC: vec4<f32>;
    var avgClr: vec4<f32>;

    let _e22 = global.cell_size;
    if (_e22 == 0f) {
        {
            let _e25 = isf_FragNormCoord_1;
            let _e27 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e25, 0f);
            isf_FragColor = _e27;
            return;
        }
    } else {
        {
            let _e32 = isf_FragNormCoord_1.x;
            xy.x = _e32;
            let _e36 = isf_FragNormCoord_1.y;
            xy.y = _e36;
            let _e37 = global.cell_size;
            let _e38 = global.min_cell_size;
            max_cell_size = (_e37 + _e38);
            let _e41 = max_cell_size;
            CellWidth = _e41;
            let _e43 = max_cell_size;
            CellHeight = _e43;
            let _e45 = global.shape;
            if (_e45 == 0i) {
                {
                    let _e48 = max_cell_size;
                    let _e49 = global.resolution;
                    let _e52 = global.resolution;
                    CellHeight = ((_e48 * _e49.x) / _e52.y);
                }
            }
            let _e55 = xy;
            let _e57 = CellWidth;
            let _e60 = CellWidth;
            x1_ = (floor((_e55.x / _e57)) * _e60);
            let _e63 = xy;
            let _e65 = CellHeight;
            let _e68 = CellHeight;
            y1_ = (floor((_e63.y / _e65)) * _e68);
            let _e71 = max_cell_size;
            newCellSize = _e71;
            let _e73 = global.round_to_divisions;
            if (_e73 > 0i) {
                {
                    let _e76 = global.min_cell_size;
                    if (_e76 > 0f) {
                        let _e79 = max_cell_size;
                        let _e80 = global.min_cell_size;
                        local = (_e79 / _e80);
                    } else {
                        local = 10f;
                    }
                    let _e84 = local;
                    maxCount = _e84;
                    let _e87 = maxCount;
                    let _e91 = global.rSeed;
                    let _e93 = x1_;
                    let _e94 = y1_;
                    let _e98 = rand((vec2((0.1f + _e91)) + vec2<f32>(_e93, _e94)));
                    val = floor((1f + ((_e87 - 1f) * _e98)));
                    let _e103 = newCellSize;
                    let _e104 = global.round_to_divisions;
                    let _e106 = val;
                    newCellSize = (_e103 / pow(f32(_e104), _e106));
                }
            } else {
                {
                    let _e109 = global.min_cell_size;
                    let _e110 = global.cell_size;
                    let _e111 = global.min_cell_size;
                    let _e114 = global.rSeed;
                    let _e116 = x1_;
                    let _e117 = y1_;
                    let _e121 = rand((vec2((0.1f + _e114)) + vec2<f32>(_e116, _e117)));
                    newCellSize = (_e109 + ((_e110 - _e111) * _e121));
                }
            }
            let _e124 = newCellSize;
            CellWidth = _e124;
            let _e125 = newCellSize;
            CellHeight = _e125;
            let _e126 = global.shape;
            if (_e126 == 0i) {
                {
                    let _e129 = newCellSize;
                    let _e130 = global.resolution;
                    let _e133 = global.resolution;
                    CellHeight = ((_e129 * _e130.x) / _e133.y);
                }
            }
            let _e136 = xy;
            let _e138 = CellWidth;
            let _e141 = CellWidth;
            x1_ = (floor((_e136.x / _e138)) * _e141);
            let _e143 = xy;
            let _e145 = CellWidth;
            let _e148 = CellWidth;
            x2_ = clamp((ceil((_e143.x / _e145)) * _e148), 0f, 1f);
            let _e154 = xy;
            let _e156 = CellHeight;
            let _e159 = CellHeight;
            y1_ = (floor((_e154.y / _e156)) * _e159);
            let _e161 = xy;
            let _e163 = CellHeight;
            let _e166 = CellHeight;
            y2_ = clamp((ceil((_e161.y / _e163)) * _e166), 0f, 1f);
            let _e172 = x1_;
            let _e173 = y1_;
            let _e176 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e172, _e173), 0f);
            let _e177 = x2_;
            let _e178 = y1_;
            let _e181 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e177, _e178), 0f);
            avgX = ((_e176 + _e181) / vec4(2f));
            let _e187 = x1_;
            let _e188 = y1_;
            let _e191 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e187, _e188), 0f);
            let _e192 = x1_;
            let _e193 = y2_;
            let _e196 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e192, _e193), 0f);
            avgY = ((_e191 + _e196) / vec4(2f));
            let _e202 = x1_;
            let _e203 = CellWidth;
            let _e207 = y2_;
            let _e208 = CellHeight;
            let _e214 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>((_e202 + (_e203 / 2f)), (_e207 + (_e208 / 2f))), 0f);
            avgC = _e214;
            let _e216 = avgX;
            let _e217 = avgY;
            let _e219 = avgC;
            avgClr = (((_e216 + _e217) + _e219) / vec4(3f));
            let _e225 = avgClr;
            isf_FragColor = vec4<f32>(_e225);
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e29 = isf_FragColor;
    return FragmentOutput(_e29);
}
