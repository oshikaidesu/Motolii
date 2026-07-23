# Native Stage gizmo反対側レビュー（2026-07-21）

状態: **縮小採用**。命題「gizmoはGPUへ任せ、WebはCSS/flexの領域へ置く」は所有境界として採る。
ただし「GPUへ任せる」を入力、picking、履歴、accessibility、canonical renderまで広げる案は退ける。

## 反証と処分

1. **GPU描画ならGPU pickingも自然ではないか**: 少数で既知形状のgizmoにID-buffer readbackを入れると、
   wgpu mapping/pollとGPU/CPU同期をhot pathへ増やす。Bevy/Unrealのscreen-space/ray hit-test先例があり、
   CPU解析判定を第一選択とする。dense scene object pickingは別問題として保留する。
2. **`motolii-render`へ一体化すれば再利用しやすいのではないか**: 同crateのcanonical outputへ入れると
   export汚染とpreview/export不一致を招く。native Stage presentation overlayとして分離する。
3. **transform-gizmoをそのまま採れば独自開発ゼロではないか**: 0.9.0 coreは`epaint`系へ直接依存し、
   固定sourceのunit testは0件だった。M5 Scale/Depth分離も証明しないため、比較spikeを先にする。
4. **native描画ならWeb UI runtime再選定は不要ではないか**: gizmoはruntime採否から外せるが、panel、Browser、
   Timeline、community sandbox、IME、offline配布は残る。G0-9は継続する。
5. **native canvasはaccessibilityを失うのではないか**: 視覚handleをDOM化せず、tool選択、現在値、操作説明、
   keyboard等価操作をbounded semantic proxyへ投影する。proxy nodeをobject数へ比例させない。

## 最終境界

- GPU/native Stage: visual overlayとframe同期
- CPU transient interaction: analytic hit-test、drag constraint、preview
- D2/single writer: commit、Undo、Cancel
- Web/host UI kit: flex layout、controls、説明、accessibility projection

この縮小を取り込んだ決定正本は
[Native Stage gizmo所有境界](2026-07-21-native-stage-gizmo-ownership.md)とする。
