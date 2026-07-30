# CU-107TC Place候補terminal原因分類実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-107PV`、`CU-0B05`

## 1. 実装

通常製品Hostの一active Rectangle dragが生じる候補terminalを、privateな閉集合
`Escape / OutsideStage / CaptureLoss / NoNonCommitCause`のちょうど一つへ分類した。

- focused windowでarm後に観測した非repeat Escape key-downをtyped cancelへ変換する。
- focus lossはcapture lossとしてEscapeと同時成立させない。
- Releaseは最新native Host layoutでStage hit-testし、内側を`NoNonCommitCause`、
  外側を`OutsideStage`とする。
- Release時にlayoutが無ければ第五の曖昧な原因を作らず、typed runtime errorでfail closedする。
- 分類結果はsource、capture generation、必要な場合だけlayout epoch / Stage NDCを持つ
  private Transient stateであり、wire、serde、Document、journalへ出さない。

既存のStage内`PendingStageDrop`は次粒への未配送候補として維持したが、本粒はadmission、
accepted terminal配送、D2、Undoを行わない。

## 2. 合格

- Stage内Release、Stage外Release、Escape、capture lossが四つのenum variantへ排他分類される。
- Escape/capture lossはactive captureを一度だけ終了し、後続pollで再生成されない。
- production Release / Cancel armが分類へ到達し、Document edit、Undo / Redo、
  admissionへ到達しない。
- `motolii-ui` unit / production structural test / clippyが緑。

## 3. 後続へ残す責任

`NoNonCommitCause`だけを入力にstale / duplicateを拒否し、一active dragで高々一件へ抑える
責任は`CU-107AD`、admit後の単一下流commit境界への配送は`CU-107TD`が所有する。
本粒はexact wire tuple、dedupe table、accepted delivery、D2契約を先取りしない。

## 4. 次

`CU-107TC`を`DONE`とする。次PRODUCT-ASSET `DO`は`CU-107AD`。
