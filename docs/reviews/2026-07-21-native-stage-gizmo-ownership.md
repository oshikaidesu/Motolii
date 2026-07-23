# Native Stage gizmo所有境界（2026-07-21）

状態: **決定**。2D object handle、selection outline、3D transform gizmo、Depth rail/axisの可視描画は
native wgpu Stageのpresentation overlayが所有する。Web/Reactを採る理由はshell、panel、Browser、form、
toolbar、hot reload、community UI kitであり、Stage上の直接操作をWebへ寄せる理由にはしない。

本決定はM3のUI runtime採否、M5の既存transform意味、Document、公開API、plugin契約を変更しない。
特に「GPU所有」はGPU readbackによるpickingや、canonical render出力へのgizmo焼き込みを意味しない。

## 1. Motolii側の正本と現行事実

- M5 P2Uは単一XYZ世界のまま、Scaleを`scale.x/y`だけ、Depth Moveを`position.z`だけへ確定する。
  3D modeや第二のDepth fieldは作らず、1 gestureをD2 command/Undo 1回へする。
- M3はUIをDocumentの投影とし、UI threadの同期readback、toolkit型のdomain流出、px/DPIの永続化を禁じる。
- 現行`motolii-ui`のStageはnative preview textureを登録して表示するだけで、製品用overlay passと
  picking systemはまだない。
- `motolii-render`はpreview/exportで共有するcanonical画素の所有者である。編集用gizmoをここへ入れると
  export画素とpreview/export同一評価を汚染するため、Stage presentationの後段へ分離する。

したがって所有境界は次のとおりとする。

```text
React / WebView                    Native Stage presentation
  shell / panel / toolbar            canonical display texture
  community UI kit                   + wgpu gizmo/handle overlay
  bounded a11y controls               + CPU analytic hit-test
          | typed intent                       |
          +---------------- transient preview -+
                               |
                         D2 commit / Undo
```

## 2. 一次資料と固定source監査

| 先例 | 確認した事実 | Motoliiでの裁定 |
|---|---|---|
| [transform-gizmo 0.9.0 / `e8e1d8e`](https://github.com/urholaukkarinen/transform-gizmo/tree/e8e1d8eb9f46762bef3b5b53b48ab1b465c61a08) | framework非依存を掲げ、interactionとviewport vertex生成を分離。translate/rotate/scale、world/local、snap、pixels-per-pointを持つ。一方coreは`emath`/`epaint`/`ecolor`へ直接依存し、Rust 1.92を要求。固定sourceで`cargo test -p transform-gizmo`はunit test 0、doc test 1件 | **SPIKE CANDIDATE**。mode/geometry比較に使うが直接依存を決めない。`epaint`型をStage/APIへ持ち込まない |
| [Bevy transform gizmo / `0ecdfaa`](https://github.com/bevyengine/bevy/blob/0ecdfaa0b6909b581bbb51759adf5999ea663b7a/crates/bevy_gizmos/src/transform_gizmo.rs) | screen-space pixel threshold、画面一定サイズ、world/local、snapを持つ。axis/ringをviewportへ投影してCPU距離判定し、dragはray-planeで解く。release欠落にも耐える | **PATTERN**。CPU解析判定、画面一定サイズ、cancel/release設計だけ転用。Bevy `Transform`直書きや依存は採らない |
| [Unreal `UTransformGizmo`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Editor/EditorInteractiveToolsFramework/UTransformGizmo) / [InteractiveToolsFramework](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/InteractiveToolsFramework) | view context、hit target、parameter source、transform/state target、interaction前後、screen-space hit thresholdを分離 | **PATTERN**。描画、hit-test、preview、transactionを別責務にする |
| [Blender viewport gizmos](https://docs.blender.org/manual/en/latest/editors/3dview/display/gizmo.html) / [transform orientation](https://docs.blender.org/manual/en/latest/editors/3dview/controls/orientation.html) | move/rotate/scaleを別形状で表示し、axis/plane、global/local、snap/fine adjustmentを提供 | **INTERACTION ORACLE**。形とmodeの識別をM5 fixtureで審判する |
| [wgpu `BufferSlice::map_async`](https://docs.rs/wgpu/latest/wgpu/struct.BufferSlice.html#method.map_async) | mappingはGPUが安全になるまで待ち、callback進行にはpoll/submitが必要。mapped中はGPU利用と排他的 | gizmo hover/drag hot pathのGPU ID readbackを**REJECT**。dense scene pickingが将来必要なら非同期・generation破棄の別spikeにする |

`transform-gizmo`の「framework非依存」は、Motoliiのtoolkit非依存公開境界をそのまま証明しない。
固定sourceのCargo依存と試験面を優先し、READMEの表現だけで採用しない。

## 3. 決定

1. **描画**: canonical display textureを作った後、同じnative wgpu device/surface上のpresentation passで
   handle/gizmo/outline/railを描く。canonical texture、export、plugin画素へ混ぜない。
2. **hit-test**: 少数の固定gizmoはCPU側でscreen-space segment/ring/planeとray-planeを解析判定する。
   logical pixel、DPI、camera projectionはTransient inputで、Documentへ保存しない。
3. **編集**: drag中はTransient previewだけを更新し、releaseでD2 commandを1回確定する。Escape、focus loss、
   pointer cancelは変更ゼロまたは既存SafetyInterrupt規則へ正規化する。
4. **Web**: Web/Reactはtool選択、数値欄、Inspector、toolbar、説明、bounded accessibility proxyを所有できる。
   native Stageの上へtransparent WebViewを重ねることをgizmoの成立条件にしない。
5. **意味**: 2D/3Dの描画実装からDocument schema、公開raw transform API、plugin自由GPU APIを逆算しない。
   M5 Scale/Depth、local/world、perspective/orthographic、common/mixed parentの既存fixtureが審判である。

## 4. 候補の処分

| 候補 | 処分 | 理由 |
|---|---|---|
| transform-gizmo直接依存 | 比較spike待ち | epaint結合、Rust version、semantic test不足、M5 Scale/Depthへの適合が未証明 |
| Bevy gizmo依存 | 不採用 | engine/Transform所有を持ち込まず、source patternだけで足りる |
| Three.js / Konva handle | 先例・interaction oracleのみ | 既存機構の成立証拠は保持するが、製品Stage runtime選定の根拠から外す |
| Motolii独自の全gizmo engine | 不採用 | 既知geometry/interaction patternを検索・比較して必要な薄いadapterだけを持つ |
| GPU ID-buffer readbackでhover | 不採用 | UI thread待機とGPU/CPU同期を増やす。固定gizmoはCPU解析判定で足りる |
| transparent WebView overlay | gizmo要件から除外 | Web UIのflex/layoutとnative Stage直接操作は非重複siblingで分担できる |

## 5. 次のnative Stage spike合格条件

- canonical preview/export画像がoverlayの有無でbit不変
- 同じnative wgpu device/surfaceを使い、frameごとのresource生成とGPU readbackが0
- DPI、window resize、camera zoomでvisual sizeとlogical hit targetが一定
- perspective/orthographic、world/local、axis/plane、camera操作排他、occlusion/on-top方針をfixture化
- M5 Scaleは`scale.x/y`だけ、Depth Moveは`position.z`だけを変更し、互いの負例が0
- common parentの複数選択を扱い、mixed parentの一括Z編集を既存診断で拒否
- move中semantic write 0、releaseでD2/Undo 1回、Escape/focus lossで確定変更0
- keyboard等価操作とbounded accessibility proxyを同じtool語彙へ接続
- transform-gizmoを比較する場合は依存差分、生成vertex、hit-test、cancel、camera/DPI、license、
  Rust toolchainを固定reportへ残し、不適合なら既知のmath patternを薄く移植する

このspikeはM3/M5 Stage責務の検証であり、G0-9のReact/WebView runtime採否を単独で閉じない。

## 6. 非目標と停止線

- 本書だけでnative overlay実装、WebView製品統合、plugin GPU surfaceを開始しない。
- gizmoをcanonical renderer、Document、journal、公開plugin契約へ追加しない。
- GPUで描くことを理由にpicking、D2 commit、accessibilityまでGPUへ集約しない。
- Three.js/Konva/Bevy/transform-gizmoの型、scene、serialization、transform所有を正本にしない。
- 既存M5 fixtureに合わない場合はライブラリ都合で期待値を変えず、比較spikeを不合格にする。

反対側の縮小条件は[Native Stage gizmo反対側レビュー](2026-07-21-native-stage-gizmo-counter-review.md)に記録した。
