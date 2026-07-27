# U3a-2A windowed native Timeline renderer採択決定

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2A: **DONE**

## 1. 目的と非目標

`U3a-2`（windowed native Timeline）区分 (D) の **renderer 採択判断**を docs で確定する。
判断は **confirmation 型**である。すなわち「CU-0G02B raw に、既決の
`egui = 製品runtime非採用 / 比較baseline保持` と
`direct wgpu primitive batch 主経路 + Vello 局所pass = 第一候補 architecture` を
**改訂させる欠陥が存在しない**」ことを確認し、その結果として `direct_vello` を採択する。

**`direct_vello` が性能で勝ったとは書かない。** raw は mixed であり、
反対側の行（§4 表）を一行も削除・圧縮・脚注化せずに本文へ残す。

本粒は docs-only。Rust / JS / JSX / CSS / fixture / bench / golden / spike raw / manifest /
lockfile / package.json は読むだけで 1 byte も変更しない。

非目標（§7 と同義）:

- egui baseline / fixture / spike の削除、`DELETE-LATER` の発火。
- 絶対性能閾値、fps / ms / MB / 件数の合否基準、製品 60fps 公約、有意性判定。
- 製品 window 結合、Motolii Studio Preview、W0b / H1b、Distribution Ready、**G0-9D** の解禁または閉集合変更。
- Windows / 追加 monitor / HDR / 追加 hardware への外挿。
- semantic zoom 段階の中身・閾値・切替条件・描画内容。
- playhead / visible range の owner 決定、および`U3a-2P`以外の新 ID の採番。
- CU-106P / CU-106F / U2h-1P の実装または `DO` 昇格、production pointer 入力、`TimelineHit` production caller。
- 公開 API、`DomainIntent`、Document、serde、journal、Undo / history、plugin 契約の変更。新しい serde default の発明。
- Rust / JS / JSX / CSS / fixture / bench / golden / visual 期待値 / spike raw / manifest /
  lockfile / package.json / `docs/mocks-ui/**` / `crates/**` / `ui/**` / `spikes/**` の変更。`npm install` の実行。
- `U3a-2S` / `U3a-2S-R2` / `U3a-2S-R3` / `U3a-2R` / `U3a-2Z` / `CU-105R` / `CU-106S` / `U2h-1PR` の
  決定内容・状態・順序・負例の書き換え（current mirror 1 行の同期のみ許可）。
- 外部製品（Rerun を含む）を根拠・再利用箇所・変更案に含めること。
- 隣接チケットへの拡張、TODO stub、部分適用（mirror を片方だけ更新して終える）。

## 2. authority から引いた事実

### 2.1 entry gate（U3a-2R §7）

[U3a-2R](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) §7 の 4 条件は BASE_SHA 事実で成立している（§3 表）。

### 2.2 raw 健全性（gpu-*-raw.json `completeness` ほか）

両 arm とも `completeness` = `complete`（`gpu-direct-vello-raw.json` / `gpu-egui-vello-raw.json`）。
acquire = present（direct 1,922/1,922、egui 1,921/1,921）、`readback_count` = 0、
`query_result_readbacks` = 1、
`resource_creations.warmup` と `.measured` の pipelines / buffers / bind_groups / textures / query_sets が
すべて 0、`reconfigure_count` = 0、`timestamp_period_ns` = 1.0。
skip は direct 3 / egui 4（同 JSON key）。

### 2.3 測定 session と provenance（L1追補 §3、raw `toolchain`）

両 raw の `toolchain.measurement_session` = `cu-0g02b-20260724-01`。
rustc `1.96.1` / cargo `1.96.1` / `execution_commit` `7c3a590e33874d60f7fbb1e1ac40173011db7649` /
`lockfile_sha256` `6217d5946a84665bf61fcc4c3072d814364c5323c3376b0dbe9ba1ff40c26086` が両 arm 一致。
`scenario_digest` `089cbd00…b8ed1618`、`input_digest` `56517a58…d8d42718`、`source_digest` `14316ebe…3ee2331`、`font_digest` `833776a6…e1b3475`、`glyph_digest` `2a6986e5…abb44ddf`、
`conditions` = `Apple M4|Metal` / `Bgra8UnormSrgb|fifo|1` / `2880x1708@2` / `2-opaque-offline-child` / `g0-9-windowed-timeline.v1|1000-clips|100000-keys` が両 arm 一致。
測定は 30.450 s（1,802 frames）と 30.489 s（1,801 frames）。

### 2.4 反対側 review（L1追補 §6）

CU-0G02BH は Grok R2 P0/P1=0 `ACCEPT`、CU-0G02B は Grok が typed raw と比較 artifact を再照合し P0/P1=0 `ACCEPT`（L1追補 §6）。

### 2.5 既決 architecture（再選定 冒頭・§2.2・§3・§4、L1追補 §2、U3a-2R §3〜§6、G0-9段階化 §7、CU-105R §3、U3a-2Z §5）

- egui は **製品 runtime 非採用・比較 baseline 保持**（[再選定](2026-07-21-native-surface-renderer-reselection.md) 冒頭・§4）。Vello は採択済み局所 renderer（同 §2.2）。
- L1 比較 arm は `direct_vello` と `egui_vello` の二つだけ（[L1追補](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) §2）。
- [U3a-2R](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) §3 candidate 閉集合、§4 admissibility 表、§5 owner、§6 第一候補の身分。
- [U3a-2R](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) §5(b) / [U3a-2Z](2026-07-27-u3a-2z-semantic-zoom-responsibility-decision.md) A7：採択結果がどちらでも time surface owner は `motolii-ui` 内 native Rust/wgpu module。
- [G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §7：W0b / H1b / Motolii Studio Preview / 通常製品 window 結合 / G0-9D /
  egui baseline と fixture の削除 / Document・journal・plugin ABI・永続 layout 形式は停止継続。
- [CU-105R](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) §3 / [U3a-2Z](2026-07-27-u3a-2z-semantic-zoom-responsibility-decision.md) §5：playhead / visible range の owner は **未決（`STOP`）**。

他候補の処分は [再選定](2026-07-21-native-surface-renderer-reselection.md) §3 の表どおり（本粒では再分類しない）。

| 対象 | 処分（再選定 §3） |
|---|---|
| GPUI | **PATTERN** |
| Slint / Iced / Qt Quick / Skia | **REJECT for product path**（Qt Quick は **REJECT**） |
| lyon | **PATTERN / fallback candidate** |
| glyphon / cosmic-text | **REJECT as duplicate stack** |

## 3. entry gate 4 条件の充足表

| 条件 | 成立根拠（authority 内の実値） |
|---|---|
| (1) 両 arm の同一 session raw | 両 raw の `toolchain.measurement_session` = `cu-0g02b-20260724-01` |
| (2) provenance 一致 | rustc `1.96.1` / cargo `1.96.1` / `execution_commit` `7c3a590e33874d60f7fbb1e1ac40173011db7649` / `lockfile_sha256` `6217d5946a84665bf61fcc4c3072d814364c5323c3376b0dbe9ba1ff40c26086` が両 arm 一致 |
| (3) scenario / input / source / font / glyph digest・window・present mode・WebView 枚数・warm-up・測定時間の一致 | `scenario_digest` `089cbd00…b8ed1618`、`input_digest` `56517a58…d8d42718`、`source_digest` `14316ebe…3ee2331`、`font_digest` `833776a6…e1b3475`、`glyph_digest` `2a6986e5…abb44ddf`、`conditions` = `Apple M4\|Metal` / `Bgra8UnormSrgb\|fifo\|1` / `2880x1708@2` / `2-opaque-offline-child` / `g0-9-windowed-timeline.v1\|1000-clips\|100000-keys` が両 arm 一致。測定は 30.450 s（1,802 frames）と 30.489 s（1,801 frames） |
| (4) 反対側 review P0/P1 = 0 | L1追補 §6：CU-0G02BH は Grok R2 P0/P1=0 `ACCEPT`、CU-0G02B は Grok が typed raw と比較 artifact を再照合し P0/P1=0 `ACCEPT` |

## 4. raw 全景表

`gpu-direct-vello-raw.json` / `gpu-egui-vello-raw.json` / `gpu-comparison.json` から引用。導出統計は作らない。

| 指標 | direct_vello | egui_vello |
|---|---|---|
| `completeness` | `complete` | `complete` |
| acquire / present（成功/試行） | 1,922 / 1,922 | 1,921 / 1,921 |
| `readback_count` | 0 | 0 |
| `query_result_readbacks` | 1 | 1 |
| `resource_creations`（warmup / measured） pipelines・buffers・bind_groups・textures・query_sets | すべて 0 | すべて 0 |
| `reconfigure_count` | 0 | 0 |
| `timestamp_period_ns` | 1.0 | 1.0 |
| skip 回数 | 3 | 4 |
| CPU frame median (ms) | 12.473791 | 12.59275 |
| CPU frame p95 (ms) | 14.381041 | 14.082375 |
| CPU frame max (ms) | 17.163583 | 18.553375 |
| input median (ms) | 1.999792 | 2.169583 |
| input p95 (ms) | 2.403166 | 2.539792 |
| input max (ms) | 4.466708 | 6.819084 |
| RSS (B) | 152,322,048 | 129,138,688 |
| GPU sum median (ms) | 4.696875 | 4.757833 |
| GPU sum p95 (ms) | 5.140583（ほぼ同値） | 5.131959（ほぼ同値） |
| GPU sum max (ms) | 9.454708 | 8.771125 |
| Vello pass median (ms) | 4.381291 | 4.412583 |
| Vello pass p95 (ms) | 4.86275 | 4.821834 |
| native pass p95 (ms) | 2.746917（ほぼ同値） | 2.755083（ほぼ同値） |
| `gpu_timing.egui` pass（arm 固有） | `null` | median 0.011958 / p95 0.283584 / max 3.23075 (ms) |

## 5. 採択判断（confirmation 型）

CU-0G02B raw は entry gate 4 条件を満たし、両 arm complete・measured resource 生成 0・
pixel readback 0 であり、既決の `egui 製品runtime非採用` と
`direct wgpu primitive batch + Vello 局所pass 第一候補` を**改訂させる欠陥を含まない**。
よって製品 native Timeline renderer 経路として `direct_vello` を採択する。

本採択は `direct_vello` が `egui_vello` へ性能で勝ったことを根拠にしない。
raw は mixed であり、§4 表の反対側行は本決定によって否定されない。

## 6. 採択が意味しない範囲

- egui baseline / fixture / spike の削除禁止（比較 baseline 保持）。
- 絶対性能閾値、fps / ms / MB / 件数の合否基準、製品 60fps 公約、有意性判定の新設なし。
- 製品 window 結合、Motolii Studio Preview、W0b / H1b、Distribution Ready、**G0-9D** の解禁または閉集合変更なし。
- Windows / 追加 monitor / HDR / 追加 hardware への外挿なし。
- semantic zoom 段階の中身・閾値・切替条件・描画内容の決定なし。
- playhead / visible range owner、viewport 値 shape / default / 復元規則の決定なし。
- production pointer 入力と `TimelineHit` production caller、CU-106P/F / U2h-1P 入場なし。
- Rust / UI / fixture / bench / golden / spike raw の変更なし。

## 7. owner

[U3a-2R](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) §5 を踏襲する。

### (a) 判断 owner

PRODUCT-ASSET lane の `U3a-2` 系 docs 粒（主担当 Codex）。

### (b) surface owner

`motolii-ui` 内の native Rust / wgpu module。採択結果が `direct_vello` でも time surface（rail / bar / key / playhead /
高頻度 scrub）の owner は React 製品 package へ移らない。

## 8. 未決として残す点

- semantic zoom 段階の中身（閾値、切替条件、描画内容）。
- playhead / visible range owner と不変規則（CU-105R §3 / U3a-2Z §5 は `STOP` のみ）。
- viewport 値 shape / default / 復元規則。
- production pointer 入力と `TimelineHit` production caller、CU-106P/F / U2h-1P 入場。
- G0-9D、Windows / 追加 hardware。

## 9. STOP 条件

1. 採択を書くために `direct_vello` の**性能優位・勝者・推奨**を主張しないと文章が成立しない。
2. 採択を書くために絶対閾値、合否基準、有意性判定、製品公約を新設する必要が出た。
3. 反対側行（§4 表の egui 側が小さい／大きい行）を残すと決定が矛盾すると見え、行を削る・弱める・脚注化したくなった。
4. authority の**節番号または JSON key で裏づけられない**数値・事実を書く必要が出た。
5. egui baseline / fixture / spike の削除、G0-9D 閉集合の変更、W0b / H1b / 製品 window /
   Distribution Ready の解禁が必要に見えた。
6. playhead / visible range の owner、semantic zoom 段階の中身、viewport 値 shape / default /
   復元規則を「もっともらしい既定」で埋めないと閉じない。
7. 主担当Codexが採番済みの`U3a-2P`以外の新しい粒 ID を採番しないと次粒が書けないと見えた。
8. 公開 API / Document / serde / journal / Undo / plugin 契約 / 永続 layout 形式の変更が要る。
9. `crates/**` / `ui/**` / `docs/mocks-ui/**` / `docs/spikes/**` / fixture / bench / golden /
   lockfile を変更したくなった、または `npm install` を実行したくなった。
10. `./scripts/check-docs.sh` または guard test が緑にならず、
    **期待値・固定 hash・guard 側・golden を書き換えれば通る**と見えた。
11. PRODUCT-ASSET lane の `DO` が 2 件以上になる、または親名 `U3a-2` / `CU-105` / `CU-106` で
    closed order を作れると書きたくなった。
12. mirror の一部だけを更新して残りを古いまま残すことになった。
13. 外部製品を根拠・再利用箇所・変更案に含めたくなった。
14. allowlist 外の file を 1 byte でも変更する必要が出た。

## 10. 必須負例 N1〜N12

- **N1**: 決定本文が `direct_vello` を**勝者・優位・推奨・高速**と書く、または `egui_vello` を劣位と書く。
- **N2**: §4 表の反対側行（egui の CPU frame p95 / RSS / GPU sum p95 / Vello pass p95 / GPU sum max）を
  削除・圧縮・脚注化・「誤差」と断定する。
- **N3**: 新規の合否閾値（fps / ms / MB / 件数）、有意性判定、統計的検定、製品 60fps 公約を書く。
- **N4**: 比較 arm を 3 つ以上にする、pure direct wgpu を第三 arm として復活させる、新 renderer 候補を足す。
- **N5**: egui baseline / fixture / spike の削除、`DELETE-LATER` 発火、G0-9D 閉集合変更を書く。
- **N6**: `G0-9L: PASS` を U3a-2 入場可・親 G0-9 完了・Distribution Ready・Windows / 追加 monitor 合格と同義に書く、
  または Mac 結果を Windows / 追加 monitor / HDR へ外挿する。
- **N7**: native time surface（rail / bar / key / playhead / 高頻度 scrub）を React 製品 package の責任へ移す、
  または `KEYS` / `LAYERS` の所有を変える。
- **N8**: playhead / visible range owner、semantic zoom 段階の中身、viewport 値 shape / default / 復元規則を決める、
  または`U3a-2P`以外の新しい粒 ID を採番する。
- **N9**: `timeline-bench` 1k / 100k、`g0-9-timeline-visual-parity`、`g0-10-multi-surface-window`、
  CU-0G02 raw への GPU 値後付け、片 arm 再実行、CU-0G02 と CU-0G02B の数値連結を採択合格証拠へ昇格させる
  （U3a-2R §4 で「いいえ」の行）。
- **N10**: mirror（ledger / decision-index / spec / docs/README / 縦 slice / CU-106S / U2h-1P）のいずれかだけを更新して
  他を古いまま残す、または既存決定の意味・状態・順序を書き換える。
- **N11**: `docs/mocks-ui` を現行実装として更新する、`npm install` を実行する、
  guard 側の期待値・固定 hash・golden を書き換える。
- **N12**: PRODUCT-ASSET lane の `DO` を 2 件以上にする、親名 `U3a-2` / `CU-105` / `CU-106` で
  closed order を作れると書く、または外部製品を根拠・再利用箇所・変更案に含める。

## 11. 次の最小粒

playhead / visible range owner が最小残余である。owner は未決のまま、主担当Codexが採番した
`U3a-2P`だけを次の`DO`へ上げる。

| 候補 | 状態 | 内容 |
|---|---|---|
| `U3a-2P` | **DO** | playhead / visible range owner docs 粒。CU-105R §3 と U3a-2Z §5 が `STOP`（owner 未決）としている残余。**この粒で owner を決めない。** |
| `U3a-2` 本体 | WAIT | windowed 実装は範囲・責任・採択の docs 閉包後、かつ製品 window / consumer 入力の成立後 |
| `CU-106P` / `CU-106F` / `U2h-1P` | WAIT | 実 consumer surface 待ち（据え置き） |
| `CU-0A08BT` / `CU-0A08IT` / `U2c-2` | WAIT | 既存依存待ち（据え置き） |

## 12. 建設的所見（非拘束）

1. §4 の「raw 全景表」を direct 列 / egui 列の 2 列に固定しておくと、将来 G0-9D の hardware 追加時に**行追加だけ**で拡張でき、表の作り直しと再解釈を避けられる。
2. confirmation 型の判断文を「欠陥不在 → 既決 architecture 維持 → 採択」の 3 段で固定しておくと、後続の Windows / 追加 hardware 粒が「性能で再判定する」圧力を先に断てる。
3. mirror 7 面の同期は U3a-2R / U3a-2Z の 2 commit と同一 file 集合であり、
   `git show b2f9213f --stat` / `git show deee1574 --stat` を差分設計の照合に使える（内容の複製は禁止）。
4. `U3a-2P`を唯一の`DO`として明記し、owner未決と採番済みIDを分離すると、
   次の担当がownerを推測したり別IDを勝手に採番する事故を防げる。
5. playhead / visible range owner 粒は、U3a-2Z §3 の 5 列責任所在表へ**行 1 本を足すだけ**で書ける形になっている。
   本粒でその表を壊さないことが、次粒の最小化に直結する。

## 13. React 境界（読取専用確認）

time / Z 軸の rail・bar・key・playhead・高頻度 scrub は
native Rust/wgpu が所有し、React 製品所有は Browser / Inspector / form / panel /
`KEYS`・`LAYERS` / Stage chrome のまま。本粒は React source asset を 1 byte も変更しない。
