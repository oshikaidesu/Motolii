# CU-105 dense Timeline責任再確認

- 日付: 2026-07-27
- 状態: **決定**
- CU-105R: **DONE**
- CU-105: **SPLIT**

## 1. 再確認の目的

CU-105は「U3a layout/hit-test/dense Timeline projection」と
「1000 clip/100k key、zoom境界でselection/playhead/range不変」を一行に束ねていた。
完了済みU3a-1Iと現行コードへ再照合し、重複実装せず各責任を`PASS / REDUCE / STOP`で処分する。

## 2. コード事実

- `timeline_projection.rs`はDocumentからtop-level Clip / Position keyを読み取り、
  RationalTime正本、全clipのfirst-fit band、viewport cull、key優先Manhattan hit-test、
  typed unsupported / overflowを実装済みである。
- 公開境界からのintegration testはsmall deterministic fixtureでlayout / cull / hit-test /
  tie-break / typed reject / finite coordinateを固定済みである。
- 同moduleはselection、playhead、semantic zoom、owned rangeを保持しない。
  `TimelineViewport`とmetricsはcaller注入値である。
- 1k clip / 100k keyは`spikes/timeline-bench`にcapacity証拠がある。
  `motolii-testkit::perf`には同spikeの外部bench slotが既にあり、数値閾値を固定しない。
- M3仕様は既存1k/100kをcapacity証拠に限り、headless正しさやD2の証明にしない。
  遠景density〜近景individualのsemantic zoomはU3a-2へ置いている。

## 3. PASS / REDUCE / STOP

| CU-105要素 | 処分 | 現行責任 |
|---|---|---|
| Document→layout / cull / hit-test | `PASS` | U3a-1I `DONE`。再実装しない |
| 1k clip / 100k key | `REDUCE` | 既存spikeをcapacity evidenceとして保持。testkit外部bench slotを再利用し、CI絶対閾値や第2 fixtureを作らない |
| numeric metrics / viewport境界 | `PASS` | U3a-1Iのtyped validation / overflow / finite-coordinate test |
| semantic zoom境界 | `STOP` | U3a-2。G0-9待ちのwindowed native Timelineで遠景density〜近景individualを比較する |
| selection不変 | `STOP` | CU-106P。U2h-1P P5と実在callerが成立した後に非vacuousなoracleを置く |
| playhead / range不変 | `STOP` | owner未決。CU-106分割またはU3a-2で状態層を決めるまで実装しない |

CU-105の親行は`SPLIT`とする。U3a-1Iの成立済み責任を再び実装粒へ戻さず、
capacity / semantic zoom / selection-familyを別ownerへ配送したことをCU-105Rの完了とする。

## 4. 次の判断

`CU-106S`は[CU-106 selection consumer分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md)で
`DONE`となった。CU-106の「U2h selection kernelとessential focus」を
primary selection / essential focusへ分け、次を確認した。

1. U2h-1P producerと同じ差分で成立する最小production callerは現行runtimeに無い。
2. callerはU3a-2入場範囲と実consumer surface待ちであり、CU-106P/Fを`WAIT`とした。
3. primary selectionへessential focus、三surface接続、hidden件数、additive/range/marquee/AXを束ねない。

CU-105Rの`PASS`を、windowed native Timeline、semantic zoom、selection consumerの完成証拠にしない。

## 5. 非目標

- Rust / JS / fixture / benchmark / guard / golden / threshold変更。
- 1k/100k fixture、projection、hit-test、perf harnessの重複新設。
- U3a-2 / CU-106 / U2h-1Pの実装。
- semantic zoom段階、playhead / range owner、production input eventの決定。
- 公開API、Document、serde、journal、Undo/history、plugin契約の変更。

## 6. STOP

1. CU-105Rでsemantic zoomまたはplayhead / range ownerを決める必要がある。
2. 1k/100kをheadless正しさ、D2、selection consumerの完成証拠へ拡大したくなる。
3. 既存spike / testkit slotと別のfixture、bench、絶対CI閾値が必要に見える。
4. CU-106S再確認前にU2h-1P producerまたはdummy callerを実装したくなる。
