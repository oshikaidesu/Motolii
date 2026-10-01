/*{
  "ID": "motolii.noise",
  "LABEL": "Noise",
  "STAGE": "pass",
  "DESCRIPTION": "Sprinkles grain over the layer that changes every frame, the same on every play (After Effects' Noise). Dust and film texture on a close-up",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "amount", "LABEL": "Amount of Noise", "TYPE": "float", "DEFAULT": 10.0, "MIN": 0.0, "MAX": 100.0, "HERO": true },
    { "NAME": "color_noise", "LABEL": "Use Color Noise", "TYPE": "bool", "DEFAULT": true },
    { "NAME": "grain", "LABEL": "Grain Size", "TYPE": "float", "DEFAULT": 1.0, "MIN": 1.0, "MAX": 64.0, "SUBTYPE": "DISTANCE" }
  ]
}*/

float hash(vec3 p) {
    p = fract(p * vec3(443.897, 441.423, 437.195));
    p += dot(p, p.yzx + 19.19);
    return fract((p.x + p.y) * p.z);
}

void main() {
    vec4 c = IMG_THIS_PIXEL(inputImage);
    if (c.a <= 1e-5) { gl_FragColor = vec4(0.0); return; }
    vec2 cell = floor(gl_FragCoord.xy / max(grain, 1.0));
    float frame = floor(TIME * 60.0);
    float mono = hash(vec3(cell, frame));
    vec3 coloured = vec3(mono, hash(vec3(cell, frame + 17.0)), hash(vec3(cell, frame + 43.0)));
    vec3 n = mix(vec3(mono), coloured, step(0.5, color_noise));
    vec3 rgb = pow(max(c.rgb / c.a, 0.0), vec3(1.0 / 2.2));
    rgb = clamp(rgb + (n - 0.5) * amount / 100.0, 0.0, 1.0);
    gl_FragColor = vec4(pow(rgb, vec3(2.2)) * c.a, c.a);
}
