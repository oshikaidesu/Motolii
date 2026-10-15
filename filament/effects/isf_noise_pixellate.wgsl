// Noise Pixellate: ISF-Files' Noise Pixellate.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@cell_size:cell_size,0.001,0.5,0.025,0.001
// --track@sigGain:sigGain,0.0,1.0,0.0,0.001
// --track@mode:mode,0,1,1,1
// --track@shape:shape,0,1,0,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    cell_size: f32,
    sigGain: f32,
    mode: i32,
    shape: i32,
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
            let _e39 = global.shape;
            if (_e39 == 0i) {
                {
                    let _e42 = global.cell_size;
                    let _e43 = global.resolution;
                    let _e46 = global.resolution;
                    CellHeight = ((_e42 * _e43.x) / _e46.y);
                }
            }
            let _e49 = xy;
            let _e51 = CellWidth;
            let _e54 = CellWidth;
            x1_ = (floor((_e49.x / _e51)) * _e54);
            let _e57 = xy;
            let _e59 = CellWidth;
            let _e62 = CellWidth;
            x2_ = clamp((ceil((_e57.x / _e59)) * _e62), 0f, 1f);
            let _e68 = xy;
            let _e70 = CellHeight;
            let _e73 = CellHeight;
            y1_ = (floor((_e68.y / _e70)) * _e73);
            let _e76 = xy;
            let _e78 = CellHeight;
            let _e81 = CellHeight;
            y2_ = clamp((ceil((_e76.y / _e78)) * _e81), 0f, 1f);
            let _e87 = isf_FragNormCoord_1;
            let _e89 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e87, 0f);
            original = _e89;
            let _e91 = x1_;
            let _e92 = y1_;
            let _e95 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e91, _e92), 0f);
            let _e96 = x2_;
            let _e97 = y1_;
            let _e100 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e96, _e97), 0f);
            avgX = ((_e95 + _e100) / vec4(2f));
            let _e106 = x1_;
            let _e107 = y1_;
            let _e110 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e106, _e107), 0f);
            let _e111 = x1_;
            let _e112 = y2_;
            let _e115 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e111, _e112), 0f);
            avgY = ((_e110 + _e115) / vec4(2f));
            let _e121 = x1_;
            let _e122 = CellWidth;
            let _e126 = y2_;
            let _e127 = CellHeight;
            let _e133 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>((_e121 + (_e122 / 2f)), (_e126 + (_e127 / 2f))), 0f);
            avgC = _e133;
            let _e135 = avgX;
            let _e136 = avgY;
            let _e138 = avgC;
            avgClr = (((_e135 + _e136) + _e138) / vec4(3f));
            let _e145 = original;
            avgClr.w = _e145.w;
            let _e147 = avgClr;
            let _e149 = avgClr;
            let _e152 = avgClr;
            let _e155 = avgClr;
            thresh = ((((_e147.x + _e149.y) + _e152.z) * _e155.w) / 3f);
            let _e161 = global.mode;
            if (_e161 == 0i) {
                {
                    let _e165 = global.sigGain;
                    let _e167 = xy;
                    let _e168 = thresh;
                    let _e171 = rand((_e167 + vec2(_e168)));
                    rVal = ((1f + _e165) * _e171);
                    let _e174 = rVal;
                    if (_e174 > 1f) {
                        local = 1f;
                    } else {
                        let _e178 = rVal;
                        local = _e178;
                    }
                    let _e180 = local;
                    rVal = _e180;
                    let _e181 = avgClr;
                    let _e183 = avgClr;
                    let _e185 = rVal;
                    let _e186 = (_e183.xyz * _e185);
                    avgClr.x = _e186.x;
                    avgClr.y = _e186.y;
                    avgClr.z = _e186.z;
                }
            } else {
                let _e193 = global.mode;
                if (_e193 == 1i) {
                    {
                        let _e196 = xy;
                        let _e197 = thresh;
                        let _e200 = rand((_e196 + vec2(_e197)));
                        let _e202 = global.sigGain;
                        rVal_1 = (_e200 / (1f + _e202));
                        let _e206 = rVal_1;
                        if (_e206 > 1f) {
                            local_1 = 1f;
                        } else {
                            let _e210 = rVal_1;
                            local_1 = _e210;
                        }
                        let _e212 = local_1;
                        rVal_1 = _e212;
                        let _e213 = thresh;
                        let _e214 = rVal_1;
                        if (_e213 > _e214) {
                            let _e216 = avgClr;
                            local_2 = _e216;
                        } else {
                            let _e220 = original;
                            local_2 = vec4<f32>(0f, 0f, 0f, _e220.w);
                        }
                        let _e224 = local_2;
                        avgClr = _e224;
                    }
                }
            }
            let _e225 = avgClr;
            isf_FragColor = _e225;
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
