// RGB Halftone: ISF-Files' RGB Halftone.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by zoidberg), its GLSL made WGSL by naga (filament/effects/from_isf.py)
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

@group(0) @binding(0) 
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;
var<private> gridRot: vec3<f32> = vec3<f32>(15f, 45f, 75f);
var<private> gl_FragCoord_1: vec4<f32>;

fn main_1() {
    var rgbaAmounts: vec4<f32> = vec4(0f);
    var i: i32 = 0i;
    var rotRad: f32;
    var ccTrans: mat2x2<f32>;
    var cTrans: mat2x2<f32>;
    var gridFragLoc: vec2<f32>;
    var gridDotLoc: vec2<f32>;
    var renderDotLoc: vec2<f32>;
    var renderDotImageColorRGB: vec4<f32>;
    var channelAmount: f32;
    var dotRadius: f32;
    var fragDistanceToGridCenter: f32;
    var smoothDist: f32;

    loop {
        let _e20 = i;
        if !((_e20 < 3i)) {
            break;
        }
        {
            let _e27 = i;
            let _e29 = gridRot[_e27];
            rotRad = radians(_e29);
            let _e32 = rotRad;
            let _e34 = rotRad;
            let _e36 = vec2<f32>(cos(_e32), sin(_e34));
            let _e39 = rotRad;
            let _e42 = rotRad;
            let _e44 = vec2<f32>((-1f * sin(_e39)), cos(_e42));
            ccTrans = mat2x2<f32>(vec2<f32>(_e36.x, _e36.y), vec2<f32>(_e44.x, _e44.y));
            let _e53 = rotRad;
            let _e57 = rotRad;
            let _e60 = vec2<f32>(cos(_e53), (-1f * sin(_e57)));
            let _e61 = rotRad;
            let _e63 = rotRad;
            let _e65 = vec2<f32>(sin(_e61), cos(_e63));
            cTrans = mat2x2<f32>(vec2<f32>(_e60.x, _e60.y), vec2<f32>(_e65.x, _e65.y));
            let _e75 = cTrans;
            let _e76 = gl_FragCoord_1;
            gridFragLoc = (_e75 * _e76.xy);
            let _e80 = gridFragLoc;
            let _e82 = global.gridSize;
            let _e85 = global.gridSize;
            let _e87 = gridFragLoc;
            let _e89 = global.gridSize;
            let _e92 = global.gridSize;
            gridDotLoc = vec2<f32>((floor((_e80.x / _e82)) * _e85), (floor((_e87.y / _e89)) * _e92));
            let _e96 = gridDotLoc;
            let _e97 = global.gridSize;
            gridDotLoc = (_e96 + vec2((_e97 / 2f)));
            let _e102 = ccTrans;
            let _e103 = gridDotLoc;
            renderDotLoc = (_e102 * _e103);
            let _e106 = renderDotLoc;
            let _e107 = global.resolution;
            let _e110 = textureSampleLevel(isf_picture, isf_sampler, (_e106 / _e107), 0f);
            renderDotImageColorRGB = _e110;
            let _e112 = i;
            let _e114 = renderDotImageColorRGB[_e112];
            channelAmount = _e114;
            let _e116 = channelAmount;
            let _e117 = global.gridSize;
            dotRadius = (_e116 * (_e117 / 2f));
            let _e122 = gl_FragCoord_1;
            let _e124 = renderDotLoc;
            fragDistanceToGridCenter = distance(_e122.xy, _e124);
            let _e127 = global.smoothing;
            let _e128 = global.gridSize;
            smoothDist = (_e127 * (_e128 / 6f));
            let _e133 = i;
            let _e135 = i;
            let _e137 = rgbaAmounts[_e135];
            let _e138 = dotRadius;
            let _e139 = dotRadius;
            let _e140 = dotRadius;
            let _e141 = global.smoothing;
            let _e144 = fragDistanceToGridCenter;
            rgbaAmounts[_e133] = (_e137 + smoothstep(_e138, (_e139 - (_e140 * _e141)), _e144));
        }
        continuing {
            let _e24 = i;
            i = (_e24 + 1i);
        }
    }
    rgbaAmounts.w = 1f;
    let _e149 = rgbaAmounts;
    isf_FragColor = _e149;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e29 = isf_FragColor;
    return FragmentOutput(_e29);
}
