/*{
  "ID": "motolii.four_color_gradient",
  "LABEL": "4-Color Gradient",
  "STAGE": "pass",
  "DESCRIPTION": "Fills the layer's shape with a soft field of four colours pinned at four points (After Effects' 4-Color Gradient; Cavalry's Multi-Point Gradient). Points are in percent of the layer, measured from its top left (a proposal: AE uses pixels)",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "point1", "LABEL": "Point 1", "TYPE": "point2D", "DEFAULT": [0.0, 0.0] },
    { "NAME": "color1", "LABEL": "Color 1", "TYPE": "color", "DEFAULT": [1.0, 0.85, 0.2, 1.0] },
    { "NAME": "point2", "LABEL": "Point 2", "TYPE": "point2D", "DEFAULT": [100.0, 0.0] },
    { "NAME": "color2", "LABEL": "Color 2", "TYPE": "color", "DEFAULT": [0.2, 0.85, 0.3, 1.0] },
    { "NAME": "point3", "LABEL": "Point 3", "TYPE": "point2D", "DEFAULT": [0.0, 100.0] },
    { "NAME": "color3", "LABEL": "Color 3", "TYPE": "color", "DEFAULT": [0.9, 0.2, 0.8, 1.0] },
    { "NAME": "point4", "LABEL": "Point 4", "TYPE": "point2D", "DEFAULT": [100.0, 100.0] },
    { "NAME": "color4", "LABEL": "Color 4", "TYPE": "color", "DEFAULT": [0.2, 0.3, 0.95, 1.0] },
    { "NAME": "blend", "LABEL": "Blend", "TYPE": "float", "DEFAULT": 100.0, "MIN": 1.0, "MAX": 1000.0, "HERO": true },
    { "NAME": "opacity", "LABEL": "Opacity", "TYPE": "float", "DEFAULT": 100.0, "MIN": 0.0, "MAX": 100.0 }
  ]
}*/

// 点ごとの重みは距離の逆冪(Shepard)。Blend が大きいほど指数が小さく、色が遠くまで混ざる。
void main() {
    vec4 c = IMG_THIS_PIXEL(inputImage);
    vec2 p = vec2(isf_FragNormCoord.x, 1.0 - isf_FragNormCoord.y) * 100.0;
    float power = 2.0 * 100.0 / blend + 0.5;
    float w1 = 1.0 / pow(max(distance(p, point1), 0.01), power);
    float w2 = 1.0 / pow(max(distance(p, point2), 0.01), power);
    float w3 = 1.0 / pow(max(distance(p, point3), 0.01), power);
    float w4 = 1.0 / pow(max(distance(p, point4), 0.01), power);
    vec3 field = (color1.rgb * w1 + color2.rgb * w2 + color3.rgb * w3 + color4.rgb * w4) / (w1 + w2 + w3 + w4);
    gl_FragColor = vec4(mix(c.rgb, field * c.a, opacity / 100.0), c.a);
}
