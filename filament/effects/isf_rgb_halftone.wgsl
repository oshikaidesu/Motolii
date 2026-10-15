// RGB Halftone: ISF-Files' RGB Halftone.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by zoidberg), its GLSL made WGSL by naga (filament/effects/from_isf.py)
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

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
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
        let _e22 = i;
        if !((_e22 < 3i)) {
            break;
        }
        {
            let _e29 = i;
            let _e31 = gridRot[_e29];
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
            let _e112 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (_e108 / _e109), 0f);
            renderDotImageColorRGB = _e112;
            let _e114 = i;
            let _e116 = renderDotImageColorRGB[_e114];
            channelAmount = _e116;
            let _e118 = channelAmount;
            let _e119 = global.gridSize;
            dotRadius = (_e118 * (_e119 / 2f));
            let _e124 = gl_FragCoord_1;
            let _e126 = renderDotLoc;
            fragDistanceToGridCenter = distance(_e124.xy, _e126);
            let _e129 = global.smoothing;
            let _e130 = global.gridSize;
            smoothDist = (_e129 * (_e130 / 6f));
            let _e135 = i;
            let _e137 = i;
            let _e139 = rgbaAmounts[_e137];
            let _e140 = dotRadius;
            let _e141 = dotRadius;
            let _e142 = dotRadius;
            let _e143 = global.smoothing;
            let _e146 = fragDistanceToGridCenter;
            rgbaAmounts[_e135] = (_e139 + smoothstep(_e140, (_e141 - (_e142 * _e143)), _e146));
        }
        continuing {
            let _e26 = i;
            i = (_e26 + 1i);
        }
    }
    rgbaAmounts.w = 1f;
    let _e151 = rgbaAmounts;
    isf_FragColor = _e151;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e31 = isf_FragColor;
    return FragmentOutput(_e31);
}
