/*{
  "ID": "motolii.caustic_light",
  "LABEL": "Caustic Light",
  "STAGE": "pass",
  "DESCRIPTION": "Paints the layer with a drifting four-colour mesh gradient and the caustics of light through moving water. Phase 0→1 is one full cycle, so keying it linearly over a loop range repeats without a seam",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "color1", "LABEL": "Color 1", "TYPE": "color", "DEFAULT": [0.02, 0.03, 0.10, 1.0] },
    { "NAME": "color2", "LABEL": "Color 2", "TYPE": "color", "DEFAULT": [0.10, 0.05, 0.30, 1.0] },
    { "NAME": "color3", "LABEL": "Color 3", "TYPE": "color", "DEFAULT": [0.00, 0.35, 0.45, 1.0] },
    { "NAME": "color4", "LABEL": "Color 4", "TYPE": "color", "DEFAULT": [0.55, 0.12, 0.40, 1.0] },
    { "NAME": "caustics", "LABEL": "Caustics", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 4.0, "HERO": true },
    { "NAME": "scale", "LABEL": "Scale", "TYPE": "float", "DEFAULT": 420.0, "MIN": 10.0, "MAX": 10000.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "tint", "LABEL": "Caustic Color", "TYPE": "color", "DEFAULT": [0.75, 0.95, 1.0, 1.0] },
    { "NAME": "phase", "LABEL": "Phase", "TYPE": "float", "DEFAULT": 0.0, "SUBTYPE": "TIME" }
  ]
}*/

const float TAU = 6.28318530718;

// Four colour points orbit on closed paths; each pixel mixes them by inverse distance (a mesh gradient).
vec3 mesh(vec2 uv, float t) {
    vec2 p1 = vec2(0.20, 0.25) + 0.18 * vec2(cos(t), sin(t));
    vec2 p2 = vec2(0.80, 0.30) + 0.16 * vec2(cos(t + 2.1), sin(2.0 * t));
    vec2 p3 = vec2(0.30, 0.80) + 0.20 * vec2(sin(t + 1.3), cos(t));
    vec2 p4 = vec2(0.75, 0.75) + 0.17 * vec2(cos(2.0 * t + 0.7), sin(t + 2.9));
    float w1 = 1.0 / pow(distance(uv, p1) + 0.05, 2.2);
    float w2 = 1.0 / pow(distance(uv, p2) + 0.05, 2.2);
    float w3 = 1.0 / pow(distance(uv, p3) + 0.05, 2.2);
    float w4 = 1.0 / pow(distance(uv, p4) + 0.05, 2.2);
    return (color1.rgb * w1 + color2.rgb * w2 + color3.rgb * w3 + color4.rgb * w4) / (w1 + w2 + w3 + w4);
}

// Water caustics: an iterated interference of waves. Time enters only through cos/sin of t, so the
// pattern closes on itself after one cycle.
float caustic(vec2 q, float t) {
    vec2 p = q - 250.0;
    vec2 i = p;
    float c = 1.0;
    float inten = 0.005;
    for (int n = 0; n < 5; n++) {
        float fn_ = float(n) + 1.0;
        vec2 drift = vec2(cos(t + fn_ * 1.7), sin(t - fn_ * 0.9)) * (0.6 / fn_);
        i = p + drift + vec2(cos(i.y * 1.3 + fn_), sin(i.x * 1.3 - fn_));
        c += 1.0 / length(vec2(p.x / (sin(i.x + drift.y) / inten), p.y / (cos(i.y + drift.x) / inten)));
    }
    c /= 5.0;
    c = 1.17 - pow(c, 1.4);
    return clamp(pow(abs(c), 8.0), 0.0, 1.0);
}

void main() {
    vec4 base = IMG_THIS_PIXEL(inputImage);
    if (base.a <= 1e-5) { gl_FragColor = vec4(0.0); return; }
    float t = TAU * phase;
    vec2 uv = vec2(isf_FragNormCoord.x, 1.0 - isf_FragNormCoord.y);
    vec3 color = mesh(uv, t);
    float light = caustic(uv * RENDERSIZE.xy / max(scale, 1.0) * TAU, t);
    light = light * light * 1.5;
    color += tint.rgb * light * caustics * (0.35 + 0.65 * smoothstep(0.0, 0.9, uv.y));
    gl_FragColor = vec4(color * base.a, base.a);
}
