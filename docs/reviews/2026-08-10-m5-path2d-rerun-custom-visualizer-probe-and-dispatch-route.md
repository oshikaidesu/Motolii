# M5 Path2D Rerun custom visualizer probeと発注route

状態: **決定**（2026-08-10）

## 1. 利用者成果

M5の最初の可視成果は、M3 Stageを主役surfaceに保ったまま、同じ`z=0`平面へ2D図形を置けることとする。
最初の閉じた図形はRectとCircle、合成は明示`draw_order`によるpremultiplied source-overとする。
Rect／Circleは別々のrenderer意味を持たず、作者向けparameter recipeから既存
`motolii_doc::pathgeom::Path`へlowerする。後続のSVG shape path編集、Vism `Path2D → Path2D`、
複雑pathもこの共通Path2Dを消費する。

RerunはSpatial2D view、visualizer query、camera、wgpu draw phase、picking、outlineを所有する。
Motoliiはshape recipe、Path2D、fill色、draw order、Document／D2／Undo、Preview／Export意味を所有する。
Rerun store、Blob、Blueprintを第二Documentまたは公開Vism schemaにしない。

## 2. 既知実装preflight

| 項目 | 裁定 |
|---|---|
| MECHANISM CLASS | Rerun Spatial2D内のfilled Path2D表示 |
| KNOWN IMPLEMENTATION SEARCH | 固定Rerun `954bf95a`の`Points2D`、`Boxes2D`、`Ellipses2D`、`custom_visualizer`、`re_renderer` transparent phaseと、現行`pathgeom::Path`／Vello依存を実コード照合 |
| CANDIDATES | built-in primitive、custom archetype＋visualizer＋renderer、Vello scene直結 |
| ADOPTION ROUTE | custom visualizerを`ADOPT / WRAP`し、Rect／Circleを共通Pathへlowerする。複雑pathのtessellationは既存Vello系を再選定する |
| REJECTED CANDIDATES | `Boxes2D`／`Ellipses2D`はoutlineで共通filled pathにならない。`Points2D`の円だけでRectと任意pathを別機構へ分裂させない。probe段階でVello scene bridgeを新設しない |
| THIN MOTOLII SEAM | Path2D payload投影、fill色、draw order、semantic identity mapping |
| THIN MOTOLII RESIDUAL | authoring recipe、D2、Undo、Preview／Export、複雑path／hole、resource admission |
| RETIREMENT | primitive別renderer、probe Blob codec、convex fan、view-fit用透明Points2Dは製品へ昇格しない |
| BUILD JUSTIFICATION | NONE。固定Rerun custom visualizerと現行Pathを採択する |

## 3. 成立したproof

[private probe](../../spikes/rerun-path2d-probe/README.md)は次を実行する。

- Rectと4-cubic Circleを既存`pathgeom::Path`へlowerする
- probe内だけのBlob payloadをcustom archetypeからvisualizerへ渡す
- Rerun `Spatial2DView`のcustom rendererで両方を`z=0`へ描く
- Circleの`draw_order=1`をRectの`draw_order=0`より後にsource-over合成する
- picking layerとoutline maskのRerun draw phaseを保持する

実画面は[rerun-path2d-z0-overlap.png](../../spikes/rerun-path2d-probe/rerun-path2d-z0-overlap.png)。
これはRerun上の可視proofであり、RN製品Stage、Document、Vism公開SDK、Preview／Export接続の完成ではない。

既知の限界は次の通り。

1. tessellationはRect／Circleだけを対象にしたconvex triangle fanで、concave contour、hole、strokeを扱わない。
2. payload codecとview-fit用透明`Points2D`はprobe器具であり、恒久形式でもruntime projection設計でもない。
3. 頂点／index／uniformをshape追加ごとに作る。製品のresource reuse／ResourceLedger接続を証明しない。
4. 固定Rerunを`native_viewer`付きgit dependencyにすると初回解決が約976 packageとなる。製品依存はexact crate閉包を再選定する。
5. Rerun workspace外ではexample shaderが自動梱包されないため、probeはWGSLをRerunのvirtual filesystemへ明示登録する。

## 4. 発注順

### M5-PATH2D-P0 — private proof

`DONE / PROBE ONLY`。本変更がownerであり、製品台帳の完成数へ加えない。

### M5-PATH2D-S1 — product Stage seat compile

状態は`ISSUE`。次の一契約は図形機能の追加ではなく、現行RN StageへRerun Spatial2D subsystemを載せる
exact seatのcompileである。次のcapsuleを満たすまでは実装担当を起動しない。

- **BASE**: 本変更がmain到達したcommit
- **AUTHORITY**: 本決定、M5 Rerun再締結、M5仕様、UI runtime責任境界
- **CURRENT STATE**: P0はRerun別windowで成立。RN `renderer_core.rs`は独自wgpu Stageを所有し、Rerunは未依存
- **OWNER**: M3 Stage Host。Rerun store／Blueprintはruntime projection readerに限定
- **EXACT TARGET**: `ui/motolii-rn/native-renderer/src/renderer_core.rs`の既存Stage render passと、同crateのdevice／queue／surface lifetime。別device／別surfaceを作らずRerun draw outputを同じStageへ渡せる一つのcall siteを特定する
- **ALLOWLIST**: 最初のdocs／compile粒では本決定、M5採択地図、実装台帳だけ。code allowlistはexact call siteとCargo feature閉包を返した次waveで固定する
- **READ SET**: `renderer_core.rs`、同crate `Cargo.toml`／`Cargo.lock`、P0の5 Rust／1 WGSL、固定Rerun `custom_visualizer` example、`re_view_spatial`／`re_renderer`の公開re-exportだけ
- **POSITIVE ORACLE**: 一つのRN Stage surfaceでRect＋Circleがz=0に表示され、重なりpixelがP0 source-over oracleと一致する
- **NEGATIVE ORACLE**: Rerun未使用Stage画素不変、wgpu device／surface生成数が増えない、Document write 0、frame内pipeline／shader生成0
- **NON-GOALS**: SVG editor、concave／hole／stroke、public Path2D schema、GLB、Depth Rail、Group Depth、Preview／Export完成、Rerun UI全体の埋め込み
- **RETURN**: `RESEARCH_RETURN`としてexact call site、必要crate／feature、device共有可否、採否、不適合理由、最小代替edgeを返す。`native_viewer`一括依存または第二surfaceしか成立しない場合は実装しない

### M5-PATH2D-S2 — product Path2D projection

`WAIT`。S1が同一Stage／device seatを閉じた後だけ、既存`pathgeom::Path`からruntime projectionを渡す。
この粒でprobe Blob codecとconvex fanを退役し、複雑path要件が入る時点で既存Vello tessellationを再選定する。
Document／Vism公開schemaは別decisionなしに変更しない。

## 5. 完了判定

今回閉じたのは`P0: Rerun custom visualizerでz=0のRect／Circle／overlapを表示できる`まで。
M5製品runtimeは未接続であり、次に発注可能なのはS1のseat compileだけである。S1 return後は古いP1→P2列へ
戻らず、current codeからS2または別の最小接続edgeを再選定する。
