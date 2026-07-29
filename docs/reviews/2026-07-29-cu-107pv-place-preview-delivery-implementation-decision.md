# CU-107PV Place preview配送実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-0B05`、既存Host pointer capture

## 1. 実装

通常製品Hostの一active Rectangle dragで、AppKit pointer captureが生成する
`Moved`をprivate Host Transient `PlacePreviewPhase`へ配送した。

- preview progressは既存typed source、capture generation、最新Host layout epoch、
  latest layoutで変換したoptional Stage NDCだけを保持する。
- Stage外の`Moved`も同じ非terminal progressとして更新し、古いStage NDCを残さない。
- 配送後はredrawを要求し、pointer releaseを失わない既存pollを継続する。
- Release / Cancel / Browser replacementではpreviewを破棄するが、本粒は原因分類、
  admission、accepted terminal配送を行わない。

stateは`product_runtime`内のprivate型で、公開API、wire、serde、Document、journalへ出さない。
visual token、Rectangle appearance、preview用D2、別Surfaceは追加していない。

## 2. 合格

- inside Stageの`Moved`がsource / generation / latest layout epoch / Y-up NDCを持つ
  非空のpreview phaseを作る。
- outside Stageへの次の`Moved`はphaseを維持しつつNDCを`None`へ更新する。
- production `Moved` armはpreview delivery、redraw、pollだけを行い、
  `PendingStageDrop`、active source消費、Document edit、Undo / Redoへ到達しない。
- MacBook実機の通常製品windowでBrowser Rectangle dragからnative parent focus移譲まで継続した。

## 3. 後続へ残す責任

Esc / outside / capture lossの候補terminal原因分類は`CU-107TC`、stale / duplicateの
at-most-once admissionは`CU-107AD`、admit後の単一下流配送は`CU-107TD`が所有する。
本粒は同一previewのdedupe、generation rejection、terminal verdictを先取りしない。

## 4. 次

`CU-107PV`を`DONE`とする。次PRODUCT-ASSET `DO`は`CU-107TC`。
