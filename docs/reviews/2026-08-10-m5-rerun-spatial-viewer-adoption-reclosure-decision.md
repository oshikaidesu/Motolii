# M5 Rerun Spatial Viewer 採択再締結決定

日付: 2026-08-10
状態: **決定／旧M5全面休止を撤回。製品依存・runtime接続・実装orderは未開始**

## 1. 結論

M5の主部は、Rerunを機構ごとの`PATTERN`として分解して独自実装するのではなく、固定commit
`954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`の**Spatial Viewer系を一つの既知実装として
`ADOPT / WRAP`し、Motolii Stageへ接続する**。

M3の主役surfaceはStageである。M3はRN shell、rust-skia overlay、window／input、D2 single writer、
Preview／Exportの製品ownerを維持し、M5はそのStage内で3D／spatial sceneを評価・表示する中核をRerunから
採択する。Rerunのstore、Blueprint、selection、playheadを第二のDocument／writer／製品authorityにはしない。

旧[M5休止・M3意味開放契約](2026-08-02-m5-pause-until-m3-semantic-release.md)の全面休止は撤回する。
M5 spatialをM3完成後まで止めると、主役であるStageを閉じるためのRerun接続自体が待たされる循環になる。
以後はM3 Stageの成果に必要なRerun接続を同じoutcome内で一契約ずつ進められる。第二writer、別world、
別Preview／Export、未決公開schemaを作らないという旧休止契約の負例は維持する。

この決定は「Rerunの全製品をそのまま埋め込む」ことも「Rerun sourceをfile単位で複製する」ことも意味しない。
採択単位をrenderer、picking、camera等の断片から、相互接続済みのSpatial Viewer subsystemへ引き上げる。

## 2. 実コードで確認した閉包

固定archiveのSHA-256は既存inventory記載値
`a891a52e4a56ced5f9d438527894d295fefe0f0ba9e10bf0d47a219f94f07af4`と一致した。

| 閉包 | 固定commitの実コード | 確認した事実 |
|---|---|---|
| View登録 | [`default_views.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_viewer/src/default_views.rs) | `SpatialView2D`／`SpatialView3D`が他の製品Viewと同じregistryへ登録される |
| time／selection query | [`system_execution.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_viewport/src/system_execution.rs) | visible entityだけを現在timeline／time／highlightでqueryし、active Viewを並列実行する |
| 3D Stage | [`ui_3d.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_view_spatial/src/ui_3d.rs) | camera、focus／track、picking、outline、grid、全draw data、wgpu targetが一つのViewで合流する |
| GPU合成 | [`re_renderer_callback.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_viewer_context/src/gpu_bridge/re_renderer_callback.rs) | `ViewBuilder::draw`のcommand bufferと`composite`を既存egui-wgpu passへ接続する |
| 表示対象 | [`visualizers/mod.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_view_spatial/src/visualizers/mod.rs) | mesh／Asset3D、画像、動画、点、線、camera、transform axes等が同じ3D Viewへ登録される |
| 拡張 | [`custom_visualizer`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/examples/rust/custom_visualizer) | 内蔵3D Viewをforkせず、独自archetype／visualizer／rendererをpicking／outline込みで追加できる |
| Host callback | [`viewer_callbacks`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/examples/rust/viewer_callbacks/src/main.rs) | play／pause、time、timeline、selectionを外側のHostへ通知できる |
| 依存整合 | Rerun root `Cargo.toml`とMotolii root `Cargo.toml` | 両者とも`egui 0.35`／`wgpu 29`。概念比較だけでなく同一世代の接続候補である |

以上により、旧採択地図の「Rerunはimporter、scene、renderer、camera、depth、pickingの各classで
`PATTERN`のみ」という裁定は、実コードが持つ接続済みsubsystemを過小評価していた。各機構を別々に再実装すると、
既知実装優先規律に反してSpatial ViewerをMotolii内で再構築することになる。

## 3. Motoliiに残す薄い責任

Rerun採択後にもMotoliiが所有するのは次だけである。

1. Documentの評価snapshot／stable identity／時刻をRerun側のview入力へ写すprojectionと、確定編集をD2 commandへ戻すsingle-writer seam。
2. `Scale / Depth Move`、Depth Rail、1 gesture = 1 Undo、Escape／stale epoch変更0等のauthoring意味とrust-skia overlay。
3. `Layer Order / Group Depth / AE-style Bins`、linear-premultiplied合成、Preview／Export同一路の製品policy。
4. faithful glTF、core metallic-roughness、`KHR_materials_unlit`、neutral environment。固定Rerun commitのmesh shaderはbase color＋固定2灯であり、このPBR範囲を満たさない。
5. ResourceLedgerによるhard budget、admission、evictionと、Motoliiのtyped diagnostic。

これらはRerunを拒否する理由ではなく、採択済みsubsystemの周囲へ置くMotolii固有のresidualである。

## 4. 採択境界

### MECHANISM CLASS

Spatial scene query、View lifecycle、camera navigation、visualizer dispatch、wgpu draw/composite、picking、outline、
bounds、mesh／image／video／point／line表示、custom visualizer extension。

### KNOWN IMPLEMENTATION SEARCH

固定Rerun commitの`re_viewer`、`re_viewport`、`re_view_spatial`、`re_renderer`、custom viewer examplesを実コードで追跡した。

### CANDIDATES

- Rerun Spatial Viewer subsystem: `ADOPT / WRAP`
- 現行Motolii wgpu／Stage route: Host surface、output、single writerとして`REUSE`
- Bevy／renderling／rend3: このsubsystemの代替候補から外す。既存の限定oracle／比較結果だけを保持する

### ADOPTION ROUTE

M3 Stageの既存Host／device／surface／snapshot ownerを維持し、Rerun Spatial Viewerの登録・query・visualizer・
renderer系を採択する。exact crate集合、storeを使うかdirect projectionに縮めるか、device共有方法、cutoverは
current codeから一契約ずつcompileする。このdecisionだけから依存追加や実装を発注しない。

### REJECTED CANDIDATES

- Rerunをclassごとの`PATTERN`へ解体して同等機構をMotoliiで再実装するroute
- Rerun store／BlueprintをDocument、journal、Undo、selection、playheadの第二authorityにするroute
- Rerun UI全体をRN shell／rust-skia Timeline／Inspectorの代替にするroute
- 固定2灯mesh shaderをMotoliiのPBR完成として採用するroute

### THIN MOTOLII SEAM

snapshot/time/identity projection、Host callback、D2 terminal intent、Stage composite、resource admission。

### THIN MOTOLII RESIDUAL

authoring gizmo／Depth Rail、occlusion policy、faithful PBR／unlit、Preview／Export policy、hard budget。

### RETIREMENT

旧M5地図のRerun `PATTERN`のみという裁定と、それを前提にした独自spatial renderer／camera／picking再構築routeを退役する。
既存private fixtureとoracleは、採択後の適合検査へ再利用する。

### BUILD JUSTIFICATION

`NONE`。一般Spatial Viewer機構の新設は禁止する。Motolii固有residualだけを、採択routeで埋められないことを
current codeで確認した後に実装できる。

## 5. 状態と次の再入場

本変更はauthority再締結だけで、製品runtime完成、Rerun依存追加、Stage接続、PBR完成を意味しない。
次に実装を始める場合は、Rerun Spatial Viewerを一つの候補subsystemとして、現在のRN + rust-skia + wgpu
Stageへ接続する最初の一契約境界をcurrent codeから選ぶ。旧P1→P2粒列や旧`PATTERN`別実装を自動再開しない。
