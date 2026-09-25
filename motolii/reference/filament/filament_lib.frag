#version 450
// Filament shading functions, copied verbatim from google/filament@41f996de8fcc2d6b60b73159aa1bc44a05a40700
// (shaders/src/*.fs, Apache-2.0), wrapped in a stub entry point so naga's GLSL frontend emits them.
// Only the few lines marked GLUE are not Filament's.

// GLUE: common_math.glsl / common_defines.glsl pieces these functions use
#define PI 3.14159265359
#define MEDIUMP_FLT_MAX 65504.0
#define saturate(x) clamp(x, 0.0, 1.0)
#define PREVENT_DIV0(n, d, magic) ((n) / max(d, magic))
float pow5(float x) { float x2 = x * x; return x2 * x2 * x; }
float sq(float x) { return x * x; }

// ---- surface_brdf.fs ----
float D_GGX(float roughness, float NoH, const vec3 h) {
    float oneMinusNoHSquared = 1.0 - NoH * NoH;
    float a = NoH * roughness;
    float k = min(roughness / (oneMinusNoHSquared + a * a), 453.5);
    float d = k * (k * (1.0 / PI));
    return d;
}

float V_SmithGGXCorrelated(float roughness, float NoV, float NoL) {
    float a2 = roughness * roughness;
    float lambdaV = NoL * sqrt((NoV - a2 * NoV) * NoV + a2);
    float lambdaL = NoV * sqrt((NoL - a2 * NoL) * NoL + a2);
    float v = PREVENT_DIV0(0.5, lambdaV + lambdaL, 0.0000077);
    return v;
}

float V_Kelemen(float LoH) {
    return PREVENT_DIV0(0.25, LoH * LoH, 0.0000039);
}

vec3 F_Schlick(const vec3 f0, float f90, float VoH) {
    return f0 + (f90 - f0) * pow5(1.0 - VoH);
}

float F_Schlick(float f0, float f90, float VoH) {
    return f0 + (f90 - f0) * pow5(1.0 - VoH);
}

float Fd_Lambert() {
    return 1.0 / PI;
}

// ---- surface_light_indirect.fs ----
// GLUE: frameUniforms.iblSH -> a parameter array
vec3 Irradiance_SphericalHarmonics(const vec3 n, const vec3 iblSH[9]) {
    vec3 sphericalHarmonics = iblSH[0];
    sphericalHarmonics +=
              iblSH[1] * (n.y)
            + iblSH[2] * (n.z)
            + iblSH[3] * (n.x);
    sphericalHarmonics +=
              iblSH[4] * (n.y * n.x)
            + iblSH[5] * (n.y * n.z)
            + iblSH[6] * (3.0 * n.z * n.z - 1.0)
            + iblSH[7] * (n.z * n.x)
            + iblSH[8] * (n.x * n.x - n.y * n.y);
    return max(sphericalHarmonics, 0.0);
}

vec3 getSpecularDominantDirection(const vec3 n, const vec3 r, float roughness) {
    return mix(r, n, roughness * roughness);
}

struct Refraction {
    vec3 position;
    vec3 direction;
    float d;
};

// GLUE: shading_position (a Filament global) -> parameter
void refractedRaySolidSphere(const vec3 shading_position, const vec3 r_in, float NoR_in, float sin2Theta_in,
        float etaIR, float etaRI, float thickness, const vec3 n, out Refraction ray) {
    float k = 1.0 - etaIR * etaIR * sin2Theta_in;
    vec3 rr = etaIR * r_in - (etaIR * NoR_in + sqrt(max(k, 0.0))) * n;
    float NoR = dot(n, rr);
    float d = thickness * -NoR;
    ray.d = d;
    ray.position = shading_position + rr * d;
    vec3 n1 = normalize(NoR * rr - n * 0.5);
    ray.direction = refract(rr, n1, etaRI);
}

// ---- surface_ambient_occlusion.fs ----
float SpecularAO_Lagarde(float NoV, float visibility, float roughness) {
    return saturate(pow(NoV + visibility, exp2(-16.0 * roughness - 1.0)) - 1.0 + visibility);
}

vec3 gtaoMultiBounce(float visibility, const vec3 albedo) {
    vec3 a =  2.0404 * albedo - 0.3324;
    vec3 b = -4.7951 * albedo + 0.6417;
    vec3 c =  2.7552 * albedo + 0.6903;
    return max(vec3(visibility), ((visibility * a + b) * visibility + c) * visibility);
}

// ---- filament/src/ToneMapper.cpp PBRNeutralToneMapper (C++ -> GLSL, same statements) ----
vec3 PBRNeutralToneMapper(vec3 color) {
    const float startCompression = 0.8 - 0.04;
    const float desaturation = 0.15;
    float x = min(color.r, min(color.g, color.b));
    float offset = x < 0.08 ? x - 6.25 * x * x : 0.04;
    color -= offset;
    float peak = max(color.r, max(color.g, color.b));
    if (peak < startCompression) return color;
    const float d = 1.0 - startCompression;
    float newPeak = 1.0 - d * d / (peak + d - startCompression);
    color *= newPeak / peak;
    float g = 1.0 - 1.0 / (desaturation * (peak - newPeak) + 1.0);
    return mix(color, vec3(newPeak), g);
}

// ---- surface_light_indirect.fs calculateDispersion(): the spectral integration ----
vec3 dispersionIntegrate(const vec3 s0, const vec3 s1, const vec3 s2, const vec3 s3) {
    const mat3 K0 = mat3(
         0.00581637, 0.02312851, 0.01689631,
        -0.11782236, 0.11316202, 0.11098148,
        -0.45422013, 0.04493517, 0.98249798
    ); // 486.1nm
    const mat3 K1 = mat3(
         0.14291703, 0.10429778, -0.01556522,
        -0.27560148, 0.57678541, -0.06412244,
         0.06839811, 0.02732891,  0.01602064
    ); // 546.1nm
    const mat3 K2 = mat3(
        0.70106120, -0.09440402, -0.00241699,
        0.29545674,  0.29931852, -0.04351961,
        0.31884400, -0.05627069,  0.00083808
    ); // 589.3nm
    const mat3 K3 = mat3(
        0.15020522, -0.03302213,  0.00108589,
        0.09796715,  0.01073410, -0.00333946,
        0.06697807, -0.01599341,  0.00064333
    ); // 656.3nm
    return max(K0 * s0 + K1 * s1 + K2 * s2 + K3 * s3, 0.0);
}

// GLUE: stub entry point that keeps every function alive for naga
layout(location = 0) in vec3 v_in;
layout(location = 0) out vec4 o_color;
void main() {
    vec3 sh[9] = vec3[9](v_in, v_in, v_in, v_in, v_in, v_in, v_in, v_in, v_in);
    Refraction ray;
    refractedRaySolidSphere(v_in, v_in, v_in.x, v_in.y, 0.6, 1.5, 1.0, v_in, ray);
    float s = D_GGX(v_in.x, v_in.y, v_in) + V_SmithGGXCorrelated(v_in.x, v_in.y, v_in.z) + V_Kelemen(v_in.x)
        + F_Schlick(0.04, 1.0, v_in.x) + Fd_Lambert() + SpecularAO_Lagarde(v_in.x, v_in.y, v_in.z);
    vec3 c = F_Schlick(v_in, 1.0, v_in.x) + Irradiance_SphericalHarmonics(v_in, sh)
        + getSpecularDominantDirection(v_in, v_in, s) + gtaoMultiBounce(s, v_in) + PBRNeutralToneMapper(v_in)
        + dispersionIntegrate(v_in, v_in, v_in, v_in) + ray.position + ray.direction * ray.d;
    o_color = vec4(c, s);
}
