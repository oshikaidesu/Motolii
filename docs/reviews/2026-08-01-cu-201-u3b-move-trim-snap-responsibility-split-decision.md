# CU-201 U3b move / trim / snap 責任分割決定

- 日付: 2026-08-01
- 状態: **SPEC DONE / CU-201 SPLIT**
- 親: CU-201 / U3b / VS-2

## 1. 結論

CU-201を一つの実装orderへ送らない。現行Document commandにはClipの`start` / `duration`を
変更する契約が無く、move、trim、snap、native gesture、系列property oracleを同時に決めると、
恒久command意味とUI都合が混ざるためである。

次のDAGへ分割する。

1. `CU-201M-S SPEC`: Clip moveの永続意味、拒否、inverse、journal / Undo oracle
2. `CU-201M-C CORE`: 決定済みmove commandとWriter prepareを実装
3. `CU-201T-S SPEC`: in/out trimと`TimeMap`の関係、拒否、inverseを決定
4. `CU-201T-C CORE`: 決定済みtrim commandとWriter prepareを実装
5. `CU-201N-S SPEC`: snap対象、優先順、許容距離の単位、no-snap条件を決定
6. `CU-201P PRODUCT`: native Timeline gestureを既存hit identityとsingle writerへ接続
7. `CU-201R ORACLE`: random move/trim列、相対位置、全Undo、Cancel 0を固定
8. `CU-201E E2E`: 通常製品windowのmove→trim→Undo/Redo→reopenを完走

実行順は`M-S → (M-C || T-S) → T-C → N-S → P → R → E`とする。
M-S後、code所有のM-Cとdocs所有のT-Sは並列可能だが、同じ
`crates/motolii-doc/src/command.rs`とsingle writerを所有するM-C/T-Cは直列にする。
最初の`DO`はdocs-only `CU-201M-S`だけである。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [M3 U3b](../specs/M3-ui-integration.md)、[快適利用Work Map W2](2026-07-22-m3-comfortable-use-work-map.md)、[縦slice VS-2](2026-07-24-m3-vertical-slice-execution-decision.md) |
| `INTERNAL TARGET` | `Clip::{start,duration,time_map}`、`TimelineHit`のstable Layer identity、`DocumentWriter::apply_macro`、`DocumentEditRuntime::commit_command` |
| `OWNER` | start/duration/TimeMapはDocument。drag candidateとsnap previewはHost Transient。hoverはlocal presentation |
| `WRITE ROUTE` | 将来のtyped command → journal-first D2 → history → published snapshot。drag updateはDocument write 0、release成功だけ1 Undo |
| `GAP` | Clip intervalを変更するCommand / Writer prepare 0。move/trim rejection、TimeMap trim、snap集合・優先順・単位が未決 |
| `RESOLUTION ROUTE` | 既存projection/hit/selection/commit routeを`REUSE`し、未決の恒久意味だけをM-S/T-S/N-Sへ`REDUCE` |
| `DISPOSITION` | `RESOLVE`。CU-201親を直接実装せず、M-Sから順に閉じる |

## 3. 再利用する成立済み責任

- CU-105/U3a-1IのDocument→Timeline projection、layout、cull、typed hit-test
- CU-106Pのnative Timeline click→`TimelineHit`→primary selection publish
- `DocumentWriter::apply_macro`のatomic apply / rollback
- `DocumentEditRuntime::commit_command`のjournal-first→live apply→history→single publish
- `RationalTime`と現行validationのpositive duration、composition end上限

既存Rectangle plannerは`AddTrackItem`だけを構築する。move/trim plannerとして複製・一般化しない。
`SetProperty`はItemEnvelope / Effect parameter用であり、Clip interval変更へ流用しない。

## 4. 未決を次粒へ残す

### CU-201M-S

- moveが同一lane内の`start`だけを変えるのか、lane変更も含むのか
- composition先頭/末尾を越える入力のreject/clamp
- overlap許可、collision、rippleの有無
- command payload、inverse、merge可否、rejection precedence

### CU-201T-S

- in trim / out trimのpayload
- `source_start`、speed、overrun modeとclip durationの関係
- trimでsource timeを保つ境界
- zero/negative duration、composition外、overflowの拒否順

### CU-201N-S

- frame、playhead、他item edge等の現行target候補
- target priority、tie-break、許容距離をtime / logical pxのどちらで持つか
- zoom、DPI、fps変更時の決定性

これらを本分割で採択しない。

## 5. 依存再締結

CU-105 / CU-106は親が`SPLIT`だが、CU-201が再利用する実子責任は成立済みである。

- CU-105RとU3a-1I: projection/layout/hit-test成立
- CU-106P: primary selection consumer成立
- CU-106Fのessential focusはCU-201M-Sの永続command意味を止めない
- U3a-2 semantic zoom、playhead range、transportもM-Sの前提にしない

CU-204PはInspector diagnostic laneであり、CU-201の依存ではない。

## 6. STOP

- beat gridまたはuser markerをsnapへ入れる。これはU7 + GAP-16である
- marker persistence、BPM意味、Auto ripple、collision policyを推測する
- `SetProperty`やraw `&mut Document`でClip timeを変更する
- schema / journalを迂回するUI専用commandを作る
- hover、focus、marquee、semantic zoom、visible range、transportを束ねる
- moveとtrimの恒久commandを一つの曖昧なinterval editへ先取り統合する
- random testの期待値から未決意味を逆算する
