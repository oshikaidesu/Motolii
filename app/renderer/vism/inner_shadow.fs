/*{
  "ID": "motolii.inner_shadow",
  "LABEL": "Inner Shadow",
  "STAGE": "pass",
  "DESCRIPTION": "A soft shadow cast inside the layer's own edge, as if the shape were cut into the page (Photoshop's and Cavalry's Inner Shadow)",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "color", "LABEL": "Color", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "opacity", "LABEL": "Opacity", "TYPE": "float", "DEFAULT": 60.0, "MIN": 0.0, "MAX": 100.0, "HERO": true },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": 120.0, "MIN": -360.0, "MAX": 360.0, "SUBTYPE": "ANGLE" },
    { "NAME": "distance", "LABEL": "Distance", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.0, "MAX": 1000.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "size", "LABEL": "Size", "TYPE": "float", "DEFAULT": 12.0, "MIN": 0.0, "MAX": 1000.0, "SUBTYPE": "DISTANCE" }
  ]
}*/

// 光の来る向き(Photoshop と同じく 120° は左上から)の逆へ形をずらし、ずらした形の外に当たる所が影。
// Size は 2 重の円周 24 点の平均でぼかす。
void main() {
    vec4 c = IMG_THIS_PIXEL(inputImage);
    if (c.a <= 1e-5) { gl_FragColor = vec4(0.0); return; }
    float a = radians(angle);
    vec2 toward = vec2(cos(a), -sin(a));
    vec2 shift = toward * distance / RENDERSIZE;
    float covered = 0.0;
    for (int ring = 0; ring < 2; ring++) {
        for (int i = 0; i < 12; i++) {
            float t = (float(i) + 0.5 * float(ring)) * 6.2831853 / 12.0;
            vec2 o = vec2(cos(t), sin(t)) * size * (0.5 + 0.5 * float(ring)) / RENDERSIZE;
            covered += IMG_NORM_PIXEL(inputImage, isf_FragNormCoord - shift + o).a;
        }
    }
    covered /= 24.0;
    float k = (1.0 - covered) * opacity / 100.0 * color.a;
    gl_FragColor = vec4(c.rgb * (1.0 - k) + color.rgb * c.a * k, c.a);
}
