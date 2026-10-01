/*{
  "ID": "motolii.hue_saturation",
  "LABEL": "Hue/Saturation",
  "STAGE": "pass",
  "DESCRIPTION": "Turns the hue wheel, drains or pumps colour, and lightens or darkens (After Effects' Hue/Saturation, Master channel). A red flower turns blue, a photo turns black and white",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "hue", "LABEL": "Master Hue", "TYPE": "float", "DEFAULT": 0.0, "MIN": -180.0, "MAX": 180.0, "SUBTYPE": "ANGLE", "HERO": true },
    { "NAME": "saturation", "LABEL": "Master Saturation", "TYPE": "float", "DEFAULT": 0.0, "MIN": -100.0, "MAX": 100.0 },
    { "NAME": "lightness", "LABEL": "Master Lightness", "TYPE": "float", "DEFAULT": 0.0, "MIN": -100.0, "MAX": 100.0 }
  ]
}*/

// 色相・彩度・明るさは見た目の空間(ガンマ 2.2)で回す(AE と同じ手触り)。出口は乗算済み線形へ戻す。

vec3 rgb2hsl(vec3 c) {
    float hi = max(c.r, max(c.g, c.b)), lo = min(c.r, min(c.g, c.b));
    float l = (hi + lo) * 0.5, d = hi - lo;
    if (d < 1e-6) return vec3(0.0, 0.0, l);
    float s = l > 0.5 ? d / (2.0 - hi - lo) : d / (hi + lo);
    float h = hi == c.r ? (c.g - c.b) / d + (c.g < c.b ? 6.0 : 0.0) : hi == c.g ? (c.b - c.r) / d + 2.0 : (c.r - c.g) / d + 4.0;
    return vec3(h / 6.0, s, l);
}
float hue2rgb(float p, float q, float t) {
    t = fract(t);
    if (t < 1.0 / 6.0) return p + (q - p) * 6.0 * t;
    if (t < 0.5) return q;
    if (t < 2.0 / 3.0) return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    return p;
}
vec3 hsl2rgb(vec3 h) {
    if (h.y < 1e-6) return vec3(h.z);
    float q = h.z < 0.5 ? h.z * (1.0 + h.y) : h.z + h.y - h.z * h.y;
    float p = 2.0 * h.z - q;
    return vec3(hue2rgb(p, q, h.x + 1.0 / 3.0), hue2rgb(p, q, h.x), hue2rgb(p, q, h.x - 1.0 / 3.0));
}

void main() {
    vec4 c = IMG_THIS_PIXEL(inputImage);
    if (c.a <= 1e-5) { gl_FragColor = vec4(0.0); return; }
    vec3 rgb = pow(max(c.rgb / c.a, 0.0), vec3(1.0 / 2.2));
    vec3 h = rgb2hsl(rgb);
    h.x = fract(h.x + hue / 360.0);
    h.y = saturation >= 0.0 ? mix(h.y, 1.0, saturation / 100.0 * h.y) : h.y * (1.0 + saturation / 100.0);
    h.z = lightness >= 0.0 ? mix(h.z, 1.0, lightness / 100.0) : h.z * (1.0 + lightness / 100.0);
    vec3 out_rgb = pow(clamp(hsl2rgb(h), 0.0, 1.0), vec3(2.2));
    gl_FragColor = vec4(out_rgb * c.a, c.a);
}
