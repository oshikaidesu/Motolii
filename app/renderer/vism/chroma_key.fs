/*{
  "ID": "motolii.chroma_key",
  "LABEL": "Chroma Key",
  "STAGE": "pass",
  "DESCRIPTION": "Takes a colour out of the picture and leaves what is left (After Effects' Keylight, Premiere's Ultra Key). What remains is also what the physics collides with, because the solver reads the key",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "key", "LABEL": "Key Color", "TYPE": "color", "DEFAULT": [0.0, 1.0, 0.0, 1.0] },
    { "NAME": "tolerance", "LABEL": "Tolerance", "TYPE": "float", "DEFAULT": 30.0, "MIN": 0.0, "MAX": 100.0, "HERO": true },
    { "NAME": "softness", "LABEL": "Softness", "TYPE": "float", "DEFAULT": 10.0, "MIN": 0.0, "MAX": 100.0 },
    { "NAME": "spill", "LABEL": "Spill", "TYPE": "float", "DEFAULT": 60.0, "MIN": 0.0, "MAX": 100.0 }
  ]
}*/

// 色の近さは、明るさで割った色味で測る。明るさごと測ると、圧縮で暗くなった緑(動画)や
// 影の入った緑が残る。入力は線形なので、欄のキー色も線形へ写してから比べる。
vec2 chroma_of(vec3 rgb) {
    float y = max(dot(rgb, vec3(0.2126, 0.7152, 0.0722)), 1e-4);
    return vec2(rgb.b - y, rgb.r - y) / y;
}

vec3 to_linear(vec3 c) { return pow(max(c, vec3(0.0)), vec3(2.2)); }

void main() {
    vec4 c = IMG_THIS_PIXEL(inputImage);
    if (c.a <= 1e-5) { gl_FragColor = vec4(0.0); return; }
    vec3 rgb = c.rgb / c.a;
    vec3 keyl = to_linear(key.rgb);
    float near = distance(chroma_of(rgb), chroma_of(keyl));
    float inner = tolerance / 100.0 * 0.9;
    float outer = inner + max(softness, 0.0) / 100.0 * 0.9 + 1e-4;
    // 内側は抜く、外側は残す、間はなめらかに。
    float keep = smoothstep(inner, outer, near);
    // 残った縁に乗った色かぶりを引く(髪の緑を抜く)。
    vec3 pure = rgb;
    float cast = max(dot(normalize(chroma_of(rgb) + 1e-6), normalize(chroma_of(keyl) + 1e-6)), 0.0);
    float y = dot(rgb, vec3(0.2126, 0.7152, 0.0722));
    pure = mix(rgb, mix(rgb, vec3(y), cast), spill / 100.0);
    float a = c.a * keep;
    gl_FragColor = vec4(pure * a, a);
}
