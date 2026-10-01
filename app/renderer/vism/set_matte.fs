/*{
  "ID": "motolii.set_matte",
  "LABEL": "Set Matte",
  "STAGE": "pass",
  "DESCRIPTION": "Cut this layer by another layer's alpha or luminance (AE の Set Matte の型)",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "matte", "TYPE": "image", "LAYER": "layer" },
    { "NAME": "layer", "LABEL": "Layer", "TYPE": "layer", "HERO": true },
    { "NAME": "channel", "LABEL": "Use", "TYPE": "long", "DEFAULT": 0, "LABELS": ["Alpha", "Luminance", "Inverted Alpha", "Inverted Luminance"] },
    { "NAME": "stretch", "LABEL": "Stretch", "TYPE": "bool", "DEFAULT": true }
  ]
}*/

// 列は乗算済み線形。切るのは 4 成分まとめて掛けるだけ。
float coverage(vec4 m) {
    float lum = dot(m.rgb, vec3(0.2126, 0.7152, 0.0722));
    if (channel < 0.5) return m.a;
    if (channel < 1.5) return lum;
    if (channel < 2.5) return 1.0 - m.a;
    return 1.0 - lum;
}

void main() {
    vec4 src = IMG_THIS_PIXEL(inputImage);
    // 相手の絵は自分の絵の寸法へ伸ばして重ねる(AE の Stretch Matte to Fit)。伸ばさないなら左上を揃える。
    vec2 uv = stretch > 0.5 ? isf_FragNormCoord : isf_FragNormCoord * RENDERSIZE / vec2(textureSize(matte, 0));
    vec4 m = IMG_NORM_PIXEL(matte, uv);
    gl_FragColor = src * clamp(coverage(m), 0.0, 1.0);
}
