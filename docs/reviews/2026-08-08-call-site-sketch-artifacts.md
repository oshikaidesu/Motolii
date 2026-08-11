# 仮コード成果物 — 保全（非authority / 非compile / 器具）

日付: 2026-08-08
状態: **観察 / 器具の保全**

## 0. この文書の扱い

`docs/reviews/2026-08-07-provisional-call-site-sketch-instrument-decision.md` の器具である。

- **compileしない。`crates/`へ置かない。authorityにしない**
- **closed orderのAUTHORITY欄へ引かない**。仕様・schema・公開APIの根拠にしない
- closed orderへ変換され次第、該当部分は役目を終える

session scratchpad は session固有で失われるため、**再現に必要な成果物だけ**を本文書へ保全する。
prompt、evidence log、統合版(Fable 16,464字)は保全対象外である（再生成できる）。

## 1. gate通過状況

| 鎖 | 鎖のgate | 備考 |
|---|---|---|
| 背骨（outcome A） | **通過・修正済み** | `ERRORS: 12 / SEAM_BLOCKED: 4` を反映 |
| outcome B（media / 保存 / 再生） | **通過せず `NEEDS_REVISION`**(2026-08-09) | `ERRORS` 14 / `SEAM_BLOCKED` 3 / 完成条件を塞ぐ3件 |
| outcome C（書き出し / 日常操作 / panel） | **通過せず `NEEDS_REVISION`**(2026-08-09) | `ERRORS` 9 / `SEAM_BLOCKED` 4。CLI exportが音声muxへ到達しない誤りを含む |
| Tune / Compose | **通過せず `NEEDS_REVISION`**(2026-08-09) | `ERRORS` 11 / 完成条件を塞ぐ2件 |
| Inspect / Fork | **通過せず `NEEDS_REVISION`**(2026-08-09) | `gesture_identity` の戻り型と適用範囲が誤り |
| Author | **通過せず `NEEDS_REVISION`**(2026-08-09) | 起草時 `PARTIAL`。存在しない script を実名として記載 |
| Publish / Reuse | **通過せず `NEEDS_REVISION`**(2026-08-09) | `ERRORS` 15 / `SEAM_BLOCKED` 6（最多） |

**未通過の鎖は、施工を駆動する前に鎖のgateへ通すこと**（器具境界決定 §6.45）。
背骨で12件出たため、他も同程度の誤りを含むと想定する。

---

## 2. 背骨（outcome A）— **gate通過・修正済み**

## A. 背骨の呼び出し側（最短経路）

> **2026-08-08 鎖のgate（Sol / OpenAI）で `ERRORS: 12 / SEAM_BLOCKED: 4` を検出し修正済み。**
> 修正前は実名12件がずれ、挿入不可のseamを4件前提にしていた。

```rust
// ── 1. 表示中objectを選ぶ ───────────────────────────────── 【実装済 2026-08-07】
let projection = project_stage_geometry(&document, EvaluationTime::new(host.current_time), &tracks)?;
//   stage_hit_test.rs:25-30 / :77-80 — 2関数に分かれている
let canonical = view_local_to_canonical(x, y, width, height);
let hit       = hit_test_projected_layers(canonical, &projection);
match hit {
    StageHit::Layer(layer) => queue.push_replace_primary(layer),
    StageHit::Miss         => queue.push_clear_primary(),
}
// !! seam注意: process_next は Result<Option<PublishedDocument>> を返し、
//    RN側helper(rn_product_host.rs:727-739)が Some(published) を consume して
//    primary/generation だけ反映する。**helper呼出し後に PublishedDocument は使えない**
//    (document_edit_runtime.rs:414-419)

// ── 2. gizmo drag（transient・Documentへ書かない） ────────────────
// 型は実在。RN Host に保持先が無いだけ【配置】 product_runtime.rs:228-235
let baseline: PositionGestureBaseline = ???_place_in_rn_host(host.primary?, host.current_time, &document);

// 逆写像。affine.rs:50-53 — transform_point(self, x: f64, y: f64) -> [f64; 2]
let inv = (geom.camera_view * geom.world).try_invert()?;   // affine.rs:81
let a = inv.transform_point(cur[0],   cur[1]);
let b = inv.transform_point(start[0], start[1]);
let delta = [a[0] - b[0], a[1] - b[1]];                    // 配列同士は減算できない

let preview = ???_place_in_rn_host_transient(baseline, delta);   // 【配置】

// 表示。RN Stage に `canvas` は無い。draw_stage_preview が SurfaceTexture から
// wgpu render pass を作る(rn_product_host.rs:1076-1108)。
// !! seam: draw_stage_preview 呼出し**後**に同じframeへ overlay できない(:981-998)。
//    挿入できるのは **draw_stage_preview 内部の submit/present 前 render pass**(:1086-1108)
???_overlay_inside_preview_pass(/* skia raster → texture → compose */);

// ── 3. release で Position key を一回だけ書く ────────────────────
// !! seam: raw な baseline/delta から直接確定できない(product_runtime.rs:3396-3421, 2397-2427)。
//    実routeは admission を通してから積む
queue.push_set_position_key_value(SetPositionKeyValueRequest {   // document_edit_runtime.rs:108-111
    target: baseline.target, key: baseline.key,
    old: baseline.value, new: [baseline.value[0] + delta[0], baseline.value[1] + delta[1]],
});
// prepare_set_position_key_value は new == old で Ok(None) = 自動 no-op (command.rs:1828-1837)

// ── 4. Timeline へ同じ identity を投影 ──────────────────────────
// timeline_projection.rs:88-92 — project_timeline(document, metrics, viewport)
// visible_range 単体では呼べない。viewport の owner が U3a-2Q-V で WAIT
let tl = project_timeline(&document, metrics, ???_viewport_owner);
// NativeTimelineRenderer は旧route専用ではない。ProductSurface::render へ接続済み
//   (native_timeline_renderer.rs:35-45 / product_runtime.rs:3651-3658, 3762-3767)
// seam: Timeline の同一 product render pass は **挿入できる**

// ── 5. Easing を適用 ───────────────────────────────────────
// product_runtime.rs:3231-3238 — field は layer / left_id / left_t / right_id / right_t / left_interp
let interval: PositionActiveInterval = position_active_interval(...);   // :3480-3484 実在
// !! seam: active interval から直接 interp request を確定できない(:3270-3295, 2609-2632)。
//    実routeは admit_easing_terminal で generation・layout epoch・interval再導出・
//    same-value を拒否してから積む
queue.push_set_position_key_interp(SetPositionKeyInterpRequest {   // document_edit_runtime.rs:103-106
    target: interval.layer, key: interval.left_id, interp: chosen,
});
```

## B. Host ABI を per-kind で書いてみる（分解可能性の反証テスト）

現行は 15-field union struct + 単一 `match intent.kind`。
本日の3粒（pointer / time / selection）は意味的に disjoint なのにここで直列化した。
per-kind で書けるかを試す:

```rust
// 各 capability が自分の payload 型だけを所有する
trait HostIntent {
    type Payload: DeserializeOwned;
    const KIND: &'static str;
    fn apply(&mut self, host: &mut HostCore, p: Self::Payload) -> Outcome;
}

struct StagePointer;  // phase, view_local_x, view_local_y, sequence  ← grain 2 が所有
struct SetTime;       // frame                                        ← grain 3 が所有
struct StageSelect;   // (payload なし。pointer transient を読む)      ← grain 4 が所有

// kernel が持つのは routing と不可分責任だけ（分解の目標形）
struct HostCore {
    runtime: DocumentEditRuntime,   // ← Document mutation 順序（不可分）
    projection_generation: u64,     // ← snapshot coherence（不可分）
    // width/height/scale_factor/focused/phase/frame ... は kernel が持たない
}
// !! 訂正(鎖のgate): 現行 RnProductHost は上記2つに加え
//    current_time / primary / stages / destroyed / GPU owner も保持する
//    (rn_product_host.rs:333-343)。分解対象は15 field union と単一matchであって、
//    これら全部が消えるわけではない
```

**書けた。** kernel に残るのは `runtime` と `projection_generation`、つまり §6.1 の不可分責任だけ。
15 field のうち **13 は capability 側へ落ちる**。

→ 現行の直列は**構造由来であり規律由来ではない**という §6.2 の主張が、呼び出し側の形からも支持される。

## C. `???` 一覧（= 既知実装調査の検索対象）

| # | `???` | 機構class | survey 対応 |
|---|---|---|---|
性質が3つに分かれる（2026-08-08 更新）。

| # | `???` | 種別 | 現状 |
|---|---|---|---|
| 1 | gesture baseline の RN 保持先 | **配置** | 型は `PositionGestureBaseline`(`product_runtime.rs:228`) として実在 |
| 2 | transient preview の RN 保持先 | **配置** | 同上 |
| 6 | active interval への RN 到達 | **配置** | 型は `PositionActiveInterval`(`product_runtime.rs:3231`) として実在 |
| 3 | `draw_stage_overlay` | **依存＋裁定待ち** | rust-skia は `PROBE_ONLY`(依存ゲート通過済み)。gizmo採択は `BUILD JUSTIFICATION` 未確定 |
| 5 | `draw_timeline` | **依存待ち** | #3 と同一 renderer |
| 4 | `visible_range` owner | **本当に未決** | `U3a-2Q-V` が WAIT。playhead 側は `U3a-2Q-P2` で「復元しない」が決定済み |

**背骨で「本当に何も無い」のは #4 の1件だけ**である。#1/#2/#6 は配置、#3/#5 は依存追加と裁定。

## D. 相互検証（規約 §4 の必須手続き）

`???` 6件 と survey `ABSENT` の照合:

| `???` | survey 側 | 判定 |
|---|---|---|
| #1 #2 #6 | `PARTIAL`（旧routeに実在、RN未接続） | **一致**。新規発明ではなく接続作業 |
| #4 | `PARTIAL`（U3a-2Q-V が WAIT） | **一致**。未決の仕様1問 |
| #3 #5 | survey には node として現れない | **不一致 → 要注意** |

**#3/#5 の不一致が最重要の発見。**
overlay renderer（rust-skia）は survey の node 表に現れない。
地図が node として持っていないのに、背骨の呼び出し側を書くと**必ず現れる**。
つまり地図には「描く場所」の node が欠けている。

survey `ABSENT` 側で背骨に現れなかったもの（`R3-OPS-RENAME`、`R3-CLIPBOARD`、
`R3-A11Y-TREE`、`R3-ACTIVITY`、`R3-TELEMETRY`、`M4-P10/11/12`、`M5-R1/R2`）は
**背骨が要求しない** = 今は要らない、と読める。

## E. M4 / M5 接点

背骨（選択 → gizmo → key → Timeline → Easing）を最短で書くと、
**M4 と M5 の呼び出しが一つも現れなかった。**

- M4（cache / resource / disk artifact）: 背骨は現在時刻1frameしか評価せず、
  再生も書き出しもしないため呼ばない
- M5（3D / import / post）: 背骨は Rect の Position だけを扱うため呼ばない

→ M4/M5 は**最初の利用者outcomeのcritical pathに乗っていない**。
これは「M3 が hero として牽引する」という構造の裏づけであり、
同時に **M4/M5 の実装順を背骨より前に置く理由が無い**ことの根拠になる。

---

## 3. 未通過の鎖（起草時のまま。gate未適用）

### seg-sketch-b1

> ⚠️ **2026-08-09 鎖のgate結果: `NEEDS_REVISION`。この区間の実名を信用しないこと。**
> 実名・型・引数・行番号のずれ、挿入できないseam、過剰な `???` が検出されている。
> 指摘の全量は[鎖のgate 6区間の結果](2026-08-09-chain-gate-results-and-audio-path.md)を見ること。
> **本文の実名をorderへ写す前に、必ずcurrent codeで再確認する。**


Planファイルへの書き込みツールが使えないため、sketchを直接ここに出力します。

## OUTCOME: mediaを入れる

### 呼び出し側
```rust
// file選択: motolii-uiに未接続。rfdは孤立probeのみ(ADOPTION_PROBE、製品接続なし)
let path: std::path::PathBuf = ???_pick_media_file;
// probe: 実在、そのまま呼べる
let container = motolii_media::probe_container(&path)?; // crates/motolii-media/src/probe.rs:154
let video = motolii_media::select_video_stream(&container, 0)?; // crates/motolii-media/src/probe.rs:207

// thumbnail生成: 実装が存在しない
let thumb = ???_thumbnail;

// asset登録: AssetTableはDocumentWriter/Command系の外側にある
let asset_id = doc.assets.allocate(name, "video/mp4", content_hash)?;
// crates/motolii-doc/src/asset.rs:170 — 全呼び出し例(72件)はテストfixtureが生Documentへ直接呼ぶもので、
// DocumentWriterに同等メソッドは無い(allocate_layer_id/reserve_layer_idはあるがasset版は無い)
let clip_source = motolii_doc::ClipSource::asset_video_only(asset_id); // crates/motolii-doc/src/schema.rs:580

let gesture = writer.begin_gesture(); // crates/motolii-doc/src/lib.rs:407
writer.apply_command(gesture, motolii_doc::Command::AddTrackItem {
    parent: motolii_doc::ParentLocator::Track(existing_track_id), // 挿入先track選択UIは???
    index: ???_insertion_index,
    item: motolii_doc::TrackItem::Clip(clip),
    layer_names,
})?; // crates/motolii-doc/src/command.rs:388, crates/motolii-doc/src/lib.rs:449

// 「1 Undo」要件: apply_commandはCommand 1個のみを履歴化する。asset.allocateは
// undo履歴の対象外のため、undoしてもAssetTableの行は残る可能性がある
writer.undo()?; // crates/motolii-doc/src/lib.rs:483(実在) — assets側は巻き戻らない(??? 参照)
```

### ??? 一覧
| # | ??? | なぜ埋まらないか | 探した場所（検索語・path） |
|---|---|---|---|
| 1 | pick_media_file | motolii-uiにfile dialog接続が無い。rfdはP06-C1固定Mac隔離probeのみで製品未接続 | docs/reviews/2026-08-03-p06-c1-mac-rfd-adoption-probe-observation.md; Grep "rfd"/"FileDialog" in crates/motolii-ui |
| 2 | thumbnail | thumbnail生成実装が存在しない | Grep "thumbnail" 全体 → docs/のみヒット、crates/には無し |
| 3 | insertion_index / target track選択 | media挿入のintentが未定義(push_place_rectangle等はあるがplace_media相当が無い) | crates/motolii-ui/src/document_edit_runtime.rs:83-128(push_*一覧) |
| 4 | asset.allocateのundo境界 | AssetTableはCommand enumのどのvariantにも現れず、undo/redoの対象外 | Grep "\.assets\." crates/motolii-doc/src/command.rs → 該当なし |

### 既存で埋まったもの
| 呼び出し | 実体 file:line |
|---|---|
| probe_container | crates/motolii-media/src/probe.rs:154 |
| select_video_stream | crates/motolii-media/src/probe.rs:207 |
| doc.assets.allocate | crates/motolii-doc/src/asset.rs:170 |
| ClipSource::asset_video_only | crates/motolii-doc/src/schema.rs:580 |
| Command::AddTrackItem | crates/motolii-doc/src/command.rs:388 |
| DocumentWriter::begin_gesture / apply_command | crates/motolii-doc/src/lib.rs:407, 449 |
| DocumentWriter::undo | crates/motolii-doc/src/lib.rs:483 |

---

## OUTCOME: 保存して開き直す

### 呼び出し側
```rust
// New: Document生成自体は実在するが、product向け「新規project作成」経路が無い
let doc = motolii_doc::Document::new_current(); // crates/motolii-doc/src/lib.rs:156
let session = ???_create_project_at(path); // ProjectSession::acquireはlock取得のみ(初期save経路は未接続)

// Open: 実在、通常経路も確認済み
let limits = motolii_doc::ResourceLimits::production();
let (mut session, opened) = motolii_doc::ProjectSession::open(&path, &limits)?;
// crates/motolii-doc/src/journal/session.rs:110, 呼び出し元: crates/motolii-ui/src/shell.rs:60
let writer = motolii_doc::DocumentWriter::new(opened.document, catalog)?; // crates/motolii-doc/src/lib.rs:360
let runtime = crate::document_edit_runtime::DocumentEditRuntime::new(session, writer, catalog);
// crates/motolii-ui/src/document_edit_runtime.rs:353

// opened.open_mode(ReadWrite/ReadOnlyNewer/Reject)はここで捨てられる
let _ = ???_open_mode_gate; // shell.rsはOpenModeを受け取らず、ReadOnlyNewerでもwritable runtimeへ入れてしまう

// Save: compaction/checkpointのみ、初回durabilityの代替ではない
session.save_with_journal(&doc, &options)?; // crates/motolii-doc/src/journal/session.rs:129

// Save As: destination transactionが未実装
???_save_as(new_path);

// 再open後「同じrevisionへ戻る」:
let (_, reopened) = motolii_doc::ProjectSession::open(&path, &limits)?; // contentはjournal replayで復元
let writer2 = motolii_doc::DocumentWriter::new(reopened.document, catalog)?;
assert_eq!(writer2.revision, ???_expected_revision);
```

### ??? 一覧
| # | ??? | なぜ埋まらないか | 探した場所（検索語・path） |
|---|---|---|---|
| 1 | create_project_at (New) | product向け新規project作成経路が無い。ProjectSession::acquire(crates/motolii-doc/src/journal/session.rs:88)はlock取得のみ | Grep "new_project"/"initialize_project" → test helperのみ(crates/motolii-ui/tests/cu109_session_backed_edit_entry.rs:72) |
| 2 | open_mode_gate | opened.open_modeがshell.rsで破棄され、ReadOnlyNewerを弾く接続が無い | docs/reviews/2026-08-03-p12-c1-document-lifecycle-adoption-decision.md:26; crates/motolii-ui/src/shell.rs:58-66 |
| 3 | save_as | Save Asのdestination transactionが`SPEC_ONLY`のまま未実装 | docs/reviews/2026-08-03-p12-c1-document-lifecycle-adoption-decision.md「connection residual」節 |
| 4 | expected_revision（再open後のUndo履歴の姿） | `UndoHistory::from_restored`が定義のみで呼び出し元ゼロ。`DocumentWriter::new`は常にrevision=0で構築する | Grep "from_restored" 全体 → crates/motolii-doc/src/undo.rs:214の定義のみ; crates/motolii-doc/src/lib.rs:379 |

### 既存で埋まったもの
| 呼び出し | 実体 file:line |
|---|---|
| Document::new_current | crates/motolii-doc/src/lib.rs:156 |
| ProjectSession::open | crates/motolii-doc/src/journal/session.rs:110（呼び出し元: crates/motolii-ui/src/shell.rs:60） |
| DocumentWriter::new | crates/motolii-doc/src/lib.rs:360 |
| DocumentEditRuntime::new | crates/motolii-ui/src/document_edit_runtime.rs:353 |
| ProjectSession::save_with_journal | crates/motolii-doc/src/journal/session.rs:129 |
| OpenedDocument{document, open_mode} | crates/motolii-doc/src/persist.rs:78-80 |

---

## OUTCOME: 再生する

### 呼び出し側
```rust
// playhead進行 / audio主clock: ほぼ全て実在
let start_frame = canonical_playback_start_frame(editor_playhead.current)?;
// crates/motolii-ui/src/product_runtime.rs:3326
let session = motolii_transport::PlaybackSession::open_default(
    program, start_frame, fps, motolii_core::Quality::DRAFT, Some(&gpu),
)?; // crates/motolii-transport/src/playback.rs:26, 呼び出し元: crates/motolii-ui/src/product_runtime.rs:1134

let plan = session.transport_mut().next_frame_plan()?;
// crates/motolii-transport/src/lib.rs:152, 呼び出し元: crates/motolii-ui/src/product_runtime.rs:1186
editor_playhead.set(plan.timeline_time); // crates/motolii-ui/src/product_runtime.rs:1213,281

// pause: 現在位置をplayheadへ書き戻してsessionを閉じる
let time = session.transport().perceptual_time()?; // crates/motolii-transport/src/lib.rs:146
editor_playhead.set(time); // crates/motolii-ui/src/product_runtime.rs:1163
// playback_session.take()相当、crates/motolii-ui/src/product_runtime.rs:1161

// scrub: EditorPlayheadのbegin/update/finish/cancelで完結。再生中はまず一時停止させる
if playback_lifecycle.state() == StagePlaybackState::Playing {
    stop_playback_for_scrub(event_loop)?; // crates/motolii-ui/src/product_runtime.rs:1254
}
editor_playhead.begin(layout_epoch, pointer_time); // crates/motolii-ui/src/product_runtime.rs:247
editor_playhead.update(layout_epoch, pointer_time); // crates/motolii-ui/src/product_runtime.rs:255
editor_playhead.finish(layout_epoch); // crates/motolii-ui/src/product_runtime.rs:259

// 再生"中"にplayhead位置を音を切らず移動させる操作(seek-while-playing)は存在しない
???_seek_while_playing;
```

### ??? 一覧
| # | ??? | なぜ埋まらないか | 探した場所（検索語・path） |
|---|---|---|---|
| 1 | seek_while_playing | `Transport`にseek/set_origin相当のメソッドが無い。`timeline_origin`はコンストラクタでのみ設定される(`Transport::new`)。位置変更は必ずセッションを閉じて新規`open_default`する設計に見えるが、それを1操作として書いた既存呼び出しは無い | Grep "fn seek\|timeline_origin =" crates/motolii-transport/src全体 → コンストラクタ代入のみ |

### 既存で埋まったもの
| 呼び出し | 実体 file:line |
|---|---|
| canonical_playback_start_frame | crates/motolii-ui/src/product_runtime.rs:3326 |
| PlaybackSession::open_default | crates/motolii-transport/src/playback.rs:26 |
| Transport::next_frame_plan | crates/motolii-transport/src/lib.rs:152 |
| Transport::perceptual_time | crates/motolii-transport/src/lib.rs:146 |
| EditorPlayhead{begin,update,finish,cancel,set} | crates/motolii-ui/src/product_runtime.rs:247,255,259,270,281 |
| stop_playback_for_scrub | crates/motolii-ui/src/product_runtime.rs:1254 |
| pause_playback | crates/motolii-ui/src/product_runtime.rs:1150 |

---

SKETCHED: 3 / RESOLVED: 20 / UNKNOWN: 9

(このセッションでは plan ファイルへの書き込みツールが利用できなかったため、出力はこの応答テキストのみです。)
### seg-sketch-b2

> ⚠️ **2026-08-09 鎖のgate結果: `NEEDS_REVISION`。この区間の実名を信用しないこと。**
> 実名・型・引数・行番号のずれ、挿入できないseam、過剰な `???` が検出されている。
> 指摘の全量は[鎖のgate 6区間の結果](2026-08-09-chain-gate-results-and-audio-path.md)を見ること。
> **本文の実名をorderへ写す前に、必ずcurrent codeで再確認する。**


Write/ExitPlanModeツールがこの環境では利用できないため、調査結果を直接ここに出力します(read-onlyでGlob/Grep/Readのみ使用)。

## OUTCOME: 書き出す

### 呼び出し側
```rust
// crates/motolii-cli/src/project.rs:379-414 実在パス
let prepared = motolii_cli::prepare_project_export(&project_path)?; // crates/motolii-cli/src/project.rs:379
let gpu = GpuCtx::new_headless()?; // crates/motolii-cli/src/main.rs:45

// crates/motolii-cli/src/project.rs:430 PreparedProject::export
let report: ExportReport = prepared.export(&gpu)?;
// 内部で crates/motolii-export/src/lib.rs:221 export_document_video
// (D3経路。document_export.rs:10-34 export_document_file も同型の同期呼び出し)

let ???_progress_stream = (); // 実行中のprogress通知が無い理由: 探した
let ???_cancel_handle = ();   // 実行中断のcancelが無い理由: 探した

// 完了後、実在するのはstdout printのみ
println!(
    "wrote {} frames: {}x{} @ {}/{} fps -> {}",
    report.frames_written, // crates/motolii-export/src/lib.rs:52 ExportReport.frames_written
    report.desc.width, report.desc.height,
    report.fps.num(), report.fps.den(),
    prepared.output_path.display(),
); // crates/motolii-cli/src/main.rs:48-56

let ???_artifact_publish_event = (); // 完成artifactを外部へ通知/公開する仕組みが無い理由: 探した
```

### ??? 一覧
| # | ??? | なぜ埋まらないか | 探した場所（検索語・path） |
|---|---|---|---|
| 1 | `???_progress_stream` | `ExportJob`/`export_document_video`にprogress callback・channel引数が無い。ループは`crates/motolii-export/src/lib.rs:254-324`で`frames_written`をローカル変数として積むだけで、外部へ出す経路が無い | `progress\|Progress` in crates/motolii-cli、crates/motolii-export/src/lib.rs全体read |
| 2 | `???_cancel_handle` | `ExportJob`/`ExportOverlayRequest`にcancel token・flagのfieldが無い。ループ条件は`job.frame_count`/`composition.duration`のみ(lib.rs:254,263) | `cancel\|Cancel\|AbortHandle\|CancelToken` in crates/motolii-cli — 0件。`DomainIntent::CancelInFlightGesture`は入力gesture用で書き出しjobとは無関係(motolii-ui/src/domain_intent.rs:12) |
| 3 | `???_artifact_publish_event` | `ExportReport`は呼び出し元へ戻るだけの値(lib.rs:50-55)。CLI側は`println!`のみ(main.rs:48-56)で、他コンポーネントへ「artifactができた」を伝える型・イベントが無い | `ExportReport`使用箇所全体、`Publish` grep — `PublishedDocument`はDocument編集専用(motolii-ui/src/document_edit_runtime.rs:1042-1050)で書き出しartifactとは無関係 |

### 既存で埋まったもの
| 呼び出し | 実体 file:line |
|---|---|
| `prepare_project_export` | crates/motolii-cli/src/project.rs:379 |
| `PreparedProject::export` | crates/motolii-cli/src/project.rs:430 |
| `export_document_video` | crates/motolii-export/src/lib.rs:221 |
| `ExportJob` | crates/motolii-export/src/lib.rs:108 |
| `ExportReport { frames_written, desc, fps }` | crates/motolii-export/src/lib.rs:51-55 |
| stdout report print | crates/motolii-cli/src/main.rs:48-56 |

---

## OUTCOME: 日常操作（delete / duplicate / rename / copy-paste、keymap+menuから同一CommandIdへ）

### 呼び出し側
```rust
// delete: keymap → CommandId → registry → intent → prepared request まで通っている唯一の経路
let id = CommandId::try_new("motolii.edit.delete_targeted_items")?; // crates/motolii-ui/src/command_registry.rs:132-135
let mut router = InputRouter::new(builtin_command_registry()?); // input_router.rs:69, command_registry.rs:129
let output = router.route(NormalizedInput::Command { phase: InputPhase::Click, id })?; // input_router.rs:89,112-139
let request = DocumentCommandRequest::try_new(
    DomainIntent::DeleteTargetedItems, // crates/motolii-ui/src/domain_intent.rs:8
    vec![Command::RemoveTrackItem { parent, index, item, layer_names }], // motolii-doc/src/command.rs:395-401
)?; // crates/motolii-ui/src/document_command_request.rs:15-30 (intent==DeleteTargetedItemsのみ許可)
queue.push_prepared(output, Some(request))?; // crates/motolii-ui/src/app.rs:141-150

// duplicate: 下層に構築関数はあるが、上の経路に一切繋がっていない
let command: Command = motolii_doc::duplicate_track_item(&mut doc, source_layer)?; // crates/motolii-doc/src/duplicate.rs:44-47
let ???_duplicate_command_id = ();   // 探した
let ???_duplicate_intent = ();       // 探した
let ???_duplicate_keymap_binding = (); // 探した

// rename: Command enum・DomainIntentどちらにも変異操作が存在しない
let ???_rename_command_variant = (); // 探した
let ???_rename_intent = ();          // 探した

// copy-paste: clipboard/paste向けの型・関数が存在しない
let ???_copy_to_clipboard = ();  // 探した
let ???_paste_from_clipboard = (); // 探した

// menu: CommandIdを引く実装済みmenuコンポーネントは存在しない
let ???_menu_dispatches_command_id = (); // 探した
```

### ??? 一覧
| # | ??? | なぜ埋まらないか | 探した場所（検索語・path） |
|---|---|---|---|
| 1 | `???_duplicate_command_id` / `???_duplicate_intent` / `???_duplicate_keymap_binding` | `DomainIntent::ALL`は7件で`Delete/EnableReduceMotion/ResetWorkspaceProfile/FitStageView/CancelInFlightGesture/Undo/Redo`のみ(domain_intent.rs:18-26)。`builtin_command_registry()`も同7件のみ(command_registry.rs:129-160)。Duplicate系のCommandId・intentは存在しない | `enum DomainIntent`、`builtin_command_registry`全体read |
| 2 | `???_rename_command_variant` / `???_rename_intent` | `Command`列挙(command.rs:265行以降)を全走査したがdisplay_name/層名を書き換えるvariantは無い。`ids.rs:146-220`の`display_name`/`allocate`は作成・複製時にのみ書き込み、以後変更するCommandが無い | `Rename\|rename` grep on command.rs — マッチ0(`#[serde(rename = "use")]`のみ、無関係) |
| 3 | `???_copy_to_clipboard` / `???_paste_from_clipboard` | `clipboard\|Clipboard\|paste\|Paste`でヒットしたのは`CopyLocalEffect`(effect definitionのlocal複製、diagnostic_projection.rs:263)のみで、track item/layer単位のクリップボードではない | `clipboard\|Clipboard\|copy_local\|CopyLocal\|paste\|Paste` type:rust 全体 |
| 4 | `???_menu_dispatches_command_id` | 実在する唯一のmenu実装(`egui::MenuBar`, app.rs:355-369)は`view_role_button`(app.rs:1029-1045)経由で`LayoutAction::Hide/Restore`を直接生成し、CommandId/CommandRegistry/InputRouterを一切経由しない。同ファイルに`ShellLifecycleInput`/`LifecycleSmokeOutcome`があり製品menuでない可能性があるが、それを判定する別実装は見つからなかった | `\bmenu\b\|Menu` type:rust — app.rs以外に実装なし |

### 既存で埋まったもの
| 呼び出し | 実体 file:line |
|---|---|
| `CommandId::try_new` | crates/motolii-ui/src/command_registry.rs:12-21 |
| `builtin_command_registry`(delete/undo/redo等7件登録) | crates/motolii-ui/src/command_registry.rs:129-160 |
| `InputRouter::route` | crates/motolii-ui/src/input_router.rs:89-99,112-139 |
| `Command::RemoveTrackItem` | crates/motolii-doc/src/command.rs:395-401 |
| `DocumentCommandRequest::try_new`(deleteのみ許可) | crates/motolii-ui/src/document_command_request.rs:15-30 |
| `DocumentEditQueue::push_prepared`(deleteのみ受理) | crates/motolii-ui/src/app.rs:132-163 |
| `duplicate_track_item`(生成のみ、配線先なし) | crates/motolii-doc/src/duplicate.rs:44-82 |
| `LayerTable::display_name`/`allocate` | crates/motolii-doc/src/ids.rs:146,161 |

---

## OUTCOME: panel配置とdiagnostics

### 呼び出し側
```rust
// panel open/close/resize: toolkit非依存の正本はLayoutAuthority(全pub(crate)、crate内限定)
let mut authority = LayoutAuthority::built_in()?; // crates/motolii-ui/src/layout_authority.rs:20-28
authority.apply(
    LayoutAction::Hide(PanelRole::Inspector), // layout.rs:77, 9-14
    LayoutConstraints { viewport_width: vw, stage_min_width: sw }, // layout.rs:62-65
)?; // layout_authority.rs:55-66

// resize(ドラッグ中の連続編集→commit)
authority.reconcile_runtime_frame(
    cancelled, RuntimeFrameEdit::Continuous, gesture_finished, constraints,
)?; // layout_authority.rs:68-105、RuntimeFrameEdit定義: layout_authority.rs:7-11

// dock復元(セッションを跨いだ保存済みlayoutの復元)
let ???_load_saved_layout = ();   // 探した
let ???_save_layout_on_change = (); // 探した

// diagnostics: disabled/invalid理由の表示は既存のDiagnosticEnvelope経路に乗る想定だが、
// panel/layout由来のエラーに対するadapterが存在しない
let layout_err: LayoutError = authority.apply(action, constraints).unwrap_err(); // layout.rs:87-122
let ???_adapt_layout_error_to_diagnostic = (); // 探した
```

### ??? 一覧
| # | ??? | なぜ埋まらないか | 探した場所（検索語・path） |
|---|---|---|---|
| 1 | `???_load_saved_layout` / `???_save_layout_on_change` | `LayoutAuthority::built_in()`(layout_authority.rs:20-28)が唯一の構築子で`PanelLayout::built_in()`固定値から始まる(layout.rs:126,133-156)。`layout_runtime.rs`にserde/save/load/persistの語は0件。`LayoutAction::ResetPreset`(layout.rs:79)はbuilt-inへ戻すだけで「保存済みprofileの読込」ではない | `serde\|Serialize\|save\|load\|persist` on layout_runtime.rs — 0件。layout.rs/layout_authority.rs全体read |
| 2 | `???_adapt_layout_error_to_diagnostic` | `crates/motolii-ui/src/diagnostic.rs`の`adapt_*`群は`InputRouterError`(108-119)、`DocumentCommandRequestError`(121-168)、`CommandError`(170-197)の3種のみ対応。`LayoutError`(layout.rs:87-122、`MissingRole`/`NonAuxiliarySubject`/`InvalidConstraints`等)を`DiagnosticEnvelope`へ渡す関数は無い | diagnostic.rs全体read(`adapt_input_router_error`/`adapt_document_command_request_error`/`adapt_command_error`のみ実在) |

### 既存で埋まったもの
| 呼び出し | 実体 file:line |
|---|---|
| `LayoutAuthority::built_in` | crates/motolii-ui/src/layout_authority.rs:20-28 |
| `LayoutAuthority::apply` | crates/motolii-ui/src/layout_authority.rs:55-66 |
| `LayoutAuthority::reconcile_runtime_frame` | crates/motolii-ui/src/layout_authority.rs:68-105 |
| `LayoutAction::Hide/Restore/ResetPreset/Separator` | crates/motolii-ui/src/layout.rs:76-85 |
| `LayoutConstraints` | crates/motolii-ui/src/layout.rs:62-65 |
| `LayoutError`(disabled理由の生データとして実在) | crates/motolii-ui/src/layout.rs:87-122 |
| `DiagnosticEnvelope`と`adapt_*`(LayoutError版は無いが型自体は実在) | crates/motolii-ui/src/diagnostic.rs:54-197 |
| menu上のHide/Restoreボタン(CommandId非経由の直結配線として実在) | crates/motolii-ui/src/app.rs:1029-1045, 355-369 |

---

SKETCHED: 3 / RESOLVED: 19 / UNKNOWN: 9

---

補足: このセッションのツールセットにWrite/ExitPlanModeが含まれていなかったため、plan fileへの書き込みと承認フローは実施していません。Glob/Grep/Readのみで完結する調査だったため、この場での直接出力としました。
### seg-seg-tune

> ⚠️ **2026-08-09 鎖のgate結果: `NEEDS_REVISION`。この区間の実名を信用しないこと。**
> 実名・型・引数・行番号のずれ、挿入できないseam、過剰な `???` が検出されている。
> 指摘の全量は[鎖のgate 6区間の結果](2026-08-09-chain-gate-results-and-audio-path.md)を見ること。
> **本文の実名をorderへ写す前に、必ずcurrent codeで再確認する。**


## 区間: Tune / Compose

### 呼び出し側
```rust
// 実名: InspectorCandidate.onPositionScrubStart/Move/End + cancelPositionScrub
// ui/motolii-web/src/candidates/InspectorCandidate.jsx:495-540
// 決定由来の語: 決定: U4b0V Position key value edit (transient preview -> terminal commit/cancel)
emitPositionGesture("start"|"update"|"commit"|"cancel", axis, value)
  // -> inspectorHostCodec.js send -> InspectorPositionKeyValueGesture inbox
  // -> product_runtime.rs:3416 Command::SetPositionKeyValue{target,key,old,new}
  // -> motolii-doc/src/lib.rs:619 DocumentWriter::prepare_set_position_key_value
  //    (command.rs:1828 prepare_set_position_key_value / :2541 apply_set_position_key_value)

// 実名: onAddPositionKey button
// ui/motolii-web/src/candidates/InspectorCandidate.jsx:669-678
// 決定由来の語: 決定: CU-0A08ITIB one-shot intent ({kind:"add-position-key",sequence})
onClick={onAddPositionKey}
  // -> motolii-doc/src/position_key_prepare.rs:40 prepare_add_position_key(target, editor_playhead.current)

// 実名: onEffectParamGesture (Opacity) via ScrubControl in "active effect" section
// ui/motolii-web/src/candidates/InspectorCandidate.jsx:396-451, :605-624
// 決定由来の語: 決定: Vism Inspector source (parameter, typed input/output をHost契約から投影)
emitProductGesture("start"|"update"|"commit"|"cancel", param, value)
  // -> inspectorHostCodec.js "effect-param-gesture" -> DocumentEditRuntime SetEffectParamRequest

// ???: 上記3経路は全てui/motolii-web(WebView island, CU-110PIH)所属。RN runtime rebaseline
//      (決定, 2026-08-07)は旧WebView islandへの新規実装を凍結。RN側 InspectorInitialReadPanel
//      (ui/motolii-rn-legacy/src/inspector/InspectorInitialReadPanel.tsx:1-46)は読取専用decodeのみで
//      gesture/inbox/queueに対応する型が索引にも正本にも無い
???_rn_inspector_parameter_edit_route(target, axis, phase, value)
```

```rust
// 実名: Command::AddEffect{target, index, effect, introduced_definition}
// crates/motolii-doc/src/command.rs:287-294
// 決定由来の語: 決定: plugin authoring / Vism A3 general lowering (line145)
AddEffect { target, index, effect, introduced_definition }

// 実名: Command::SetEffectEnabled{target, effect: EffectId, old, new}
// crates/motolii-doc/src/command.rs:302-307 (apply: :796-820, def.enabled = *new, Shared Effect決定line147)
SetEffectEnabled { target, effect, old, new }
// ???: SetEffectEnabled は EffectDefinition.enabled (schema.rs:301, 共有)を書き換える実装済み
//      Commandだが、ui/(web・rn) 全体にこのCommandへの呼び出しが0件(grep該当なし)。
//      索引Shared Effect(line147)はdelete/unlink/copy-localのみ扱い、enable/disableのUI入口を
//      定義していない
???_effect_enabled_toggle_ui(target, effect_use_id)

// ???: 効果の並べ替え(reorder)は AddEffect の index を使った remove+insert 以外に専用Commandが
//      無く、索引にも正本にも記述が無い。InspectorCandidate.jsx:913-920 の
//      <span className="grip">::</span> は mode==="installed" mock分岐のみの装飾で
//      handlerが無い(実装粒未着手)
???_reorder_effect(target, from_index, to_index)
```

```rust
// ???: 複数Vismを接続して一つの表現にする経路(Kit/Rack相当)は現行codeに型が存在しない。
//      grep結果: `Rack`/`VismKit`/struct Kit は docs以外に0件。
//      決定(line42, Vism入口・並列解禁の根本マップ)は
//      「A9前に二つ以上のVism実装を同時起動しない」と明記し、Kit接続(line54/57)の
//      protocol/schema/UIは明示的に未決のまま維持されている
???_connect_vism_into_kit(vism_a: VismInstance, vism_b: VismInstance) -> ???_KitComposite
```

### ??? 一覧
| # | ??? | 索引/正本を探した範囲 | 真の未決か検索失敗か |
|---|---|---|---|
| 1 | `???_rn_inspector_parameter_edit_route` | decision-index.md line60/110(RN rebaseline)、ui/motolii-rn-legacy/src/inspector/*、docs/reviews/2026-08-04-inspector-position-key-one-shot-intent-contract.md | 真の未決。WebView側は実装済みだがRN移行先の接続決定が存在しない |
| 2 | `???_effect_enabled_toggle_ui` | decision-index.md line147(Shared Effect)、ui/motolii-web・ui/motolii-rn-legacy全体をgrep | 真の未決。Command実装済みだがUI入口の決定が無い |
| 3 | `???_reorder_effect` | decision-index.md全体(reorder/並べ替え関連行のみ検索)、crates/motolii-doc/src/command.rs CommandKind一覧 | 真の未決。Command/決定とも存在しない |
| 4 | `???_connect_vism_into_kit` | decision-index.md line42/54/57、crates全体でRack/Kit/TypedConnection grep | 真の未決(索引が明示的にA9まで未着手と宣言) |

### 合成失敗（最重要）
| 種別 | 決定A | 決定B | どう噛み合わないか | 根拠 |
|---|---|---|---|---|
| 矛盾(索引 vs 正本) | decision-index.md line313 CU-110PIR(2026-07-29)「mock state、S値、editing callback…Document writerなし」 | docs/reviews/2026-08-04-inspector-position-key-one-shot-intent-contract.md / u4b0v-position-key-value-edit-contract.md (未索引) | 索引はInspectorを読取専用と要約するが、3日後の正本と現行コード(InspectorCandidate.jsx:485-577)はwrite route(SetPositionKeyValue/AddPositionKey)を実装済みとして閉じている。索引更新義務(decision-index.md:11)が本チェーン5文書全てで未履行 | InspectorCandidate.jsx:495-576, command.rs:422 SetPositionKeyValue |
| 順序不能 | decision-index.md line60/110 M3 RN runtime rebaseline(2026-08-07)「旧WebView islands…新規実装を凍結」 | CU-0A08ITIA/ITIB, U4b0V(2026-08-04)によるWebView Inspector上のparameter編集チェーン全体 | RN側Inspector(decodeInspectorInitialRead.ts, InspectorInitialReadPanel.tsx)はread-only decodeのみでgesture/inbox相当が存在しない。RNが新route合格までWebViewは凍結されるが、parameter編集機能はWebView側にしか実装がなく、RNへ移す接続決定が無いため「RN routeでparameterを編集する」が現状どの決定順でも成立しない | ui/motolii-rn-legacy/src/inspector/decodeInspectorInitialRead.ts:1-18(gesture型皆無) |
| 断絶 | crates/motolii-doc/src/command.rs:302 SetEffectEnabled(実装済み、Shared Effect決定line147の対象) | (無し) | Document層のCommandは完成しているが、ui/motolii-web・ui/motolii-rn-legacy いずれにも呼び出しが0件。「effectをenable/disableする」呼び出し側の実体が存在しない | grep "SetEffectEnabled\|EffectEnabled" ui/ → No files found |

### 実名で埋まったもの
| 呼び出し | 実体 file:line または 決定 |
|---|---|
| Position key preview→commit→cancel | InspectorCandidate.jsx:495-540 / command.rs:1828 prepare_set_position_key_value |
| Add Position Key one-shot | InspectorCandidate.jsx:669-678 / position_key_prepare.rs:40 prepare_add_position_key |
| Opacity effect param gesture | InspectorCandidate.jsx:396-451 / inspectorHostCodec.js "effect-param-gesture" |
| Effect追加(index付き挿入) | command.rs:287 Command::AddEffect |
| Effect enable/disable(Shared) | command.rs:302 SetEffectEnabled / schema.rs:301 EffectDefinition.enabled / 決定: Shared Effect(line147) |
| RN Inspector読取専用decode | decodeInspectorInitialRead.ts:387 / InspectorInitialReadPanel.tsx:10 |

SEGMENT: Tune/Compose / RESOLVED: 6 / UNKNOWN: 4 / COMPOSITION_FAILURES: 3
### seg-seg-fork

> ⚠️ **2026-08-09 鎖のgate結果: `NEEDS_REVISION`。この区間の実名を信用しないこと。**
> 実名・型・引数・行番号のずれ、挿入できないseam、過剰な `???` が検出されている。
> 指摘の全量は[鎖のgate 6区間の結果](2026-08-09-chain-gate-results-and-audio-path.md)を見ること。
> **本文の実名をorderへ写す前に、必ずcurrent codeで再確認する。**


まず `docs/decision-index.md` を読み、Inspect/Fork区間に関係する主題行を特定します。Row 144はまだ未読なので確認する。row45/49(作者連続性・Inspector)、144(VSM-A4S)は確認済み。313行目のCU-110PIR(Inspector実装現況: read-only, editing callbackなし)が区間の核心的手がかりなので、実コードを特定する。核心となる決定文書3本(作者連続性契約、Inspector/Automation境界、VSM-A4S)を正本で確認する。row49の正本(Inspector・作者source・Automation境界決定)を確認し、space/temporal mode/Bakeの記述を正確に引用する。十分な材料が揃った。プランファイルへ最終出力を書く。Writeツールがこの環境で無効化されているため、プランファイルへの保存はできません。代わりに、区間分析の結果をそのままここに提示します(read-onlyでの調査結果であり、ファイル変更は一切行っていません)。

## 区間: Inspect / Fork

### 呼び出し側

```rust
// 1. Inspect: 選択中Vismの意味をInspectorから読む
// 実名: gesture_identity() -> InspectorGestureIdentity{layer_id,effect_use_id,definition_id,
//        plugin_id,effect_version,param_id} // crates/motolii-ui/src/inspector_host_runtime.rs:798-841
// 実名: position_projection() -> Const|Key|Animated // :964
// 実名: InspectorSnapshot{target,position,nodes,active_effect_use_id} // :1063-1073
// 決定由来の語: 作用先/typed input/typed output/space/temporal mode/diagnostics
// 決定: Vism Inspector source 外部IDE Automation (reviews/2026-08-01-vism-inspector-source-
//        automation-boundary-decision.md:52-58)
fn inspect_selected_vism(document: &Document, primary: LayerId, active_effect: Option<EffectId>) {
    let identity = gesture_identity(document, Some(primary), active_effect); // 作用先+typed input
    let space = ???_space; // 決定にはあるがInspectorSnapshotにfieldが無い
    let temporal_mode = ???_temporal_mode; // 同上
    let diagnostics = ???_diagnostics; // 同上
}
```

```rust
// 2. Fork: Inspect → Fork → 候補 → preflight → atomic adoption
// 決定: 作者連続性 変更カプセル (reviews/2026-07-31-authoring-continuity-capsule-goal-contract.md ACG-O2)
// 決定: VSM-A4S外部crate作者scaffold (reviews/2026-08-01-vsm-a4s-external-crate-author-scaffold-spec.md §3-4)
fn fork_selected_vism(identity: InspectorGestureIdentity) {
    // 断絶: identityをFork入口へ渡す経路が無い。A4S入口はCLI固定引数
    // --from core.layer_source.radial_repeater (A4S§3)であり、Document selectionを受けない
    let candidate_source = ???_scaffold_from_selection(identity);
    // 実名(未実装、A4S§3で命名のみ): scripts/new_plugin_crate.py --from <kind_id> --vendor <v> --out-dir <dir>
    let preflight_result = ???_host_conformance_check(candidate_source); // A4S§4 step4: 一時harness
    let adopted = ???_atomic_adopt(preflight_result); // 下記「合成失敗」参照
}
```

```rust
// 3. fork後も作品・identity・型付き入力・Preview・診断・versionを失わない
// 実名: EffectUse{id,definition_id} // crates/motolii-doc/src/schema.rs:261
// 実名: EffectDefinition{id,plugin_id,effect_version,params,extra} // crates/motolii-doc/src/schema.rs:295
// 決定: 作者連続性ゴール契約 §6-2 (reviews/2026-07-31-...md:126)
fn after_fork_adoption(pre_fork_effect_use: EffectUse) {
    // 断絶: A4S§4 step5「composition rootへ明示登録」+ step6「rebuild/restart」はprocess終了を伴う
    let surviving_work = ???_work_after_restart; // rebuild/restart後にDocument/Undo/Previewが
    let preview_continuity = ???_preview_continuity; // どう継続するか、索引にも正本にも記述が無い
}
```

### ??? 一覧

| # | ??? | 索引/正本を探した範囲 | 真の未決か検索失敗か |
|---|---|---|---|
| 1 | `???_space` | decision-index行49、reviews/2026-08-01-vism-inspector-source-automation-boundary-decision.md:55、`InspectorSnapshot`全field(inspector_host_runtime.rs:1063-1073) | 真の未決(決定は要求するがfieldとして未実装。正本にも実装計画の記述なし) |
| 2 | `???_temporal_mode` | 同上:56 | 真の未決(同上) |
| 3 | `???_diagnostics` | 同上:58 | 真の未決(同上) |
| 4 | `???_scaffold_from_selection` | decision-index行45/144、reviews/2026-08-01-vsm-a4s-external-crate-author-scaffold-spec.md §1「local Vismを成立させない」、crates全体で`fork`/`Fork`をgrep(0件) | 真の未決。A4S自身が一般選択→fork化を明示的に範囲外にしている |
| 5 | `???_host_conformance_check` | A4S§4 step4、A4S§7 disposition「PASS/SPEC、実装はVSM-A4Iへ分離」 | 検索失敗ではなく計画済み未実装(`scripts/new_plugin_crate.py`は名前のみ存在、コード無し) |
| 6 | `???_atomic_adopt` | ACG-O2 (reviews/2026-07-31-...md:47)、A4S§4 step5、crates全体で`atomic`/`adopt`をgrep→`journal/session.rs:149`の`migrate_legacy_sidecar`のみ(無関係) | 真の未決。Fork用のatomic adoption関数は存在しない |
| 7 | `???_work_after_restart` | ACG契約§6項2「現在の対象、入力、Preview、versionを保ったFork」、A4S§4 step6 | 真の未決。rebuild/restartとDocument/session継続の関係が正本に記述無し |
| 8 | `???_preview_continuity` | 同上 | 真の未決(同上) |

### 合成失敗（最重要）

| 種別 | 決定A | 決定B | どう噛み合わないか | 根拠 |
|---|---|---|---|---|
| 矛盾 | ACG-O2 atomic adoption「開始revisionを照合し、Hostの全体preflight後に一回だけ採用される。Document変更なら1 macro」 | VSM-A4S§4 step5-6「first-party composition rootへ明示登録」→「rebuild/restart後、標準parameter projectionとrender経路で変更を確認する」 | A4Sの唯一の「採用」手段はプロセス再起動を要すcomposition root登録であり、revision照合も1 Document macroも介さない。両者は単体では正しいが、A4SをFork/atomic adoptionの実装先として使うと1つのDocumentコマンドとして閉じられない | reviews/2026-07-31-authoring-continuity-capsule-goal-contract.md:47、reviews/2026-08-01-vsm-a4s-external-crate-author-scaffold-spec.md:39-50 |
| 順序不能 | Fork/atomic adoptionはInspect時点のDocument開始revision(ACG-O2)とライブselection(`InspectorGestureIdentity`)を前提にする | A4S§4 step5-6のadoptionはcomposition root登録後の**rebuild/restart**を要求する | Fork完了(A)はrestart後のbinary(B)のruntimeを要求するが、そのrestart(B)はAが検査対象としていた実行中session(InspectorHostRuntime、gesture inbox、Document in-memory state)を終了させる。Aの前提条件をBが破棄する | crates/motolii-ui/src/inspector_host_runtime.rs:511-518(session state)、reviews/2026-08-01-vsm-a4s-...md:39-50 |
| 断絶 | 決定49「作用先、typed input／output、space、temporal mode／Bake、diagnosticをHost契約から投影」 | 実装`InspectorSnapshot`(inspector_host_runtime.rs:1063-1073)は`target{layer_id}`、`position`、`nodes[].params`、`active_effect_use_id`のみ | space/temporal mode/diagnosticsの投影経路が無い。決定を読んだ実装者が「target/positionで足りる」と誤認する余地がある | reviews/2026-08-01-vism-inspector-source-automation-boundary-decision.md:52-58、crates/motolii-ui/src/inspector_host_runtime.rs:1063-1073 |
| 断絶 | 決定45(作者連続性)「Use→Tune→Compose→Inspect→Fork」は任意の選択中Vismに一般的に適用される経路 | VSM-A4S§1「first-party参照Vismを作者が別crateへfork」に限定、§1「第三者package、動的load、**local Vism**...を成立させない」 | Inspectorが実際に選択できる対象(任意のEffectUse/LayerId)とFork可能な対象(A4Sが認める`core.layer_source.radial_repeater`等の固定candidate)が一致しない。接続する決定が無い | reviews/2026-07-31-authoring-continuity-capsule-goal-contract.md:194(接続票「Inspect／Fork製品面は未実装」)、reviews/2026-08-01-vsm-a4s-external-crate-author-scaffold-spec.md:7-9 |

### 実名で埋まったもの

| 呼び出し | 実体 file:line または 決定 |
|---|---|
| Inspector snapshot生成 | `InspectorHostRuntime::publish` crates/motolii-ui/src/inspector_host_runtime.rs:724 |
| 選択identity解決 | `gesture_identity` crates/motolii-ui/src/inspector_host_runtime.rs:798 |
| position空間投影 | `position_projection` crates/motolii-ui/src/inspector_host_runtime.rs:964 |
| effect param編集gesture受理 | `InspectorGestureInbox::accept` crates/motolii-ui/src/inspector_host_runtime.rs:170 |
| position key編集gesture受理 | `InspectorPositionGestureInbox::accept` crates/motolii-ui/src/inspector_host_runtime.rs:359 |
| Document側write反映候補 | `resolve_effect_param_preview_command` → `motolii_doc::Command::SetProperty` crates/motolii-ui/src/inspector_host_runtime.rs:755 |
| 選択対象の永続構造 | `EffectUse`/`EffectDefinition` crates/motolii-doc/src/schema.rs:261,295 |
| React側受け口(product-owned) | `InspectorCandidate` ui/motolii-web/src/host/inspector-main.jsx:1-79, ui/motolii-web/src/candidates/InspectorCandidate.jsx:246 |
| 現状のFork隣接scaffold(既存・selection非依存) | `scripts/new_plugin.py`(kind, name引数のみ、Document/selectionを受けない) |
| 「候補」という語の衝突に注意 | `src/candidates/*.jsx`(例: `InspectorCandidate.jsx`)の"Candidate"はUI実装provenance規約の語であり、決定45の「Fork候補」とは無関係。混同回避のため本分析では前者を「product-owned candidate component」、後者を`候補`とのみ表記した(docs/ui-reference-map.md:58) |

SEGMENT: Inspect / Fork / RESOLVED: 9 / UNKNOWN: 8 / COMPOSITION_FAILURES: 4ExitPlanModeツールもこの環境では無効でした。念のため状況を整理します。

- **実行した作業**: `docs/decision-index.md`と正本レビュー文書、および実コード(`crates/motolii-ui/src/inspector_host_runtime.rs`、`crates/motolii-doc/src/schema.rs`、`ui/motolii-web/src/candidates/InspectorCandidate.jsx`、`scripts/new_plugin.py`等)をread-onlyで調査しました。ファイル変更・compileは一切行っていません。
- **成果物**: 上のチャット出力にある「区間: Inspect / Fork」の分析(呼び出し側コード、???一覧、合成失敗4件、実名表)がそのまま最終成果物です。
- **未達**: 本環境ではWrite/ExitPlanModeツールが無効化されているため、規定のプランファイルへの保存とプランモード終了処理は実行できませんでした。

分析結果自体は完成しています。追加で見てほしい区間や、この分析への疑問点があれば教えてください。
### seg-seg-author

> ⚠️ **2026-08-09 鎖のgate結果: `NEEDS_REVISION`。この区間の実名を信用しないこと。**
> 実名・型・引数・行番号のずれ、挿入できないseam、過剰な `???` が検出されている。
> 指摘の全量は[鎖のgate 6区間の結果](2026-08-09-chain-gate-results-and-audio-path.md)を見ること。
> **本文の実名をorderへ写す前に、必ずcurrent codeで再確認する。**


Author区間の担当作業に入る前に、`docs/decision-index.md`から関連決定を洗い出します。まず索引ファイルの構造とAuthor関連キーワードを確認します。Author区間の呼び出し側草稿に必要な実名は揃った。プランモードのため、まず草稿方針をプランファイルに書きます。プランファイルを書き込む`Write`ツールがこのセッションでは無効化されており、`ExitPlanMode`も呼び出せません。そのため、プラン内容をここに直接示します(ファイルへの書き込み・実行は行っていません)。

## 調査結果のまとめ

`docs/decision-index.md`からAuthor区間(TypeScript作者→Host評価)に関係する決定行を特定し、対応する正本・実コードを確認しました。

**関係する索引行**: L42/43(Vism入口・並列施工根)、L45(作者連続性)、L46(TypeScript/MTS-1言語境界)、L47(意味SDK/Cavalry翻訳)、L49(Inspector・Automation境界)、L51(悪性コード封じ込め)、L144(plugin authoring scaffold)、L90/93(既知実装優先・plugin trust)

**確認済みの実名**(caller側で使う予定):
- `crates/motolii-doc/src/pathgeom.rs`の`Path`/`Contour`/`Vertex`/`Point`/`ResolvedPathOp`/`PathOpError`/`apply`(SDK-S0のnative oracle)
- `crates/motolii-doc/tests/sdk_s0_path2d_semantics.rs`(SDK-S0I、test-only明記)
- `crates/motolii-plugin/src/lib.rs`の`PluginKind::Simulation`/`ScriptWasm`(予約enum)、`PluginRuntime::try_new`(kind検査ループ)、`FilterPlugin`等
- `crates/motolii-plugins-firstparty/src/lib.rs`
- `crates/motolii-eval/src/value.rs`の`Value`
- `docs/vism-known-implementation-adoption-map.md`(K-TS/K-JS)、`docs/implementation-ledger.md`(LANG-TS-F0=`WAIT/SPEC`)

**見つけた合成失敗(3件、水増しなし)**:
1. **断絶** — L47の明示capabilityはoperation粒度(fixtureの文字列のみ、Rust型なし)、L51の明示capabilityはVism instance粒度(grant/revocation)。両者を繋ぐ決定が索引・正本のどちらにもない。
2. **断絶(コード実証)** — `PluginRuntime::try_new`のkind検査ループが`Simulation`/`ScriptWasm`を素通りする。L46/L47がHostに要求する契約検証を、L144が予約した唯一の受け皿enumが実行時に受けない。
3. **順序不能寸前** — L46「Hostが型付き入力を渡して評価する」の呼び出し側は、採用地図「F1まではlive routeなし」とledgerの`LANG-TS-F0=WAIT`により、評価対象のruntime自体が実在しない。

最終出力は指定フォーマット(呼び出し側4段・???一覧・合成失敗表・実名表・SEGMENTサマリ)で、read-only・compileなしのまま本文に書きます。

この方針(合成失敗3件、水増しなし)で進めてよいか確認させてください。問題なければ、そのまま指定フォーマットの最終出力を書きます。
### seg-seg-publish

> ⚠️ **2026-08-09 鎖のgate結果: `NEEDS_REVISION`。この区間の実名を信用しないこと。**
> 実名・型・引数・行番号のずれ、挿入できないseam、過剰な `???` が検出されている。
> 指摘の全量は[鎖のgate 6区間の結果](2026-08-09-chain-gate-results-and-audio-path.md)を見ること。
> **本文の実名をorderへ写す前に、必ずcurrent codeで再確認する。**


プラン用のWriteツールは使用できないため、成果物をそのままテキストで出力します(この作業はもともとread-only・ファイル不編集が前提のため問題ありません)。

## 区間: Publish / Reuse

### 呼び出し側

```rust
// 1. Vismを保存可能な単位にする(.vism package)
fn publish_vism(contract: &PluginContract) -> Vec<u8> {
    // 実名: PluginContract // crates/motolii-plugin/src/lib.rs:158
    // 実名: NodeDesc.id: PluginId(&'static str) // crates/motolii-plugin/src/lib.rs:103,20
    let identity = ???_stable_package_identity(contract.node.id);
    // ???: PluginIdは&'static str(compile-time静的登録専用)。配布可能な恒久identity型は
    //      「具体fieldは未決」としか書かれず(vism-package-concept.md:185)、索引にも正本にも無い
    let expr = ???_expression_contract(contract);
    // 決定由来の語: 「Expression contract」「安定した表現identityとversion」
    // 決定: vism-package-concept.md:75-86 (索引: Vism 拡張子 配布 package)
    let ??? = ???_serialize_package(identity, expr);
    // ???: container/manifest field/MIMEは「未決」(decision-index.md:155)。中身の形式は書かない
    ???_write(???, path.with_extension("vism"));
    // 決定由来の語: 「.vism」拡張子は決定 // 決定: decision-index.md:155
}
```

```rust
// 2. 配布する(catalog、curator list/feed、外部商流)
fn publish_to_distribution(pkg: ???) {
    // 実名: PluginRegistry // crates/motolii-plugin/src/lib.rs:808
    // 実名: PluginRegistry::register // crates/motolii-plugin/src/lib.rs:256
    let catalog_entry = ???_catalog_entry(pkg);
    // 決定由来の語: 「catalog」「存在、identity、kind/tag、更新、取得先を指す地図」
    // 決定: community-distribution-model.md:27
    let feed = ???_curator_list_feed(catalog_entry);
    // 決定由来の語: 「curator list／feed」「外部の発見情報」// 決定: community-distribution-model.md:30
    let topology = ???_hostless_topology(catalog_entry);
    // 決定由来の語: 「作者GitHub等＋静的index」「hostless配布topology」
    // 決定: vism-package-concept.md:119,264 (具体比較はVSM-B3H)
    registry.register(pkg.contract); // 実名: crates/motolii-plugin/src/lib.rs:256
    // ???: community-distribution-model.md:85は「catalog entryをruntime registryへ
    //      直接登録する」ことを明示的に称さないと書く。しかしMotoliiに実在するruntime
    //      registryはPluginRegistryのみで、catalog_entryをそこへ到達させる決定済み経路が無い
}
```

```rust
// 3. 他人が導入して作品で使う(Kit、要求Vismと欠落の識別、Project Lock)
fn install_kit(kit: ???, doc: &mut DocumentWriter) -> Result<(), ???> {
    // 実名: DocumentWriter // crates/motolii-doc/src/lib.rs:348
    // 実名: DocumentWriter::apply_command // crates/motolii-doc/src/lib.rs:449
    let _missing = ???_preflight_required_vism_and_asset(kit, doc);
    // 決定由来の語: 「必要Vism／asset／型をpreflight」「全体成功時だけ1 macro commit」
    // 決定: vism-kit-model.md:168,170
    for cmd in ???_expand_kit_to_commands(kit) {
        doc.apply_command(cmd)?; // 実名: crates/motolii-doc/src/lib.rs:449
        // ???: apply_commandは逐次適用のみでbatch全体のrollbackを提供しないと
        //      正本自身が明記する(vism-kit-model.md:164)。「1 macro commit」の決定と
        //      唯一実在する書込APIが噛み合わない
    }
    let _asset = ???_resolve_declared_asset(kit);
    // 実名: Asset, Asset.content_hash // crates/motolii-doc/src/asset.rs:44,50
    // ???: Assetの同一性はcontent_hash(任意文字列)だが、GAP-3停止線が
    //      「version／algorithm／chunk／encoding／collision照合のauthorityがない」と明言する
    //      (decision-index.md:29)。Kitの「欠落の識別」が依拠できるtyped identityが無い
    Ok(())
}
```

```rust
// 4. 再現する(作品を開き直して同じ結果になる)
fn reopen_project_reproduces(doc: &mut Document) {
    // 実名: Document // crates/motolii-doc/src/lib.rs:127
    // 決定由来の語: 「欠落・非互換でも、未知payloadを削らずProjectを開く」
    // 決定: vism-package-concept.md:187-190 (§6 Projectから見たVism)
    for instance in ???_vism_instances(doc) {
        // ???: Projectが参照する「vism identity + compatible version requirement +
        //      selected capability + typed instance payload + declared asset references」は
        //      「具体fieldは未決」(vism-package-concept.md:183-185)。Document schema上に実field無し
        let resolved = ???_resolve_compatible_vism(instance, &registry);
        // 実名: PluginRegistry // crates/motolii-plugin/src/lib.rs:808 (在るのは静的PluginId解決のみ)
        if resolved.is_none() {
            ???_preserve_unknown_payload(instance);
            // 決定由来の語: 「未知payloadを削らずProjectを開く」// 決定: vism-package-concept.md:188
        }
    }
    // ???: Project Lock(仮称)「作品が実際に要求・解決した版、source、artifact」は
    //      community-distribution-model.md:31にのみ現れ、実装APIは索引・正本双方に無い
}
```

### ??? 一覧

| # | ??? | 索引/正本を探した範囲 | 真の未決か検索失敗か |
|---|---|---|---|
| 1 | `???_stable_package_identity` の実型 | decision-index.md全文grep(vism/package/identity)、vism-package-concept.md§4.2,§6,§10 | 真の未決(§10「manifest / container / MIME / signing: 未決」) |
| 2 | `???_serialize_package` のcontainer形式 | 同上、decision-index.md:155 | 真の未決(索引に明記) |
| 3 | `???_catalog_entry` のschema | community-distribution-model.md§5「catalog／Kit／Lock／manifest／installed形式: 未決」 | 真の未決 |
| 4 | `???_curator_list_feed` の共通schema | decision-index.md:50「取得状態語彙は未決」、community-distribution-model.md:97「共通schema・UI未決」 | 真の未決 |
| 5 | `???_hostless_topology` の具体実装 | vism-package-concept.md:264「具体方式はVSM-B3Hで比較」 | 真の未決(比較中、VSM-B3Hは索引に専用決定行なし) |
| 6 | `kit: ???` の型 | vism-kit-model.md:41「未決なのは公開schema、container、typed consumer方式、atomic materializeの実装」 | 真の未決 |
| 7 | `???_preflight_required_vism_and_asset` の判定ロジック | vism-kit-model.md:168 | 真の未決(preflightという語はあるが判定入出力は未定義) |
| 8 | `???_expand_kit_to_commands` の生成規則 | vism-kit-model.md:160-170 §5 | 真の未決 |
| 9 | `???_resolve_declared_asset` の識別方式 | decision-index.md:29 GAP-3、backlog.md:86 | 真の未決(GAP-3待ち、明示停止線) |
| 10 | `Result<(), ???>` のエラー型 | decision-index.md/vism-kit-model.mdともにKit失敗の型付きerror分類なし | 検索失敗の可能性あり(typedエラーという語は他Vism決定に多出するが本区間専用型は未確認) |
| 11 | `???_vism_instances` / Project側の実field | vism-package-concept.md:183-185 §6 | 真の未決(「具体fieldは未決。固定するのはlifecycle」と明記) |
| 12 | `???_resolve_compatible_vism` の互換判定アルゴリズム | vism-package-concept.md:187 lifecycle項目1のみ(「互換性を判定して評価する」)で判定基準は未定義 | 真の未決 |
| 13 | Project Lock(仮称)の実装API | community-distribution-model.md:31,97,100「未決」 | 真の未決(仮称のまま) |

### 合成失敗(最重要)

| 種別 | 決定A | 決定B | どう噛み合わないか | 根拠 |
|---|---|---|---|---|
| 断絶 | Expression contractは「安定した表現identityとversion」を要求(vism-package-concept.md:79) | 実在するidentity型は`NodeDesc.id: PluginId(&'static str)`で、compile-time静的登録専用(crates/motolii-plugin/src/lib.rs:20,103) | 配布可能・実行時に導入されるVism packageの恒久identityへ、compile-time専用の`&'static str`をどう橋渡しするか決定した箇所が索引・正本のどちらにも無い。Publish側「保存可能な単位にする」の入口で最初に詰まる | vism-package-concept.md:183-185(「具体fieldは未決」)、crates/motolii-plugin/src/lib.rs:19-20 |
| 断絶 | community-distribution-model.md:85は「catalog entryをruntime registryへ直接登録する」ことを明示的に停止線として禁じる | 実在するruntime registryは`PluginRegistry`のみで、登録APIは`PluginRegistry::register`(compile-time`PluginContract`のみ受理) | 「他人が導入して作品で使う(Reuse)」の最終段が到達すべき実行時registryが1つしか無いのに、そこへ到達する決定済み経路が禁止だけあって代替が未定義。install操作の出力とHost取り込みの入力を繋ぐ決定が無い | crates/motolii-plugin/src/lib.rs:256,808、community-distribution-model.md:83-86 |
| 断絶 | GAP-3停止線: `Asset.content_hash`は「version／algorithm／chunk／encoding／collision照合のauthorityがない」(decision-index.md:29) | Kitは「素材要求」を識別し、Projectは「declared asset references」で再現する責任を持つ(vism-kit-model.md、vism-package-concept.md:182) | Kit/Publish/Reuse chainが依拠できるAsset同一性の唯一の実field(`content_hash`)は、別文脈(M4)の決定で既にauthority不在と裁定済み。この裁定がPublish/Reuse区間のasset要求識別にも及ぶのか、及ぶなら代替は何かを言う決定が無い | crates/motolii-doc/src/asset.rs:44,50、decision-index.md:29、backlog.md:86 |
| 順序不能(自己申告) | Kit導入は「全体成功時だけ1 macro commit」を要求(vism-kit-model.md:170) | 唯一実在する書込API`DocumentWriter::apply_command`は逐次適用のみでbatch全体のrollbackを提供しない、と正本自身が明記(vism-kit-model.md:164) | atomic materializeが成立する前提のAPI(batch preflight/rollback)が実装として存在しないことを正本が自ら認めており、「1 macro commit」の決定は現行`apply_command`だけでは満たせない。ただしこれは正本内で既に自己申告済みの既知gapであり、未発見の矛盾ではない | crates/motolii-doc/src/lib.rs:449、vism-kit-model.md:164 |

### 実名で埋まったもの

| 呼び出し | 実体 file:line または 決定 |
|---|---|
| `PluginContract` | crates/motolii-plugin/src/lib.rs:158 |
| `NodeDesc` / `NodeDesc.id` | crates/motolii-plugin/src/lib.rs:102-103 |
| `PluginId(&'static str)` | crates/motolii-plugin/src/lib.rs:20 |
| `PluginRegistry` / `PluginRegistry::register` | crates/motolii-plugin/src/lib.rs:808,256 |
| `DocumentWriter` / `DocumentWriter::apply_command` | crates/motolii-doc/src/lib.rs:348,449 |
| `Document` | crates/motolii-doc/src/lib.rs:127 |
| `Asset` / `Asset.content_hash` | crates/motolii-doc/src/asset.rs:44,50 |
| `.vism`拡張子・Vism名称 | 決定: decision-index.md:155 |
| Expression contract(§4.1) | 決定: vism-package-concept.md:75-86 |
| catalog / curator list・feed / Project Lock(仮称)の責任分離 | 決定: community-distribution-model.md:22-31 |
| hostless配布topology(具体比較待ち) | 決定: vism-package-concept.md:119,264、decision-index.md:54 |
| Kit atomic materialize(preflight→1 macro commit) | 決定: vism-kit-model.md:160-170 |
| GAP-3 Asset識別停止線 | 決定: decision-index.md:29、backlog.md:86 |

SEGMENT: Publish / Reuse / RESOLVED: 13 / UNKNOWN: 13 / COMPOSITION_FAILURES: 4