/*{
  "ID": "motolii.xdog",
  "LABEL": "XDoG",
  "STAGE": "pass",
  "DESCRIPTION": "Ink lines and flat tones that follow the picture's own strokes (flow-based extended difference of Gaussians). Sketch, manga and cel looks without machine learning; keep the colour to lay the lines over the image",
  "FILTER": "linear",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "sigma", "LABEL": "Line Width", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.3, "MAX": 5.0, "SUBTYPE": "DISTANCE", "HERO": true },
    { "NAME": "sharpen", "LABEL": "Sharpen", "TYPE": "float", "DEFAULT": 20.0, "MIN": 0.0, "MAX": 100.0 },
    { "NAME": "threshold", "LABEL": "Threshold", "TYPE": "float", "DEFAULT": 0.45, "MIN": 0.0, "MAX": 1.5 },
    { "NAME": "softness", "LABEL": "Softness", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.005, "MAX": 1.0 },
    { "NAME": "flow", "LABEL": "Flow Length", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 10.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "keep", "LABEL": "Keep Colour", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ],
  "PASSES": [
    { "TARGET": "tensor", "FLOAT": true },
    { "TARGET": "across", "FLOAT": true },
    { "TARGET": "smoothed", "FLOAT": true },
    { "TARGET": "edges", "FLOAT": true },
    { }
  ]
}*/

// Winnemöller, Kyprianidis, Olsen「XDoG: An eXtended difference-of-Gaussians compendium」(2012) の流れに沿う版。
// 構造テンソルから流れ(接線)を取り、勾配の向きに DoG((1 + p)·Gσ − p·Gkσ)、接線に沿って積分し、柔らかい閾値で線と面にする。

vec3 straight(vec4 c) { return c.a > 1e-5 ? c.rgb / c.a : vec3(0.0); }
// 閾値は見た目の明るさに当てる(効果の列は線形なので、そのままだと中間調が全部暗い側へ落ちる)。
float luma(vec4 c) { return pow(max(dot(straight(c), vec3(0.2126, 0.7152, 0.0722)), 0.0), 1.0 / 2.2); }

const float GAUSS[9] = float[9](0.028, 0.066, 0.124, 0.180, 0.204, 0.180, 0.124, 0.066, 0.028);
const float K = 1.6;

vec2 tangent_at(vec3 g) {
    float e = g.x, f = g.z, h = g.y;
    float root = sqrt(max((e - h) * (e - h) + 4.0 * f * f, 0.0));
    float lambda2 = 0.5 * (e + h - root);
    // 固有ベクトルの式は 2 通りあり、軸に沿う縁では片方が 0 に潰れる。長い方を取る。
    vec2 t1 = vec2(lambda2 - h, f);
    vec2 t2 = vec2(f, lambda2 - e);
    vec2 t = dot(t1, t1) >= dot(t2, t2) ? t1 : t2;
    return length(t) > 1e-7 ? normalize(t) : vec2(1.0, 0.0);
}

void main() {
    vec2 uv = isf_FragNormCoord;
    vec2 texel = 1.0 / RENDERSIZE;
    if (PASSINDEX == 0) {
        float gx = (luma(IMG_NORM_PIXEL(inputImage, uv + vec2(texel.x, -texel.y))) + 2.0 * luma(IMG_NORM_PIXEL(inputImage, uv + vec2(texel.x, 0.0))) + luma(IMG_NORM_PIXEL(inputImage, uv + texel))
                  - luma(IMG_NORM_PIXEL(inputImage, uv - texel)) - 2.0 * luma(IMG_NORM_PIXEL(inputImage, uv - vec2(texel.x, 0.0))) - luma(IMG_NORM_PIXEL(inputImage, uv + vec2(-texel.x, texel.y)))) / 4.0;
        float gy = (luma(IMG_NORM_PIXEL(inputImage, uv + vec2(-texel.x, texel.y))) + 2.0 * luma(IMG_NORM_PIXEL(inputImage, uv + vec2(0.0, texel.y))) + luma(IMG_NORM_PIXEL(inputImage, uv + texel))
                  - luma(IMG_NORM_PIXEL(inputImage, uv - texel)) - 2.0 * luma(IMG_NORM_PIXEL(inputImage, uv - vec2(0.0, texel.y))) - luma(IMG_NORM_PIXEL(inputImage, uv + vec2(texel.x, -texel.y)))) / 4.0;
        gl_FragColor = vec4(gx * gx, gy * gy, gx * gy, 1.0);
    } else if (PASSINDEX == 1 || PASSINDEX == 2) {
        vec2 axis = PASSINDEX == 1 ? vec2(texel.x, 0.0) : vec2(0.0, texel.y);
        vec4 sum = vec4(0.0);
        for (int k = 0; k < 9; k++) {
            vec2 at = uv + axis * float(k - 4);
            sum += (PASSINDEX == 1 ? IMG_NORM_PIXEL(tensor, at) : IMG_NORM_PIXEL(across, at)) * GAUSS[k];
        }
        gl_FragColor = sum;
    } else if (PASSINDEX == 3) {
        // 勾配の向き(接線に直交)に 1 次元の DoG。
        vec2 t = tangent_at(IMG_NORM_PIXEL(smoothed, uv).rgb);
        vec2 n = vec2(-t.y, t.x);
        float s1 = max(sigma, 0.3), s2 = s1 * K;
        int reach = int(ceil(3.0 * s2));
        float a = 0.0, b = 0.0, wa = 0.0, wb = 0.0;
        for (int i = -16; i <= 16; i++) {
            if (i < -reach || i > reach) continue;
            float x = float(i);
            float l = luma(IMG_NORM_PIXEL(inputImage, uv + n * x * texel));
            float g1 = exp(-x * x / (2.0 * s1 * s1));
            float g2 = exp(-x * x / (2.0 * s2 * s2));
            a += l * g1; wa += g1;
            b += l * g2; wb += g2;
        }
        float dog = (1.0 + sharpen) * (a / wa) - sharpen * (b / wb);
        gl_FragColor = vec4(dog, 0.0, 0.0, 1.0);
    } else {
        // 接線に沿って両向きに辿りながら平均する(line integral convolution)。
        float sm = max(flow, 0.0);
        int steps = int(ceil(2.0 * sm));
        float sum = IMG_NORM_PIXEL(edges, uv).r;
        float total = 1.0;
        for (int dir = 0; dir < 2; dir++) {
            vec2 p = uv;
            vec2 prev = tangent_at(IMG_NORM_PIXEL(smoothed, uv).rgb) * (dir == 0 ? 1.0 : -1.0);
            for (int k = 1; k <= 20; k++) {
                if (k > steps) break;
                vec2 t = tangent_at(IMG_NORM_PIXEL(smoothed, p).rgb);
                if (dot(t, prev) < 0.0) t = -t;
                p += t * texel;
                prev = t;
                float w = exp(-float(k * k) / (2.0 * sm * sm + 1e-4));
                sum += IMG_NORM_PIXEL(edges, p).r * w;
                total += w;
            }
        }
        float d = sum / total;
        float tone = d >= threshold ? 1.0 : 1.0 + tanh((d - threshold) / max(softness, 1e-3));
        vec4 c = IMG_NORM_PIXEL(inputImage, uv);
        vec3 ink = mix(vec3(tone), straight(c) * tone, clamp(keep, 0.0, 1.0));
        gl_FragColor = vec4(ink * c.a, c.a);
    }
}
