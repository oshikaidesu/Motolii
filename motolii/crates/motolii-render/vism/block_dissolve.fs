/*{
  "ID": "motolii.block_dissolve",
  "LABEL": "Block Dissolve",
  "STAGE": "pass",
  "DESCRIPTION": "Makes the layer disappear in random blocks as the completion rises (After Effects' Block Dissolve). The same seed always drops the same blocks in the same order",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "completion", "LABEL": "Transition Completion", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 100.0, "HERO": true },
    { "NAME": "block_width", "LABEL": "Block Width", "TYPE": "float", "DEFAULT": 20.0, "MIN": 1.0, "MAX": 4000.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "block_height", "LABEL": "Block Height", "TYPE": "float", "DEFAULT": 20.0, "MIN": 1.0, "MAX": 4000.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "seed", "LABEL": "Random Seed", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 10000.0 }
  ]
}*/

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21) + seed * 0.618);
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

void main() {
    vec2 block = floor(gl_FragCoord.xy / vec2(max(block_width, 1.0), max(block_height, 1.0)));
    float gone = step(hash(block), completion / 100.0 - 1e-4);
    gl_FragColor = IMG_THIS_PIXEL(inputImage) * (1.0 - gone);
}
