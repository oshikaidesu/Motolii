// Triangles: ISF-Files' Triangles.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@cell_size:cell_size,0.001,0.5,0.125,0.001
// --track@style:style,0,1,0,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    cell_size: f32,
    style: i32,
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
    let _e18 = center_1;
    let _e20 = pt_1;
    let _e25 = center_1;
    let _e27 = pt_1;
    tmp = (pow((_e18.x - _e20.x), 2f) + pow((_e25.y - _e27.y), 2f));
    let _e34 = tmp;
    return pow(_e34, 0.5f);
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

    let _e14 = global.cell_size;
    if (_e14 == 0f) {
        {
            let _e17 = isf_FragNormCoord_1;
            let _e19 = textureSampleLevel(isf_picture, isf_sampler, _e17, 0f);
            isf_FragColor = _e19;
            return;
        }
    } else {
        {
            let _e24 = isf_FragNormCoord_1.x;
            xy.x = _e24;
            let _e28 = isf_FragNormCoord_1.y;
            xy.y = _e28;
            let _e29 = global.cell_size;
            CellWidth = _e29;
            let _e31 = global.cell_size;
            CellHeight = _e31;
            let _e33 = global.cell_size;
            let _e34 = global.resolution;
            let _e37 = global.resolution;
            CellHeight = ((_e33 * _e34.x) / _e37.y);
            let _e40 = xy;
            let _e42 = CellWidth;
            let _e45 = CellWidth;
            x1_ = (floor((_e40.x / _e42)) * _e45);
            let _e48 = xy;
            let _e50 = CellWidth;
            let _e53 = CellWidth;
            x2_ = clamp((ceil((_e48.x / _e50)) * _e53), 0f, 1f);
            let _e59 = xy;
            let _e61 = CellHeight;
            let _e64 = CellHeight;
            y1_ = (floor((_e59.y / _e61)) * _e64);
            let _e67 = xy;
            let _e69 = CellHeight;
            let _e72 = CellHeight;
            y2_ = clamp((ceil((_e67.y / _e69)) * _e72), 0f, 1f);
            let _e78 = xy;
            let _e80 = x1_;
            let _e82 = CellWidth;
            x = ((_e78.x - _e80) / _e82);
            let _e85 = xy;
            let _e87 = y1_;
            let _e89 = CellHeight;
            y = ((_e85.y - _e87) / _e89);
            let _e95 = global.style;
            if (_e95 == 0i) {
                {
                    let _e98 = x;
                    let _e99 = y;
                    if (_e98 < _e99) {
                        {
                            let _e101 = x1_;
                            let _e102 = y1_;
                            let _e105 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e101, _e102), 0f);
                            let _e106 = x1_;
                            let _e107 = y2_;
                            let _e110 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e106, _e107), 0f);
                            avgL = ((_e105 + _e110) / vec4(2f));
                            let _e116 = x2_;
                            let _e117 = y2_;
                            let _e120 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e116, _e117), 0f);
                            avgR = _e120;
                            let _e122 = x1_;
                            let _e123 = CellWidth;
                            let _e127 = y2_;
                            let _e128 = CellHeight;
                            let _e134 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>((_e122 + (_e123 / 2f)), (_e127 + (_e128 / 2f))), 0f);
                            avgC = _e134;
                            let _e136 = avgL;
                            let _e137 = avgR;
                            let _e139 = avgC;
                            avgClr = (((_e136 + _e137) + _e139) / vec4(3f));
                        }
                    } else {
                        {
                            let _e144 = x2_;
                            let _e145 = y1_;
                            let _e148 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e144, _e145), 0f);
                            let _e149 = x2_;
                            let _e150 = y2_;
                            let _e153 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e149, _e150), 0f);
                            avgR_1 = ((_e148 + _e153) / vec4(2f));
                            let _e159 = x1_;
                            let _e160 = y1_;
                            let _e163 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e159, _e160), 0f);
                            avgL_1 = _e163;
                            let _e165 = x1_;
                            let _e166 = CellWidth;
                            let _e170 = y2_;
                            let _e171 = CellHeight;
                            let _e177 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>((_e165 + (_e166 / 2f)), (_e170 + (_e171 / 2f))), 0f);
                            avgC_1 = _e177;
                            let _e179 = avgL_1;
                            let _e180 = avgR_1;
                            let _e182 = avgC_1;
                            avgClr = (((_e179 + _e180) + _e182) / vec4(3f));
                        }
                    }
                }
            } else {
                {
                    let _e187 = x;
                    let _e188 = y;
                    let _e190 = x;
                    let _e192 = y;
                    if ((_e187 > _e188) && (_e190 < (1f - _e192))) {
                        {
                            let _e196 = x1_;
                            let _e197 = y1_;
                            let _e200 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e196, _e197), 0f);
                            avgL_2 = _e200;
                            let _e202 = x2_;
                            let _e203 = y1_;
                            let _e206 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e202, _e203), 0f);
                            avgR_2 = _e206;
                            let _e208 = x1_;
                            let _e209 = CellWidth;
                            let _e213 = y2_;
                            let _e214 = CellHeight;
                            let _e220 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>((_e208 + (_e209 / 2f)), (_e213 + (_e214 / 2f))), 0f);
                            avgC_2 = _e220;
                            let _e222 = avgL_2;
                            let _e223 = avgR_2;
                            let _e225 = avgC_2;
                            avgClr = (((_e222 + _e223) + _e225) / vec4(3f));
                        }
                    } else {
                        let _e230 = x;
                        let _e231 = y;
                        let _e233 = x;
                        let _e235 = y;
                        if ((_e230 < _e231) && (_e233 < (1f - _e235))) {
                            {
                                let _e239 = x1_;
                                let _e240 = y1_;
                                let _e243 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e239, _e240), 0f);
                                avgL_3 = _e243;
                                let _e245 = x1_;
                                let _e246 = y2_;
                                let _e249 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e245, _e246), 0f);
                                avgR_3 = _e249;
                                let _e251 = x1_;
                                let _e252 = CellWidth;
                                let _e256 = y2_;
                                let _e257 = CellHeight;
                                let _e263 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>((_e251 + (_e252 / 2f)), (_e256 + (_e257 / 2f))), 0f);
                                avgC_3 = _e263;
                                let _e265 = avgL_3;
                                let _e266 = avgR_3;
                                let _e268 = avgC_3;
                                avgClr = (((_e265 + _e266) + _e268) / vec4(3f));
                            }
                        } else {
                            let _e273 = x;
                            let _e275 = y;
                            let _e278 = x;
                            let _e279 = y;
                            if ((_e273 > (1f - _e275)) && (_e278 < _e279)) {
                                {
                                    let _e282 = x1_;
                                    let _e283 = y2_;
                                    let _e286 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e282, _e283), 0f);
                                    avgL_4 = _e286;
                                    let _e288 = x2_;
                                    let _e289 = y2_;
                                    let _e292 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e288, _e289), 0f);
                                    avgR_4 = _e292;
                                    let _e294 = x1_;
                                    let _e295 = CellWidth;
                                    let _e299 = y2_;
                                    let _e300 = CellHeight;
                                    let _e306 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>((_e294 + (_e295 / 2f)), (_e299 + (_e300 / 2f))), 0f);
                                    avgC_4 = _e306;
                                    let _e308 = avgL_4;
                                    let _e309 = avgR_4;
                                    let _e311 = avgC_4;
                                    avgClr = (((_e308 + _e309) + _e311) / vec4(3f));
                                }
                            } else {
                                {
                                    let _e316 = x2_;
                                    let _e317 = y1_;
                                    let _e320 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e316, _e317), 0f);
                                    avgL_5 = _e320;
                                    let _e322 = x2_;
                                    let _e323 = y2_;
                                    let _e326 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e322, _e323), 0f);
                                    avgR_5 = _e326;
                                    let _e328 = x1_;
                                    let _e329 = CellWidth;
                                    let _e333 = y2_;
                                    let _e334 = CellHeight;
                                    let _e340 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>((_e328 + (_e329 / 2f)), (_e333 + (_e334 / 2f))), 0f);
                                    avgC_5 = _e340;
                                    let _e342 = avgL_5;
                                    let _e343 = avgR_5;
                                    let _e345 = avgC_5;
                                    avgClr = (((_e342 + _e343) + _e345) / vec4(3f));
                                }
                            }
                        }
                    }
                }
            }
            let _e350 = avgClr;
            isf_FragColor = _e350;
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e21 = isf_FragColor;
    return FragmentOutput(_e21);
}
