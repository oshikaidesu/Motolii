/*{
  "ID": "motolii.four_color_gradient",
  "LABEL": "4-Color Gradient",
  "STAGE": "pass",
  "DESCRIPTION": "Fills the layer's shape with a soft field of four colours pinned at its four corners (After Effects' 4-Color Gradient; Cavalry's Multi-Point Gradient). The points are fixed to the corners for now: vism binds one uniform per input, and four points would pass the limit",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "color1", "LABEL": "Top Left", "TYPE": "color", "DEFAULT": [1.0, 0.85, 0.2, 1.0] },
    { "NAME": "color2", "LABEL": "Top Right", "TYPE": "color", "DEFAULT": [0.2, 0.85, 0.3, 1.0] },
    { "NAME": "color3", "LABEL": "Bottom Left", "TYPE": "color", "DEFAULT": [0.9, 0.2, 0.8, 1.0] },
    { "NAME": "color4", "LABEL": "Bottom Right", "TYPE": "color", "DEFAULT": [0.2, 0.3, 0.95, 1.0] },
    { "NAME": "blend", "LABEL": "Blend", "TYPE": "float", "DEFAULT": 100.0, "MIN": 1.0, "MAX": 1000.0, "HERO": true }
  ]
}*/

// 点ごとの重みは距離の逆冪(Shepard)。Blend が大きいほど指数が小さく、色が遠くまで混ざる。
void main() {
    vec4 c = IMG_THIS_PIXEL(inputImage);
    vec2 p = isf_FragNormCoord;
    float power = 2.0 * 100.0 / blend + 0.5;
    float w1 = 1.0 / pow(max(distance(p, vec2(0.0, 1.0)), 0.001), power);
    float w2 = 1.0 / pow(max(distance(p, vec2(1.0, 1.0)), 0.001), power);
    float w3 = 1.0 / pow(max(distance(p, vec2(0.0, 0.0)), 0.001), power);
    float w4 = 1.0 / pow(max(distance(p, vec2(1.0, 0.0)), 0.001), power);
    vec3 field = (color1.rgb * w1 + color2.rgb * w2 + color3.rgb * w3 + color4.rgb * w4) / (w1 + w2 + w3 + w4);
    gl_FragColor = vec4(field * c.a, c.a);
}
