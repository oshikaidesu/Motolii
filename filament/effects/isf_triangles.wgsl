// Triangles: ISF-Files' Triangles.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Stylize
// --alpha (on a thing: its cells are its shape, each as clear as what it gathered)
// --track@cell_size:cell_size,0.001,0.5,0.125,0.001
// --track@style:style,0,1,0,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    cell_size: f32,
    style: i32,
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
    let _e20 = center_1;
    let _e22 = pt_1;
    let _e27 = center_1;
    let _e29 = pt_1;
    tmp = (pow((_e20.x - _e22.x), 2f) + pow((_e27.y - _e29.y), 2f));
    let _e36 = tmp;
    return pow(_e36, 0.5f);
}

fn main_1() {
    var xy: vec2<f32>;
    var CellWidth: f32;
    var CellHeight: f32;
    var x1_: f32;
    var x2_: f32;
    var y1_: f32;
    var y2_: f32;
    var x: f32;
    var y: f32;
    var avgClr: vec4<f32> = vec4(0f);
    var avgL: vec4<f32>;
    var avgR: vec4<f32>;
    var avgC: vec4<f32>;
    var avgR_1: vec4<f32>;
    var avgL_1: vec4<f32>;
    var avgC_1: vec4<f32>;
    var avgL_2: vec4<f32>;
    var avgR_2: vec4<f32>;
    var avgC_2: vec4<f32>;
    var avgL_3: vec4<f32>;
    var avgR_3: vec4<f32>;
    var avgC_3: vec4<f32>;
    var avgL_4: vec4<f32>;
    var avgR_4: vec4<f32>;
    var avgC_4: vec4<f32>;
    var avgL_5: vec4<f32>;
    var avgR_5: vec4<f32>;
    var avgC_5: vec4<f32>;

    let _e16 = global.cell_size;
    if (_e16 == 0f) {
        {
            let _e19 = isf_FragNormCoord_1;
            let _e21 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e19, 0f);
            isf_FragColor = _e21;
            return;
        }
    } else {
        {
            let _e26 = isf_FragNormCoord_1.x;
            xy.x = _e26;
            let _e30 = isf_FragNormCoord_1.y;
            xy.y = _e30;
            let _e31 = global.cell_size;
            CellWidth = _e31;
            let _e33 = global.cell_size;
            CellHeight = _e33;
            let _e35 = global.cell_size;
            let _e36 = global.resolution;
            let _e39 = global.resolution;
            CellHeight = ((_e35 * _e36.x) / _e39.y);
            let _e42 = xy;
            let _e44 = CellWidth;
            let _e47 = CellWidth;
            x1_ = (floor((_e42.x / _e44)) * _e47);
            let _e50 = xy;
            let _e52 = CellWidth;
            let _e55 = CellWidth;
            x2_ = clamp((ceil((_e50.x / _e52)) * _e55), 0f, 1f);
            let _e61 = xy;
            let _e63 = CellHeight;
            let _e66 = CellHeight;
            y1_ = (floor((_e61.y / _e63)) * _e66);
            let _e69 = xy;
            let _e71 = CellHeight;
            let _e74 = CellHeight;
            y2_ = clamp((ceil((_e69.y / _e71)) * _e74), 0f, 1f);
            let _e80 = xy;
            let _e82 = x1_;
            let _e84 = CellWidth;
            x = ((_e80.x - _e82) / _e84);
            let _e87 = xy;
            let _e89 = y1_;
            let _e91 = CellHeight;
            y = ((_e87.y - _e89) / _e91);
            let _e97 = global.style;
            if (_e97 == 0i) {
                {
                    let _e100 = x;
                    let _e101 = y;
                    if (_e100 < _e101) {
                        {
                            let _e103 = x1_;
                            let _e104 = y1_;
                            let _e107 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e103, _e104), 0f);
                            let _e108 = x1_;
                            let _e109 = y2_;
                            let _e112 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e108, _e109), 0f);
                            avgL = ((_e107 + _e112) / vec4(2f));
                            let _e118 = x2_;
                            let _e119 = y2_;
                            let _e122 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e118, _e119), 0f);
                            avgR = _e122;
                            let _e124 = x1_;
                            let _e125 = CellWidth;
                            let _e129 = y2_;
                            let _e130 = CellHeight;
                            let _e136 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>((_e124 + (_e125 / 2f)), (_e129 + (_e130 / 2f))), 0f);
                            avgC = _e136;
                            let _e138 = avgL;
                            let _e139 = avgR;
                            let _e141 = avgC;
                            avgClr = (((_e138 + _e139) + _e141) / vec4(3f));
                        }
                    } else {
                        {
                            let _e146 = x2_;
                            let _e147 = y1_;
                            let _e150 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e146, _e147), 0f);
                            let _e151 = x2_;
                            let _e152 = y2_;
                            let _e155 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e151, _e152), 0f);
                            avgR_1 = ((_e150 + _e155) / vec4(2f));
                            let _e161 = x1_;
                            let _e162 = y1_;
                            let _e165 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e161, _e162), 0f);
                            avgL_1 = _e165;
                            let _e167 = x1_;
                            let _e168 = CellWidth;
                            let _e172 = y2_;
                            let _e173 = CellHeight;
                            let _e179 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>((_e167 + (_e168 / 2f)), (_e172 + (_e173 / 2f))), 0f);
                            avgC_1 = _e179;
                            let _e181 = avgL_1;
                            let _e182 = avgR_1;
                            let _e184 = avgC_1;
                            avgClr = (((_e181 + _e182) + _e184) / vec4(3f));
                        }
                    }
                }
            } else {
                {
                    let _e189 = x;
                    let _e190 = y;
                    let _e192 = x;
                    let _e194 = y;
                    if ((_e189 > _e190) && (_e192 < (1f - _e194))) {
                        {
                            let _e198 = x1_;
                            let _e199 = y1_;
                            let _e202 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e198, _e199), 0f);
                            avgL_2 = _e202;
                            let _e204 = x2_;
                            let _e205 = y1_;
                            let _e208 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e204, _e205), 0f);
                            avgR_2 = _e208;
                            let _e210 = x1_;
                            let _e211 = CellWidth;
                            let _e215 = y2_;
                            let _e216 = CellHeight;
                            let _e222 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>((_e210 + (_e211 / 2f)), (_e215 + (_e216 / 2f))), 0f);
                            avgC_2 = _e222;
                            let _e224 = avgL_2;
                            let _e225 = avgR_2;
                            let _e227 = avgC_2;
                            avgClr = (((_e224 + _e225) + _e227) / vec4(3f));
                        }
                    } else {
                        let _e232 = x;
                        let _e233 = y;
                        let _e235 = x;
                        let _e237 = y;
                        if ((_e232 < _e233) && (_e235 < (1f - _e237))) {
                            {
                                let _e241 = x1_;
                                let _e242 = y1_;
                                let _e245 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e241, _e242), 0f);
                                avgL_3 = _e245;
                                let _e247 = x1_;
                                let _e248 = y2_;
                                let _e251 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e247, _e248), 0f);
                                avgR_3 = _e251;
                                let _e253 = x1_;
                                let _e254 = CellWidth;
                                let _e258 = y2_;
                                let _e259 = CellHeight;
                                let _e265 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>((_e253 + (_e254 / 2f)), (_e258 + (_e259 / 2f))), 0f);
                                avgC_3 = _e265;
                                let _e267 = avgL_3;
                                let _e268 = avgR_3;
                                let _e270 = avgC_3;
                                avgClr = (((_e267 + _e268) + _e270) / vec4(3f));
                            }
                        } else {
                            let _e275 = x;
                            let _e277 = y;
                            let _e280 = x;
                            let _e281 = y;
                            if ((_e275 > (1f - _e277)) && (_e280 < _e281)) {
                                {
                                    let _e284 = x1_;
                                    let _e285 = y2_;
                                    let _e288 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e284, _e285), 0f);
                                    avgL_4 = _e288;
                                    let _e290 = x2_;
                                    let _e291 = y2_;
                                    let _e294 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e290, _e291), 0f);
                                    avgR_4 = _e294;
                                    let _e296 = x1_;
                                    let _e297 = CellWidth;
                                    let _e301 = y2_;
                                    let _e302 = CellHeight;
                                    let _e308 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>((_e296 + (_e297 / 2f)), (_e301 + (_e302 / 2f))), 0f);
                                    avgC_4 = _e308;
                                    let _e310 = avgL_4;
                                    let _e311 = avgR_4;
                                    let _e313 = avgC_4;
                                    avgClr = (((_e310 + _e311) + _e313) / vec4(3f));
                                }
                            } else {
                                {
                                    let _e318 = x2_;
                                    let _e319 = y1_;
                                    let _e322 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e318, _e319), 0f);
                                    avgL_5 = _e322;
                                    let _e324 = x2_;
                                    let _e325 = y2_;
                                    let _e328 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e324, _e325), 0f);
                                    avgR_5 = _e328;
                                    let _e330 = x1_;
                                    let _e331 = CellWidth;
                                    let _e335 = y2_;
                                    let _e336 = CellHeight;
                                    let _e342 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>((_e330 + (_e331 / 2f)), (_e335 + (_e336 / 2f))), 0f);
                                    avgC_5 = _e342;
                                    let _e344 = avgL_5;
                                    let _e345 = avgR_5;
                                    let _e347 = avgC_5;
                                    avgClr = (((_e344 + _e345) + _e347) / vec4(3f));
                                }
                            }
                        }
                    }
                }
            }
            let _e352 = avgClr;
            isf_FragColor = _e352;
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e23 = isf_FragColor;
    return FragmentOutput(_e23);
}
