// CMYK Halftone: ISF-Files' CMYK Halftone.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by zoidberg), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@gridSize:gridSize,1,256,45,0.001
// --track@smoothing:smoothing,0,1,0.15,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    gridSize: f32,
    smoothing: f32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const gridRot: vec4<f32> = vec4<f32>(15f, 45f, 0f, 75f);

@group(0) @binding(0) 
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;

fn RGBAtoCMYK(inColor: vec4<f32>) -> vec4<f32> {
    var inColor_1: vec4<f32>;
    var ret: vec4<f32>;

    inColor_1 = inColor;
    let _e20 = inColor_1;
    let _e22 = inColor_1;
    let _e25 = inColor_1;
    ret.w = (1f - max(max(_e20.x, _e22.y), _e25.z));
    let _e31 = inColor_1;
    let _e34 = ret;
    let _e38 = ret;
    ret.x = (((1f - _e31.x) - _e34.w) / (1f - _e38.w));
    let _e44 = inColor_1;
    let _e47 = ret;
    let _e51 = ret;
    ret.y = (((1f - _e44.y) - _e47.w) / (1f - _e51.w));
    let _e57 = inColor_1;
    let _e60 = ret;
    let _e64 = ret;
    ret.z = (((1f - _e57.z) - _e60.w) / (1f - _e64.w));
    let _e68 = ret;
    return _e68;
}

fn CMYKtoRGBA(inColor_2: vec4<f32>) -> vec4<f32> {
    var inColor_3: vec4<f32>;
    var ret_1: vec4<f32>;

    inColor_3 = inColor_2;
    let _e18 = ret_1;
    let _e21 = inColor_3;
    let _e26 = inColor_3;
    let _e29 = ((vec3(1f) - _e21.xyz) * (1f - _e26.w));
    ret_1.x = _e29.x;
    ret_1.y = _e29.y;
    ret_1.z = _e29.z;
    ret_1.w = 1f;
    let _e38 = ret_1;
    return _e38;
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
        let _e20 = i;
        if !((_e20 < 4i)) {
            break;
        }
        {
            let _e27 = i;
            let _e31 = local[_e27];
            rotRad = radians(_e31);
            let _e34 = rotRad;
            let _e36 = rotRad;
            let _e38 = vec2<f32>(cos(_e34), sin(_e36));
            let _e41 = rotRad;
            let _e44 = rotRad;
            let _e46 = vec2<f32>((-1f * sin(_e41)), cos(_e44));
            ccTrans = mat2x2<f32>(vec2<f32>(_e38.x, _e38.y), vec2<f32>(_e46.x, _e46.y));
            let _e55 = rotRad;
            let _e59 = rotRad;
            let _e62 = vec2<f32>(cos(_e55), (-1f * sin(_e59)));
            let _e63 = rotRad;
            let _e65 = rotRad;
            let _e67 = vec2<f32>(sin(_e63), cos(_e65));
            cTrans = mat2x2<f32>(vec2<f32>(_e62.x, _e62.y), vec2<f32>(_e67.x, _e67.y));
            let _e77 = cTrans;
            let _e78 = gl_FragCoord_1;
            gridFragLoc = (_e77 * _e78.xy);
            let _e82 = gridFragLoc;
            let _e84 = global.gridSize;
            let _e87 = global.gridSize;
            let _e89 = gridFragLoc;
            let _e91 = global.gridSize;
            let _e94 = global.gridSize;
            gridDotLoc = vec2<f32>((floor((_e82.x / _e84)) * _e87), (floor((_e89.y / _e91)) * _e94));
            let _e98 = gridDotLoc;
            let _e99 = global.gridSize;
            gridDotLoc = (_e98 + vec2((_e99 / 2f)));
            let _e104 = ccTrans;
            let _e105 = gridDotLoc;
            renderDotLoc = (_e104 * _e105);
            let _e108 = renderDotLoc;
            let _e109 = global.resolution;
            let _e112 = textureSampleLevel(isf_picture, isf_sampler, (_e108 / _e109), 0f);
            renderDotImageColorRGB = _e112;
            let _e114 = renderDotImageColorRGB;
            let _e115 = RGBAtoCMYK(_e114);
            renderDotImageColorCMYK = _e115;
            let _e117 = i;
            let _e119 = renderDotImageColorCMYK[_e117];
            channelAmount = _e119;
            let _e121 = channelAmount;
            let _e122 = global.gridSize;
            dotRadius = (_e121 * (_e122 / 2f));
            let _e127 = gl_FragCoord_1;
            let _e129 = renderDotLoc;
            fragDistanceToGridCenter = distance(_e127.xy, _e129);
            let _e132 = global.smoothing;
            let _e133 = global.gridSize;
            smoothDist = (_e132 * (_e133 / 6f));
            let _e138 = i;
            let _e140 = i;
            let _e142 = cmykAmounts[_e140];
            let _e143 = dotRadius;
            let _e144 = dotRadius;
            let _e145 = dotRadius;
            let _e146 = global.smoothing;
            let _e149 = fragDistanceToGridCenter;
            cmykAmounts[_e138] = (_e142 + smoothstep(_e143, (_e144 - (_e145 * _e146)), _e149));
        }
        continuing {
            let _e24 = i;
            i = (_e24 + 1i);
        }
    }
    let _e152 = cmykAmounts;
    let _e153 = CMYKtoRGBA(_e152);
    isf_FragColor = _e153;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e25 = isf_FragColor;
    return FragmentOutput(_e25);
}
