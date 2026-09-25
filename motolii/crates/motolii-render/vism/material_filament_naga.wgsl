// Generated: naga 29 GLSL frontend over motolii/reference/filament/filament_lib.frag (Filament shaders/src,
// google/filament@41f996de8fcc2d6b60b73159aa1bc44a05a40700, Apache-2.0). Do not edit by hand.
struct fil_Refraction {
    position: vec3<f32>,
    direction: vec3<f32>,
    d: f32,
}



fn fil_pow5_(x: f32) -> f32 {
    var x_1: f32;
    var x2_: f32;

    x_1 = x;
    let _e2 = x_1;
    let _e3 = x_1;
    x2_ = (_e2 * _e3);
    let _e6 = x2_;
    let _e7 = x2_;
    let _e9 = x_1;
    return ((_e6 * _e7) * _e9);
}

fn fil_sq(x_2: f32) -> f32 {
    var x_3: f32;

    x_3 = x_2;
    let _e2 = x_3;
    let _e3 = x_3;
    return (_e2 * _e3);
}

fn fil_D_GGX(roughness: f32, NoH: f32, h: vec3<f32>) -> f32 {
    var roughness_1: f32;
    var NoH_1: f32;
    var oneMinusNoHSquared: f32;
    var a: f32;
    var k: f32;
    var d: f32;

    roughness_1 = roughness;
    NoH_1 = NoH;
    let _e6 = NoH_1;
    let _e7 = NoH_1;
    oneMinusNoHSquared = (1f - (_e6 * _e7));
    let _e11 = NoH_1;
    let _e12 = roughness_1;
    a = (_e11 * _e12);
    let _e15 = roughness_1;
    let _e16 = oneMinusNoHSquared;
    let _e17 = a;
    let _e18 = a;
    k = min((_e15 / (_e16 + (_e17 * _e18))), 453.5f);
    let _e25 = k;
    let _e26 = k;
    d = (_e25 * (_e26 * 0.31830987f));
    let _e33 = d;
    return _e33;
}

fn fil_V_SmithGGXCorrelated(roughness_2: f32, NoV: f32, NoL: f32) -> f32 {
    var roughness_3: f32;
    var NoV_1: f32;
    var NoL_1: f32;
    var a2_: f32;
    var lambdaV: f32;
    var lambdaL: f32;
    var v: f32;

    roughness_3 = roughness_2;
    NoV_1 = NoV;
    NoL_1 = NoL;
    let _e6 = roughness_3;
    let _e7 = roughness_3;
    a2_ = (_e6 * _e7);
    let _e10 = NoL_1;
    let _e11 = NoV_1;
    let _e12 = a2_;
    let _e13 = NoV_1;
    let _e16 = NoV_1;
    let _e18 = a2_;
    lambdaV = (_e10 * sqrt((((_e11 - (_e12 * _e13)) * _e16) + _e18)));
    let _e23 = NoV_1;
    let _e24 = NoL_1;
    let _e25 = a2_;
    let _e26 = NoL_1;
    let _e29 = NoL_1;
    let _e31 = a2_;
    lambdaL = (_e23 * sqrt((((_e24 - (_e25 * _e26)) * _e29) + _e31)));
    let _e37 = lambdaV;
    let _e38 = lambdaL;
    v = (0.5f / max((_e37 + _e38), 0.0000077f));
    let _e44 = v;
    return _e44;
}

fn fil_V_Kelemen(LoH: f32) -> f32 {
    var LoH_1: f32;

    LoH_1 = LoH;
    let _e3 = LoH_1;
    let _e4 = LoH_1;
    return (0.25f / max((_e3 * _e4), 0.0000039f));
}

fn fil_F_Schlick(f0_: vec3<f32>, f90_: f32, VoH: f32) -> vec3<f32> {
    var f90_1: f32;
    var VoH_1: f32;

    f90_1 = f90_;
    VoH_1 = VoH;
    let _e5 = f90_1;
    let _e9 = VoH_1;
    let _e11 = fil_pow5_((1f - _e9));
    return (f0_ + ((vec3(_e5) - f0_) * _e11));
}

fn fil_F_Schlick_1(f0_1: f32, f90_2: f32, VoH_2: f32) -> f32 {
    var f0_2: f32;
    var f90_3: f32;
    var VoH_3: f32;

    f0_2 = f0_1;
    f90_3 = f90_2;
    VoH_3 = VoH_2;
    let _e6 = f0_2;
    let _e7 = f90_3;
    let _e8 = f0_2;
    let _e11 = VoH_3;
    let _e13 = fil_pow5_((1f - _e11));
    return (_e6 + ((_e7 - _e8) * _e13));
}

fn fil_Fd_Lambert() -> f32 {
    return 0.31830987f;
}

fn fil_Irradiance_SphericalHarmonics(n: vec3<f32>, iblSH: array<vec3<f32>, 9>) -> vec3<f32> {
    var sphericalHarmonics: vec3<f32>;

    sphericalHarmonics = iblSH[0];
    let _e5 = sphericalHarmonics;
    sphericalHarmonics = (_e5 + (((iblSH[1] * n.y) + (iblSH[2] * n.z)) + (iblSH[3] * n.x)));
    let _e21 = sphericalHarmonics;
    sphericalHarmonics = (_e21 + (((((iblSH[4] * (n.y * n.x)) + (iblSH[5] * (n.y * n.z))) + (iblSH[6] * (((3f * n.z) * n.z) - 1f))) + (iblSH[7] * (n.z * n.x))) + (iblSH[8] * ((n.x * n.x) - (n.y * n.y)))));
    let _e65 = sphericalHarmonics;
    return max(_e65, vec3(0f));
}

fn fil_getSpecularDominantDirection(n_1: vec3<f32>, r: vec3<f32>, roughness_4: f32) -> vec3<f32> {
    var roughness_5: f32;

    roughness_5 = roughness_4;
    let _e4 = roughness_5;
    let _e5 = roughness_5;
    return mix(r, n_1, vec3((_e4 * _e5)));
}

fn fil_refractedRaySolidSphere(shading_position: vec3<f32>, r_in: vec3<f32>, NoR_in: f32, sin2Theta_in: f32, etaIR: f32, etaRI: f32, thickness: f32, n_2: vec3<f32>, ray: ptr<function, fil_Refraction>) {
    var NoR_in_1: f32;
    var sin2Theta_in_1: f32;
    var etaIR_1: f32;
    var etaRI_1: f32;
    var thickness_1: f32;
    var k_1: f32;
    var rr: vec3<f32>;
    var NoR: f32;
    var d_1: f32;
    var n1_: vec3<f32>;

    NoR_in_1 = NoR_in;
    sin2Theta_in_1 = sin2Theta_in;
    etaIR_1 = etaIR;
    etaRI_1 = etaRI;
    thickness_1 = thickness;
    let _e15 = etaIR_1;
    let _e16 = etaIR_1;
    let _e18 = sin2Theta_in_1;
    k_1 = (1f - ((_e15 * _e16) * _e18));
    let _e22 = etaIR_1;
    let _e24 = etaIR_1;
    let _e25 = NoR_in_1;
    let _e27 = k_1;
    rr = ((_e22 * r_in) - (((_e24 * _e25) + sqrt(max(_e27, 0f))) * n_2));
    let _e35 = rr;
    NoR = dot(n_2, _e35);
    let _e38 = thickness_1;
    let _e39 = NoR;
    d_1 = (_e38 * -(_e39));
    let _e44 = d_1;
    (*ray).d = _e44;
    let _e46 = rr;
    let _e47 = d_1;
    (*ray).position = (shading_position + (_e46 * _e47));
    let _e50 = NoR;
    let _e51 = rr;
    n1_ = normalize(((_e50 * _e51) - (n_2 * 0.5f)));
    let _e59 = rr;
    let _e60 = n1_;
    let _e61 = etaRI_1;
    (*ray).direction = refract(_e59, _e60, _e61);
    return;
}

fn fil_SpecularAO_Lagarde(NoV_2: f32, visibility: f32, roughness_6: f32) -> f32 {
    var NoV_3: f32;
    var visibility_1: f32;
    var roughness_7: f32;

    NoV_3 = NoV_2;
    visibility_1 = visibility;
    roughness_7 = roughness_6;
    let _e6 = NoV_3;
    let _e7 = visibility_1;
    let _e11 = roughness_7;
    let _e19 = visibility_1;
    return clamp(((pow((_e6 + _e7), exp2(((-16f * _e11) - 1f))) - 1f) + _e19), 0f, 1f);
}

fn fil_gtaoMultiBounce(visibility_2: f32, albedo: vec3<f32>) -> vec3<f32> {
    var visibility_3: f32;
    var a_1: vec3<f32>;
    var b: vec3<f32>;
    var c: vec3<f32>;

    visibility_3 = visibility_2;
    a_1 = ((2.0404f * albedo) - vec3(0.3324f));
    b = ((-4.7951f * albedo) + vec3(0.6417f));
    c = ((2.7552f * albedo) + vec3(0.6903f));
    let _e22 = visibility_3;
    let _e24 = visibility_3;
    let _e25 = a_1;
    let _e27 = b;
    let _e29 = visibility_3;
    let _e31 = c;
    let _e33 = visibility_3;
    return max(vec3(_e22), (((((_e24 * _e25) + _e27) * _e29) + _e31) * _e33));
}

fn fil_PBRNeutralToneMapper(color: vec3<f32>) -> vec3<f32> {
    var color_1: vec3<f32>;
    var startCompression: f32 = 0.76f;
    var desaturation: f32 = 0.15f;
    var x_4: f32;
    var local: f32;
    var offset: f32;
    var peak: f32;
    var d_2: f32;
    var newPeak: f32;
    var g: f32;

    color_1 = color;
    let _e8 = color_1;
    let _e10 = color_1;
    let _e12 = color_1;
    x_4 = min(_e8.x, min(_e10.y, _e12.z));
    let _e17 = x_4;
    if (_e17 < 0.08f) {
        let _e20 = x_4;
        let _e22 = x_4;
        let _e24 = x_4;
        local = (_e20 - ((6.25f * _e22) * _e24));
    } else {
        local = 0.04f;
    }
    let _e29 = local;
    offset = _e29;
    let _e31 = color_1;
    let _e32 = offset;
    color_1 = (_e31 - vec3(_e32));
    let _e35 = color_1;
    let _e37 = color_1;
    let _e39 = color_1;
    peak = max(_e35.x, max(_e37.y, _e39.z));
    let _e44 = peak;
    let _e45 = startCompression;
    if (_e44 < _e45) {
        let _e47 = color_1;
        return _e47;
    }
    let _e49 = startCompression;
    d_2 = (1f - _e49);
    let _e53 = d_2;
    let _e54 = d_2;
    let _e56 = peak;
    let _e57 = d_2;
    let _e59 = startCompression;
    newPeak = (1f - ((_e53 * _e54) / ((_e56 + _e57) - _e59)));
    let _e64 = color_1;
    let _e65 = newPeak;
    let _e66 = peak;
    color_1 = (_e64 * (_e65 / _e66));
    let _e71 = desaturation;
    let _e72 = peak;
    let _e73 = newPeak;
    g = (1f - (1f / ((_e71 * (_e72 - _e73)) + 1f)));
    let _e81 = color_1;
    let _e82 = newPeak;
    let _e84 = g;
    return mix(_e81, vec3(_e82), vec3(_e84));
}

fn fil_dispersionIntegrate(s0_: vec3<f32>, s1_: vec3<f32>, s2_: vec3<f32>, s3_: vec3<f32>) -> vec3<f32> {
    var K0_: mat3x3<f32> = mat3x3<f32>(vec3<f32>(0.00581637f, 0.02312851f, 0.01689631f), vec3<f32>(-0.11782236f, 0.11316202f, 0.11098148f), vec3<f32>(-0.45422012f, 0.04493517f, 0.982498f));
    var K1_: mat3x3<f32> = mat3x3<f32>(vec3<f32>(0.14291704f, 0.10429778f, -0.01556522f), vec3<f32>(-0.27560148f, 0.5767854f, -0.06412244f), vec3<f32>(0.06839811f, 0.02732891f, 0.01602064f));
    var K2_: mat3x3<f32> = mat3x3<f32>(vec3<f32>(0.7010612f, -0.09440402f, -0.00241699f), vec3<f32>(0.29545674f, 0.29931852f, -0.04351961f), vec3<f32>(0.318844f, -0.05627069f, 0.00083808f));
    var K3_: mat3x3<f32> = mat3x3<f32>(vec3<f32>(0.15020522f, -0.03302213f, 0.00108589f), vec3<f32>(0.09796715f, 0.0107341f, -0.00333946f), vec3<f32>(0.06697807f, -0.01599341f, 0.00064333f));

    let _e72 = K0_;
    let _e74 = K1_;
    let _e77 = K2_;
    let _e80 = K3_;
    return max(((((_e72 * s0_) + (_e74 * s1_)) + (_e77 * s2_)) + (_e80 * s3_)), vec3(0f));
}
