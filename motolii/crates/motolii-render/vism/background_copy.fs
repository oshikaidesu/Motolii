/*{
  "ID": "motolii.background_copy",
  "LABEL": "Background Copy",
  "STAGE": "pass",
  "BACKDROP_INPUT": "backdrop",
  "DESCRIPTION": "The layer's picture becomes what is composited beneath it (アライトモーションの「背景のコピー」)",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "backdrop", "TYPE": "image" }
  ]
}*/

// 自分の形(α)の中に、下の合成を写す — 続く効果はその絵に掛かる。形の外は触らない
// (層は形を持つ。窓ぶん全部を写すと、続く効果が画面全体に掛かってしまう)。
void main() {
    vec4 below = IMG_THIS_PIXEL(backdrop);
    float shape = IMG_THIS_PIXEL(inputImage).a;
    gl_FragColor = below * shape;
}
