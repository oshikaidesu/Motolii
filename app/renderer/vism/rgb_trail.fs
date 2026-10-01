/*{
  "ID": "motolii.rgb_trail",
  "LABEL": "RGB Trail",
  "STAGE": "pass",
  "DESCRIPTION": "Afterimage that decays per channel and drifts: red lingers, blue fades, the tail unravels into colour. The history is kept by the host (feedback), so scrubbing lands on the same picture",
  "FILTER": "linear",
  "PADDING": { "PARAM": "reach", "SCALE": 1.0 },
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "reach", "LABEL": "Reach", "TYPE": "float", "DEFAULT": 200.0, "MIN": 0.0, "MAX": 2000.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "decay", "LABEL": "Decay (R, G, B)", "TYPE": "color", "DEFAULT": [0.94, 0.86, 0.72, 1.0], "HERO": true },
    { "NAME": "drift", "LABEL": "Drift", "TYPE": "point2D", "DEFAULT": [3.0, 0.0] }
  ],
  "PASSES": [ { "TARGET": "history", "PERSISTENT": true, "FLOAT": true }, { } ]
}*/

// 前のフレームの残像(history)を drift だけ流し、色ごとに減衰させ、今の絵と max で重ねる。
// history の持ち主は host(docs/plugin-resources.md §6-3): 効果は何も覚えない。
void main() {
    if (PASSINDEX == 0) {
        vec4 now = IMG_THIS_PIXEL(inputImage);
        vec2 uv = isf_FragNormCoord - drift / RENDERSIZE;
        vec4 past = IMG_NORM_PIXEL(history, uv);
        vec3 faded = past.rgb * decay.rgb;
        float alpha = max(now.a, past.a * max(decay.r, max(decay.g, decay.b)));
        gl_FragColor = vec4(max(now.rgb, faded), alpha);
    } else {
        gl_FragColor = IMG_THIS_PIXEL(history);
    }
}
