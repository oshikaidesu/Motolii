# Motolii egui Timeline engine 正本

日付: 2026-08-15  
状態: **決定**

> **訂正(2026-08-16)**: 下の「Timeline engine／皮の正本は `timeline_egui.rs`」は**撤回された**。
> 現物では `timeline_egui` は製品のどこからも呼ばれておらず、製品 Timeline は以前から
> Skia(`timeline_skia_raster.rs`)である。egui Timeline は移行が途中で止まった残骸で、
> 3,338行ごと撤去した。→ [製品Timelineの正本はSkia](2026-08-16-skia-timeline-authority-correction.md)
> 本文は歴史として残す。**「Rerun Time Panel へ engine 相乗りしない」は維持**。

## 決定

利用者裁定(2026-08-15): 製品 Timeline の **engine** は Motolii egui（`crates/motolii-ui/src/timeline_egui/mod.rs` — 2026-08-16 に撤去済み。原文は `git show f209da9d^:crates/motolii-ui/src/timeline_egui/mod.rs`）である。Rerun Time Panel への engine 相乗りはしない。改造コストより、普通の Timeline を現行 egui へ翻訳し切る。

Stage spatial の Rerun `ADOPT / WRAP` は[採択再締結](2026-08-10-m5-rerun-spatial-viewer-adoption-reclosure-decision.md)のまま。この切り方を Stage／Inspector／Browser／Host へ広げない。

| 座席 | 正本 | 役割 |
|---|---|---|
| Timeline engine／皮 | Motolii egui `timeline_egui.rs` | 描画・hit・pointer。トンマナは現行 token |
| 操作カタログ | [Godot編集系PORT](2026-08-13-godot-editing-system-adoption.md) | 席の一覧。Godot chrome／theme／Node は持ち込まない |
| 編集意味 | Document／D2 | clip／key／layer。egui は writer にしない |
| Stage spatial | Rerun Spatial Viewer | Timeline engine とは別席 |

[2026-08-14 Rerun相乗り先](2026-08-14-rerun-body-skin-meaning-decision.md)の **Timeline engine 相乗り**だけを撤回する。本文は歴史として残す。layer／clip／key を Time Panel のキー writer にしない、という負例は維持する。

## トンマナ

現行 Motolii token を維持する。行高、`PALETTE`、`DESKTOP` 面は `timeline_egui.rs` と Skia 意味源（`timeline_skia.rs`）から取る。Godot／egui-keyframe の見た目は採らない。

## 既知実装

```text
MECHANISM CLASS: dope-sheet 時間編集（clip移動/trim、key時刻、選択）
KNOWN IMPLEMENTATION SEARCH: decision-index、Godot PORT、product_runtime gesture、egui-keyframe 0.1.0
CANDIDATES:
  - REUSE: DocumentEditQueue / TimelineMoveGesture / TimelineTrimGesture / SetPositionKeyTime
  - PATTERN: egui-keyframe DopeSheet（box select、key drag、AnimationCommand）
  - PORT: Godot AnimationTrackEditor の席（既決）
ADOPTION ROUTE: PATTERN + REUSE。crate ADOPT しない
REJECTED:
  - `egui-keyframe` を依存追加（egui 0.36。Motolii/Rerun共用は 0.35）
  - 彼らの Track/Keyframe をDocument代わりにする
  - 今波で CurveEditor 面を立てる
  - Rerun Time Panel を製品Timeline engineにする
THIN MOTOLII SEAM: intent → 既存queue → DocumentEditRuntime
BUILD JUSTIFICATION: NONE
```

[`egui-keyframe`](https://github.com/virtualritz/egui-keyframe) は DopeSheet 操作の PATTERN だけに使う。crate を ADOPT しない（egui 0.36 vs Motolii 0.35）。彼らの `Track`／`Keyframe` を Document にしない。

## 今波の契約境界

clip の move／trim と `SetPositionKeyTime` を、既存 `DocumentEditQueue`／`TimelineMoveGesture`／`TimelineTrimGesture` へ通す。新しい Document writer や第二 Undo は作らない。

## 非目標

- CurveEditor panel、値軸ドラッグ
- `cargo add egui-keyframe`、egui 0.36 上げ
- RN／`timeline_skia.rs` 製品経路の変更
- 新 Command、第二 Undo
- Stage spatial の Rerun 採択変更
