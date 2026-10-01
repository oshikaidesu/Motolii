/*{
  "ID": "motolii.depth_map",
  "LABEL": "Depth Map",
  "STAGE": "pass",
  "DESCRIPTION": "Guesses depth from the picture without machine learning: lower in frame is nearer, hazy (bright and pale) is farther, sharp detail is nearer. White is near, black is far. Feed it to effects that take a layer",
  "FILTER": "linear",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "height", "LABEL": "Height", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0, "HERO": true },
    { "NAME": "haze", "LABEL": "Haze", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "focus", "LABEL": "Focus", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "smoothness", "LABEL": "Smoothness", "TYPE": "float", "DEFAULT": 4.0, "MIN": 0.0, "MAX": 16.0 },
    { "NAME": "contrast", "LABEL": "Contrast", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "invert", "LABEL": "Invert", "TYPE": "bool", "DEFAULT": false }
  ],
  "PASSES": [
    { "TARGET": "cue", "WIDTH": "$WIDTH/4", "HEIGHT": "$HEIGHT/4", "FLOAT": true },
    { "TARGET": "across", "WIDTH": "$WIDTH/4", "HEIGHT": "$HEIGHT/4", "FLOAT": true },
    { "TARGET": "down", "WIDTH": "$WIDTH/4", "HEIGHT": "$HEIGHT/4", "FLOAT": true },
    { }
  ]
}*/

// 手掛かりは 3 つ(単眼の奥行きの古典的な手掛かり): 画面の下ほど近い、かすむ(明るく色が薄い)ほど遠い、
// 細部がくっきりしているほど近い。1/4 の寸法で混ぜてぼかし、元の明るさを手掛かりに縁へ戻す(joint bilateral)。

vec3 straight(vec4 c) { return c.a > 1e-5 ? c.rgb / c.a : vec3(0.0); }
float luma(vec3 c) { return pow(max(dot(c, vec3(0.2126, 0.7152, 0.0722)), 0.0), 1.0 / 2.2); }

const float GAUSS[9] = float[9](0.028, 0.066, 0.124, 0.180, 0.204, 0.180, 0.124, 0.066, 0.028);

void main() {
    vec2 uv = isf_FragNormCoord;
    if (PASSINDEX == 0) {
        vec2 texel = 1.0 / (RENDERSIZE * 4.0);
        vec4 c = IMG_NORM_PIXEL(inputImage, uv);
        vec3 rgb = straight(c);
        float l = luma(rgb);
        float saturation = max(rgb.r, max(rgb.g, rgb.b)) - min(rgb.r, min(rgb.g, rgb.b));
        float detail = 0.0;
        for (int j = -1; j <= 2; j++) {
            for (int i = -1; i <= 2; i++) {
                vec2 at = uv + vec2(float(i) - 0.5, float(j) - 0.5) * texel;
                float here = luma(straight(IMG_NORM_PIXEL(inputImage, at)));
                float right = luma(straight(IMG_NORM_PIXEL(inputImage, at + vec2(texel.x, 0.0))));
                float up = luma(straight(IMG_NORM_PIXEL(inputImage, at + vec2(0.0, texel.y))));
                detail += abs(right - here) + abs(up - here);
            }
        }
        detail /= 16.0;
        float hazy = l * (1.0 - clamp(saturation, 0.0, 1.0));
        gl_FragColor = vec4(hazy, detail / (detail + 0.02), l, c.a);
    } else if (PASSINDEX == 1 || PASSINDEX == 2) {
        vec2 axis = PASSINDEX == 1 ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
        vec2 step_uv = axis * max(smoothness, 0.0) * 0.5 / RENDERSIZE;
        vec4 sum = vec4(0.0);
        for (int k = 0; k < 9; k++) {
            vec2 at = uv + step_uv * float(k - 4);
            sum += (PASSINDEX == 1 ? IMG_NORM_PIXEL(cue, at) : IMG_NORM_PIXEL(across, at)) * GAUSS[k];
        }
        gl_FragColor = sum;
    } else {
        vec4 c = IMG_NORM_PIXEL(inputImage, uv);
        float l = luma(straight(c));
        vec2 texel = 4.0 / RENDERSIZE;
        vec4 cue_here = vec4(0.0);
        float total = 0.0;
        for (int j = -1; j <= 1; j++) {
            for (int i = -1; i <= 1; i++) {
                vec4 s = IMG_NORM_PIXEL(down, uv + vec2(float(i), float(j)) * texel);
                float w = exp(-float(i * i + j * j) * 0.5) * exp(-pow((s.b - l) * 12.0, 2.0));
                cue_here += s * w;
                total += w;
            }
        }
        cue_here /= max(total, 1e-5);
        // ISF の uv は下端が 0。
        float d = height * (1.0 - uv.y) + focus * cue_here.g - haze * cue_here.r;
        float span = max(height + focus + haze, 1e-3);
        d = clamp(0.5 + (d / span - 0.5 + haze / span * 0.5) * contrast, 0.0, 1.0);
        if (invert > 0.5) d = 1.0 - d;
        gl_FragColor = vec4(vec3(d), 1.0) * c.a;
    }
}
