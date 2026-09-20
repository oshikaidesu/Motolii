/*{
  "ID": "motolii.echo",
  "LABEL": "Echo",
  "STAGE": "pass",
  "DESCRIPTION": "Afterimage: where the layer has been over the last Hold frames, fading as it ages. The history is kept by the host (feedback), so a long tail costs the same one pass a frame as a short one, and scrubbing lands on the same picture",
  "FILTER": "linear",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "hold", "LABEL": "Hold", "TYPE": "float", "DEFAULT": 12.0, "MIN": 1.0, "MAX": 240.0, "SUBTYPE": "COUNT", "HERO": true },
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0, "SUBTYPE": "OPACITY" },
    { "NAME": "stack", "LABEL": "Stack", "TYPE": "long", "DEFAULT": 0, "LABELS": ["Composite", "Add", "Maximum"] }
  ],
  "PASSES": [ { "TARGET": "history", "PERSISTENT": true, "FLOAT": true }, { } ]
}*/

// AE の Echo を feedback で: 前のコマの history を Hold コマで 1% まで落ちる率で減衰させ、今の絵と重ねる。
// 枚数を数えない — 尾の長さは減衰率で、費用は毎コマ 1 パスのまま(RGB Trail と同じ形)。
// history の持ち主は host(docs/plugin-resources.md §6-3): 効果は何も覚えない。

vec4 stacked(vec4 now, vec4 past) {
    int how = int(stack + 0.5);
    if (how == 1) { return now + past; }
    if (how == 2) { return max(now, past); }
    return now + past * (1.0 - now.a);
}

void main() {
    vec4 now = IMG_THIS_PIXEL(inputImage);
    if (PASSINDEX == 0) {
        float decay = pow(0.01, 1.0 / max(hold, 1.0));
        gl_FragColor = stacked(now, IMG_THIS_PIXEL(history) * decay);
    } else {
        gl_FragColor = mix(now, IMG_THIS_PIXEL(history), clamp(strength, 0.0, 1.0));
    }
}
