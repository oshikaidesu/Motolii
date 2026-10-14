// Halftone: three.js's HalftoneShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)
// --track@radius:radius,0.0,16.0,4.0,0.001
// --track@rotateR:rotateR,0.0,1.0,0.0,0.001
// --track@rotateG:rotateG,0.0,1.0,0.0,0.001
// --track@rotateB:rotateB,0.0,1.0,0.0,0.001
// --track@scatter:scatter,0.0,1.0,0.0,0.001
// --track@width:width,0.0,4.0,1.0,0.001
// --track@height:height,0.0,4.0,1.0,0.001
// --track@shape:shape,0,4,1,1
// --check@disable_on:disable,0
// --track@blending:blending,0.0,4.0,1.0,0.001
// --track@blendingMode:blendingMode,0,4,1,1
// --check@greyscale_on:greyscale,0

struct Knobs {
    radius: f32,
    rotateR: f32,
    rotateG: f32,
    rotateB: f32,
    scatter: f32,
    width: f32,
    height: f32,
    shape: i32,
    disable_on: i32,
    blending: f32,
    blendingMode: i32,
    greyscale_on: i32,
}

struct Cell {
    normal: vec2<f32>,
    p1_: vec2<f32>,
    p2_: vec2<f32>,
    p3_: vec2<f32>,
    p4_: vec2<f32>,
    samp2_: f32,
    samp1_: f32,
    samp3_: f32,
    samp4_: f32,
}

struct FragmentOutput {
    @location(0) three_FragColor: vec4<f32>,
}

const samples: i32 = 8i;

@group(0) @binding(0) 
var tDiffuse_picture: texture_2d<f32>;
@group(0) @binding(1) 
var tDiffuse_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> vUV_1: vec2<f32>;
var<private> three_FragColor: vec4<f32>;

fn blend(a: f32, b: f32, t: f32) -> f32 {
    var a_1: f32;
    var b_1: f32;
    var t_1: f32;

    a_1 = a;
    b_1 = b;
    t_1 = t;
    let _e35 = a_1;
    let _e37 = t_1;
    let _e40 = b_1;
    let _e41 = t_1;
    return ((_e35 * (1f - _e37)) + (_e40 * _e41));
}

fn hypot(x: f32, y: f32) -> f32 {
    var x_1: f32;
    var y_1: f32;

    x_1 = x;
    y_1 = y;
    let _e33 = x_1;
    let _e34 = x_1;
    let _e36 = y_1;
    let _e37 = y_1;
    return sqrt(((_e33 * _e34) + (_e36 * _e37)));
}

fn rand(seed: vec2<f32>) -> f32 {
    var seed_1: vec2<f32>;

    seed_1 = seed;
    let _e31 = seed_1;
    return fract((sin(dot(_e31.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn distanceToDotRadius(channel: f32, coord: vec2<f32>, normal: vec2<f32>, p: vec2<f32>, angle: f32, rad_max: f32) -> f32 {
    var channel_1: f32;
    var coord_1: vec2<f32>;
    var normal_1: vec2<f32>;
    var p_1: vec2<f32>;
    var angle_1: f32;
    var rad_max_1: f32;
    var dist: f32;
    var rad: f32;
    var dot_p: f32;
    var dot_p_1: f32;
    var theta: f32;
    var sin_t: f32;
    var cos_t: f32;
    var local: f32;
    var angle45_: f32 = 0.7853982f;
    var theta_1: f32;
    var sin_t_1: f32;
    var cos_t_1: f32;
    var local_1: f32;

    channel_1 = channel;
    coord_1 = coord;
    normal_1 = normal;
    p_1 = p;
    angle_1 = angle;
    rad_max_1 = rad_max;
    let _e41 = coord_1;
    let _e43 = p_1;
    let _e46 = coord_1;
    let _e48 = p_1;
    let _e51 = hypot((_e41.x - _e43.x), (_e46.y - _e48.y));
    dist = _e51;
    let _e53 = channel_1;
    rad = _e53;
    let _e55 = global.shape;
    if (_e55 == 1i) {
        {
            let _e58 = rad;
            let _e62 = rad_max_1;
            rad = (pow(abs(_e58), 1.125f) * _e62);
        }
    } else {
        let _e64 = global.shape;
        if (_e64 == 2i) {
            {
                let _e67 = rad;
                let _e71 = rad_max_1;
                rad = (pow(abs(_e67), 1.125f) * _e71);
                let _e73 = dist;
                if (_e73 != 0f) {
                    {
                        let _e76 = p_1;
                        let _e78 = coord_1;
                        let _e81 = dist;
                        let _e83 = normal_1;
                        let _e86 = p_1;
                        let _e88 = coord_1;
                        let _e91 = dist;
                        let _e93 = normal_1;
                        dot_p = abs(((((_e76.x - _e78.x) / _e81) * _e83.x) + (((_e86.y - _e88.y) / _e91) * _e93.y)));
                        let _e99 = dist;
                        let _e104 = dot_p;
                        let _e105 = dist;
                        dist = ((_e99 * 0.79289323f) + ((_e104 * _e105) * 0.41421357f));
                    }
                }
            }
        } else {
            let _e110 = global.shape;
            if (_e110 == 3i) {
                {
                    let _e113 = rad;
                    let _e117 = rad_max_1;
                    rad = (pow(abs(_e113), 1.5f) * _e117);
                    let _e119 = p_1;
                    let _e121 = coord_1;
                    let _e124 = normal_1;
                    let _e127 = p_1;
                    let _e129 = coord_1;
                    let _e132 = normal_1;
                    dot_p_1 = (((_e119.x - _e121.x) * _e124.x) + ((_e127.y - _e129.y) * _e132.y));
                    let _e137 = normal_1;
                    let _e139 = dot_p_1;
                    let _e141 = normal_1;
                    let _e143 = dot_p_1;
                    let _e145 = hypot((_e137.x * _e139), (_e141.y * _e143));
                    dist = _e145;
                }
            } else {
                let _e146 = global.shape;
                if (_e146 == 4i) {
                    {
                        let _e149 = p_1;
                        let _e151 = coord_1;
                        let _e154 = p_1;
                        let _e156 = coord_1;
                        let _e160 = angle_1;
                        theta = (atan2((_e149.y - _e151.y), (_e154.x - _e156.x)) - _e160);
                        let _e163 = theta;
                        sin_t = abs(sin(_e163));
                        let _e167 = theta;
                        cos_t = abs(cos(_e167));
                        let _e171 = rad;
                        rad = pow(abs(_e171), 1.4f);
                        let _e175 = rad_max_1;
                        let _e176 = rad;
                        let _e177 = sin_t;
                        let _e178 = cos_t;
                        if (_e177 > _e178) {
                            let _e180 = rad;
                            let _e181 = sin_t;
                            let _e182 = rad;
                            local = (_e180 - (_e181 * _e182));
                        } else {
                            let _e185 = rad;
                            let _e186 = cos_t;
                            let _e187 = rad;
                            local = (_e185 - (_e186 * _e187));
                        }
                        let _e191 = local;
                        rad = (_e175 * (_e176 + _e191));
                    }
                } else {
                    let _e194 = global.shape;
                    if (_e194 == 5i) {
                        {
                            let _e201 = p_1;
                            let _e203 = coord_1;
                            let _e206 = p_1;
                            let _e208 = coord_1;
                            let _e212 = angle_1;
                            let _e214 = angle45_;
                            theta_1 = ((atan2((_e201.y - _e203.y), (_e206.x - _e208.x)) - _e212) - _e214);
                            let _e217 = theta_1;
                            sin_t_1 = abs(sin(_e217));
                            let _e221 = theta_1;
                            cos_t_1 = abs(cos(_e221));
                            let _e225 = rad;
                            rad = pow(abs(_e225), 1.4f);
                            let _e229 = rad_max_1;
                            let _e230 = rad;
                            let _e231 = sin_t_1;
                            let _e232 = cos_t_1;
                            if (_e231 > _e232) {
                                let _e234 = rad;
                                let _e235 = sin_t_1;
                                let _e236 = rad;
                                local_1 = (_e234 - (_e235 * _e236));
                            } else {
                                let _e239 = rad;
                                let _e240 = cos_t_1;
                                let _e241 = rad;
                                local_1 = (_e239 - (_e240 * _e241));
                            }
                            let _e245 = local_1;
                            rad = (_e229 * (_e230 + _e245));
                        }
                    }
                }
            }
        }
    }
    let _e248 = rad;
    let _e249 = dist;
    return (_e248 - _e249);
}

fn getSample(point: vec2<f32>) -> vec4<f32> {
    var point_1: vec2<f32>;
    var tex: vec4<f32>;
    var base: f32;
    var step_: f32 = 0.7853982f;
    var dist_1: f32;
    var i: i32 = 0i;
    var r: f32;
    var coord_2: vec2<f32>;

    point_1 = point;
    let _e31 = point_1;
    let _e33 = global.width;
    let _e35 = point_1;
    let _e37 = global.height;
    let _e40 = textureSample(tDiffuse_picture, tDiffuse_sampler, vec2<f32>((_e31.x / _e33), (_e35.y / _e37)));
    tex = _e40;
    let _e42 = point_1;
    let _e45 = point_1;
    let _e49 = rand(vec2<f32>(floor(_e42.x), floor(_e45.y)));
    base = (_e49 * 6.2831855f);
    let _e58 = global.radius;
    dist_1 = (_e58 * 0.66f);
    loop {
        let _e64 = i;
        if !((_e64 < samples)) {
            break;
        }
        {
            let _e70 = base;
            let _e71 = step_;
            let _e72 = i;
            r = (_e70 + (_e71 * f32(_e72)));
            let _e77 = point_1;
            let _e78 = r;
            let _e80 = dist_1;
            let _e82 = r;
            let _e84 = dist_1;
            coord_2 = (_e77 + vec2<f32>((cos(_e78) * _e80), (sin(_e82) * _e84)));
            let _e89 = tex;
            let _e90 = coord_2;
            let _e92 = global.width;
            let _e94 = coord_2;
            let _e96 = global.height;
            let _e99 = textureSample(tDiffuse_picture, tDiffuse_sampler, vec2<f32>((_e90.x / _e92), (_e94.y / _e96)));
            tex = (_e89 + _e99);
        }
        continuing {
            let _e67 = i;
            i = (_e67 + 1i);
        }
    }
    let _e101 = tex;
    tex = (_e101 / vec4(9f));
    let _e108 = tex;
    return _e108;
}

fn getDotColour(c: Cell, p_2: vec2<f32>, channel_2: i32, angle_2: f32, aa: f32) -> f32 {
    var c_1: Cell;
    var p_3: vec2<f32>;
    var channel_3: i32;
    var angle_3: f32;
    var aa_1: f32;
    var dist_c_1_: f32;
    var dist_c_2_: f32;
    var dist_c_3_: f32;
    var dist_c_4_: f32;
    var res: f32;
    var local_2: f32;
    var local_3: f32;
    var local_4: f32;
    var local_5: f32;

    c_1 = c;
    p_3 = p_2;
    channel_3 = channel_2;
    angle_3 = angle_2;
    aa_1 = aa;
    let _e44 = channel_3;
    if (_e44 == 0i) {
        {
            let _e48 = c_1;
            let _e50 = getSample(_e48.p1_);
            c_1.samp1_ = _e50.x;
            let _e53 = c_1;
            let _e55 = getSample(_e53.p2_);
            c_1.samp2_ = _e55.x;
            let _e58 = c_1;
            let _e60 = getSample(_e58.p3_);
            c_1.samp3_ = _e60.x;
            let _e63 = c_1;
            let _e65 = getSample(_e63.p4_);
            c_1.samp4_ = _e65.x;
        }
    } else {
        let _e67 = channel_3;
        if (_e67 == 1i) {
            {
                let _e71 = c_1;
                let _e73 = getSample(_e71.p1_);
                c_1.samp1_ = _e73.y;
                let _e76 = c_1;
                let _e78 = getSample(_e76.p2_);
                c_1.samp2_ = _e78.y;
                let _e81 = c_1;
                let _e83 = getSample(_e81.p3_);
                c_1.samp3_ = _e83.y;
                let _e86 = c_1;
                let _e88 = getSample(_e86.p4_);
                c_1.samp4_ = _e88.y;
            }
        } else {
            {
                let _e91 = c_1;
                let _e93 = getSample(_e91.p1_);
                c_1.samp1_ = _e93.z;
                let _e96 = c_1;
                let _e98 = getSample(_e96.p3_);
                c_1.samp3_ = _e98.z;
                let _e101 = c_1;
                let _e103 = getSample(_e101.p2_);
                c_1.samp2_ = _e103.z;
                let _e106 = c_1;
                let _e108 = getSample(_e106.p4_);
                c_1.samp4_ = _e108.z;
            }
        }
    }
    let _e110 = c_1;
    let _e112 = c_1;
    let _e114 = c_1;
    let _e116 = p_3;
    let _e117 = angle_3;
    let _e118 = global.radius;
    let _e119 = distanceToDotRadius(_e110.samp1_, _e112.p1_, _e114.normal, _e116, _e117, _e118);
    dist_c_1_ = _e119;
    let _e120 = c_1;
    let _e122 = c_1;
    let _e124 = c_1;
    let _e126 = p_3;
    let _e127 = angle_3;
    let _e128 = global.radius;
    let _e129 = distanceToDotRadius(_e120.samp2_, _e122.p2_, _e124.normal, _e126, _e127, _e128);
    dist_c_2_ = _e129;
    let _e130 = c_1;
    let _e132 = c_1;
    let _e134 = c_1;
    let _e136 = p_3;
    let _e137 = angle_3;
    let _e138 = global.radius;
    let _e139 = distanceToDotRadius(_e130.samp3_, _e132.p3_, _e134.normal, _e136, _e137, _e138);
    dist_c_3_ = _e139;
    let _e140 = c_1;
    let _e142 = c_1;
    let _e144 = c_1;
    let _e146 = p_3;
    let _e147 = angle_3;
    let _e148 = global.radius;
    let _e149 = distanceToDotRadius(_e140.samp4_, _e142.p4_, _e144.normal, _e146, _e147, _e148);
    dist_c_4_ = _e149;
    let _e150 = dist_c_1_;
    if (_e150 > 0f) {
        let _e153 = dist_c_1_;
        let _e154 = aa_1;
        local_2 = clamp((_e153 / _e154), 0f, 1f);
    } else {
        local_2 = 0f;
    }
    let _e161 = local_2;
    res = _e161;
    let _e162 = res;
    let _e163 = dist_c_2_;
    if (_e163 > 0f) {
        let _e166 = dist_c_2_;
        let _e167 = aa_1;
        local_3 = clamp((_e166 / _e167), 0f, 1f);
    } else {
        local_3 = 0f;
    }
    let _e174 = local_3;
    res = (_e162 + _e174);
    let _e176 = res;
    let _e177 = dist_c_3_;
    if (_e177 > 0f) {
        let _e180 = dist_c_3_;
        let _e181 = aa_1;
        local_4 = clamp((_e180 / _e181), 0f, 1f);
    } else {
        local_4 = 0f;
    }
    let _e188 = local_4;
    res = (_e176 + _e188);
    let _e190 = res;
    let _e191 = dist_c_4_;
    if (_e191 > 0f) {
        let _e194 = dist_c_4_;
        let _e195 = aa_1;
        local_5 = clamp((_e194 / _e195), 0f, 1f);
    } else {
        local_5 = 0f;
    }
    let _e202 = local_5;
    res = (_e190 + _e202);
    let _e204 = res;
    res = clamp(_e204, 0f, 1f);
    let _e208 = res;
    return _e208;
}

fn getReferenceCell(p_4: vec2<f32>, origin: vec2<f32>, grid_angle: f32, step_1: f32) -> Cell {
    var p_5: vec2<f32>;
    var origin_1: vec2<f32>;
    var grid_angle_1: f32;
    var step_2: f32;
    var c_2: Cell;
    var n: vec2<f32>;
    var threshold: f32;
    var dot_normal: f32;
    var dot_line: f32;
    var offset: vec2<f32>;
    var offset_normal: f32;
    var local_6: f32;
    var normal_dir: f32;
    var local_7: f32;
    var normal_scale: f32;
    var offset_line: f32;
    var local_8: f32;
    var line_dir: f32;
    var local_9: f32;
    var line_scale: f32;
    var off_mag: f32;
    var off_angle: f32;
    var local_10: f32;
    var normal_step: f32;
    var local_11: f32;
    var line_step: f32;

    p_5 = p_4;
    origin_1 = origin;
    grid_angle_1 = grid_angle;
    step_2 = step_1;
    let _e38 = grid_angle_1;
    let _e40 = grid_angle_1;
    n = vec2<f32>(cos(_e38), sin(_e40));
    let _e44 = step_2;
    threshold = (_e44 * 0.5f);
    let _e48 = n;
    let _e50 = p_5;
    let _e52 = origin_1;
    let _e56 = n;
    let _e58 = p_5;
    let _e60 = origin_1;
    dot_normal = ((_e48.x * (_e50.x - _e52.x)) + (_e56.y * (_e58.y - _e60.y)));
    let _e66 = n;
    let _e69 = p_5;
    let _e71 = origin_1;
    let _e75 = n;
    let _e77 = p_5;
    let _e79 = origin_1;
    dot_line = ((-(_e66.y) * (_e69.x - _e71.x)) + (_e75.x * (_e77.y - _e79.y)));
    let _e85 = n;
    let _e87 = dot_normal;
    let _e89 = n;
    let _e91 = dot_normal;
    offset = vec2<f32>((_e85.x * _e87), (_e89.y * _e91));
    let _e95 = offset;
    let _e97 = offset;
    let _e99 = hypot(_e95.x, _e97.y);
    let _e100 = step_2;
    offset_normal = (_e99 - (floor((_e99 / _e100)) * _e100));
    let _e106 = dot_normal;
    if (_e106 < 0f) {
        local_6 = 1f;
    } else {
        local_6 = -1f;
    }
    let _e113 = local_6;
    normal_dir = _e113;
    let _e115 = offset_normal;
    let _e116 = threshold;
    if (_e115 < _e116) {
        let _e118 = offset_normal;
        local_7 = -(_e118);
    } else {
        let _e120 = step_2;
        let _e121 = offset_normal;
        local_7 = (_e120 - _e121);
    }
    let _e124 = local_7;
    let _e125 = normal_dir;
    normal_scale = (_e124 * _e125);
    let _e128 = p_5;
    let _e130 = offset;
    let _e133 = origin_1;
    let _e136 = p_5;
    let _e138 = offset;
    let _e141 = origin_1;
    let _e144 = hypot(((_e128.x - _e130.x) - _e133.x), ((_e136.y - _e138.y) - _e141.y));
    let _e145 = step_2;
    offset_line = (_e144 - (floor((_e144 / _e145)) * _e145));
    let _e151 = dot_line;
    if (_e151 < 0f) {
        local_8 = 1f;
    } else {
        local_8 = -1f;
    }
    let _e158 = local_8;
    line_dir = _e158;
    let _e160 = offset_line;
    let _e161 = threshold;
    if (_e160 < _e161) {
        let _e163 = offset_line;
        local_9 = -(_e163);
    } else {
        let _e165 = step_2;
        let _e166 = offset_line;
        local_9 = (_e165 - _e166);
    }
    let _e169 = local_9;
    let _e170 = line_dir;
    line_scale = (_e169 * _e170);
    let _e174 = n;
    c_2.normal = _e174;
    let _e177 = p_5;
    let _e179 = n;
    let _e181 = normal_scale;
    let _e184 = n;
    let _e186 = line_scale;
    c_2.p1_.x = ((_e177.x - (_e179.x * _e181)) + (_e184.y * _e186));
    let _e191 = p_5;
    let _e193 = n;
    let _e195 = normal_scale;
    let _e198 = n;
    let _e200 = line_scale;
    c_2.p1_.y = ((_e191.y - (_e193.y * _e195)) - (_e198.x * _e200));
    let _e203 = global.scatter;
    if (_e203 != 0f) {
        {
            let _e206 = global.scatter;
            let _e207 = threshold;
            off_mag = ((_e206 * _e207) * 0.5f);
            let _e212 = c_2;
            let _e216 = c_2;
            let _e221 = rand(vec2<f32>(floor(_e212.p1_.x), floor(_e216.p1_.y)));
            off_angle = (_e221 * 6.2831855f);
            let _e227 = c_2;
            let _e230 = off_angle;
            let _e232 = off_mag;
            c_2.p1_.x = (_e227.p1_.x + (cos(_e230) * _e232));
            let _e237 = c_2;
            let _e240 = off_angle;
            let _e242 = off_mag;
            c_2.p1_.y = (_e237.p1_.y + (sin(_e240) * _e242));
        }
    }
    let _e245 = normal_dir;
    let _e246 = offset_normal;
    let _e247 = threshold;
    if (_e246 < _e247) {
        let _e249 = step_2;
        local_10 = _e249;
    } else {
        let _e250 = step_2;
        local_10 = -(_e250);
    }
    let _e253 = local_10;
    normal_step = (_e245 * _e253);
    let _e256 = line_dir;
    let _e257 = offset_line;
    let _e258 = threshold;
    if (_e257 < _e258) {
        let _e260 = step_2;
        local_11 = _e260;
    } else {
        let _e261 = step_2;
        local_11 = -(_e261);
    }
    let _e264 = local_11;
    line_step = (_e256 * _e264);
    let _e269 = c_2;
    let _e272 = n;
    let _e274 = normal_step;
    c_2.p2_.x = (_e269.p1_.x - (_e272.x * _e274));
    let _e279 = c_2;
    let _e282 = n;
    let _e284 = normal_step;
    c_2.p2_.y = (_e279.p1_.y - (_e282.y * _e284));
    let _e289 = c_2;
    let _e292 = n;
    let _e294 = line_step;
    c_2.p3_.x = (_e289.p1_.x + (_e292.y * _e294));
    let _e299 = c_2;
    let _e302 = n;
    let _e304 = line_step;
    c_2.p3_.y = (_e299.p1_.y - (_e302.x * _e304));
    let _e309 = c_2;
    let _e312 = n;
    let _e314 = normal_step;
    let _e317 = n;
    let _e319 = line_step;
    c_2.p4_.x = ((_e309.p1_.x - (_e312.x * _e314)) + (_e317.y * _e319));
    let _e324 = c_2;
    let _e327 = n;
    let _e329 = normal_step;
    let _e332 = n;
    let _e334 = line_step;
    c_2.p4_.y = ((_e324.p1_.y - (_e327.y * _e329)) - (_e332.x * _e334));
    let _e337 = c_2;
    return _e337;
}

fn blendColour(a_2: f32, b_2: f32, t_2: f32) -> f32 {
    var a_3: f32;
    var b_3: f32;
    var t_3: f32;

    a_3 = a_2;
    b_3 = b_2;
    t_3 = t_2;
    let _e35 = global.blendingMode;
    if (_e35 == 1i) {
        {
            let _e38 = a_3;
            let _e39 = b_3;
            let _e41 = t_3;
            let _e43 = blend(_e38, _e39, (1f - _e41));
            return _e43;
        }
    } else {
        let _e44 = global.blendingMode;
        if (_e44 == 3i) {
            {
                let _e47 = a_3;
                let _e49 = a_3;
                let _e50 = b_3;
                let _e53 = t_3;
                let _e54 = blend(_e47, min(1f, (_e49 + _e50)), _e53);
                return _e54;
            }
        } else {
            let _e55 = global.blendingMode;
            if (_e55 == 2i) {
                {
                    let _e58 = a_3;
                    let _e60 = a_3;
                    let _e61 = b_3;
                    let _e64 = t_3;
                    let _e65 = blend(_e58, max(0f, (_e60 * _e61)), _e64);
                    return _e65;
                }
            } else {
                let _e66 = global.blendingMode;
                if (_e66 == 4i) {
                    {
                        let _e69 = a_3;
                        let _e70 = a_3;
                        let _e71 = b_3;
                        let _e73 = t_3;
                        let _e74 = blend(_e69, max(_e70, _e71), _e73);
                        return _e74;
                    }
                } else {
                    let _e75 = global.blendingMode;
                    if (_e75 == 5i) {
                        {
                            let _e78 = a_3;
                            let _e79 = a_3;
                            let _e80 = b_3;
                            let _e82 = t_3;
                            let _e83 = blend(_e78, min(_e79, _e80), _e82);
                            return _e83;
                        }
                    } else {
                        {
                            let _e84 = a_3;
                            let _e85 = b_3;
                            let _e87 = t_3;
                            let _e89 = blend(_e84, _e85, (1f - _e87));
                            return _e89;
                        }
                    }
                }
            }
        }
    }
}

fn main_1() {
    var p_6: vec2<f32>;
    var origin_2: vec2<f32> = vec2<f32>(0f, 0f);
    var local_12: f32;
    var aa_2: f32;
    var cell_r: Cell;
    var cell_g: Cell;
    var cell_b: Cell;
    var r_1: f32;
    var g: f32;
    var b_4: f32;
    var colour: vec4<f32>;

    let _e29 = global.disable_on;
    if !((_e29 != 0i)) {
        {
            let _e33 = vUV_1;
            let _e35 = global.width;
            let _e37 = vUV_1;
            let _e39 = global.height;
            p_6 = vec2<f32>((_e33.x * _e35), (_e37.y * _e39));
            let _e49 = global.radius;
            if (_e49 < 2.5f) {
                let _e52 = global.radius;
                local_12 = (_e52 * 0.5f);
            } else {
                local_12 = 1.25f;
            }
            let _e57 = local_12;
            aa_2 = _e57;
            let _e59 = p_6;
            let _e60 = origin_2;
            let _e61 = global.rotateR;
            let _e62 = global.radius;
            let _e63 = getReferenceCell(_e59, _e60, _e61, _e62);
            cell_r = _e63;
            let _e65 = p_6;
            let _e66 = origin_2;
            let _e67 = global.rotateG;
            let _e68 = global.radius;
            let _e69 = getReferenceCell(_e65, _e66, _e67, _e68);
            cell_g = _e69;
            let _e71 = p_6;
            let _e72 = origin_2;
            let _e73 = global.rotateB;
            let _e74 = global.radius;
            let _e75 = getReferenceCell(_e71, _e72, _e73, _e74);
            cell_b = _e75;
            let _e77 = cell_r;
            let _e78 = p_6;
            let _e80 = global.rotateR;
            let _e81 = aa_2;
            let _e82 = getDotColour(_e77, _e78, 0i, _e80, _e81);
            r_1 = _e82;
            let _e84 = cell_g;
            let _e85 = p_6;
            let _e87 = global.rotateG;
            let _e88 = aa_2;
            let _e89 = getDotColour(_e84, _e85, 1i, _e87, _e88);
            g = _e89;
            let _e91 = cell_b;
            let _e92 = p_6;
            let _e94 = global.rotateB;
            let _e95 = aa_2;
            let _e96 = getDotColour(_e91, _e92, 2i, _e94, _e95);
            b_4 = _e96;
            let _e98 = vUV_1;
            let _e99 = textureSample(tDiffuse_picture, tDiffuse_sampler, _e98);
            colour = _e99;
            let _e101 = r_1;
            let _e102 = colour;
            let _e104 = global.blending;
            let _e105 = blendColour(_e101, _e102.x, _e104);
            r_1 = _e105;
            let _e106 = g;
            let _e107 = colour;
            let _e109 = global.blending;
            let _e110 = blendColour(_e106, _e107.y, _e109);
            g = _e110;
            let _e111 = b_4;
            let _e112 = colour;
            let _e114 = global.blending;
            let _e115 = blendColour(_e111, _e112.z, _e114);
            b_4 = _e115;
            let _e116 = global.greyscale_on;
            if (_e116 != 0i) {
                {
                    let _e119 = r_1;
                    let _e120 = b_4;
                    let _e122 = g;
                    let _e125 = (((_e119 + _e120) + _e122) / 3f);
                    b_4 = _e125;
                    g = _e125;
                    r_1 = _e125;
                }
            }
            let _e126 = r_1;
            let _e127 = g;
            let _e128 = b_4;
            three_FragColor = vec4<f32>(_e126, _e127, _e128, 1f);
            return;
        }
    } else {
        {
            let _e131 = vUV_1;
            let _e132 = textureSample(tDiffuse_picture, tDiffuse_sampler, _e131);
            three_FragColor = _e132;
            return;
        }
    }
}

@fragment 
fn main(@location(0) vUV: vec2<f32>) -> FragmentOutput {
    vUV_1 = vUV;
    main_1();
    let _e37 = three_FragColor;
    return FragmentOutput(_e37);
}
