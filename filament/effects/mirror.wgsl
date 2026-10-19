// Mirror: three.js's MirrorShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --track@side:side,0,4,1,1

struct Knobs {
    side: i32,
}

struct FragmentOutput {
    @location(0) three_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var tDiffuse_picture: texture_2d<f32>;
@group(0) @binding(1) 
var tDiffuse_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> vUv_1: vec2<f32>;
var<private> three_FragColor: vec4<f32>;

fn main_1() {
    var p: vec2<f32>;
    var color: vec4<f32>;

    let _e6 = vUv_1;
    p = _e6;
    let _e8 = global.side;
    if (_e8 == 0i) {
        {
            let _e11 = p;
            if (_e11.x > 0.5f) {
                let _e17 = p;
                p.x = (1f - _e17.x);
            }
        }
    } else {
        let _e20 = global.side;
        if (_e20 == 1i) {
            {
                let _e23 = p;
                if (_e23.x < 0.5f) {
                    let _e29 = p;
                    p.x = (1f - _e29.x);
                }
            }
        } else {
            let _e32 = global.side;
            if (_e32 == 2i) {
                {
                    let _e35 = p;
                    if (_e35.y < 0.5f) {
                        let _e41 = p;
                        p.y = (1f - _e41.y);
                    }
                }
            } else {
                let _e44 = global.side;
                if (_e44 == 3i) {
                    {
                        let _e47 = p;
                        if (_e47.y > 0.5f) {
                            let _e53 = p;
                            p.y = (1f - _e53.y);
                        }
                    }
                }
            }
        }
    }
    let _e56 = p;
    let _e57 = textureSample(tDiffuse_picture, tDiffuse_sampler, _e56);
    color = _e57;
    let _e59 = color;
    three_FragColor = _e59;
    return;
}

@fragment 
fn main(@location(0) vUv: vec2<f32>) -> FragmentOutput {
    vUv_1 = vUv;
    main_1();
    let _e13 = three_FragColor;
    return FragmentOutput(_e13);
}
