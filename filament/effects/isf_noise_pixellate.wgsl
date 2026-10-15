// Noise Pixellate: ISF-Files' Noise Pixellate.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@cell_size:cell_size,0.001,0.5,0.025,0.001
// --track@sigGain:sigGain,0.0,1.0,0.0,0.001
// --track@mode:mode,0,1,1,1
// --track@shape:shape,0,1,0,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    cell_size: f32,
    sigGain: f32,
    mode: i32,
    shape: i32,
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
    let _e22 = center_1;
    let _e24 = pt_1;
    let _e29 = center_1;
    let _e31 = pt_1;
    tmp = (pow((_e22.x - _e24.x), 2f) + pow((_e29.y - _e31.y), 2f));
    let _e38 = tmp;
    return pow(_e38, 0.5f);
}

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
    var x1_: f32;
    var x2_: f32;
    var y1_: f32;
    var y2_: f32;
    var original: vec4<f32>;
    var avgX: vec4<f32>;
    var avgY: vec4<f32>;
    var avgC: vec4<f32>;
    var avgClr: vec4<f32>;
    var thresh: f32;
    var rVal: f32;
    var local: f32;
    var rVal_1: f32;
    var local_1: f32;
    var local_2: vec4<f32>;

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
            let _e37 = global.shape;
            if (_e37 == 0i) {
                {
                    let _e40 = global.cell_size;
                    let _e41 = global.resolution;
                    let _e44 = global.resolution;
                    CellHeight = ((_e40 * _e41.x) / _e44.y);
                }
            }
            let _e47 = xy;
            let _e49 = CellWidth;
            let _e52 = CellWidth;
            x1_ = (floor((_e47.x / _e49)) * _e52);
            let _e55 = xy;
            let _e57 = CellWidth;
            let _e60 = CellWidth;
            x2_ = clamp((ceil((_e55.x / _e57)) * _e60), 0f, 1f);
            let _e66 = xy;
            let _e68 = CellHeight;
            let _e71 = CellHeight;
            y1_ = (floor((_e66.y / _e68)) * _e71);
            let _e74 = xy;
            let _e76 = CellHeight;
            let _e79 = CellHeight;
            y2_ = clamp((ceil((_e74.y / _e76)) * _e79), 0f, 1f);
            let _e85 = isf_FragNormCoord_1;
            let _e87 = textureSampleLevel(isf_picture, isf_sampler, _e85, 0f);
            original = _e87;
            let _e89 = x1_;
            let _e90 = y1_;
            let _e93 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e89, _e90), 0f);
            let _e94 = x2_;
            let _e95 = y1_;
            let _e98 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e94, _e95), 0f);
            avgX = ((_e93 + _e98) / vec4(2f));
            let _e104 = x1_;
            let _e105 = y1_;
            let _e108 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e104, _e105), 0f);
            let _e109 = x1_;
            let _e110 = y2_;
            let _e113 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e109, _e110), 0f);
            avgY = ((_e108 + _e113) / vec4(2f));
            let _e119 = x1_;
            let _e120 = CellWidth;
            let _e124 = y2_;
            let _e125 = CellHeight;
            let _e131 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>((_e119 + (_e120 / 2f)), (_e124 + (_e125 / 2f))), 0f);
            avgC = _e131;
            let _e133 = avgX;
            let _e134 = avgY;
            let _e136 = avgC;
            avgClr = (((_e133 + _e134) + _e136) / vec4(3f));
            let _e143 = original;
            avgClr.w = _e143.w;
            let _e145 = avgClr;
            let _e147 = avgClr;
            let _e150 = avgClr;
            let _e153 = avgClr;
            thresh = ((((_e145.x + _e147.y) + _e150.z) * _e153.w) / 3f);
            let _e159 = global.mode;
            if (_e159 == 0i) {
                {
                    let _e163 = global.sigGain;
                    let _e165 = xy;
                    let _e166 = thresh;
                    let _e169 = rand((_e165 + vec2(_e166)));
                    rVal = ((1f + _e163) * _e169);
                    let _e172 = rVal;
                    if (_e172 > 1f) {
                        local = 1f;
                    } else {
                        let _e176 = rVal;
                        local = _e176;
                    }
                    let _e178 = local;
                    rVal = _e178;
                    let _e179 = avgClr;
                    let _e181 = avgClr;
                    let _e183 = rVal;
                    let _e184 = (_e181.xyz * _e183);
                    avgClr.x = _e184.x;
                    avgClr.y = _e184.y;
                    avgClr.z = _e184.z;
                }
            } else {
                let _e191 = global.mode;
                if (_e191 == 1i) {
                    {
                        let _e194 = xy;
                        let _e195 = thresh;
                        let _e198 = rand((_e194 + vec2(_e195)));
                        let _e200 = global.sigGain;
                        rVal_1 = (_e198 / (1f + _e200));
                        let _e204 = rVal_1;
                        if (_e204 > 1f) {
                            local_1 = 1f;
                        } else {
                            let _e208 = rVal_1;
                            local_1 = _e208;
                        }
                        let _e210 = local_1;
                        rVal_1 = _e210;
                        let _e211 = thresh;
                        let _e212 = rVal_1;
                        if (_e211 > _e212) {
                            let _e214 = avgClr;
                            local_2 = _e214;
                        } else {
                            let _e218 = original;
                            local_2 = vec4<f32>(0f, 0f, 0f, _e218.w);
                        }
                        let _e222 = local_2;
                        avgClr = _e222;
                    }
                }
            }
            let _e223 = avgClr;
            isf_FragColor = _e223;
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
