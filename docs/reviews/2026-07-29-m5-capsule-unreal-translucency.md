# Unreal translucency／refraction証拠capsule

状態: **観察** / `FROZEN / DELETE-LATER` / 製品import禁止

- source: Unreal Engine `5.8`公式docs、取得日2026-07-29
- URLs: <https://dev.epicgames.com/documentation/unreal-engine/using-transparency-in-unreal-engine-materials>,
  <https://dev.epicgames.com/documentation/unreal-engine/temporal-super-resolution-in-unreal-engine>,
  <https://dev.epicgames.com/documentation/unreal-engine/using-refraction-in-unreal-engine>
- license: Epic公式docsの利用条件に従う。本文転載なし
- 削除条件: P2D-RCIで公式URLへの直接引用へ置換後

## 観察

- translucent重なりにはsort問題があり、depth bufferだけでは前後を決められないと説明される。
- translucencyは複数のpass位置を持ち、scene colorへのblendとdepth／velocityの扱いが同一ではない。
- overdrawは層数に応じた性能問題になり、sort priorityは意図的な上書きである。
- refractionはtranslucent material側の機能として扱われ、方式／pass／screen-space制約を伴う。

## 非証明範囲

Unreal material／pass／sort priority／IOR UIをMotoliiの公開語彙、Document、phase enumにしない。
