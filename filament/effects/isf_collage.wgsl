// Collage: ISF-Files' Collage.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@seed:seed,0.01,1,0.5,0.001
// --track@cell_size:cell_size,0.01,1,0.125,0.001
// --check@allow_flips_h_on:allow_flips_h,1
// --check@allow_flips_v_on:allow_flips_v,0

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    seed: f32,
    cell_size: f32,
    allow_flips_h_on: i32,
    allow_flips_v_on: i32,
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

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e20 = co_1;
    return fract((sin(dot(_e20.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn main_1() {
    var xy: vec2<f32>;
    var CellWidth: f32;
    var CellHeight: f32;
    var rows: f32;
    var cols: f32;
    var count: f32;
    var region: f32;
    var translated: f32;
    var flipx: f32;
    var flipy: f32;

    let _e18 = global.cell_size;
    if (_e18 == 0f) {
        {
            let _e21 = isf_FragNormCoord_1;
            let _e23 = textureSampleLevel(isf_picture, isf_sampler, _e21, 0f);
            isf_FragColor = _e23;
            return;
        }
    } else {
        {
            let _e28 = isf_FragNormCoord_1.x;
            xy.x = _e28;
            let _e32 = isf_FragNormCoord_1.y;
            xy.y = _e32;
            let _e33 = global.cell_size;
            CellWidth = _e33;
            let _e35 = global.cell_size;
            CellHeight = _e35;
            let _e38 = CellHeight;
            rows = floor((1f / _e38));
            let _e43 = CellWidth;
            cols = floor((1f / _e43));
            let _e47 = rows;
            let _e48 = cols;
            count = floor((_e47 * _e48));
            let _e52 = cols;
            let _e53 = xy;
            let _e55 = CellWidth;
            let _e59 = xy;
            let _e61 = CellHeight;
            region = ((_e52 * floor((_e53.x / _e55))) + floor((_e59.y / _e61)));
            let _e66 = region;
            let _e67 = count;
            let _e69 = global.seed;
            let _e71 = rand(vec2<f32>((_e66 / _e67), _e69));
            translated = clamp(_e71, 0f, 1f);
            let _e76 = count;
            let _e77 = translated;
            translated = floor((_e76 * _e77));
            let _e81 = xy;
            let _e83 = xy;
            let _e85 = CellWidth;
            let _e88 = CellWidth;
            let _e91 = CellWidth;
            let _e92 = translated;
            let _e93 = rows;
            xy.x = ((_e81.x - (floor((_e83.x / _e85)) * _e88)) + (_e91 * floor((_e92 / _e93))));
            let _e99 = xy;
            let _e101 = xy;
            let _e103 = CellHeight;
            let _e106 = CellHeight;
            let _e109 = CellHeight;
            let _e110 = translated;
            let _e111 = cols;
            xy.y = ((_e99.y - (floor((_e101.y / _e103)) * _e106)) + (_e109 * floor((_e110 - (floor((_e110 / _e111)) * _e111)))));
            let _e119 = global.allow_flips_h_on;
            if (_e119 != 0i) {
                {
                    let _e122 = translated;
                    let _e123 = global.seed;
                    let _e125 = rand(vec2<f32>(_e122, _e123));
                    flipx = _e125;
                    let _e127 = flipx;
                    if (_e127 > 0.5f) {
                        {
                            let _e132 = xy;
                            xy.x = (1f - _e132.x);
                        }
                    }
                }
            }
            let _e135 = global.allow_flips_v_on;
            if (_e135 != 0i) {
                {
                    let _e138 = translated;
                    let _e139 = global.seed;
                    let _e141 = rand(vec2<f32>(_e138, _e139));
                    flipy = _e141;
                    let _e143 = flipy;
                    if (_e143 > 0.5f) {
                        {
                            let _e148 = xy;
                            xy.y = (1f - _e148.y);
                        }
                    }
                }
            }
            let _e151 = xy;
            let _e153 = textureSampleLevel(isf_picture, isf_sampler, _e151, 0f);
            isf_FragColor = _e153;
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e25 = isf_FragColor;
    return FragmentOutput(_e25);
}
