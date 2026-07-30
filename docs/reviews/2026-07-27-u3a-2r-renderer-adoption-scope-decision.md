# U3a-2R windowed native Timeline renderer採択範囲決定

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2R: **DONE**

## 1. 目的と非目標

`U3a-2`（windowed native Timeline）の区分 (D) について、**renderer 採択判断そのものではなく**、
後続の比較・採択粒が閉じた境界の内側で動くための docs 範囲を確定する。本粒は docs-only である。
Rust / UI / spike raw / manifest は読むだけで変更しない。

非目標: renderer 勝者の選定、`direct_vello` / `egui_vello` のどちらかの採択、egui baseline 削除、
絶対性能閾値、semantic zoom 段階の中身、playhead / range owner、selection 入力、製品 window 結合、
公開 API・Document・永続形式の変更。

## 2. authority から引いた事実

### candidate 閉集合（L1 追補 §2）

[L1測定追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §2 は L1 比較 arm を
`direct_vello` と `egui_vello` の**二つだけ**に限定する。pure direct wgpu を第三 arm にしない
（同 §2：別 glyph renderer 新造か fixture 縮退が必要になり同条件比較でなくなる）。

### 他候補の処分（再選定 §3）

[renderer再選定](2026-07-21-native-surface-renderer-reselection.md) §3 は、direct wgpu primitive batch を
**FIRST CANDIDATE**、Vello 0.9 を **ADJUNCT / 既存採択**、GPUI を **PATTERN**、Slint / Iced / Qt Quick /
Skia を **REJECT for product path**、lyon を **PATTERN / fallback candidate**、glyphon / cosmic-text を
**REJECT as duplicate stack** と処分する。

### 第一候補の身分（decision-index・反対側 §4・L1 追補 §4）

[decision-index](../decision-index.md) の `native surface renderer …` 行の状態は **比較中**。
[反対側レビュー](2026-07-21-native-surface-renderer-counter-review.md) §4 は「第一候補は維持。
**採否の正本は本回答でなく再選定の合格条件を満たす spike 証拠**」とする。
[L1測定追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §4 は「絶対閾値、renderer 勝者、
egui 削除を決めない」とする。よって第一候補は既決ではなく、優先順位つきの比較入力である。

### 判定条件（G0-9 段階化 §5 L1）

[G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §5 L1 は、同一 Mac / OS / display / window /
present mode、同一 1,000 clip / 100,000 key fixture、同一 WKWebView 枚数・操作列・warm-up・測定時間、
CPU frame / input latency・pass 別 GPU timestamp・RSS・resource 生成回数・readback 回数の raw を要求する。
測定後に絶対閾値を追加しない。採択根拠は正しさ、resource hot-loop 生成 0、readback 0、
既存 G0-4 手順の p50 / p95 と外れ値である。

### 停止範囲（G0-9 段階化 §7）

[G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §7 は W0b / H1b / Motolii Studio Preview /
通常製品 window 結合 / G0-9D / egui baseline と fixture の削除 / Document・journal・plugin ABI・
永続 layout 形式を引き続き停止するとする。

### surface owner（decision-index UI runtime 行）

[decision-index](../decision-index.md) `UI runtime責任境界` 行は、time / Z 軸の rail・bar・key・playhead・
Preview・handle・gizmo・高頻度 scrub を native Rust / wgpu module が所有する。React 製品所有は
Browser / Inspector / form / panel / `KEYS`・`LAYERS` tool panel / Stage chrome である。

### 証拠カプセル分類（U3a-2S §4）

証拠カプセルの処分表は [U3a-2S readiness分割](2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md) §4
をそのまま参照する。本粒では再分類しない。

### 責任処分語彙（依存優先ゲート §3）

[依存優先ゲート](2026-07-24-dependency-first-responsibility-gate.md) §3 の `PASS / REDUCE / STOP` と
`FROZEN` / `DELETE-LATER` / `KEEP-AS-EVIDENCE` のみを用いる。

## 3. candidate 閉集合表

| arm | 意味 |
|---|---|
| `direct_vello` | [L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §2 の製品候補 stack（direct wgpu primitive batch 主経路 + Vello 局所 pass） |
| `egui_vello` | 同 §2 の egui integration cost を加えた現行 baseline |

**比較対象外**（L1 比較 arm に含めない。処分語は [再選定](2026-07-21-native-surface-renderer-reselection.md) §3 と
[L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §2 のまま）

| 対象 | 処分 |
|---|---|
| pure direct wgpu 第三 arm | L1追補 §2：同条件比較ではないため除外 |
| GPUI | 再選定 §3：**PATTERN** |
| Slint | 再選定 §3：**REJECT for product path** |
| Iced | 再選定 §3：**REJECT for product path** |
| Qt Quick | 再選定 §3：**REJECT** |
| Skia | 再選定 §3：**REJECT** |
| lyon | 再選定 §3：**PATTERN / fallback candidate** |
| glyphon / cosmic-text | 再選定 §3：**REJECT as duplicate stack** |

## 4. 証拠 admissibility 表

[U3a-2S §4](2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md) の証拠カプセル分類を前提とする。

| 証拠 | 採択判断へ使ってよい | 採択判断へ使ってはならない |
|---|---|---|
| CU-0G02B 同一 session raw（両 arm） | はい | — |
| CU-0G02 CPU / input / RSS raw | はい（**粒の証拠としてのみ**） | 両 arm GPU raw の代替、勝者導出 |
| G0-9L manifest（`PASS_FIXED_MAC_PREREQUISITE_EVIDENCE_ONLY`） | はい（prerequisite 文脈） | 親 G0-9 完了、製品粒入場可、Distribution Ready |
| `g0-9-surface-host` topology 部分証拠 | はい（topology 部分） | renderer 採用、G0-9 全 platform 合格 |
| headless `U3a-1I` の正しさ | はい（layout / cull / hit-test） | windowed renderer 勝者、製品 60fps |
| `timeline-bench` 1k / 100k | いいえ | capacity 証拠のみ。headless 正しさ・D2・製品 60fps ではない |
| `g0-9-timeline-visual-parity` | いいえ | 製品未接続の外観 oracle |
| `g0-10-multi-surface-window` | いいえ | (B) G0-9D 候補・未証明。G0-9L 通常 topology の代替ではない |
| CU-0G02 raw への GPU 値後付け | いいえ | L1追補 §5 負例 |
| 片 arm だけの再実行 | いいえ | L1追補 §3 |
| CU-0G02 raw と CU-0G02B raw の数値連結 | いいえ | L1追補 §3 |
| Mac 結果の Windows / 追加 monitor / HDR 外挿 | いいえ | G0-9段階化 §8 |
| Rerun | いいえ | Motolii authority 外 |

## 5. 責任 owner

### (a) 判断 owner

PRODUCT-ASSET lane の `U3a-2` 系 docs 粒（主担当 Codex）。renderer 採択判断を実装粒・spike・
Grok 検収・外部 model へ委譲しない。勝者・閾値・egui 削除は別の closed order で閉じる。

### (b) surface owner

`motolii-ui` 内の native Rust / wgpu module（[decision-index](../decision-index.md) UI runtime 行）。
採択結果が `direct_vello` でも `egui_vello` でも、time surface（rail / bar / key / playhead /
高頻度 scrub）の owner は React 製品 package へ移らない。

## 6. 第一候補の身分

「direct wgpu primitive batch + Vello 局所」は **比較中の優先順位つき比較入力であり、採択済み契約ではない**。

根拠: [decision-index](../decision-index.md) `native surface renderer …` 行の状態語 **比較中**、
[反対側レビュー](2026-07-21-native-surface-renderer-counter-review.md) §4、
[L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §4。

既存の非対称は変更しない。[再選定](2026-07-21-native-surface-renderer-reselection.md) §2.2 は Vello を
採択済み局所 renderer とし、§4 は製品 native renderer の第一候補 stack を direct wgpu + Vello 局所とする。
egui は製品 runtime 非採用・比較 baseline 保持（再選定 冒頭・§4 の現行記述）。

## 7. 採択判断を開始してよい条件（entry gate）

次を**すべて**満たす場合に限り、renderer **採択**粒（勝者選定）を起票する。一つでも欠けたら起票しない。

1. 両 arm の同一 session raw が存在する（[L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §3）。
2. provenance（rustc / cargo / locked dependency / 実行 commit / 固定 Mac 構成）が両 arm で一致する（同 §3）。
3. scenario / input / source / font / glyph digest / window / present mode / WebView 枚数 / warm-up /
   測定時間が一致する（同 §3）。
4. 反対側 review P0 / P1 = 0（L1追補 §6 完了条件の二段階審判を満たした証拠のみ採用）。

## 8. STOP 条件

本粒（範囲決定）と後続採択粒の双方に効く。

1. 範囲を閉じるために renderer 勝者・恒久 renderer 契約・公開境界・絶対閾値・依存採択を決める必要が出た。
2. authority の節番号で裏づけられない事実を書く必要が出た。
3. 比較 arm を L1 追補 §2 以外へ広げたくなった（第三 arm、新 renderer、pure direct 復活）。
4. `timeline-bench` 1k / 100k や visual parity spike を採択合格証拠へ昇格させたくなった。
5. G0-9L PASS を製品粒入場可・親 G0-9 完了・Distribution Ready と同義に書いた。
6. native time surface を React 製品 package の責任へ移した、または `KEYS` / `LAYERS` 所有を変えた。
7. 台帳・decision-index・spec・README・縦 slice・handoff の鏡を片方だけ更新した。
8. `docs/mocks-ui` を現行実装として扱う、または npm install で guard を通そうとした。
9. `DO` を 2 件以上にした、または親名 `U3a-2` / `CU-106` で closed order を作れると書いた。
10. Rerun を根拠・再利用・変更案に含めた。

## 9. 必須負例 N1〜N10

- **N1**: 決定本文が `direct_vello` / `egui_vello` の一方を勝者・優位・推奨と書く。
- **N2**: 比較 arm を 3 つ以上にする、pure direct wgpu を第三 arm として復活させる、新 renderer 候補を足す。
- **N3**: 数値閾値、fps、ms、MB の**合否基準**を新規に書く（raw 引用と合否閾値は別物）。
- **N4**: `timeline-bench` 1k / 100k を headless 正しさ・D2・selection consumer・renderer 採択・製品 60fps の合格証拠へ昇格させる。
- **N5**: G0-9L PASS を U3a-2 入場可、親 G0-9 完了、Distribution Ready、Windows / 追加 monitor 合格と同義に書く。
- **N6**: native time surface（rail / bar / key / playhead / 高頻度 scrub）を React 製品 package の責任へ移す、または `KEYS` / `LAYERS` の所有を変える。
- **N7**: 台帳 / decision-index / spec / docs/README / 縦 slice / CU-106S / U2h-1P のいずれかだけを更新して他を古いまま残す。
- **N8**: `docs/mocks-ui` を現行実装として更新する、または npm install を実行する。
- **N9**: `DO` を 2 件以上にする、または親名 `U3a-2` / `CU-106` で closed order を作れると書く。
- **N10**: `U3a-2S` / `U3a-2S-R2` / `U3a-2S-R3` の決定内容・状態・順序を書き換える、または重複文言修理を `U3a-2R` lane 行の外へ広げる。

## 10. 次の最小粒

| ID | 状態 | 内容 |
|---|---|---|
| `U3a-2Z` | **DO** | semantic zoom **責任所在** docs 粒（中身は決めない） |
| `U3a-2` 本体 | WAIT | windowed 実装は範囲・責任・採択の docs 閉包後 |
| `CU-106P` / `CU-106F` / `U2h-1P` | WAIT | 実 consumer surface 待ち（据え置き） |
| `CU-0A08BT` / `CU-0A08IT` / `U2c-2` | WAIT | 既存依存待ち（据え置き） |

`U3a-2Z` を `DO` にした根拠: 本決定 `DONE`、
[CU-105R](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) §3 で semantic zoom が `STOP`（U3a-2）のまま、
[U3a-2S](2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md) §5 順位 2（`U3a-2R` 完了後に入場条件を再照合）が変わっていない。

## 11. React 境界（読取専用確認）

製品 React は Browser / Inspector / form / panel / `KEYS`・`LAYERS` tool panel（`KeyToolsCandidate`）を所有する。
time / Z 軸の rail・bar・key・playhead は native Rust / wgpu が所有する（decision-index UI runtime 行、
[React移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)）。
本粒は固定 React source asset（commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`）を 1 byte も変更しない。

## 12. 建設的所見（非拘束）

1. §7 entry gate は CU-0G02B raw 検証手順の再利用 checklist としてそのまま転用できる。
2. §4 admissibility 表は将来の G0-9D 証拠にも同じ列構造で拡張でき、表の作り直しを避けられる。
3. 判断 owner を PRODUCT-ASSET docs 粒へ固定したことで、`U3a-2Z` semantic zoom と renderer 採択粒を
   同一粒へ束ねる圧力を先に断てる。
