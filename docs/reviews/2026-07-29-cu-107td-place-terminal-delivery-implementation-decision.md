# CU-107TD Place terminal配送実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-107AD`、`CU-0B05`

## 1. 実装

`CU-107AD`がadmitしたprivate terminalだけを、既存の単一下流commit候補境界
`PendingStageDrop`へ一回配送した。

- deliveryはsource、capture generation、layout epoch、canonical Stage NDCをそのまま渡す。
- delivered high-water以下のduplicate / replayは拒否する。
- noncommit原因、layout / NDC欠落、admitされていないterminalは配送しない。
- 配送先はprivate slot一つだけで、D2、Document、journal、Undoへは到達しない。

## 2. 合格

- admit済みinside-Stage terminalが`PendingStageDrop`へ一回だけ届く。
- 同terminalの再配送とEscape / capture lossは0件。
- production admission arm以外から配送境界へ入らない。
- `motolii-ui` unit / structural testが緑。

## 3. 次

`CU-107TD`を`DONE`とし、`CU-107`の4前提
`PV → TC → AD → TD`をすべて閉じた。次PRODUCT-ASSET `DO`は親`CU-107`の閉鎖確認。
