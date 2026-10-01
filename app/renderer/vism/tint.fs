/*{
  "ID": "motolii.tint",
  "LABEL": "Tint",
  "STAGE": "pass",
  "DESCRIPTION": "Maps the dark end of the picture to one colour and the bright end to another (After Effects' Tint). Duotones: a blue blueprint, a red print",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "black", "LABEL": "Map Black To", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "white", "LABEL": "Map White To", "TYPE": "color", "DEFAULT": [1.0, 1.0, 1.0, 1.0] },
    { "NAME": "amount", "LABEL": "Amount to Tint", "TYPE": "float", "DEFAULT": 100.0, "MIN": 0.0, "MAX": 100.0, "HERO": true }
  ]
}*/

void main() {
    vec4 c = IMG_THIS_PIXEL(inputImage);
    if (c.a <= 1e-5) { gl_FragColor = vec4(0.0); return; }
    vec3 rgb = c.rgb / c.a;
    float l = pow(max(dot(rgb, vec3(0.2126, 0.7152, 0.0722)), 0.0), 1.0 / 2.2);
    vec3 tinted = pow(mix(pow(black.rgb, vec3(1.0 / 2.2)), pow(white.rgb, vec3(1.0 / 2.2)), l), vec3(2.2));
    gl_FragColor = vec4(mix(rgb, tinted, amount / 100.0) * c.a, c.a);
}
