/*{
  "ID": "motolii.invert",
  "LABEL": "Invert",
  "STAGE": "pass",
  "DESCRIPTION": "Turns the colours inside out, keeping the shape (After Effects' Invert, RGB). A one-frame flash between two looks",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "blend", "LABEL": "Blend With Original", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 100.0, "HERO": true }
  ]
}*/

void main() {
    vec4 c = IMG_THIS_PIXEL(inputImage);
    if (c.a <= 1e-5) { gl_FragColor = vec4(0.0); return; }
    vec3 rgb = pow(max(c.rgb / c.a, 0.0), vec3(1.0 / 2.2));
    vec3 inverted = pow(clamp(1.0 - rgb, 0.0, 1.0), vec3(2.2));
    gl_FragColor = vec4(mix(inverted, c.rgb / c.a, blend / 100.0) * c.a, c.a);
}
