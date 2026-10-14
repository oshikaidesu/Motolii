// Technicolor: three.js's TechnicolorShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)

struct FragmentOutput {
    @location(0) three_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var tDiffuse_picture: texture_2d<f32>;
@group(0) @binding(1) 
var tDiffuse_sampler: sampler;
var<private> vUv_1: vec2<f32>;
var<private> three_FragColor: vec4<f32>;

fn main_1() {
    var tex: vec4<f32>;
    var newTex: vec4<f32>;

    let _e4 = vUv_1;
    let _e6 = vUv_1;
    let _e9 = textureSample(tDiffuse_picture, tDiffuse_sampler, vec2<f32>(_e4.x, _e6.y));
    tex = _e9;
    let _e11 = tex;
    let _e13 = tex;
    let _e15 = tex;
    let _e20 = tex;
    let _e22 = tex;
    newTex = vec4<f32>(_e11.x, ((_e13.y + _e15.z) * 0.5f), ((_e20.y + _e22.z) * 0.5f), 1f);
    let _e30 = newTex;
    three_FragColor = _e30;
    return;
}

@fragment 
fn main(@location(0) vUv: vec2<f32>) -> FragmentOutput {
    vUv_1 = vUv;
    main_1();
    let _e11 = three_FragColor;
    return FragmentOutput(_e11);
}
