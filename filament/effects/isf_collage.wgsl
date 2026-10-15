// Collage: ISF-Files' Collage.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@seed:seed,0.01,1,0.5,0.001
// --track@cell_size:cell_size,0.01,1,0.125,0.001
// --check@allow_flips_h_on:allow_flips_h,1
// --check@allow_flips_v_on:allow_flips_v,0

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    seed: f32,
    cell_size: f32,
    allow_flips_h_on: i32,
    allow_flips_v_on: i32,
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

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e22 = co_1;
    return fract((sin(dot(_e22.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
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

    let _e20 = global.cell_size;
    if (_e20 == 0f) {
        {
            let _e23 = isf_FragNormCoord_1;
            let _e25 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e23, 0f);
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
            CellWidth = _e35;
            let _e37 = global.cell_size;
            CellHeight = _e37;
            let _e40 = CellHeight;
            rows = floor((1f / _e40));
            let _e45 = CellWidth;
            cols = floor((1f / _e45));
            let _e49 = rows;
            let _e50 = cols;
            count = floor((_e49 * _e50));
            let _e54 = cols;
            let _e55 = xy;
            let _e57 = CellWidth;
            let _e61 = xy;
            let _e63 = CellHeight;
            region = ((_e54 * floor((_e55.x / _e57))) + floor((_e61.y / _e63)));
            let _e68 = region;
            let _e69 = count;
            let _e71 = global.seed;
            let _e73 = rand(vec2<f32>((_e68 / _e69), _e71));
            translated = clamp(_e73, 0f, 1f);
            let _e78 = count;
            let _e79 = translated;
            translated = floor((_e78 * _e79));
            let _e83 = xy;
            let _e85 = xy;
            let _e87 = CellWidth;
            let _e90 = CellWidth;
            let _e93 = CellWidth;
            let _e94 = translated;
            let _e95 = rows;
            xy.x = ((_e83.x - (floor((_e85.x / _e87)) * _e90)) + (_e93 * floor((_e94 / _e95))));
            let _e101 = xy;
            let _e103 = xy;
            let _e105 = CellHeight;
            let _e108 = CellHeight;
            let _e111 = CellHeight;
            let _e112 = translated;
            let _e113 = cols;
            xy.y = ((_e101.y - (floor((_e103.y / _e105)) * _e108)) + (_e111 * floor((_e112 - (floor((_e112 / _e113)) * _e113)))));
            let _e121 = global.allow_flips_h_on;
            if (_e121 != 0i) {
                {
                    let _e124 = translated;
                    let _e125 = global.seed;
                    let _e127 = rand(vec2<f32>(_e124, _e125));
                    flipx = _e127;
                    let _e129 = flipx;
                    if (_e129 > 0.5f) {
                        {
                            let _e134 = xy;
                            xy.x = (1f - _e134.x);
                        }
                    }
                }
            }
            let _e137 = global.allow_flips_v_on;
            if (_e137 != 0i) {
                {
                    let _e140 = translated;
                    let _e141 = global.seed;
                    let _e143 = rand(vec2<f32>(_e140, _e141));
                    flipy = _e143;
                    let _e145 = flipy;
                    if (_e145 > 0.5f) {
                        {
                            let _e150 = xy;
                            xy.y = (1f - _e150.y);
                        }
                    }
                }
            }
            let _e153 = xy;
            let _e155 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e153, 0f);
            isf_FragColor = _e155;
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
