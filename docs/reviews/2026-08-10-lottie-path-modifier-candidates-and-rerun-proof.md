# Lottie Path modifier候補とRerun再確認

日付: 2026-08-10
状態: **M5-PATHFX-P0 `DONE / PRIVATE PROBE`、候補分類済み、製品Stage未接続**

## 1. Outcomeと境界

AE由来のPath変形を、標準Shape、SVG、Text outlineへ共通適用できるeffect候補として維持し、
実際の`Path -> Path`出力をRerun Spatial2Dで観察できるところまで確認する。
新しいeffect framework、Lottie runtime依存、Document variant、第二rendererは作らない。

| preflight | 結論 |
|---|---|
| MECHANISM CLASS | cubic Bezier Pathへ順序適用する決定論的modifier |
| KNOWN IMPLEMENTATION SEARCH | `motolii-doc::pathgeom`、M2 PathOp表、lottie-web固定commit、Rerun Path2D probe |
| CANDIDATES | 既存`ResolvedPathOp` 8種、lottie-web modifier 6種、Rerun `LineStrips2D` |
| ADOPTION ROUTE | PathOpは`REUSE`、Lottie数学は`PORT / PATTERN`、Rerun観察は`ADOPT / WRAP` |
| REJECTED CANDIDATES | Lottie player組込み=`REJECT`。Document／time／renderer ownerを増やすため |
| THIN MOTOLII SEAM | `ShapeRecipe::lower -> pathgeom::apply -> sample_outline -> Spatial2DView` |
| THIN MOTOLII RESIDUAL | 正準単位、型付きparameter、決定論、Motolii oracle |
| RETIREMENT | 新規runtimeなし。probe outline helperは製品tessellationへ昇格しない |
| BUILD JUSTIFICATION | NONE |
| BUILD | FORBIDDEN |

## 2. 固定sourceと候補分類

- lottie-web: commit [`bede03d25d232826e0c9dca1733d542d8a7754fb`](https://github.com/airbnb/lottie-web/commit/bede03d25d232826e0c9dca1733d542d8a7754fb)
- Rerun: commit [`954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`](https://github.com/rerun-io/rerun/commit/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e)

| Motolii PathOp | lottie-web固定source | 処分 | このwaveで証明した範囲 |
|---|---|---|---|
| Pucker / Bloat | `PuckerAndBloatModifier.js` | `PORT` | 頂点と絶対Bezier handleの式を一致させ、曲線oracleを修正 |
| Zig Zag | `ZigZagModifier.js` | `PATTERN` | 円へのcorner Zig Zagが破裂outlineになることを実画面確認。Motoliiの`ridges`正規化とsource corner保持はLottie byte parityではない |
| Offset Paths | `OffsetPathModifier.js` | `PATTERN / REUSE` | source存在と既存席を確認。数値parityは未監査 |
| Round Corners | `RoundCornersModifier.js` | `PATTERN / REUSE` | source存在と既存席を確認。数値parityは未監査 |
| Trim Paths | `TrimModifier.js` | `PATTERN / REUSE` | source存在と既存席を確認。数値parityは未監査 |
| Repeater | `RepeaterModifier.js` | `PATTERN / REUSE` | source存在と既存席を確認。複数輪郭のRerun表示は未確認 |
| Twist | 固定commitの`player/js/utils/shapes`に実装fileなし | Motolii / AE候補を維持 | Lottie実コード同値とは数えない |
| Wiggle | 固定commitに実装fileなし | Motolii決定論variantを維持 | Lottie相互運用とは数えない |

## 3. Pucker / Bloatの訂正

lottie-webは頂点だけでなく、absolute in/out handleを重心から逆方向へ補間する。
Motoliiはrelative tangentを保持するため、変換式は次になる。

```text
d = vertex - centroid
vertex' = centroid + (1 - amount) * d
tangent' = (1 + amount) * tangent + 2 * amount * d
```

旧実装は第二項だけを足し、元接線へ`1 + amount`を掛けていなかった。cornerでは見えず、Circle等の
cubic curveだけ曲率がずれるため、M2正本、実装、意味論goldenを同じ変更で訂正した。

## 4. Rerun実画面oracle

既存custom visualizerのRect／Circle fillを残し、その下へ同じ`motolii_doc::pathgeom::Path`から生成した
Source Circle、Pucker / Bloat、Zig Zag burstをRerun標準`LineStrips2D`で表示する。
concave fillを未対応のconvex triangle fanへ偽装せず、Path変形そのものをoutlineで審判する。

![Rerun上のPucker／BloatとZig Zag burst](../../spikes/rerun-path2d-probe/rerun-pathfx-pucker-zigzag.png)

## 5. Oracleと限界

- `cargo test -p motolii-doc --test d1i2_pathop_geometry`: Pucker相対接線式を含む33件
- `cargo test --manifest-path spikes/rerun-path2d-probe/Cargo.toml`: codec、outline閉路、source-over
- `cargo run --manifest-path spikes/rerun-path2d-probe/Cargo.toml`: Rerun native viewerで3 outlineを目視
- private別windowのprobeであり、RN Stage seat、Document projection、Preview／Export、filled concave pathを証明しない
- 次の製品edgeは既決どおり`M5-PATH2D-S1` seat compile。S1前に本probeを製品rendererへ昇格しない
