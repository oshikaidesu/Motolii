/*{
  "ID": "motolii.time_difference",
  "LABEL": "Time Difference",
  "STAGE": "pass",
  "DESCRIPTION": "Shows what moved: the layer now, minus the layer a moment ago (AE の Time Difference と同じ型)",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "past", "TYPE": "image", "TIME_OFFSET": "offset" },
    { "NAME": "offset", "LABEL": "Offset", "TYPE": "float", "DEFAULT": -0.2, "MIN": -5.0, "MAX": 5.0, "HERO": true },
    { "NAME": "contrast", "LABEL": "Contrast", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 20.0 }
  ]
}*/

// 「今」と「少し前」の差。ホストが両方の時刻の絵を渡すので、効果は何も覚えない
// (docs/plugin-resources.md §6 — 覚えると追跡できなくなる)。
void main() {
    vec4 now = IMG_THIS_PIXEL(inputImage);
    vec4 then = IMG_THIS_PIXEL(past);
    gl_FragColor = vec4(abs(now.rgb - then.rgb) * contrast, max(now.a, then.a));
}
