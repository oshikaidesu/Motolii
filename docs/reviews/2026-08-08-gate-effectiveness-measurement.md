# gate の実効性測定 — 鎖のgateは効き、capsuleのgateは1周で収束しない

日付: 2026-08-08
状態: **観察 / 工程の実測記録**

## 0. 何を測ったか

2026-08-08に確立した工程

```
利用者outcome → 鎖 → 【鎖のgate】 → seam特定 → 粒割り → capsule → 【capsuleのgate】 → 施工
```

について、`N-OVERLAY-MINIMAL` を題材に**gateが自身のコストを回収するか**を実測した。
実装到達は目的ではない。

## 1. 鎖のgate — 効いた

背骨の仮コードを別family（Sol / OpenAI）へ通した結果:

```
ERRORS: 12 / VERIFIED: 15 / SEAM_BLOCKED: 4
```

### 検出された誤りの種類

| 種別 | 例 |
|---|---|
| 実名ずれ | `stage_hit_test(..)` → 実際は `view_local_to_canonical` + `hit_test_projected_layers` の2段 |
| signature誤り | `project_timeline(doc, visible_range)` → 実際は `(document, metrics, viewport)` |
| field名誤り | `interval.target/.key` → 実際は `layer/left_id` |
| 型誤り | `transform_point(cur)` → 実際は `(x, y) -> [f64;2]`。配列同士は減算できない |
| **事実誤認** | 「`native_timeline_renderer.rs` は旧route専用」→ 実際は `ProductSurface::render` へ接続済み |
| **既決違反** | rust-skia を候補として扱っていたが、当時は `REJECT for product path`（別途[裁定](2026-08-08-skia-reject-to-adopt-authority-reconciliation.md)） |

### 塞がっていたseam 4件（最重要）

```
draw_stage_preview 呼出し後に同じframeへoverlay      → できない
RN selection helper後に PublishedDocument を使う     → できない
raw な baseline/delta から直接 Position key を確定   → できない
active interval から直接 interp request を確定       → できない
```

**`BUILT_UNWIRED` を「繋ぐだけ」と表現していたのは不正確だった。**
実routeには admission 層がある（`admit_easing_terminal` が generation・layout epoch・
interval再導出・same-value を拒否してから積む）。飛ばすと stale な状態で Document を書ける。

> **「呼び出し箇所」と「挿入できる継ぎ目」は別物である。**

## 2. capsuleのgate — 1周で収束しない

| | HOLES | V1_REOPENED |
|---|---|---|
| v1（未検証の鎖から compile） | **12** | — |
| v2（鎖のgate通過後 + v1の12件を先回りで潰したと主張） | **9** | **7** |

**12件のうち7件が再指摘された。「潰した」は過大申告だった。**

### 収束しなかった理由

主担当の修正は**方向は正しいが精度が足りない**形で一貫していた。

| v2で書いた指定 | 再指摘 |
|---|---|
| 「`ensure_gpu`時に一度だけ生成」＋「surface全面1:1」 | **この2つは同時に満たせない**（resizeでextentが変わる）。自分の指定同士が矛盾 |
| dirty = 初回 / 寸法変化 / `surface_epoch`変化 | host共有なのに **stage identity をkeyに含めるか**未指定 |
| clearは「upload完了時」 | **`write_texture`復帰時 / submit時 / GPU完了時**のどれか未指定 |
| raster と upload を個別計数 | **計数の置き場所**未指定。productionから切れたcounterでも通る |
| 色は既存と区別できる値 | **実値**未指定 |
| `Rgba8Unorm`に合わせる | **何と比較するか**未指定（実surfaceは`Bgra8Unorm`の可能性） |

### 最も重い再指摘

> **raster/upload counter だけ増やして overlay draw を省略する、誤texture・誤UV・
> 透明出力を描く実装でも全列挙oracleを通せる**

v1で「fixtureの色を既存と区別せよ」と指摘され、主担当は**色を指定した**。
しかし根本は「**compose されたことを証明するoracleが無い**」ことであった。
**指摘を表面的に読み、oracleの目的を直していない。**

## 3. 結論

| gate | 実効性 |
|---|---|
| **鎖のgate** | **回収している。** 1回で12件、うち4件は施工不能なseam。通さなければ誤った前提で発注していた |
| **capsuleのgate** | **1周では収束しない。** 指摘の反映精度が律速。2周目でも7件が残った |

## 4. 推奨（未採択）

capsuleのgateが収束しない原因は、**粒が複合的でoracleが間接的**になることにある。

`N-OVERLAY-MINIMAL` は「skia raster + texture upload + compose + dirty管理」が1粒に入り、
oracleが counter や不在証明に頼らざるを得なかった。

> **粒を割って oracle を直接的にする。**
> 例: 「skiaでrasterした結果のpixelが期待値と一致する」は直接検査できるが、
> 「composeされた」は間接検査になりやすい。

**本文書はこの分割を採択しない。** 次の主担当が判断する。

## 5. 非目標

- 本文書を根拠に `N-OVERLAY-MINIMAL` を発注すること（**v2は施工不可。oracleが空実装を通す**）
- gate を省略する根拠にすること（鎖のgateは回収している）
- capsuleのgateを不要と結論すること（9件は依然として実害のある指摘である）
