# M3 UIモック

## timeline v0

![M3 timeline visual mock](m3-timeline-v0.png)

- [SVG source](m3-timeline-v0.svg)
- ステータス: **比較用の静的モック**。G0-6のtoken決定、製品UI、goldenではない
- 対象: asset browser、preview、inspector、一般的なtrack型timelineを同一画面へ置いた時の密度と視覚認知
- 表現済み: video/audio/shape/text/group、選択、keyframe、mute、warning、playhead
- 未表現: hover/focus、drag/trim、zoom、easing popup、keyboard操作、IME、reduce motion、別monitor/DPI

色値はこのSVG内だけの仮値であり、DTCG tokenへ転記しない。採択するのは値ではなく、同じfixtureで比較して合格した役割と階層だけとする。

## このモックで答える問い

1. labelを読まず、項目種別・選択項目・mute・keyframe・warningを識別できるか
2. timelineが主作業面として十分な高さと一覧性を持つか
3. preview/inspector/asset browserがtimelineより強く見えすぎないか
4. 意味色が多すぎず、AE型の文字依存へも戻っていないか
5. 新規componentだけが別製品のように浮いていないか

## 次の比較案

- v0-A: 現在案。previewとtimelineをほぼ同格
- v0-B: timelineをさらに高くし、previewを縮小
- v0-C: inspectorを必要時だけ展開し、timelineの横幅を増やす

同じfixtureとviewportでA/B/Cを比較し、印象だけでpanel比率を決めない。
