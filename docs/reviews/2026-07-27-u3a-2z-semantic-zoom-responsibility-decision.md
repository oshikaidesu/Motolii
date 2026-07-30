# U3a-2Z windowed native Timeline semantic zoom 責任所在決定

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2Z: **DONE**

## 1. 目的と非目標

`U3a-2`（windowed native Timeline）の semantic zoom について、**段階の中身を一切決めず、責任所在（どの層が何を所有し、何を所有しないか）だけ**を docs で閉じる。本粒は docs-only である。Rust / JS / JSX / CSS / fixture / bench / spike raw / manifest は読むだけで変更しない。

非目標は §7 を参照する。

## 2. authority から引いた事実

### A1 三段呼称と identity 不変（M3 後半発覚プレモーテム §2 LD-8、§5 U3a 行）

[decision-index](../decision-index.md) の M3 後半発覚プレモーテム行が指す正本（§2 LD-8、§5 U3a 行）は、三段呼称の正本を **遠景 density / 中景 cluster / 近景 individual**（Motolii 語）とする。U3a は三段を**同じ時間 range と stable ID projection** から作り、zoom 境界前後で選択 identity・playhead・visible range を保ち、**density pixel を Document object identity に使わない**とする。

### A2 Core と bundled Host module の軸分離（軸分離 §3）

[軸分離](2026-07-22-m3-surface-extension-axis-separation.md) §3 は、Timeline を全体として Core にも plugin にも置かない。**Core** = RationalTime、clip/key identity、projection 入力、selection 意味、typed intent、D2 command、Undo/Cancel。**bundled Host module** = visible range layout、**semantic zoom**、hit-test、native 描画、gesture adapter、bounded accessibility projection。**plugin 候補** = 標準意味へ typed command を提案する Tool のみ（独自 Timeline 正本 / Undo / Document 探索を持たない）とする。

### A3 native time surface owner（decision-index UI runtime 行、UI runtime 責任境界）

[decision-index](../decision-index.md) `UI runtime責任境界` 行と [UI runtime 責任境界](../ui-runtime-architecture.md) は、time/Z 軸へ row 同期する rail・bar・key・playhead、Preview、handle、gizmo、高頻度 scrub を **native Rust/wgpu module** が所有する。React 製品所有は Browser / Inspector / form / panel / `KEYS`・`LAYERS` tool panel / Stage chrome。Document は D2 single writer、Transient selection/session は Host coordinator だけが所有し、全 surface は同じ revision 付き snapshot の read-only projection とする。

### A4 五層状態と Timeline scroll/zoom（M3 仕様 U0b、decision-index、interaction prototype P48/P49、G0-2）

[M3 仕様](../specs/M3-ui-integration.md) U0b 行、[decision-index](../decision-index.md) `UI panel resize Browser Inspector Timeline hierarchy rail` 行、[interaction prototype decision ledger](2026-07-19-m3-interaction-prototype-decision-ledger.md) P48/P49、[M3着手前決定 G0-2](2026-07-16-m3-preflight-decisions.md#22-状態の持ち場と寿命) は、代表状態を Document / User settings / Workspace profile / **Project session** / Transient の 5 層へ分類する。既決の `Timeline scroll/zoom` は **Project session** に属し、Document・Undo を変えない。**新しい所有寿命や恒久 workspace/session 形式をこのタスクで発明しない**。

### A5 責任処分（CU-105R §3）

[CU-105R](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) §3 は、layout/cull/hit-test = `PASS`（U3a-1I 済、再実装しない）、1k/100k = `REDUCE`（既存 capacity evidence のみ）、**semantic zoom = `STOP`（U3a-2）**、selection 不変 = `STOP`（CU-106P）、**playhead / range 不変 = `STOP`（owner 未決）** とする。

### A6 readiness 区分（U3a-2S §3）

[U3a-2S](2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md) §3 は、(A)〜(D) 4 区分とし、(A) は G0-9L 済 fixed-Mac prerequisite evidence と headless U3a-1I・隔離 spike 読取まで、(C) は製品 window / consumer 入力待ち、(D) は renderer 採択範囲（`U3a-2R` で `DONE`）とする。

### A7 surface owner と renderer 非移管（U3a-2R §5(b)、§6）

[U3a-2R](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) §5(b)、§6 は、surface owner を `motolii-ui` 内 native Rust/wgpu module とし、採択結果が `direct_vello` でも `egui_vello` でも time surface の owner は React 製品 package へ移らない。第一候補は「比較中の優先順位つき比較入力」であり採択済み契約ではないとする。

### A8 selection consumer 到達性（CU-106S §1〜§3、§5）

[CU-106S](2026-07-27-cu-106-selection-consumer-split-decision.md) §1〜§3、§5 は、`TimelineHit` の production caller 0 / pointer 入力不在 / egui Timeline 製品面不在。CU-106P の入場は **U3a-2 入場範囲決定と non-test consumer 成立**を要する。lint 抑制・dummy caller を到達性として数えないとする。

### A9 G0-9L の意味（G0-9 段階化 §6〜§8）

[G0-9 段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §6〜§8 は、`G0-9L: PASS` が意味するのは固定 Mac の platform prerequisite evidence だけ。W0b / H1b / Motolii Studio Preview / 通常製品 window 結合 / Distribution Ready / egui baseline 削除は引き続き停止。Mac 結果の Windows・追加 monitor 外挿を拒否する。

### A10 処分語彙（依存優先ゲート §3）

[依存優先ゲート](2026-07-24-dependency-first-responsibility-gate.md) §3 は、責任処分語彙を `PASS / REDUCE / STOP` と `FROZEN / DELETE-LATER / KEEP-AS-EVIDENCE` のみとし、`PASS` 以外では実装ループへ入らないとする。

証拠カプセル分類は [U3a-2S §4](2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md) を参照する。証拠 admissibility は [U3a-2R §4](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) を参照する。本粒では再分類しない。

## 3. 責任所在表

| 区分 | owner | 処分 | 根拠節 | 本粒で解禁しないもの |
|---|---|---|---|---|
| (1) Document / domain 意味 | Core（RationalTime、clip/key identity、projection 入力、selection 意味、typed intent、D2 command、Undo/Cancel） | PASS（意味正本）/ semantic zoom 段階は Document へ焼かない | A2、A3 | density / cluster / individual の段階定義、閾値、serde への焼き込み |
| (2) Project session viewport 状態 | Project session（Host coordinator 経由） | PASS（既決分類） | A4 | zoom 値 shape、default、復元規則、新しい恒久 workspace/session 形式 |
| (3) headless projection / layout / cull / hit-test | `motolii-ui::timeline_projection`（caller 注入 viewport・metrics） | PASS（U3a-1I 済、再実装しない） | A5、[U3a-1 owner/visibility分割 §7](2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md#7-完了証跡u3a-1i) | 同 module への semantic zoom 状態追加、selection / playhead / owned range |
| (4) native Rust/wgpu windowed presentation | bundled Host module（`motolii-ui` 内 native module） | STOP（実装粒は別） | A2、A3、A7 | renderer 勝者、段階の中身、製品 window 結合 |
| (5) selection / playhead / range | Core 意味 + Host coordinator（selection）；playhead / range owner は未決 | STOP（CU-106P / owner 未決） | A5、A8 | production caller なしでの CU-106P 実装、playhead / range の既定 owner |
| (6) React 製品 surface | Browser / Inspector / form / panel / `KEYS`・`LAYERS` / Stage chrome | PASS（所有境界） | A3、[React 移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md) | time rail・bar・key・playhead・高頻度 scrub の React 所有 |

## 4. 二重所有拒否表

| owner | 所有してはならないもの |
|---|---|
| Core / Document | semantic zoom 段階の中身・閾値、viewport scroll/zoom の正本、density pixel を object identity とする解釈 |
| Project session | clip/key identity、selection 意味、D2 command 正本、Undo 履歴 |
| `timeline_projection`（headless） | semantic zoom 状態、selection / playhead / range、windowed 描画、gesture 解釈 |
| bundled Host native module | Document 正本、独自 Undo、plugin 独自 Timeline 正本、React time surface |
| React 製品 package | time/Z 軸 rail・bar・key・playhead、Timeline semantic zoom 状態、高頻度 scrub 正本 |
| density 表示 | Document object identity（A1） |

## 5. 未決として残す点

authority が層を割り当てていない、または本粒の非目標として触れない項目は次のとおり **未決** のまま置く。

- density / cluster / individual の閾値、切替条件、段階の中身、各段階の描画内容。
- viewport の値 shape、default、復元規則（A4 は Project session 所属のみ既決）。
- playhead / visible range の owner と不変規則（A5 は `STOP` のみ）。
- renderer 勝者、`direct_vello` / `egui_vello` の採択、egui baseline 削除（A7）。
- production pointer 入力、`TimelineHit` production caller、CU-106P/F / U2h-1P 実装粒の入場（A8）。

## 6. 次の最小粒

候補は主担当Codexの事前審査で固定した閉集合のみ。実装担当は別の新 ID を発明しない。

| ID | 状態 | 内容 |
|---|---|---|
| `U3a-2A` | **DO** | renderer **採択判断** docs 粒（[U3a-2R](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) §7 の 4 条件が BASE_SHA 事実で成立済み。起票前にループ外 Fable 5 read-only 助言を得る） |
| `U3a-2` 本体 | WAIT | windowed 実装は責任所在・採択 docs 閉包後 |
| `CU-106P` / `CU-106F` / `U2h-1P` | WAIT | 実 consumer surface 待ち（A8） |
| `CU-0A08BT` / `CU-0A08IT` / `U2c-2` | WAIT | 既存依存待ち |

`U3a-2A` を `DO` にした根拠: 本決定 `DONE`、[implementation-ledger](../implementation-ledger.md) 各候補の依存確認セル照合のうち、[L1測定追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §3・§6の同一session raw・provenance・反対側review完了事実が [U3a-2R](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) §7 entry gate 4 条件を満たすため、BASE_SHA 事実と本決定だけで入場可となるのは renderer 採択判断粒のみ。semantic zoom 段階内容・playhead / range owner・製品 surface 実装は束ねない。

## 7. 非目標

- density / cluster / individual の**閾値、切替条件、段階の中身、描画内容**の決定。
- renderer 勝者、`direct_vello` / `egui_vello` の採択、egui baseline 削除、絶対性能閾値、合否基準、製品公約の新設。
- production pointer / 入力 event、`TimelineHit` production caller、CU-106P/F/U2h-1P の実装または `DO` 昇格。
- 公開 API、`DomainIntent`、Document、serde、journal、Undo/history、plugin 契約の変更。新しい serde default の発明。
- Rust / JS / JSX / CSS / fixture / bench / golden / visual 期待値 / spike raw / manifest の変更。
- W0b / H1b / Motolii Studio Preview / 通常製品 window 結合 / Distribution Ready / **G0-9D** の解禁または閉集合変更。
- `U3a-2S` / `U3a-2S-R2` / `U3a-2S-R3` / `U3a-2R` / `CU-105R` / `CU-106S` / `U2h-1PR` の**決定内容・状態・順序の書き換え**（current mirror 行の同期のみ許可）。
- 外部製品を根拠・再利用箇所・変更案に含めること。

## 8. STOP 条件

1. 責任所在を閉じるために semantic zoom の**段階の中身・閾値・切替条件**を決める必要が出た。
2. renderer 勝者、恒久 renderer 契約、egui baseline 削除、絶対性能閾値を決めないと閉じない。
3. authority の**節番号で裏づけられない**事実を書く必要が出た（特に Project session viewport の値 shape・default・復元規則）。
4. 既決の Timeline scroll/zoom = Project session / Document・Undo 不変を変更したくなった、または新しい恒久 workspace/session 形式を発明したくなった。
5. 公開 API、`DomainIntent`、Document、serde、journal、Undo、plugin 契約、永続 layout 形式の追加・変更が要る。
6. production caller、pointer 入力、lint 抑制、dummy caller、`#[allow]`、test-only accessor の新設が要る。
7. `crates/**` / `ui/**` / `docs/mocks-ui/**` / `docs/spikes/**` / fixture / bench / golden を変更したくなった、または `npm install` を実行したくなった。
8. `G0-9L: PASS` を U3a-2 入場可・親 G0-9 完了・Distribution Ready・Windows/追加 monitor 合格と同義に書きたくなった、または G0-9D の閉集合へ触れたくなった。
9. PRODUCT-ASSET lane の `DO` が 2 件以上になる、または親名 `U3a-2` / `CU-105` / `CU-106` で closed order を作れると書きたくなった。
10. 外部製品を根拠・再利用箇所・変更案に含めたくなった。
11. 既存決定（U3a-2S / U3a-2S-R2 / U3a-2S-R3 / U3a-2R / CU-105R / CU-106S / U2h-1PR）の**意味・状態・順序**を変えないと整合しない。
12. docs 整合 command が緑にならず、期待値・golden・guard 側を書き換えれば通ると見えた。

## 9. 必須負例 N1〜N10

- **N1**: density / cluster / individual の閾値・切替条件・段階の内容・描画内容を決める。
- **N2**: `direct_vello` / `egui_vello` の一方を勝者・優位・推奨と書く、または比較 arm を増やす。
- **N3**: fps / ms / MB / 件数の**合否閾値**を新規に書く。
- **N4**: `timeline-bench` 1k/100k や visual parity spike を headless 正しさ・D2・selection consumer・renderer 採択・製品性能の合格証拠へ昇格させる。
- **N5**: `G0-9L: PASS` を U3a-2 入場可・親 G0-9 完了・Distribution Ready・Windows/追加 monitor 合格と同義に書く。
- **N6**: native time surface（rail / bar / key / playhead / 高頻度 scrub）を React 製品 package の責任へ移す、または `KEYS` / `LAYERS` の所有を変える。
- **N7**: §7 current mirror のいずれかだけを更新して他を古いまま残す、または U3a-2S/R2/R3 の歴史 receipt を書き換える。
- **N8**: `docs/mocks-ui` を現行実装として更新する、`npm install` を実行する、guard 側の期待値・固定 hash を書き換える。
- **N9**: PRODUCT-ASSET lane の `DO` を 2 件以上にする、または親名 `U3a-2` / `CU-105` / `CU-106` で closed order を作れると書く。
- **N10**: viewport の値 shape・default・復元規則、または playhead / range owner を「もっともらしい既定」で埋める、既決の Timeline scroll/zoom = Project session を別層へ移す、または density pixel を Document object identity として扱う記述を書く。

## 10. 建設的所見（非拘束）

1. §3 責任所在表の 5 列（区分 / owner / 処分 / 根拠節 / 解禁しないもの）は [U3a-2R §4](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) admissibility 表と同型にしておくと、playhead / range owner 粒と CU-106P 粒が行追加で足せる。
2. (3) headless module と (4) native presentation を別行に固定しておくと、「semantic zoom 状態を `timeline_projection.rs` へ足す」二重所有を表 1 行の照合で却下できる。
3. §13 相当の grep 負例（外部製品名 0 件、合否閾値 0 件、PRODUCT-ASSET `DO` 件数）は以後の docs-only 粒へ転用できる。
4. 次は `U3a-2A` renderer 採択判断 docs 粒のみ。semantic zoom 段階内容・playhead / range owner・製品 surface 実装を束ねない。
