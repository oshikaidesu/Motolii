/*{
  "ID": "motolii.kuwahara",
  "LABEL": "Kuwahara",
  "STAGE": "pass",
  "DESCRIPTION": "Painterly smoothing that keeps edges: flattens texture into brush-like patches stretched along the picture's own strokes (anisotropic Kuwahara with polynomial weights). An oil-paint look without machine learning",
  "FILTER": "linear",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "radius", "LABEL": "Radius", "TYPE": "float", "DEFAULT": 6.0, "MIN": 1.0, "MAX": 12.0, "SUBTYPE": "DISTANCE", "HERO": true },
    { "NAME": "anisotropy", "LABEL": "Stretch", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "sharpness", "LABEL": "Sharpness", "TYPE": "float", "DEFAULT": 8.0, "MIN": 1.0, "MAX": 18.0 },
    { "NAME": "hardness", "LABEL": "Hardness", "TYPE": "float", "DEFAULT": 8.0, "MIN": 1.0, "MAX": 100.0, "ADVANCED": true }
  ],
  "PASSES": [
    { "TARGET": "tensor", "FLOAT": true },
    { "TARGET": "across", "FLOAT": true },
    { "TARGET": "smoothed", "FLOAT": true },
    { }
  ]
}*/

// Kyprianidis, Kang, Döllner「Anisotropic Kuwahara Filtering with Polynomial Weighting Functions」(2010)。
// 構造テンソル → ぼかす → 流れの向きと異方性で楕円の窓を作り、8 つの扇の平均のうち、ばらつきの小さい扇ほど重く混ぜる。

vec3 straight(vec4 c) { return c.a > 1e-5 ? c.rgb / c.a : vec3(0.0); }

const float GAUSS[9] = float[9](0.028, 0.066, 0.124, 0.180, 0.204, 0.180, 0.124, 0.066, 0.028);
const float PI = 3.14159265;

void main() {
    vec2 uv = isf_FragNormCoord;
    vec2 texel = 1.0 / RENDERSIZE;
    if (PASSINDEX == 0) {
        vec3 gx = (
            -1.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(-texel.x, -texel.y))) +
            -2.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(-texel.x, 0.0))) +
            -1.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(-texel.x, texel.y))) +
             1.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(texel.x, -texel.y))) +
             2.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(texel.x, 0.0))) +
             1.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(texel.x, texel.y)))
        ) / 4.0;
        vec3 gy = (
            -1.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(-texel.x, -texel.y))) +
            -2.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(0.0, -texel.y))) +
            -1.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(texel.x, -texel.y))) +
             1.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(-texel.x, texel.y))) +
             2.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(0.0, texel.y))) +
             1.0 * straight(IMG_NORM_PIXEL(inputImage, uv + vec2(texel.x, texel.y)))
        ) / 4.0;
        gl_FragColor = vec4(dot(gx, gx), dot(gy, gy), dot(gx, gy), 1.0);
    } else if (PASSINDEX == 1 || PASSINDEX == 2) {
        vec2 axis = PASSINDEX == 1 ? vec2(texel.x, 0.0) : vec2(0.0, texel.y);
        vec4 sum = vec4(0.0);
        for (int k = 0; k < 9; k++) {
            vec2 at = uv + axis * float(k - 4);
            sum += (PASSINDEX == 1 ? IMG_NORM_PIXEL(tensor, at) : IMG_NORM_PIXEL(across, at)) * GAUSS[k];
        }
        gl_FragColor = sum;
    } else {
        vec3 g = IMG_NORM_PIXEL(smoothed, uv).rgb;
        float e = g.x, f = g.z, h = g.y;
        float root = sqrt(max((e - h) * (e - h) + 4.0 * f * f, 0.0));
        float lambda1 = 0.5 * (e + h + root);
        float lambda2 = 0.5 * (e + h - root);
        // 勾配の向き(大きい固有値の固有ベクトル)。式は 2 通りあり、軸に沿う縁では片方が 0 に潰れるので長い方。
        vec2 t1 = vec2(lambda1 - h, f);
        vec2 t2 = vec2(f, lambda1 - e);
        vec2 t = dot(t1, t1) >= dot(t2, t2) ? t1 : t2;
        t = length(t) > 1e-7 ? normalize(t) : vec2(1.0, 0.0);
        float phi = -atan(t.y, t.x);
        float A = (lambda1 + lambda2 > 1e-7) ? (lambda1 - lambda2) / (lambda1 + lambda2) : 0.0;
        float alpha = max(1.0 / max(anisotropy, 1e-3), 1e-3);
        float a = radius * clamp((alpha + A) / alpha, 0.1, 2.0);
        float b = radius * clamp(alpha / (alpha + A), 0.1, 2.0);
        float cos_phi = cos(phi), sin_phi = sin(phi);
        mat2 R = mat2(cos_phi, -sin_phi, sin_phi, cos_phi);
        mat2 S = mat2(0.5 / a, 0.0, 0.0, 0.5 / b);
        mat2 SR = S * R;
        int max_x = int(ceil(sqrt(a * a * cos_phi * cos_phi + b * b * sin_phi * sin_phi)));
        int max_y = int(ceil(sqrt(a * a * sin_phi * sin_phi + b * b * cos_phi * cos_phi)));
        max_x = min(max_x, 24);
        max_y = min(max_y, 24);

        float zeta = 2.0 / max(radius, 1.0);
        float zero_cross = 0.58;
        float sin_zero = sin(zero_cross);
        float eta = (zeta + cos(zero_cross)) / (sin_zero * sin_zero);

        vec4 m[8];
        vec3 s[8];
        for (int k = 0; k < 8; k++) { m[k] = vec4(0.0); s[k] = vec3(0.0); }

        for (int y = -24; y <= 24; y++) {
            if (y < -max_y || y > max_y) continue;
            for (int x = -24; x <= 24; x++) {
                if (x < -max_x || x > max_x) continue;
                vec2 offset = vec2(float(x), float(y));
                vec2 v = SR * offset;
                if (dot(v, v) > 0.25) continue;
                vec3 c = straight(IMG_NORM_PIXEL(inputImage, uv + offset * texel));
                float w[8];
                float sum = 0.0;
                float vxx = zeta - eta * v.x * v.x;
                float vyy = zeta - eta * v.y * v.y;
                float z = max(0.0, v.y + vxx); w[0] = z * z; sum += w[0];
                z = max(0.0, -v.x + vyy); w[2] = z * z; sum += w[2];
                z = max(0.0, -v.y + vxx); w[4] = z * z; sum += w[4];
                z = max(0.0, v.x + vyy); w[6] = z * z; sum += w[6];
                vec2 d = 0.70710678 * vec2(v.x - v.y, v.x + v.y);
                vxx = zeta - eta * d.x * d.x;
                vyy = zeta - eta * d.y * d.y;
                z = max(0.0, d.y + vxx); w[1] = z * z; sum += w[1];
                z = max(0.0, -d.x + vyy); w[3] = z * z; sum += w[3];
                z = max(0.0, -d.y + vxx); w[5] = z * z; sum += w[5];
                z = max(0.0, d.x + vyy); w[7] = z * z; sum += w[7];
                float gauss = exp(-3.125 * dot(v, v)) / max(sum, 1e-6);
                for (int k = 0; k < 8; k++) {
                    float wk = w[k] * gauss;
                    m[k] += vec4(c * wk, wk);
                    s[k] += c * c * wk;
                }
            }
        }

        vec3 color = vec3(0.0);
        float total = 0.0;
        for (int k = 0; k < 8; k++) {
            if (m[k].w <= 1e-6) continue;
            vec3 mean = m[k].rgb / m[k].w;
            vec3 variance = abs(s[k] / m[k].w - mean * mean);
            float sigma2 = variance.r + variance.g + variance.b;
            float weight = 1.0 / (1.0 + pow(hardness * 1000.0 * sigma2, 0.5 * sharpness));
            color += mean * weight;
            total += weight;
        }
        float coverage = IMG_NORM_PIXEL(inputImage, uv).a;
        vec3 straight_out = total > 0.0 ? color / total : straight(IMG_NORM_PIXEL(inputImage, uv));
        gl_FragColor = vec4(straight_out * coverage, coverage);
    }
}
