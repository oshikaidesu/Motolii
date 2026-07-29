# CU-107AD Place候補terminal admission実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-107TC`、`CU-0B05`

## 1. 実装

通常製品Hostにprivate `PlaceTerminalAdmission`を置き、`CU-107TC`が
`NoNonCommitCause`と分類した候補terminalだけを、一active dragにつき高々一件admitする。

- capture generationはBrowser/WebContent寿命の外側にあるProduct Hostが単調発行し、
  Browser replacement後も巻き戻さない。
- active generationとretired high-waterだけを保持し、matching terminalは原因にかかわらず
  activeをretireする。
- stale mismatchはcurrent activeをretireせず、duplicateとhigh-water以下のreplayは拒否する。
- admitted detailを破棄した後もretired high-waterを残すため、同世代の再適用を拒否する。
- lifecycle replacementはactiveをretireするがhigh-waterを保持する。

stateはprivate Transientであり、exact wire tuple、serde、Document、journalへ出さない。
本粒はadmitted terminalをprivate slotへ置くだけで、既存`PendingStageDrop`を生成せず、
単一下流commit境界への配送、D2、Undoを行わない。

## 2. 合格

- matching inside-Stage terminalは一回だけadmitされ、duplicateは拒否される。
- Escape / outside / capture lossは世代をretireするがadmitされない。
- stale terminalはcurrent activeを壊さず、current terminalは続けてadmitできる。
- detail eviction後もhigh-water以下のbegin / replayを拒否する。
- production terminal armがadmissionへ到達し、accepted delivery / Document edit / Undoへ到達しない。

## 3. Opus高速相談の反映

`claude-opus-5 --effort low`のread-only相談で、capture runtime再生成によるgeneration巻き戻りを
反例として確認した。generation ownerをProduct Hostへ上げ、非commit terminalでもretireし、
lifecycle後もhigh-waterを保持する形へ修正した。公開契約やscopeは増やしていない。

## 4. 次

`CU-107AD`を`DONE`とする。次PRODUCT-ASSET `DO`は`CU-107TD`。
