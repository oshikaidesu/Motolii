# 鎖のgate 6区間 — 全区間 `NEEDS_REVISION` と、完成条件の音声経路の確定

日付: 2026-08-09
状態: **観察 / gate結果の記録。仮コード本体の修正は未反映**

## 0. この文書の扱い

[仮コード器具境界決定](2026-08-07-provisional-call-site-sketch-instrument-decision.md)に従う。
本書はgate結果の記録であり、**仮コードは非authorityのまま**である。
**closed orderのAUTHORITY欄へ引かない。** ここに書かれた実名は、
orderへ写す前にcurrent codeで再確認する。

## 1. 結果 — 6区間すべて `NEEDS_REVISION`

[保全済み仮コード](2026-08-08-call-site-sketch-artifacts.md)のうち、
背骨（outcome A）を除く未通過6区間に鎖のgateを掛けた。

| 区間 | `ERRORS` | `SEAM_BLOCKED` | `OVER_UNKNOWN` | `FORBIDDEN` | `BLOCKS_COMPLETION` |
|---|---|---|---|---|---|
| b1（media / 保存 / 再生） | 14 | 3 | 3 | 1 | **3** |
| b2（書き出し / 日常操作 / panel） | 9 | 4 | 0 | 2 | 0 |
| Tune / Compose | 11 | 4 | 0 | 4 | **2** |
| Inspect / Fork | 6+ | — | — | — | 0 |
| Author | 3 | 1 | 0 | 3 | 0 |
| Publish / Reuse | 15 | 6 | 4 | 4 | 0 |

**全区間 `NEEDS_REVISION`。** 背骨が gate 1回で `ERRORS 12 / SEAM_BLOCKED 4` を出した
実績と同程度であり、想定どおりである。

実施条件: Codex direct `gpt-5.6-sol` medium × 6並列、`--sandbox read-only`、
`scripts/run-observed-cli.py` で途中stream保存。合計 input 5.9M / output 54k。
起草側がAnthropic系である可能性が高いため、§6.45 に従い**非Anthropic**を検査側に置いた
（背骨のgateと同じ）。

### 検査項目を4点から5点へ増やした

§6.45 の検査4点に、**5点目「その `???` は完成条件を塞ぐか」**を追加した。

根拠は実測である。[完成条件の鎖](2026-08-08-completion-condition-call-site-sketch.md)で
**起草者自身が「音楽同期」を拍同期編集と読み替えていた**誤りが、検査4点では出なかった。
同じ穴を6区間分素通りさせないために足した。実際、下記§2はこの5点目で出ている。

## 2. 最大の発見 — 完成条件の音声経路は**両端ともBUILT_UNWIRED**

完成条件は「3〜5分・音楽同期（=音声mux）のMVを1本書き出す」である。
gateはその両端が実装済みでありながら**製品経路から到達できない**ことを検出した。
**supervisorが現物で再確認済み。**

### 書き出し側 — muxする関数はあるが、CLI subcommandが到達しない

- `export_document_video`（**音声mux実装**）— `crates/motolii-export/src/lib.rs:221`。
  `resolve_audio_export` → `mux_soundtrack` / `mux_mixed_pcm`
- 呼び出し元は `crates/motolii-cli/src/document_export.rs:21`。
  `crates/motolii-cli/src/lib.rs:273` で `pub use document_export::export_document_file as export_document` として**公開済み**
- **しかし `main.rs` の subcommand は `ExportOverlay` / `ExportProject` / `VerifyB4` / `Help` の4つで、
  前2つはどちらも `export_overlay_video`（`lib.rs:121`、mux無しのM1最小ループ）へ行く**
- 現状 `export_document_video` を実際に呼んでいるのは test 3本
  （`d6_audio_mux.rs` / `ag4_audio_export.rs` / `d3e_preview_export_same.rs`）だけ

仮コードは `PreparedProject::export` を「音声muxへ到達する解決済み経路」と書いていた。**誤りである。**

### 取り込み側 — 音声を含める構築子はあるが、仮コードは落とす方を名指ししていた

- `build_import_clip_source(asset, ImportAvMode::VideoAndAudio { video_ordinal, audio_ordinal })`
  — `crates/motolii-doc/src/audio_edit.rs:21`
- 仮コードが名指しした `ClipSource::asset_video_only`（`schema.rs:580`）は、
  同関数の `ImportAvMode::VideoOnly` 分岐であり **`audio: Vec::new()` を構築する**

**両端とも「無い」のではなく「繋がっていない」。** 既定推定 `BUILT_UNWIRED` が正しかった。

## 3. 完成条件を塞ぐもの（gate横断で確定）

| # | 塞いでいるもの | 性質 |
|---|---|---|
| 1 | CLI subcommand が `export_document`（mux経路）へ到達しない | **配線のみ**。関数もpublic再exportも実在 |
| 2 | import が `asset_video_only` 側を使うと音声が落ちる | **呼び分けのみ**。`VideoAndAudio` は実在 |
| 3 | `pick_media_file` — 製品から素材を選ぶ入口が無い | 席が無い |
| 4 | `insertion_index` / target track選択 — mediaをDocumentへ置く製品intentが無い | 席が無い |
| 5 | `create_project_at` の製品接続 — 製品New経路が無い（低水準の初回永続化は実在） | 席が無い |
| 6 | `???_rn_inspector_parameter_edit_route` | 席が無い |
| 7 | `???_effect_enabled_toggle_ui` | 席が無い |
| 8 | `N-SOUNDTRACK-WRITE` — 楽曲bedを作品へ据える編集操作が無い（[既出](2026-08-08-completion-condition-call-site-sketch.md)） | 席が無い |

**1と2は配線だけである。** 3〜8は席を作る必要があるが、いずれも新規の製品意味ではない。

塞がないと判定されたもの: thumbnail、asset の undo境界、`open_mode_gate`、`save_as`、
`expected_revision`、`seek_while_playing`、export progress / cancel / artifact通知、
日常編集操作、layout保存、diagnostic adapter、effect reorder、Kit接続。

## 4. 区間別の主要指摘

**行番号ずれは全区間に多数ある**（b1だけで7件）。ここでは契約の形が違うものだけ挙げる。

### b1（media / 保存 / 再生）

- **`DocumentWriter` はDocumentを内部所有する。** 外側の `doc.assets` へ登録してもWriter側へ反映されない（`SEAM_BLOCKED`）
- Asset admissionは**Command境界にあり Undo可能**（`Command::AdmitAsset`、`prepare_admit_asset`）。仮コードは「Command外」と誤記し `???` を過剰計上していた
- 初回永続化経路は実在する（`ProjectSession::acquire` + `save_with_journal`、既定 `checkpoint: true`）
- `DocumentEditRuntime::new(session, ...)` は session を**値渡しで奪う**ため、以降 session は使えない（`SEAM_BLOCKED`）
- **`ReadOnlyNewer` からwritable Runtimeを構築するのは停止線違反**（`SaveRejectedReadOnlyNewer`）

### b2（書き出し / 日常操作 / panel）

- §2の音声mux経路の誤り
- **delete の製品keymap bindingが存在しない。** 製品keymapは Undo / Redo / Cancel の3本のみ
- duplicate の公開APIは `DocumentWriter::duplicate_track_item(&mut self, source) -> Result<GestureId, _>` で、
  **commandを返さずその場で適用しUndo登録まで完結する**。prepared-request経路へは入らない（`SEAM_BLOCKED`）
- menu は `LayoutAction::Hide/Restore` を直接生成しており、**CommandIdを渡す継ぎ目ではない**
- `LayoutAuthority` の構築子は `built_in()` のみで、保存値を受ける引数が無い

### Tune / Compose

- 完成条件を塞ぐ `???` が2件（RN Inspector の parameter編集route、effect enabled toggle）
- legacy / frozen な実名を現行製品routeとして扱っている箇所がある

### Inspect / Fork

- `gesture_identity` の戻り値は `Result<Option<..>, ..>` であり、しかも
  **`core.filter.opacity` v1 の `amount` 専用**。他Effectは `ActiveEffectContractMismatch`
- `scripts/new_plugin_crate.py` は**存在しない**。現行は `scripts/new-plugin-crate.sh`

### Author / Publish / Reuse

- Publish区間が最多（`ERRORS` 15 / `SEAM_BLOCKED` 6）。ただし完成条件は塞がない

## 5. 仮コードの扱い

**本書は仮コード本体を修正していない。** 6区間は依然 gate 未反映であり、
**この状態で施工を駆動しない**（§6.45）。

修正を反映するか、gate結果を直接orderのCURRENT FACTSへ写すかは、次のsupervisor判断とする。
**どちらにせよ、orderへ書く実名はcurrent codeで再確認してから使う** —
本書の実名もgateの報告であって、one more hopを挟んでいる。

## 6. 読み取れること

- **「M3は繋ぎ直し」という読みは、完成条件のcritical pathでも成立した。**
  塞いでいる8件のうち2件は配線だけ、残り6件も席の新設であって新規の製品意味ではない
- **gateは安い。** 6区間を1周、read-only、コードもmainも動かさずに、
  完成条件を塞ぐ8件と契約形の誤り約60件が出た
- **仮コードを鵜呑みにして発注していたら、6区間分の空振りを踏んでいた。**
  発注を止めた判断が正しかったことを、gateが事後に確認した形である
