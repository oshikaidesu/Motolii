// Multi-Pixellate: ISF-Files' Multi-Pixellate.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@cell_size:cell_size,0.001,0.5,0.125,0.001
// --track@min_cell_size:min_cell_size,0.001,0.5,0.05000000074505806,0.001
// --track@rSeed:rSeed,0,1,0.25,0.001
// --track@shape:shape,0,1,0,1
// --track@round_to_divisions:round_to_divisions,0,1,2,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
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
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
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
    let _e24 = center_1;
    let _e26 = pt_1;
    let _e31 = center_1;
    let _e33 = pt_1;
    tmp = (pow((_e24.x - _e26.x), 2f) + pow((_e31.y - _e33.y), 2f));
    let _e40 = tmp;
    return pow(_e40, 0.5f);
}

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e22 = co_1;
    return fract((sin(dot(_e22.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
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

    let _e20 = global.cell_size;
    if (_e20 == 0f) {
        {
            let _e23 = isf_FragNormCoord_1;
            let _e25 = textureSampleLevel(isf_picture, isf_sampler, _e23, 0f);
            isf_FragColor = _e25;
            return;
        }
    } else {
        {
            let _e30 = isf_FragNormCoord_1.x;
            xy.x = _e30;
            let _e34 = isf_FragNormCoord_1.y;
            xy.y = _e34;
            let _e35 = global.cell_size;
            let _e36 = global.min_cell_size;
            max_cell_size = (_e35 + _e36);
            let _e39 = max_cell_size;
            CellWidth = _e39;
            let _e41 = max_cell_size;
            CellHeight = _e41;
            let _e43 = global.shape;
            if (_e43 == 0i) {
                {
                    let _e46 = max_cell_size;
                    let _e47 = global.resolution;
                    let _e50 = global.resolution;
                    CellHeight = ((_e46 * _e47.x) / _e50.y);
                }
            }
            let _e53 = xy;
            let _e55 = CellWidth;
            let _e58 = CellWidth;
            x1_ = (floor((_e53.x / _e55)) * _e58);
            let _e61 = xy;
            let _e63 = CellHeight;
            let _e66 = CellHeight;
            y1_ = (floor((_e61.y / _e63)) * _e66);
            let _e69 = max_cell_size;
            newCellSize = _e69;
            let _e71 = global.round_to_divisions;
            if (_e71 > 0i) {
                {
                    let _e74 = global.min_cell_size;
                    if (_e74 > 0f) {
                        let _e77 = max_cell_size;
                        let _e78 = global.min_cell_size;
                        local = (_e77 / _e78);
                    } else {
                        local = 10f;
                    }
                    let _e82 = local;
                    maxCount = _e82;
                    let _e85 = maxCount;
                    let _e89 = global.rSeed;
                    let _e91 = x1_;
                    let _e92 = y1_;
                    let _e96 = rand((vec2((0.1f + _e89)) + vec2<f32>(_e91, _e92)));
                    val = floor((1f + ((_e85 - 1f) * _e96)));
                    let _e101 = newCellSize;
                    let _e102 = global.round_to_divisions;
                    let _e104 = val;
                    newCellSize = (_e101 / pow(f32(_e102), _e104));
                }
            } else {
                {
                    let _e107 = global.min_cell_size;
                    let _e108 = global.cell_size;
                    let _e109 = global.min_cell_size;
                    let _e112 = global.rSeed;
                    let _e114 = x1_;
                    let _e115 = y1_;
                    let _e119 = rand((vec2((0.1f + _e112)) + vec2<f32>(_e114, _e115)));
                    newCellSize = (_e107 + ((_e108 - _e109) * _e119));
                }
            }
            let _e122 = newCellSize;
            CellWidth = _e122;
            let _e123 = newCellSize;
            CellHeight = _e123;
            let _e124 = global.shape;
            if (_e124 == 0i) {
                {
                    let _e127 = newCellSize;
                    let _e128 = global.resolution;
                    let _e131 = global.resolution;
                    CellHeight = ((_e127 * _e128.x) / _e131.y);
                }
            }
            let _e134 = xy;
            let _e136 = CellWidth;
            let _e139 = CellWidth;
            x1_ = (floor((_e134.x / _e136)) * _e139);
            let _e141 = xy;
            let _e143 = CellWidth;
            let _e146 = CellWidth;
            x2_ = clamp((ceil((_e141.x / _e143)) * _e146), 0f, 1f);
            let _e152 = xy;
            let _e154 = CellHeight;
            let _e157 = CellHeight;
            y1_ = (floor((_e152.y / _e154)) * _e157);
            let _e159 = xy;
            let _e161 = CellHeight;
            let _e164 = CellHeight;
            y2_ = clamp((ceil((_e159.y / _e161)) * _e164), 0f, 1f);
            let _e170 = x1_;
            let _e171 = y1_;
            let _e174 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e170, _e171), 0f);
            let _e175 = x2_;
            let _e176 = y1_;
            let _e179 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e175, _e176), 0f);
            avgX = ((_e174 + _e179) / vec4(2f));
            let _e185 = x1_;
            let _e186 = y1_;
            let _e189 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e185, _e186), 0f);
            let _e190 = x1_;
            let _e191 = y2_;
            let _e194 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e190, _e191), 0f);
            avgY = ((_e189 + _e194) / vec4(2f));
            let _e200 = x1_;
            let _e201 = CellWidth;
            let _e205 = y2_;
            let _e206 = CellHeight;
            let _e212 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>((_e200 + (_e201 / 2f)), (_e205 + (_e206 / 2f))), 0f);
            avgC = _e212;
            let _e214 = avgX;
            let _e215 = avgY;
            let _e217 = avgC;
            avgClr = (((_e214 + _e215) + _e217) / vec4(3f));
            let _e223 = avgClr;
            isf_FragColor = vec4<f32>(_e223);
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e27 = isf_FragColor;
    return FragmentOutput(_e27);
}
