/*{
  "ID": "motolii.pixel_motion_blur",
  "LABEL": "Pixel Motion Blur",
  "STAGE": "pass",
  "DESCRIPTION": "Motion blur from the picture itself: estimates how each pixel moved since the previous frame (pyramid Lucas-Kanade optical flow) and smears it along that motion, like a camera shutter. Works on footage, pre-rendered clips and anything without keyframed motion",
  "FILTER": "linear",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "previous", "TYPE": "image", "TIME_OFFSET_FRAMES": -1 },
    { "NAME": "shutter", "LABEL": "Shutter Angle", "TYPE": "float", "DEFAULT": 180.0, "MIN": 0.0, "MAX": 720.0, "HERO": true },
    { "NAME": "samples", "LABEL": "Samples", "TYPE": "float", "DEFAULT": 16.0, "MIN": 2.0, "MAX": 64.0 },
    { "NAME": "limit", "LABEL": "Max Motion", "TYPE": "float", "DEFAULT": 0.15, "MIN": 0.0, "MAX": 1.0, "ADVANCED": true }
  ],
  "PASSES": [
    { "TARGET": "now1", "WIDTH": "$WIDTH/2", "HEIGHT": "$HEIGHT/2", "FLOAT": true },
    { "TARGET": "was1", "WIDTH": "$WIDTH/2", "HEIGHT": "$HEIGHT/2", "FLOAT": true },
    { "TARGET": "now2", "WIDTH": "$WIDTH/4", "HEIGHT": "$HEIGHT/4", "FLOAT": true },
    { "TARGET": "was2", "WIDTH": "$WIDTH/4", "HEIGHT": "$HEIGHT/4", "FLOAT": true },
    { "TARGET": "now3", "WIDTH": "$WIDTH/8", "HEIGHT": "$HEIGHT/8", "FLOAT": true },
    { "TARGET": "was3", "WIDTH": "$WIDTH/8", "HEIGHT": "$HEIGHT/8", "FLOAT": true },
    { "TARGET": "now4", "WIDTH": "$WIDTH/16", "HEIGHT": "$HEIGHT/16", "FLOAT": true },
    { "TARGET": "was4", "WIDTH": "$WIDTH/16", "HEIGHT": "$HEIGHT/16", "FLOAT": true },
    { "TARGET": "flow4", "WIDTH": "$WIDTH/16", "HEIGHT": "$HEIGHT/16", "FLOAT": true },
    { "TARGET": "flow3", "WIDTH": "$WIDTH/8", "HEIGHT": "$HEIGHT/8", "FLOAT": true },
    { "TARGET": "flow2", "WIDTH": "$WIDTH/4", "HEIGHT": "$HEIGHT/4", "FLOAT": true },
    { "TARGET": "flow1", "WIDTH": "$WIDTH/2", "HEIGHT": "$HEIGHT/2", "FLOAT": true },
    { }
  ]
}*/

// 流れ(flow)は uv の単位で持つ: 段の寸法に依らないので、粗い段の答えをそのまま細かい段の初期値にできる。
// 動き v は「前のコマの x - v に在った物が、今 x に在る」(now(x) = was(x - v))。

float gray(vec4 c) { return dot(c.rgb, vec3(0.2126, 0.7152, 0.0722)) + 0.25 * c.a; }

// naga の GLSL は sampler を関数の引数に取れないので、段ごとの Lucas-Kanade をマクロで 4 つ作る。
// NOW・WAS は同じ段の (灰, α)、guess は粗い段の答え(uv)。
#define LUCAS_KANADE(NAME, NOW, WAS) \
vec2 NAME(vec2 guess) { \
    vec2 uv = isf_FragNormCoord; \
    vec2 texel = 1.0 / RENDERSIZE; \
    vec2 v = guess; \
    for (int iteration = 0; iteration < 3; iteration++) { \
        float gxx = 0.0; float gxy = 0.0; float gyy = 0.0; float bx = 0.0; float by = 0.0; float presence = 0.0; \
        for (int j = -2; j <= 2; j++) { \
            for (int i = -2; i <= 2; i++) { \
                vec2 at = uv + vec2(float(i), float(j)) * texel; \
                vec2 here = IMG_NORM_PIXEL(NOW, at).rg; \
                float ix = (IMG_NORM_PIXEL(NOW, at + vec2(texel.x, 0.0)).r - IMG_NORM_PIXEL(NOW, at - vec2(texel.x, 0.0)).r) * 0.5; \
                float iy = (IMG_NORM_PIXEL(NOW, at + vec2(0.0, texel.y)).r - IMG_NORM_PIXEL(NOW, at - vec2(0.0, texel.y)).r) * 0.5; \
                vec2 there = IMG_NORM_PIXEL(WAS, at - v).rg; \
                float e = there.r - here.r; \
                gxx += ix * ix; gxy += ix * iy; gyy += iy * iy; \
                bx += ix * e; by += iy * e; \
                presence += min(here.g, there.g); \
            } \
        } \
        if (presence < 1.0) return vec2(0.0); \
        float det = gxx * gyy - gxy * gxy; \
        float trace = gxx + gyy; \
        if (det < 1e-7 || det < 0.02 * trace * trace) break; \
        vec2 step_px = vec2(gyy * bx - gxy * by, gxx * by - gxy * bx) / det; \
        v += step_px * texel; \
        if (length(step_px) < 0.01) break; \
    } \
    float reach = max(limit, 1e-4); \
    return length(v) > reach ? v * (reach / length(v)) : v; \
}

// 前のコマに層が居なかった所(入点の直後・画面外から入った所)は動きを作らない(presence)。
// 平らな所・線に沿う所(開口問題)は解が決まらないので、粗い段の答えのままにする(det)。
LUCAS_KANADE(solve4, now4, was4)
LUCAS_KANADE(solve3, now3, was3)
LUCAS_KANADE(solve2, now2, was2)
LUCAS_KANADE(solve1, now1, was1)

void main() {
    vec2 uv = isf_FragNormCoord;
    if (PASSINDEX == 0) { vec4 c = IMG_NORM_PIXEL(inputImage, uv); gl_FragColor = vec4(gray(c), c.a, 0.0, 1.0); }
    else if (PASSINDEX == 1) { vec4 c = IMG_NORM_PIXEL(previous, uv); gl_FragColor = vec4(gray(c), c.a, 0.0, 1.0); }
    else if (PASSINDEX == 2) { gl_FragColor = IMG_NORM_PIXEL(now1, uv); }
    else if (PASSINDEX == 3) { gl_FragColor = IMG_NORM_PIXEL(was1, uv); }
    else if (PASSINDEX == 4) { gl_FragColor = IMG_NORM_PIXEL(now2, uv); }
    else if (PASSINDEX == 5) { gl_FragColor = IMG_NORM_PIXEL(was2, uv); }
    else if (PASSINDEX == 6) { gl_FragColor = IMG_NORM_PIXEL(now3, uv); }
    else if (PASSINDEX == 7) { gl_FragColor = IMG_NORM_PIXEL(was3, uv); }
    else if (PASSINDEX == 8) { gl_FragColor = vec4(solve4(vec2(0.0)), 0.0, 1.0); }
    else if (PASSINDEX == 9) { gl_FragColor = vec4(solve3(IMG_NORM_PIXEL(flow4, uv).rg), 0.0, 1.0); }
    else if (PASSINDEX == 10) { gl_FragColor = vec4(solve2(IMG_NORM_PIXEL(flow3, uv).rg), 0.0, 1.0); }
    else if (PASSINDEX == 11) { gl_FragColor = vec4(solve1(IMG_NORM_PIXEL(flow2, uv).rg), 0.0, 1.0); }
    else {
        // シャッターが開いている間(コマの間隔 × 角度 / 360)に動いた道のりを、今のコマを中心に平均する。
        vec2 v = IMG_NORM_PIXEL(flow1, uv).rg * (shutter / 360.0);
        int n = int(clamp(samples, 2.0, 64.0));
        vec4 sum = vec4(0.0);
        for (int k = 0; k < 64; k++) {
            if (k >= n) break;
            float s = (float(k) + 0.5) / float(n) - 0.5;
            sum += IMG_NORM_PIXEL(inputImage, uv + v * s);
        }
        gl_FragColor = sum / float(n);
    }
}
