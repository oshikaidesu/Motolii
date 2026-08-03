# CU-201P-MOVE native Timeline body-drag known-semantics adoption

- 日付: 2026-08-03
- 状態: **SPEC DONE / REDUCE: MOVE-ONLY**
- 親: CU-201P / U3b / VS-2

## 1. 利用者成果

Timeline上の既存Clip bodyを左右へドラッグして配置時刻を変更する。drag中はTransient previewだけを更新し、release時に既存`SetClipStart`を一回だけsingle writerへ渡す。Escape、focus loss、pointer capture loss、無効なtarget、same-valueはDocument、journal、Undo、revision、published snapshotを変更しない。snapは既決候補の意味だけを保持し、具体的なnative threshold callerが閉じるまでこの粒では適用しない。

trim edge、slip、slide、roll、ripple、複数選択、lane変更はこの粒へ含めず、既知意味の再調査と別境界へ残す。

## 2. 先例の収束点

以下はコード移植ではなく、利用者向け意味の比較である。

| 先例 | 確認できる収束意味 | Motoliiへ持ち込まないもの |
|---|---|---|
| [Blender VSE Editing](https://docs.blender.org/manual/en/3.3/video_editing/edit/montage/editing.html) | selected stripのbody dragで時間方向へ移動し、snapを一時的に有効化できる | Blenderのshortcut、channel/ripple policy、内部operator |
| [Adobe Premiere timeline editing](https://helpx.adobe.com/uk/premiere/desktop/edit-projects/intro-to-editing/edit-video-in-premiere.html) | Timeline上のclipをrepositionし、元素材を壊さず編集する | Adobeのtool mode、商用UI、multi-track policy |
| [Adobe Premiere snapping](https://helpx.adobe.com/uk/premiere/desktop/edit-projects/change-clip-sequence/snap-clips.html) | clip edge、marker、playheadへのsnapを編集時に適用する | marker／playheadをMotoliiのsnap候補へ追加すること |
| [DaVinci Resolve Editors Guide](https://documents.blackmagicdesign.com/UserManuals/DaVinci-Resolve-18-Editors-Guide.pdf) | trim modeとsnapは一般的なTimeline編集の補助意味である | slip/slide/ripple/roll、Resolve固有のedit mode |

この比較から、今回の収束点は「既存Clipのbodyを移動し、releaseで非破壊に確定し、既決snap候補だけへ寄せる」までとする。

## 3. Authority / code fact / 接続票

| 項目 | 内容 |
|---|---|
| `AUTHORITY` | [M3 U3b](../specs/M3-ui-integration.md)、[CU-201S](2026-08-01-cu-201-u3b-move-trim-snap-responsibility-split-decision.md)、[CU-201M-S](2026-08-01-cu-201m-s-clip-start-command-contract-decision.md)、[CU-201N-S](2026-08-03-cu-201n-s-snap-target-contract-decision.md) |
| `INTERNAL TARGET` | `ProductApp`／`product_runtime.rs`、`ProductTimelineProjection::hit_test`、`TimelineBar`、`RationalTime`、`DocumentEditRuntime`、`DocumentWriter::prepare_set_clip_start` |
| `OWNER` | Documentのclip startはDocument。drag session、pointer delta、snap candidateは既存`ProductApp`内のTransient。新しいcoordinatorや第二writerは作らない |
| `WRITE ROUTE` | body hit→既存published snapshotから候補を作る→previewはread-only→release時だけ既存`SetClipStart`を`DocumentEditRuntime`へ渡す |
| `GAP` | Timeline body dragのnative event adapter、session lifecycle、既存writerへのprivate typed handoffが未接続。Document commandとsnap policyは既に閉じている |
| `RESOLUTION ROUTE` | `REUSE` projection／writer、`PATTERN`として成熟NLEのdrag lifecycleを採択、native window eventから既存ProductAppへ薄く接続。snap policyはthreshold caller成立まで保留 |
| `DISPOSITION` | `REDUCE`。move-onlyを先に実装し、trim edge hit-zoneは別粒へ送る |

## 4. 契約

1. `MouseInput::Pressed(Left)`が現行Timeline time surface内の`TimelineHit::Bar` bodyに当たった時だけsessionを開始する。Key、None、ruler、chrome、hidden Timelineは開始しない。
2. session開始時に対象`LayerId`、開始pointer時刻、開始clip start、開始projection generationを凍結する。drag中にlive Documentや表示labelを再解決しない。
3. `CursorMoved`ではpointer時刻との差分だけをpreviewする。既存`CU-201N-S`のkey／Clip edge候補へsnapする具体的なnative threshold callerが無いため、この粒ではsnapを発生させない。preview中のDocument/journal/history/revision/publishは0。
4. `Released(Left)`は開始時のgenerationとcurrent snapshotを照合し、`prepare_set_clip_start`が返す一件だけを既存single writerへ渡す。same-valueはno-op。
5. Escape、focus loss、capture loss、stale generation、対象消失、無効なRationalTime、command prepare失敗はCancelとして終了し、Document側の変更は0。
6. 一つのsessionからcommitは最大一回。release後の遅延event、duplicate release、別surfaceへの再配送は無視する。

## 5. 負例 / 非目標 / 停止線

- drag中に`DocumentWriter`、journal、Undoを呼ばない。
- snap thresholdをlogical px、DPI、fps、beat、markerから暗黙に発明しない。threshold callerは後続粒へ残す。
- Timelineのnative renderer、React Timeline Tools、Browser Place pointer captureを複製・流用しない。
- trim edge hit-zone、slip、slide、roll、ripple、multi-select、lane変更、playhead／marker／frame-grid snapは実装しない。
- `DocumentCommandRequest`の公開意味、Document schema、serde、plugin契約、永続形式を変更しない。
- native eventが製品windowへ到達する正本が閉じなければ、fixture-only成功で製品完了とせず、`EXTERNAL_GATE_PENDING`または局所STOPへ戻す。

## 6. 実装・検収出口

- `timeline_move_gesture`のpure unit oracle: body-only start、delta、既決snap、same-value、cancel、stale、duplicate release。
- `DocumentEditRuntime` oracle: release一回で`SetClipStart`一件、Undo一回、失敗時Document/history/revision不変。
- `product_runtime` compileと既存`cargo test -p motolii-ui --lib`を維持する。
- 通常製品windowで、既存Rectangleを一度配置した後のTimeline body drag→release→Undoを観測できる状態まで閉じる。trimと実Mac人間審判はこの粒の完了に数えない。
