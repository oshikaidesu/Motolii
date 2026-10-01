/*{
  "ID": "motolii.background_delay",
  "LABEL": "Background Delay",
  "STAGE": "pass",
  "DESCRIPTION": "Background Copy, a moment ago: what was composited beneath this layer at t + offset, inside the layer's own shape (datamosh-style time slips)",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "below", "TYPE": "image", "SOURCE": "below", "TIME_OFFSET": "offset" },
    { "NAME": "offset", "LABEL": "Offset", "TYPE": "float", "DEFAULT": -0.1, "MIN": -5.0, "MAX": 5.0, "HERO": true }
  ]
}*/

// 下の合成の「少し前」を、自分の形(α)の中に写す。ホストがその時刻の下の層たちを描いて渡すので、
// 効果は何も覚えない(docs/plugin-resources.md §6)。
void main() {
    vec4 past = IMG_THIS_PIXEL(below);
    float shape = IMG_THIS_PIXEL(inputImage).a;
    gl_FragColor = past * shape;
}
