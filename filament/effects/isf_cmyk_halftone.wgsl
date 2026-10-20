// CMYK Halftone: ISF-Files' CMYK Halftone.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by zoidberg), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Stylize
// --track@gridSize:gridSize,1,256,45,0.001
// --track@smoothing:smoothing,0,1,0.15,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    gridSize: f32,
    smoothing: f32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const gridRot: vec4<f32> = vec4<f32>(15f, 45f, 0f, 75f);

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;

fn RGBAtoCMYK(inColor: vec4<f32>) -> vec4<f32> {
    var inColor_1: vec4<f32>;
    var ret: vec4<f32>;

    inColor_1 = inColor;
    let _e22 = inColor_1;
    let _e24 = inColor_1;
    let _e27 = inColor_1;
    ret.w = (1f - max(max(_e22.x, _e24.y), _e27.z));
    let _e33 = inColor_1;
    let _e36 = ret;
    let _e40 = ret;
    ret.x = (((1f - _e33.x) - _e36.w) / (1f - _e40.w));
    let _e46 = inColor_1;
    let _e49 = ret;
    let _e53 = ret;
    ret.y = (((1f - _e46.y) - _e49.w) / (1f - _e53.w));
    let _e59 = inColor_1;
    let _e62 = ret;
    let _e66 = ret;
    ret.z = (((1f - _e59.z) - _e62.w) / (1f - _e66.w));
    let _e70 = ret;
    return _e70;
}

fn CMYKtoRGBA(inColor_2: vec4<f32>) -> vec4<f32> {
    var inColor_3: vec4<f32>;
    var ret_1: vec4<f32>;

    inColor_3 = inColor_2;
    let _e20 = ret_1;
    let _e23 = inColor_3;
    let _e28 = inColor_3;
    let _e31 = ((vec3(1f) - _e23.xyz) * (1f - _e28.w));
    ret_1.x = _e31.x;
    ret_1.y = _e31.y;
    ret_1.z = _e31.z;
    ret_1.w = 1f;
    let _e40 = ret_1;
    return _e40;
}

fn main_1() {
    var cmykAmounts: vec4<f32> = vec4(0f);
    var i: i32 = 0i;
    var local: vec4<f32> = gridRot;
    var rotRad: f32;
    var ccTrans: mat2x2<f32>;
    var cTrans: mat2x2<f32>;
    var gridFragLoc: vec2<f32>;
    var gridDotLoc: vec2<f32>;
    var renderDotLoc: vec2<f32>;
    var renderDotImageColorRGB: vec4<f32>;
    var renderDotImageColorCMYK: vec4<f32>;
    var channelAmount: f32;
    var dotRadius: f32;
    var fragDistanceToGridCenter: f32;
    var smoothDist: f32;

    loop {
        let _e22 = i;
        if !((_e22 < 4i)) {
            break;
        }
        {
            let _e29 = i;
            let _e33 = local[_e29];
            rotRad = radians(_e33);
            let _e36 = rotRad;
            let _e38 = rotRad;
            let _e40 = vec2<f32>(cos(_e36), sin(_e38));
            let _e43 = rotRad;
            let _e46 = rotRad;
            let _e48 = vec2<f32>((-1f * sin(_e43)), cos(_e46));
            ccTrans = mat2x2<f32>(vec2<f32>(_e40.x, _e40.y), vec2<f32>(_e48.x, _e48.y));
            let _e57 = rotRad;
            let _e61 = rotRad;
            let _e64 = vec2<f32>(cos(_e57), (-1f * sin(_e61)));
            let _e65 = rotRad;
            let _e67 = rotRad;
            let _e69 = vec2<f32>(sin(_e65), cos(_e67));
            cTrans = mat2x2<f32>(vec2<f32>(_e64.x, _e64.y), vec2<f32>(_e69.x, _e69.y));
            let _e79 = cTrans;
            let _e80 = gl_FragCoord_1;
            gridFragLoc = (_e79 * _e80.xy);
            let _e84 = gridFragLoc;
            let _e86 = global.gridSize;
            let _e89 = global.gridSize;
            let _e91 = gridFragLoc;
            let _e93 = global.gridSize;
            let _e96 = global.gridSize;
            gridDotLoc = vec2<f32>((floor((_e84.x / _e86)) * _e89), (floor((_e91.y / _e93)) * _e96));
            let _e100 = gridDotLoc;
            let _e101 = global.gridSize;
            gridDotLoc = (_e100 + vec2((_e101 / 2f)));
            let _e106 = ccTrans;
            let _e107 = gridDotLoc;
            renderDotLoc = (_e106 * _e107);
            let _e110 = renderDotLoc;
            let _e111 = global.resolution;
            let _e114 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (_e110 / _e111), 0f);
            renderDotImageColorRGB = _e114;
            let _e116 = renderDotImageColorRGB;
            let _e117 = RGBAtoCMYK(_e116);
            renderDotImageColorCMYK = _e117;
            let _e119 = i;
            let _e121 = renderDotImageColorCMYK[_e119];
            channelAmount = _e121;
            let _e123 = channelAmount;
            let _e124 = global.gridSize;
            dotRadius = (_e123 * (_e124 / 2f));
            let _e129 = gl_FragCoord_1;
            let _e131 = renderDotLoc;
            fragDistanceToGridCenter = distance(_e129.xy, _e131);
            let _e134 = global.smoothing;
            let _e135 = global.gridSize;
            smoothDist = (_e134 * (_e135 / 6f));
            let _e140 = i;
            let _e142 = i;
            let _e144 = cmykAmounts[_e142];
            let _e145 = dotRadius;
            let _e146 = dotRadius;
            let _e147 = dotRadius;
            let _e148 = global.smoothing;
            let _e151 = fragDistanceToGridCenter;
            cmykAmounts[_e140] = (_e144 + smoothstep(_e145, (_e146 - (_e147 * _e148)), _e151));
        }
        continuing {
            let _e26 = i;
            i = (_e26 + 1i);
        }
    }
    let _e154 = cmykAmounts;
    let _e155 = CMYKtoRGBA(_e154);
    isf_FragColor = _e155;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e27 = isf_FragColor;
    return FragmentOutput(_e27);
}
