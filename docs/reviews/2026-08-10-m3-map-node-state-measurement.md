# M3実行地図 node状態の実測照合（2026-08-10）

日付: 2026-08-10
状態: **観察 / 実測記録**

## 0. この文書の扱い

`docs/m3-rn-runtime-execution-map.md` の全nodeを current code と照合した記録である。
**判定は current worktree の実体だけを根拠にし、docs の他の記述を根拠にしていない**
（docs同士が食い違っている可能性そのものを測っているため）。

`WIRED` は製品コード上の到達経路が閉じていることを示し、**検査通過を意味しない**。
build / test / 実機検査は実行していない。

実施: Codex direct `gpt-5.6-sol` medium、`--sandbox read-only`、`scripts/run-observed-cli.py` 経由。
実行command 22件。照合対象 `ecc024a2`。

## 1. なぜ測ったか

`R1-GPU-BINDING` が `COMPILE`（未着手）と書かれた横で実装が main に存在する、
といった乖離が2026-08-09に6件確認された。原因は構造的である。
実装と統合がPRを介さず進んだ時期があり、**成果はbranchに残って状態語だけが取り残された**。
**この地図を信じて発注すると、既に動いているものを再実装させる。**

Codexを実装の現場監督に据えるには、現場監督が読む地図が信用できる必要がある。

## 2. 結果

## SUMMARY

- 全 node: **54**
- `DRIFT: YES`: **8**
- `MEASURED` 内訳:

| MEASURED | 件数 |
|---|---:|
| `WIRED` | 5 |
| `BUILT_UNWIRED` | 19 |
| `PARTIAL` | 16 |
| `ABSENT` | 10 |
| `EXTERNAL` | 4 |
| `UNKNOWN` | 0 |

## TABLE

| NODE | MAP_STATE | MEASURED | EVIDENCE | DRIFT |
|---|---|---|---|---|
| `R0-HOST` | `DONE` | `WIRED` | `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/AppDelegate.mm:85`; `crates/motolii-ui/src/rn_product_host.rs:337` | NO |
| `R0-MAC-SEAT` | `DONE` | `WIRED` | `ui/motolii-rn-legacy/macos/MotoliiRn.xcodeproj/project.pbxproj:101`; `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/AppDelegate.mm:151` | NO |
| `R0-STAGE-LIFECYCLE` | `DONE` | `WIRED` | `ui/motolii-rn-legacy/App.tsx:33`; `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:167` | NO |
| `R0-ACCEPT` | `DONE`。根拠は[R0 product runtime seat受入](2026-08-09-m3-r0-product-runtime-seat-acceptance.md) | `WIRED` | `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/AppDelegate.mm:115`; `ui/motolii-rn-legacy/App.tsx:23` | NO |
| `R1-SHELL` | `COMPILE`。current root、registration、publicationのexact write setを固定する | `PARTIAL` | `ui/motolii-rn-legacy/App.tsx:23`; `ui/motolii-rn-legacy/App.tsx:29`; `ui/motolii-rn-legacy/App.tsx:53` | NO |
| `R1-BROWSER` | `COMPILE`。現行RN targetとterminal intent edgeを固定する | `ABSENT` | `ui/motolii-rn-legacy/App.tsx:29` | NO |
| `R1-HOST-EDIT` | `WAIT(R1-BROWSER)` | `BUILT_UNWIRED` | `crates/motolii-ui/src/document_edit_runtime.rs:23`; `crates/motolii-ui/src/rn_product_host.rs:436` | NO |
| `R1-GPU-BINDING` | `COMPILE`。backend初期化のexact targetとMac/Windows別surface contractを固定する | `PARTIAL` | `crates/motolii-ui/src/rn_product_host.rs:806`; `crates/motolii-ui/src/rn_product_host.rs:909`; `crates/motolii-ui/Cargo.toml:33` | **YES** |
| `R1-STAGE` | `WAIT(R1-GPU-BINDING)` | `PARTIAL` | `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:247`; `crates/motolii-ui/src/rn_product_host.rs:1080` | NO |
| `R1-TIMELINE` | `WAIT(R1-GPU-BINDING)` | `BUILT_UNWIRED` | `crates/motolii-ui/src/product_runtime.rs:3651`; `ui/motolii-rn-legacy/App.tsx:53` | NO |
| `R1-INSPECTOR` | `COMPILE`。現行RN targetとread projection edgeを固定する | `PARTIAL` | `ui/motolii-rn-legacy/App.tsx:49`; `ui/motolii-rn-legacy/src/inspector/InspectorInitialReadPanel.tsx:18` | **YES** |
| `R1-E2E` | `WAIT(R1-SHELL..INSPECTOR)` | `PARTIAL` | `ui/motolii-rn-legacy/App.tsx:29`; `ui/motolii-rn-legacy/App.tsx:53` | NO |
| `R2-SELECTION-AUTHORITY` | `COMPILE / WAIT(R1-E2E)`。実在selection routeだけを閉じる | `WIRED` | `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:228`; `crates/motolii-ui/src/rn_product_host.rs:668`; `crates/motolii-ui/src/rn_product_host.rs:739` | **YES** |
| `R2-FOCUS-PLAYHEAD-AUTHORITY` | `TARGET_MISSING`。旧`P02-C3`どおり実在consumerを一件ずつ前ownerで特定する | `PARTIAL` | `crates/motolii-ui/src/rn_product_host.rs:124`; `crates/motolii-ui/src/rn_product_host.rs:605`; `crates/motolii-ui/src/rn_product_host.rs:438` | NO |
| `R2-TL-NAV` | `COMPILE`。visible-range consumerとnavigation intentを固定後、`WAIT(R2-SELECTION-AUTHORITY, R2-FOCUS-PLAYHEAD-AUTHORITY)` | `BUILT_UNWIRED` | `crates/motolii-ui/src/timeline_projection.rs:88`; `ui/motolii-rn-legacy/App.tsx:53` | NO |
| `R2-TL-EDIT` | `COMPILE`。move/trimは再利用、lane commandだけexact target再確認。`WAIT(R2-TL-NAV)` | `BUILT_UNWIRED` | `crates/motolii-ui/src/timeline_move_gesture.rs:59`; `crates/motolii-ui/src/timeline_trim_gesture.rs:85`; `ui/motolii-rn-legacy/App.tsx:53` | NO |
| `R2-STAGE-VIEW` | `WAIT(R1-E2E)` | `PARTIAL` | `crates/motolii-ui/src/rn_product_host.rs:949`; `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:269` | NO |
| `R2-STAGE-GIZMO` | `SPEC_ONLY`。target classificationとgroup transformの既存command写像を固定後、`WAIT(R2-STAGE-VIEW, R2-SELECTION-AUTHORITY)` | `PARTIAL` | `crates/motolii-ui/src/stage_hit_test.rs:74`; `crates/motolii-ui/src/stage_geometry_projection.rs:101` | NO |
| `R2-INSPECTOR-EDIT` | `WAIT(R2-SELECTION-AUTHORITY)` | `BUILT_UNWIRED` | `crates/motolii-ui/src/inspector_host_runtime.rs:637`; `ui/motolii-rn-legacy/src/inspector/InspectorInitialReadPanel.tsx:18` | **YES** |
| `R2-KEY-COMMAND` | `TARGET_MISSING`。現行`CommandKind`にkeyframe編集familyがない。UI／Curve担当へ発明させずP02 writer ownerで閉じる | `PARTIAL` | `crates/motolii-doc/src/command.rs:133`; `crates/motolii-doc/src/command.rs:423`; `crates/motolii-ui/src/document_edit_runtime.rs:29` | **YES** |
| `R2-KEY-UI` | `WAIT(R2-KEY-COMMAND, R2-INSPECTOR-EDIT)` | `BUILT_UNWIRED` | `crates/motolii-ui/src/inspector_host_runtime.rs:681`; `ui/motolii-rn-legacy/src/inspector/InspectorInitialReadPanel.tsx:18` | NO |
| `R2-CURVE-READ` | `COMPILE`。active interval projectionを固定後、`WAIT(R2-KEY-COMMAND)` | `ABSENT` | `ui/motolii-rn-legacy/App.tsx:53`; `crates/motolii-ui/src/timeline_projection.rs:33` | NO |
| `R2-CURVE-EDIT` | `WAIT(R2-CURVE-READ, R2-KEY-UI)` | `PARTIAL` | `crates/motolii-ui/src/product_easing_popup.rs:150`; `crates/motolii-doc/src/command.rs:437` | NO |
| `R2-E2E` | `WAIT(R2 nodes)`。統合受入であり実装nodeではない | `ABSENT` | `ui/motolii-rn-legacy/App.tsx:29`; `ui/motolii-rn-legacy/App.tsx:53` | NO |
| `R3-PROJECT-POLICY` | `SPEC_ONLY`。残る四問を一問ずつ閉じる | `PARTIAL` | `crates/motolii-doc/src/persist.rs:55`; `crates/motolii-doc/src/journal/session.rs:78` | NO |
| `R3-PROJECT-UI` | `WAIT(R3-PROJECT-POLICY)` | `BUILT_UNWIRED` | `crates/motolii-doc/src/journal/session.rs:109`; `ui/motolii-rn-legacy/App.tsx:23` | NO |
| `R3-MEDIA-EXPLORE` | `COMPILE`。file-kind admissionとRN callbackを固定。`WAIT(R1-E2E)` | `BUILT_UNWIRED` | `crates/motolii-media/src/lib.rs:21`; `ui/motolii-rn-legacy/App.tsx:29` | NO |
| `R3-MEDIA-PLACE` | `SPEC_ONLY`。動画placement defaultとSoundtrack policyを分離。`WAIT(R3-MEDIA-EXPLORE)` | `BUILT_UNWIRED` | `crates/motolii-doc/src/command.rs:409`; `crates/motolii-doc/src/command.rs:129`; `ui/motolii-rn-legacy/App.tsx:29` | NO |
| `R3-PLAYBACK-AUDIO` | `TARGET_MISSING`。現行PlaybackSessionは`PcmCache`／`AudioProducer`経路で、mixed `AudioProgram`接続がない | `BUILT_UNWIRED` | `crates/motolii-transport/src/playback.rs:15`; `crates/motolii-transport/src/playback.rs:68`; `crates/motolii-ui/src/product_runtime.rs:1060` | **YES** |
| `R3-TRANSPORT-SEEK` | `COMPILE / WAIT(R2-FOCUS-PLAYHEAD-AUTHORITY)`。旧P07のseek-only `REDUCE`を継承 | `BUILT_UNWIRED` | `crates/motolii-ui/src/rn_product_host.rs:438`; `ui/motolii-rn-legacy/App.tsx:22` | NO |
| `R3-TRANSPORT-PLAYBACK` | `WAIT(R3-PLAYBACK-AUDIO, R3-TRANSPORT-SEEK)` | `BUILT_UNWIRED` | `crates/motolii-ui/src/product_runtime.rs:1022`; `crates/motolii-ui/src/product_runtime.rs:1134`; `ui/motolii-rn-legacy/App.tsx:22` | NO |
| `R3-PREVIEW-PRESSURE` | `WAIT(R3-TRANSPORT-PLAYBACK, provider target)`。M3独自cache/schedulerを作らない | `BUILT_UNWIRED` | `crates/motolii-ui/src/render_worker.rs:96`; `crates/motolii-ui/src/render_worker.rs:197` | NO |
| `R3-SYNC-MEASURE` | `MEASURE / WAIT(R3-TRANSPORT-PLAYBACK)` | `EXTERNAL` | `crates/motolii-transport/src/playback.rs:24`; `ui/motolii-rn-legacy/package.json:5` | NO |
| `R3-EXPORT-PROVIDER` | `TARGET_MISSING`。現行同期APIにproduct async job snapshot／progress／cancel ownerがない | `PARTIAL` | `crates/motolii-export/src/lib.rs:50`; `crates/motolii-export/src/lib.rs:108`; `crates/motolii-export/src/lib.rs:221` | NO |
| `R3-EXPORT-UI` | `WAIT(R3-EXPORT-PROVIDER)` | `ABSENT` | `ui/motolii-rn-legacy/App.tsx:22` | NO |
| `R3-EXPORT-E2E` | `WAIT(R3-EXPORT-UI, R3-PROJECT-UI)` | `BUILT_UNWIRED` | `crates/motolii-export/src/lib.rs:237`; `crates/motolii-export/src/lib.rs:339`; `ui/motolii-rn-legacy/App.tsx:22` | NO |
| `R3-OPS-DELETE` | `COMPILE / WAIT(R2-SELECTION-AUTHORITY)`。既存D2 routeだけを再利用 | `BUILT_UNWIRED` | `crates/motolii-ui/src/command_registry.rs:128`; `crates/motolii-ui/src/document_edit_runtime.rs:141` | **YES** |
| `R3-OPS-DUPLICATE` | `COMPILE / WAIT(R2-SELECTION-AUTHORITY)`。CommandId／Host terminal boundaryを固定する | `BUILT_UNWIRED` | `crates/motolii-doc/src/duplicate.rs:44`; `crates/motolii-ui/src/command_registry.rs:130` | **YES** |
| `R3-OPS-RENAME` | `TARGET_MISSING`。現行Command/CommandKindにrename familyがないためwriter ownerへ返す | `ABSENT` | `crates/motolii-doc/src/command.rs:109`; `crates/motolii-ui/src/command_registry.rs:130` | NO |
| `R3-CLIPBOARD` | `SPEC_ONLY / ADOPTION_PROBE`。cross-document意味を先取りせず、macOS/Windows別にadapter採択証拠を取る | `ABSENT` | `crates/motolii-ui/src/command_registry.rs:130`; `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:228` | NO |
| `R3-KEYMAP-IME` | `COMPILE / WAIT(R1-E2E)`。実IMEはR4 gate | `BUILT_UNWIRED` | `crates/motolii-ui/src/input_router.rs:17`; `crates/motolii-ui/src/keymap_codec.rs:17`; `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:228` | NO |
| `R3-MENU` | `WAIT(R3-OPS-DELETE, R3-OPS-DUPLICATE, R3-OPS-RENAME, R3-KEYMAP-IME)`。OS menu要件が無ければ専用libraryを追加しない | `ABSENT` | `crates/motolii-ui/src/command_registry.rs:128`; `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/AppDelegate.mm:76` | NO |
| `R3-WORKSPACE` | `SPEC_ONLY`。detach top-levelとsurface再生成境界を固定。`WAIT(R1-E2E)` | `BUILT_UNWIRED` | `crates/motolii-ui/src/layout_runtime.rs:1`; `ui/motolii-rn-legacy/App.tsx:88` | NO |
| `R3-A11Y-TREE` | `ADOPTION_PROBE / WAIT(R2-TL-NAV, R2-STAGE-VIEW, R2-CURVE-READ)`。RN macOS/RNW両adapterを別証拠にする | `PARTIAL` | `ui/motolii-rn-legacy/App.tsx:35`; `ui/motolii-rn-legacy/App.tsx:53` | NO |
| `R3-DIAGNOSTIC` | `TARGET_MISSING`。`CU-204P`再確認どおりproduction source callが0。source成立前にadapter/test injectionを製品接続しない | `BUILT_UNWIRED` | `crates/motolii-ui/src/diagnostic_projection.rs:92`; `crates/motolii-ui/src/diagnostic_projection.rs:117` | NO |
| `R3-ACTIVITY` | `TARGET_MISSING`。共通snapshotを先に発明せずproviderごとにtargetを出す | `ABSENT` | `ui/motolii-rn-legacy/App.tsx:22`; `crates/motolii-export/src/lib.rs:50` | NO |
| `R3-TELEMETRY` | `TARGET_MISSING`。current raw値を受ける一意なtyped snapshotがない | `PARTIAL` | `crates/motolii-ui/src/ui_numeric_trace.rs:3`; `crates/motolii-ui/src/render_worker.rs:62` | NO |
| `R3-RECOVERY` | `WAIT(R3-PROJECT-UI, R3-MEDIA-EXPLORE, R3-PLAYBACK-AUDIO, R3-EXPORT-PROVIDER)` | `PARTIAL` | `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:254`; `crates/motolii-ui/src/rn_product_host.rs:1020` | NO |
| `R3-E2E` | `WAIT(R2-E2E, R3 required nodes)`。Local Alpha automated gate | `ABSENT` | `ui/motolii-rn-legacy/App.tsx:22`; `ui/motolii-rn-legacy/App.tsx:29` | NO |
| `R4-MAC-ADAPTER` | `COMPILE / WAIT(R3-E2E)`。code adapter | `PARTIAL` | `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:136`; `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:228`; `ui/motolii-rn-legacy/macos/MotoliiRn-macOS/MotoliiStageComponentView.mm:374` | NO |
| `R4-MAC-HUMAN` | `EXTERNAL_GATE / WAIT(R4-MAC-ADAPTER)` | `EXTERNAL` | `ui/motolii-rn-legacy/package.json:6`; `ui/motolii-rn-legacy/react-native.config.js:3` | NO |
| `R4-WIN-ADAPTER` | `COMPILE`。common Rust/headless contract成立後はMac human gateを待たず進められる。rfd/native dialogはWindows実機のopen/cancel/parent/failure証拠を別に取る | `ABSENT` | `ui/motolii-rn-legacy/react-native.config.js:1`; `ui/motolii-rn-legacy/package.json:11` | NO |
| `R4-WIN-PRODUCT` | `EXTERNAL_GATE / WAIT(R4-WIN-ADAPTER, R3-E2E)` | `EXTERNAL` | `ui/motolii-rn-legacy/react-native.config.js:1`; `ui/motolii-rn-legacy/package.json:5` | NO |
| `R4-DISTRIBUTION` | `EXTERNAL_GATE / WAIT(MAC and WIN product gates)` | `EXTERNAL` | `ui/motolii-rn-legacy/macos/MotoliiRn.xcodeproj/project.pbxproj:101`; `ui/motolii-rn-legacy/package.json:5` | NO |

## WORST_DRIFT

1. `R2-KEY-COMMAND`  
   `TARGET_MISSING` は事実と合わない。`AddPositionKey`、`SetPositionKeyInterp`、`SetPositionKeyValue` が writer/runtime に存在する。全 family 完成ではないが、ゼロから再実装させる危険が高い。

2. `R3-PLAYBACK-AUDIO`  
   地図の「PlaybackSession は PcmCache/AudioProducer 経路」という根拠が失効している。current code は `AudioProgram → MixProducer → PlaybackSession` を実装し、旧 product runtime からも呼んでいる。RN 接続だけが未成立。

3. `R1-GPU-BINDING`  
   RN Component View から `CAMetalLayer → wgpu Surface → Host の単一 GpuCtx` へ到達している。`rust-skia` 使用がないため `PARTIAL` だが、`COMPILE` を未着手として発注すると第二 surface/device owner を作る危険がある。

4. `R2-SELECTION-AUTHORITY`  
   Stage pointer down が geometry projection/hit-test を通り、既存 selection writer と Host snapshot へ接続済み。`WAIT(R1-E2E)` は実装状態を表していない。

5. `R1-INSPECTOR`  
   RN product root に read-only Inspector panel が既に存在する。ただし initial snapshot 固定で、更新・値表示は未完成。新設ではなく既存部分の存在を前提にしないと二重 panel を発注し得る。

## READY_NOW

地図に明示された `WAIT(...)` のうち、current code だけで依存成立を確認できるもの:

| NODE | 満たされた依存 | 根拠 |
|---|---|---|
| `R2-INSPECTOR-EDIT` | `R2-SELECTION-AUTHORITY` | `crates/motolii-ui/src/rn_product_host.rs:668` |
| `R3-OPS-DELETE` | `R2-SELECTION-AUTHORITY` | `crates/motolii-ui/src/rn_product_host.rs:739` |
| `R3-OPS-DUPLICATE` | `R2-SELECTION-AUTHORITY` | `crates/motolii-ui/src/rn_product_host.rs:739` |

これは依存消去の判定だけであり、発注内容・実装案ではない。

## EVIDENCE_GAP

`UNKNOWN` は **0件**。したがってコード照合上の evidence gap はない。

`R3-SYNC-MEASURE`、`R4-MAC-HUMAN`、`R4-WIN-PRODUCT`、`R4-DISTRIBUTION` はコード不足による不明ではなく、実機・人間・成果物を必要とするため `EXTERNAL` とした。ファイル変更は行っていない。
## 3. 地図への反映

上記 `DRIFT: YES` の8 nodeについて、`docs/m3-rn-runtime-execution-map.md` の状態語を
実測値へ更新した。**依存関係の記述と施工内容は変更していない。**

更新した node: `R1-GPU-BINDING`、`R1-INSPECTOR`、`R2-SELECTION-AUTHORITY`、
`R2-INSPECTOR-EDIT`、`R2-KEY-COMMAND`、`R3-PLAYBACK-AUDIO`、`R3-OPS-DELETE`、`R3-OPS-DUPLICATE`。

## 4. 読み取れること

**`WIRED` 5 / `BUILT_UNWIRED` 19 / `PARTIAL` 16 / `ABSENT` 10 / `EXTERNAL` 4。**

54 node中 **35 が「コードはあるが繋がっていない」**（`BUILT_UNWIRED` + `PARTIAL`）であり、
**本当に実体が無いのは10だけ**である。M3が「作る工程」ではなく
「先に作った資産を接続する統合ゾーン」であるという2026-08-07の読み直しが、
地図全体で数値として確認された。

したがって発注の既定形は **「Xを実装せよ」ではなく「既にあるXを既にあるYへ繋げ」**である。
`ABSENT` を宣言する前に名前を変えて検索する規律が、10件という数字を支えている。

## 5. 非目標

- 本書を根拠に実装を発注すること（`READY_NOW` は依存消去の判定であって発注内容ではない）
- 測定していない node の状態語を推測で更新すること
- build / test / 実機検査の結果を本書へ含めること
