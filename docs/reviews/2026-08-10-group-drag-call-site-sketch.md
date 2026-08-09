# グループroot drag 呼び出し側仮コード

日付: 2026-08-10
状態: **鎖のgate未通過 / 観察用器具**

OUTCOME: Stage上でグループrootを掴んで動かし、releaseで1回だけ確定する。

## 0. この文書の扱い

本書は[仮コード（provisional call-site sketch）の器具境界](2026-08-07-provisional-call-site-sketch-instrument-decision.md)に従い、呼び出し側から契約境界を露出させる文書である。

- **compileしない。`crates/`へ置かない。実装ではない**
- **authorityではない。closed orderの`AUTHORITY`欄へ引かず、仕様・schema・公開APIの根拠にしない**
- `???`は希望API名ではなく、現行repoで実名を置けなかった契約境界である
- `AGENTS.md`の「findingは権限ではない」に従い、報告と分類だけを行う。修理案・実装案・発注を含めない
- group bounds、gizmo採択、`BUILD JUSTIFICATION`を裁定しない

本鎖はまだ鎖のgateを通っていない。以下の実名、行番号、`???`分類はgate前の観察である。

## 1. 呼び出し側

```rust
// OUTCOME: Stage上でグループrootを掴んで動かし、releaseで1回だけ確定する。

// ── 1. pointer down → hit-test → グループroot ──────────────────
let canonical = view_local_to_canonical(x, y, width, height)?;
// crates/motolii-ui/src/stage_hit_test.rs:25-40

let rect_only_projection = project_stage_geometry(
    &document,
    EvaluationTime::new(host.current_time),
    &tracks,
)?;
// crates/motolii-ui/src/stage_geometry_projection.rs:78-121
// 現行は TrackItem::Group(_) を Unavailable(Group) にする(:101-105)。
// hit_test_projected_layers は Unavailable を飛ばすため(:74-99)、
// この projection のままでは group 自身を hit できない。

let group_pick_projection = ???_evaluated_group_bounds_for_picking(
    &document,
    EvaluationTime::new(host.current_time),
    &tracks,
    &rect_only_projection,
);
let hit = hit_test_projected_layers(canonical, &group_pick_projection);
// crates/motolii-ui/src/stage_hit_test.rs:74-99

let group_root = ???_group_root_from_hit(&document, hit);
// 契約境界: child hitから「直近のgroup」「最上位group」「現在の編集scopeのgroup」の
// どれをrootとするか、またgroup自身のbounds hitをどう優先するかが決まれば埋まる。
// 現行の StageHit は Layer/Miss だけである(stage_hit_test.rs:18-21)。

queue.push_replace_primary(group_root);
// crates/motolii-ui/src/document_edit_runtime.rs:123-126

// ── 2. グループの bounds を得る ───────────────────────────────
let bounds = ???_evaluated_group_bounds_for_picking(
    &document,
    EvaluationTime::new(host.current_time),
    &tracks,
    group_root,
);
// 【本当に未決】同名の ??? は上の picking projection と同じ契約境界を指す。
// 最低限、次が決まらなければ実名にできない:
// - content由来（評価済みの子の合成内容へ密着）か、canvas由来（compositionサイズ固定）か
// - どの座標系で返すか（group-local / canonical world / camera-view）
// - 評価時刻、非表示の子、mask/clipping、group/child effect、空groupを含める規則
// - Video/Vector/Plugin/Groupなど bounds 不明の子を Unknown とする時の伝播と拒否規則
// - bounds と picking が同一の参加境界を使うか
// ここでは content/canvas のどちらも選ばない。

// ── 3. bounds → pivot / handle位置 ─────────────────────────────
let (pivot, handles) = ???_pivot_and_handles_from_group_bounds(bounds, group_root);
// 【依存待ち】bounds契約に加え、gizmo機構の採択裁定が未了。
// docs/reviews/2026-08-08-gizmo-known-implementation-preflight.md:100-110
// boundsが埋まらない限り、pivot、handle配置、screen-size補正、hit領域、snap基準も決まらない。

// ── 4. drag delta を canonical 座標へ逆写像 ────────────────────
let inverse = (bounds.camera_view * bounds.world).try_invert()?;
// crates/motolii-doc/src/affine.rs:81 — Affine2D::try_invert
let current_local = inverse.transform_point(current.x, current.y);
let start_local = inverse.transform_point(start.x, start.y);
// crates/motolii-doc/src/affine.rs:50-53 — Affine2D::transform_point
let canonical_delta = [
    current_local[0] - start_local[0],
    current_local[1] - start_local[1],
];

let baseline = ???_group_drag_baseline_owner(
    group_root,
    host.current_time,
    &document,
    bounds,
    pivot,
);
// 【配置】PositionGestureBaselineは実在するがkeyed Position専用で、旧ProductRuntimeの
// private保持先だけにある(product_runtime.rs:204,228-235,3375-3394)。
// groupのconstant/keyed Positionを同じgestureで保持する通常RN Hostの席は実名にできない。

// ── 5. transient preview（Documentへ書かない） ─────────────────
let preview_command = match &baseline.position {
    DocParam::Keyframes(_) => Command::SetPositionKeyValue {
        target: group_root,
        key: baseline.key,
        old: baseline.value,
        new: [
            baseline.value[0] + canonical_delta[0],
            baseline.value[1] + canonical_delta[1],
        ],
    },
    DocParam::Const(DocValue::Vec2(old)) => Command::SetProperty {
        target: group_root,
        property: ScalarPropertyId::Position,
        old_value: DocParam::const_vec2(*old),
        new_value: DocParam::const_vec2([
            old[0] + canonical_delta[0],
            old[1] + canonical_delta[1],
        ]),
    },
    _ => return, // terminalへ変換せず、Document変更0で処分
};
// Command::SetProperty / ScalarPropertyId::Position は実在する。
// crates/motolii-doc/src/command.rs:34-40,282-288,1604-1614

???_group_drag_transient_owner.update(canonical_delta, preview_command.clone());
// 【配置】drag中の最新delta/previewを保持するRN Hostの席が無い。

render_client.submit_preview(render_request, preview_command)?;
// crates/motolii-ui/src/render_worker.rs:608-617
// workerはDocumentをcloneしてpreview Commandを適用するだけで、live Documentへ書かない。
// crates/motolii-ui/src/render_worker.rs:456-469

???_overlay_inside_preview_pass(handles, pivot, canonical_delta);
// 【依存待ち】N-OVERLAYは依存追加段階までで、crates/**/srcにskia_safe使用は0件。
// 同一frameのsubmit/present前へhandleを描く製品接続を実名にできない。

// ── 6. release → D2 terminal requestを1回だけ積む ──────────────
match &baseline.position {
    DocParam::Keyframes(_) => {
        queue.push_set_position_key_value(SetPositionKeyValueRequest {
            target: group_root,
            key: baseline.key,
            old: baseline.value,
            new: [
                baseline.value[0] + canonical_delta[0],
                baseline.value[1] + canonical_delta[1],
            ],
        });
        // crates/motolii-ui/src/document_edit_runtime.rs:108-111,220-226
    }
    DocParam::Const(DocValue::Vec2(old)) => {
        ???_constant_position_terminal_entry(
            &mut queue,
            Command::SetProperty {
                target: group_root,
                property: ScalarPropertyId::Position,
                old_value: DocParam::const_vec2(*old),
                new_value: DocParam::const_vec2([
                    old[0] + canonical_delta[0],
                    old[1] + canonical_delta[1],
                ]),
            },
        );
        // 【配置】D2のCommandは実在するが、DocumentEditQueueの製品入口は無い。
        // queueに実在するPosition terminalはkeyed valueだけ(:108-111)。
    }
    _ => return, // terminalへ変換せず、Document変更0で処分
}

// releaseごとにterminalを1件だけprocessする。
let published = runtime.process_next(&mut queue, primary, projection_generation)?;
// crates/motolii-ui/src/document_edit_runtime.rs:414-422
// commit側はCommand 1件を1 macroとして適用する(:705-730)。

// ── 7. Escape / focus loss / pointer cancel ────────────────────
match pointer_terminal {
    HostPointerCandidate::Cancelled {
        reason: HostPointerCancel::Escape | HostPointerCancel::CaptureLost,
        ..
    }
    | RouterOutput::SafetyCancel { .. } => {
        ???_dispose_group_drag_transient(baseline, &mut ???_group_drag_transient_owner);
        // 【配置】入力のtyped cancelは実在するが、group drag stateへの接続先が無い。
        // semantic terminalをqueueへ積まない。Document変更は0。
    }
    _ => {}
}
// HostPointerCandidate / HostPointerCancel: host_pointer_capture.rs:15-35,111-144
// focus/capture loss: input_router.rs:23-26,49-52

// ── 8. Undoは1回 ───────────────────────────────────────────────
// 上のreleaseが1 terminal → 1 Command → 1 macroなら、通常Undo intentを1回積む。
queue.push_prepared(undo_output, None)?;
// crates/motolii-ui/src/document_edit_runtime.rs:132-163
runtime.process_next(&mut queue, primary, projection_generation)?;
// runtimeはHistoryDirection::UndoをDocumentWriter::undoへ渡す。
// crates/motolii-ui/src/document_edit_runtime.rs:769-772
// DocumentWriter::undoは直前のgesture macroを丸ごと戻す。
// crates/motolii-doc/src/lib.rs:477-489
// cancel経路ではrelease Commandを積まないため、戻すDocument変更自体が無い。
```

## 2. ??? 一覧

同じ名前の複数出現は一つの契約境界として数える。分類は背骨仮コードの**配置 / 依存待ち / 本当に未決**へ揃える。

| # | `???` | 種別 | 現状と、埋まるために決まること |
|---|---|---|---|
| 1 | `???_group_drag_baseline_owner` | **配置** | keyed専用`PositionGestureBaseline`と旧ProductRuntimeの保持先は実在する。groupのconstant/keyedを扱う通常RN Hostのgesture席が置かれれば実名照合できる |
| 2 | `???_group_drag_transient_owner` | **配置** | `RenderWorkerClient::submit_preview`は実在する。group dragのlatest delta/previewのHost保持先だけを実名にできない |
| 3 | `???_constant_position_terminal_entry` | **配置** | `Command::SetProperty { property: Position }`は実在するが、`DocumentEditQueue`にconstant Position terminal入口が無い。D2意味そのものの未決とは数えない |
| 4 | `???_dispose_group_drag_transient` | **配置** | Escape、focus loss、capture lossのtyped入力は実在するが、group drag transientを破棄する接続先が無い |
| 5 | `???_pivot_and_handles_from_group_bounds` | **依存待ち** | group bounds契約とgizmo採択裁定の両方を待つ。`BUILD JUSTIFICATION`は`NOT NONE`のまま |
| 6 | `???_overlay_inside_preview_pass` | **依存待ち** | N-OVERLAYは依存追加段階まで。製品`src`に`skia_safe` consumerが無く、handleを描く先を実名にできない |
| 7 | `???_evaluated_group_bounds_for_picking` | **本当に未決** | content由来 / canvas由来、座標系、時刻、effect/mask/clipping、空group、Unknown伝播、pickingとの同一性が契約として決まれば埋まる |
| 8 | `???_group_root_from_hit` | **本当に未決** | child hitから選ぶgroup階層（直近 / 最上位 / 編集scope）とgroup自身のhit優先が決まれば埋まる |

`???_COUNT`: **配置 4 / 依存待ち 2 / 本当に未決 2 / 合計 8**。

最重要の境界は#7である。#7が埋まらない限り、group自身のhit領域、#8のroot候補、pivot、handle位置・hit領域、snap基準、previewのdirty領域を確定できない。

## 3. 既存で埋まったもの

| 呼び出し / 契約 | 実体 `file:line` | この鎖での範囲 |
|---|---|---|
| `project_stage_geometry` | `crates/motolii-ui/src/stage_geometry_projection.rs:78-121` | 評価時刻のStage投影。Groupは`:101-105`で明示的にUnavailable |
| `view_local_to_canonical` | `crates/motolii-ui/src/stage_hit_test.rs:25-40` | view-localから正準座標への変換 |
| `hit_test_projected_layers` | `crates/motolii-ui/src/stage_hit_test.rs:74-99` | Available幾何だけのhit-test。Unavailableは`:80-83`で除外 |
| `DocumentEditQueue::push_replace_primary` | `crates/motolii-ui/src/document_edit_runtime.rs:123-126` | 実在LayerIdをprimaryへ送るselection writer |
| `Affine2D::transform_point / try_invert` | `crates/motolii-doc/src/affine.rs:50-53,81` | camera/worldからlocal deltaを得る既存数学 |
| `PositionGestureBaseline` | `crates/motolii-ui/src/product_runtime.rs:228-235` | keyed Position限定。group専用でもRN Host接続済みでもない |
| `Command::SetProperty(Position)` | `crates/motolii-doc/src/command.rs:34-40,282-288,1604-1614` | constant Positionを表現できる既存D2 command |
| `Command::SetPositionKeyValue` | `crates/motolii-doc/src/command.rs:443-448` | keyed Position valueのterminal command |
| `RenderWorkerClient::submit_preview` | `crates/motolii-ui/src/render_worker.rs:608-617` | preview Commandをworkerへ渡す |
| clone Documentへのpreview適用 | `crates/motolii-ui/src/render_worker.rs:456-469` | live Documentへ書かないtransient preview |
| `DocumentEditQueue::push_set_position_key_value` | `crates/motolii-ui/src/document_edit_runtime.rs:108-111,220-226` | keyed Positionだけのqueue入口 |
| `DocumentEditRuntime::commit_command` | `crates/motolii-ui/src/document_edit_runtime.rs:705-730` | 1 Commandを1 macroとしてsingle writerへ確定 |
| `HostPointerCandidate::{Moved,Released,Cancelled}` | `crates/motolii-ui/src/host_pointer_capture.rs:22-35,111-144` | pointer release / Escape / capture lossのtyped terminal |
| `SafetyInterrupt / SafetyCancel` | `crates/motolii-ui/src/input_router.rs:23-26,49-52` | focus loss / pointer capture lossのtyped cancel |
| `DocumentWriter::undo` | `crates/motolii-doc/src/lib.rs:477-489` | 直前のgesture macroを1回で戻す |

## 4. 相互検証

本書の`???`と、[成果駆動統合地図](../outcome-driven-integration-map.md)の`ABSENT` / `PARTIAL`（地図上は一部`BUILT_UNWIRED` / `PROBE_ONLY` / `UNDECIDED`）を照合する。

| `???` | survey側 | 判定 |
|---|---|---|
| #1 / #2 / #4 | gesture baseline / transient preview = `BUILT_UNWIRED` (`outcome-driven-integration-map.md:116`) | **一致**。既存旧routeと通常Host接続の間の配置gap |
| #3 | release → Position key = `BUILT_UNWIRED` (`:118`) | **部分一致・要注意**。surveyはkeyed routeを数えるが、constant Position queue入口の不在を別nodeにしていない |
| #5 | gizmo handle描画 = `ABSENT`、gizmo survey/adoption未裁定 (`:97-103,117`) | **一致**。ただし採択済みへ繰り上げない |
| #6 | N-OVERLAY = `PROBE_ONLY` (`:73-89`) | **一致**。依存追加段階をrenderer製品接続と数えない |
| #7 | 幾何投影 = `WIRED` (`:112`) | **不一致 → 要注意**。surveyの`WIRED`はRectangleの正準rect投影であり、Groupは実コードでUnavailable。group bounds nodeが地図に無い |
| #8 | primary selection producer = `WIRED` (`:115`) | **不一致 → 要注意**。実在producerはLayer/Missを選べるが、group-root階層policyを持たない |

最重要の不一致は#7である。「幾何投影`WIRED`」をgroupへ一般化すると、`stage_geometry_projection.rs:101-105`および`stage_hit_test.rs:80-83`と衝突する。#8も同様に、selection writerの実在はgroup-rootを選ぶ契約の実在を意味しない。

関連資料の位置づけ:

- `M4-P01-REGION`（`docs/m4-known-implementation-adoption-map.md:33,71-85`）はRoD / RoI / tile extentとUnknown時の過小評価禁止・full fallbackを扱う。**render評価領域の契約であり、Stage操作用group boundsがcontent由来かcanvas由来かを決めない。** Unknown伝播を欠落扱いせず安全側へ送るpatternは関連するが、#7を埋める実名ではない
- `M5-3d-and-post.md:111-116`はHostがbounds / picking参加境界を所有すると定めるため、owner境界には関連する。**所有者を決めるだけでgroup boundsの意味は決めない**
- 同書P2DのUnknown bounds oracle（`:174`）はUnknownをsort / cull根拠にせずFinalを切らない。これはdepth/render側の負例であり、Unknown groupをStageでどう選択するか、pivotをどこへ置くかは決めない
- `concept.md:153,198-200`は、プリコンポを作らず再帰group + bakeで置換し、groupがclipと同じitem envelopeを持ち、子を合成したflat 1枚へeffectを適用し、groupにsize / resolution / durationを保存しないことを定める。**group boundsのlive導出をcontent由来 / composition canvas由来のどちらにするかは裁定していない。** むしろeffect / mask / child compositeのどの段をboundsへ含めるかが#7に残ることを示す

## 5. この鎖が要求しないもの

Rectangle前提の背骨と同じく、この鎖を最短で書いても次は要求されない。

- M4 cache / resource / disk artifact / group bake。現在時刻1 frameのStage操作であり、再生・書き出し・仮出力を行わない
- M5 3D import / Camera Provider / depth policyの実装。M5資料はbounds ownerとUnknown負例の照合にだけ使い、本鎖からM5 runtimeを呼ばない
- 新しいDocument schema、公開API、group size / resolution field
- group boundsのcontent由来 / canvas由来の裁定
- gizmo候補の採択、独自gizmo framework、`BUILD JUSTIFICATION: NONE`への繰り上げ
- overlay rendererの実装、`skia_safe` consumer、依存・描画先の追加
- constant Position command意味の新設。D2 `Command::SetProperty(Position)`は既に実在し、欠けているのは通常製品queueの入口である
- Timeline、Easing、rename、clipboard、a11y、activity、telemetry
- `crates/`変更、compile、test追加、README登録、closed order作成、発注、修理案

この文書が行うのは、現行repoで実名にできた箇所とできなかった契約境界の報告・分類だけである。
