/*{
  "ID": "import.ceil_bloom",
  "LABEL": "ceil_bloom",
  "STAGE": "pass",
  "DESCRIPTION": "PASSES の天井測り: 抽出 → 横 → 縦 → 合成",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "threshold", "LABEL": "Threshold", "TYPE": "float", "DEFAULT": 0.55, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "amount", "LABEL": "Amount", "TYPE": "float", "DEFAULT": 1.4, "MIN": 0.0, "MAX": 4.0 }
  ],
  "PASSES": [
    { "TARGET": "bright", "WIDTH": "$WIDTH/2", "HEIGHT": "$HEIGHT/2" },
    { "TARGET": "blurH", "WIDTH": "$WIDTH/2", "HEIGHT": "$HEIGHT/2" },
    { "TARGET": "blurV", "WIDTH": "$WIDTH/2", "HEIGHT": "$HEIGHT/2" },
    { }
  ]
}*/

void main() {
    vec2 uv = isf_FragNormCoord;
    if (PASSINDEX == 0) {
        vec4 c = IMG_THIS_PIXEL(inputImage);
        float l = dot(c.rgb, vec3(0.299, 0.587, 0.114));
        gl_FragColor = vec4(c.rgb * step(threshold, l), c.a);
    } else if (PASSINDEX == 1) {
        vec4 sum = vec4(0.0);
        for (int i = -4; i <= 4; i++) {
            sum += IMG_NORM_PIXEL(bright, uv + vec2(float(i) * 2.0 / RENDERSIZE.x, 0.0));
        }
        gl_FragColor = sum / 9.0;
    } else if (PASSINDEX == 2) {
        vec4 sum = vec4(0.0);
        for (int i = -4; i <= 4; i++) {
            sum += IMG_NORM_PIXEL(blurH, uv + vec2(0.0, float(i) * 2.0 / RENDERSIZE.y));
        }
        gl_FragColor = sum / 9.0;
    } else {
        vec4 base = IMG_THIS_PIXEL(inputImage);
        gl_FragColor = vec4(base.rgb + IMG_NORM_PIXEL(blurV, uv).rgb * amount, base.a);
    }
}
