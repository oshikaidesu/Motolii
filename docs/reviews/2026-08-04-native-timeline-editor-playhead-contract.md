# P02-C3 native Timeline editor playhead producer contract

状態: **決定**

日付: 2026-08-04

## 1. 利用者出口と境界

`P02-C3` は native Timeline の既存 ruler を primary press/drag して、現在時刻を一つだけ選ぶ producer/carrier を閉じる。対象は `native Timeline ruler → ProductApp private editor playhead → native renderer / Stage evaluation` である。通常 Inspector の Position 行、Add Position Key intent、focus、visible range、Transport/再生は別境界であり、本粒へ入れない。

この状態の五層 owner は既決どおり `Project session` であり、実装上の保持者は Host coordinator `ProductApp` とする。`motolii_doc::ProjectSession` は journal/lock/file capability であって、この五層 UI state owner ではない。Document、journal、Undo、公開 API、codec へ playhead を足さない。

## 2. 既知実装採択 preflight

| 項目 | 結果 |
|---|---|
| MECHANISM CLASS | native editor の current-time ruler click/scrub |
| KNOWN IMPLEMENTATION SEARCH | `native_timeline_renderer.rs` の既存 ruler/固定 `PLAYHEAD`、`ProductApp` の raw pointer lifecycle、`RationalTime`、既存 RenderWorker latest admission、Blender/Adobe の一次資料、固定 Rerun source を照合 |
| CANDIDATES | 既存 ruler geometry + `ProductApp`、既存 `RationalTime`、mock time surface、clip Move/Trim body gesture、React owner、generic snap、Transport clock、固定 ZERO |
| ADOPTION ROUTE | Motolii の既存 native ruler/input/time/render targets を REUSE。Blender/Adobe は ruler 固有の current-time 操作を PATTERN としてだけ採択 |
| REJECTED CANDIDATES | mock/React は owner 不一致、Move/Trim は hit/lifecycle 不一致、Transport は clock owner 不一致、fixed ZERO は producer 不在、generic snap は target owner 不在 |
| THIN MOTOLII SEAM | private ruler rect と app-private playhead state projection |
| THIN MOTOLII RESIDUAL | stated files 外なし |
| RETIREMENT | fixed-zero-only の native playhead 表示/evaluation |
| BUILD JUSTIFICATION | NONE |
| BUILD: FORBIDDEN | 新 framework/component/API/schema/codec |

Blender は Timeline の scrub 領域で click/drag して current frame を移動する既存操作を示す。[Blender Timeline](https://docs.blender.org/manual/fi/5.0/editors/timeline.html) Adobe Premiere は playhead と zoom scroll bar を別操作とする。[Premiere Timeline navigation](https://helpx.adobe.com/ca/premiere/desktop/edit-projects/change-clip-sequence/navigation-controls-in-the-timeline.html) After Effects は time ruler で CTI/current frame を選ぶ。[After Effects previewing](https://helpx.adobe.com/ca/after-effects/desktop/view-and-preview/preview-video-and-audio/previewing.html) Rerun の `re_time_panel` は chunk-store 結合のため source 再利用候補ではなく PATTERN evidence のみである。[Rerun fixed source](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_time_panel)

## 3. 決定した契約

- fresh `ProductApp` は `RationalTime::ZERO` で開始する。surface 再作成と document publish は同一 in-memory 値を保つが、新しい Host coordinator/project reopen は以前の playhead を復元しない。
- ruler 内の primary press は scrub を開始し、その座標を即時採用する。active scrub 中の cursor move は同じ state を更新し、release は最後の採用値を保持する。既存 track body/key/edge の Move/Trim hit と重ならない。
- ruler x を composition duration の closed interval `[ZERO, duration]` へ clamp して `RationalTime` として保持する。start/end は両端値である。
- Esc、focus loss、pointer capture loss、ruler/layout epoch 変更、ruler/layout 不在、非有限座標、不正 geometry/mapping、overflow は press 時値へ戻して scrub を消す。Document/journal/history/queue/Undo/selection/D2 write は常に 0 である。
- 値変更だけが private playhead revision を進める。Document `projection_generation` は進めない。renderer は既存の固定 ZERO 線をこの `RationalTime` の線へ置換し、Stage は既存 RenderRequest の evaluation time として同じ値を使う。既存 RenderWorker の latest generation admission が stale result を拒否する。
- snap は導入しない。frame grid、key、marker、clip edge、modifier snap と外部 playback/Transport は owner/target 未成立の非目標である。

## 4. 実装 allowlist と oracle

後続 implementation の allowlist は `crates/motolii-ui/src/product_runtime.rs`、`crates/motolii-ui/src/native_timeline_renderer.rs`、その focused `motolii-ui` test だけである。

PRIMARY_ORACLE は ruler の start/interior/end が `ZERO/interior/duration` へ写り、chrome/track/body/scrollbar は playhead を変えず、renderer と Stage evaluation が同一時刻を受け、cancel が press 時値へ戻して Document write 0 となること。REPO_LANES は focused `motolii-ui` test、`git diff --check`、通常 local validation とする。EXTERNAL_GATES はこの契約では増やさない。通常 Inspector Position row とその Add Position Key wiring は `CU-0A08ITI WAIT_TARGET` のまま別に残る。

## 5. compile

`P02-C3` の ruler producer/carrier sub-boundary だけを `IMPLEMENT` / ledger `DO` とする。親 P02-C3、essential focus、visible range、`CU-0A08ITI` の normal row/Inspector wiring を完了扱いにしない。
