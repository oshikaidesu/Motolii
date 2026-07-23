# M2 planar camera契約lineageの価値回収（Unit 4I、2026-07-23）

状態: **観察**（cutoff 5 historical blobの処分完了）

対象: `docs/reviews/2026-07-16-m2-comp-camera-decision.md`のcutoff全3版と、`docs/reviews/2026-07-18-d1k-runtime-camera-thaw-spec.md`のcutoff全2版。

関連: [planar camera決定](2026-07-16-m2-comp-camera-decision.md)、[runtime解凍記録](2026-07-18-d1k-runtime-camera-thaw-spec.md)、[Stage / Output Frame設計](2026-07-14-unified-stage-camera-design.md)、[M2仕様](../specs/M2-document-model.md)、[M3仕様](../specs/M3-ui-integration.md)、[M5仕様](../specs/M5-3d-and-post.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

このlineageは、single cameraを「UIの実装方式」や「core pluginの一種」ではなく、作品の出力意味として採択した記録である。混同を避けるため、現在地を三層へ分ける。

| 層 | 所有する意味 | 2026-07-23現在 |
|---|---|---|
| Document / semantic core | 全Compositionにcameraがちょうど1つ、2Dは同じXYZ世界の`z=0`、Output Frameはcamera aperture、aspectはComposition所有 | **決定・実装済み**。永続variantは`PlanarOrthographic`のみ |
| runtime / render | radians、orthographic可視高、必須camera入力、exact aspect、Document→GPU、preview/export共通経路 | **D1k/D3f実装・審判済み** |
| product UI | Stage View、Output Frame、枠外Draft、Camera tool、直接操作 | **未実装**。U1fは`BLOCKED`、U2dはU1f依存 |

native windowかReact windowかはproduct UI runtimeの選択であり、cameraのDocument意味を変えない。core／first-party／third-partyの供給元分類も別軸である。cameraはpluginではなく、Hostと全pluginが共有する作品・評価文脈である。

## 2. 採択された最小恒久面

- `Composition.camera`は常在し、camera layer、group camera、shot切替、第二preview cameraを持たない。
- `CompCameraDoc::PlanarOrthographic`は`center`、`roll_radians`、`height`だけを永続化する。px、DPI、viewport、degree表示、near/far、overscanは保存しない。
- output aspectはcameraへ重複保存せず、Compositionの正の有理数を使う。`FrameDesc`との不一致はtyped rejectで、stretch/crop/letterboxへ黙って変えない。
- default cameraは既存2D出力のidentityで、旧project migrationとCAM-G0 exact oracleがpixel不変を固定する。
- Stage Viewのpan/zoom/fitはDocument外、Output Frame操作はDocument cameraへのD2 commandで、preview/exportへ影響する。
- 2Dと将来3Dで別world、別transform、別camera導線を作らない。ただし現在のruntimeが証明するのはplanar投影までである。

この意味でsingle cameraと「2.5D的に同じ世界へ広げる」という発想は現行設計へ一般化されている。一方、Perspective、depth occlusion、Spatial poseまで実装済みという意味ではない。

## 3. 二つのlineageで起きた訂正

### 3.1 decision 3版

初版は一つのworld/cameraを採択しながら、恒久schemaをplanar一variantへ絞った。次版はCAM-G0を、変更可能なtest harnessと、保護されるreviewable TSV semantic oracleへ分離した。これによりAPI配線の修理と期待pixelの改変を同一ファイルへ閉じ込めない。

最終版は、古い「M3停止中」「Slintを漏らさない」という時点表現を、U0a入場後のtask依存とtoolkit非依存へ更新した。これはcamera意味の変更ではなく、UI入場状態とtoolkit選択を恒久契約から外した訂正である。

### 3.2 runtime thaw 2版

初版は旧`position + target + fov_y_degrees`、serde、`DEFAULT`、暗黙cameraを撤去し、planar `CompCamera`、typed error、必須render camera、元`FrameDesc`での事前aspect検証を凍結した。またD1kがDocument camera評価まで先取りせず、D3fへ直列化した。

次版はD1k到達を記録し、`world_to_ndc`の浮動小数点評価順序を訂正した。`height * aspect`や巨大比の中間overflowで数学的に有限な結果を誤拒否しないよう、分母を先に適用する負例を追加した。仕様の数式が同値でも、実装評価順序まで審判が必要だという回収価値である。

## 4. 現行コードとの照合

現行コードは次を保持している。

- `CompCameraDoc`はinternally taggedな`PlanarOrthographic`一variantで、Compositionに必須fieldとして存在する。
- runtime `CompCamera`のfieldはprivateで、唯一の公開constructorは`try_new`。degree/Perspective、target、serde、`Default`は無い。
- `world_to_ndc`は分母側を先に適用し、大きなaspect比と中間overflowの回帰testを持つ。
- 公開render request／graph入力は`CompCamera`を必須で持ち、元descriptorとのaspect一致をGPU処理前に検査する。
- D3fはDocument cameraを時刻`t`で評価し、非既定cameraを旧viewport transformと二重適用しない。defaultをbit一致でのみskipする。
- CAM-G0、current/migrated Document、非既定camera、preview/export同一経路の審判が存在する。

同時に、[implementation ledger](../implementation-ledger.md)ではU1fが`BLOCKED`である。したがって、planar camera基盤の完成をStage/Output Frame実UIの完成へ読み替えない。M5-P3の`Spatial`追加もdecision/schema/runtime統合待ちである。

## 5. 再利用する設計原則

- 一つの作品意味を、Document、runtime、product UIの別完了条件へ分ける。
- output raster、編集view、projection、poseを別所有にし、UI viewportやpxから永続cameraを逆算しない。
- 将来拡張は既存variantの再解釈ではなく、新variantとversion/min-reader上昇で追加する。
- schema、runtime thaw、Document接続、UI操作を一PRへ束ねない。
- default identityの互換性は、旧経路が権威のうちにsemantic oracleへ固定する。
- test harnessはAPI変更へ追随できるよう保護oracleから分け、期待値だけを不変にする。
- 数学式の等価性だけでなく、巨大値での浮動小数点評価順序を正例・負例で固定する。
- optional cameraや暗黙defaultをrender入口へ戻さず、全経路で同じ明示cameraを運ぶ。
- previewとexportの差をQualityだけにし、別cameraや別変換を作らない。

## 6. Spatial/Perspectiveの再入場条件

M5で`CompCameraDoc::Spatial`を追加する前に、orientation表現と補間、handedness／local axes／transform順、Perspective/Orthographic projection、clip policy、target constraint特異点、Planarとの切替・migrationをdecision PRで固定する。既存`center`、`roll_radians`、`height`を再解釈せず、`position + target + implicit world-up`をpose保存へ戻さない。

再入場の審判は、Spatial camera animation、Z平面のparallax、既存Planar project/pixel不変、preview/export一致、camera layer／複数cameraが構文不能であることを含む。現在のsingle-camera思想はこの将来variantを許すが、その具体shapeを先取りしていない。

## 7. 復活させないもの

- 旧core型の`position`、`target`、degree FOV、暗黙world-up、serde、`DEFAULT`をadapterとして戻すこと。
- `Option<CompCamera>`やcamera無しrender entryを作り、Host側でidentityを黙って補うこと。
- `PlanarOrthographic`へPerspectiveやSpatial poseをfield追加し、既存variantの意味を変えること。
- Stage View transform、logical/physical px、DPI、UI degreeをDocumentへ保存すること。
- Output Frameとcameraを別々の永続transformにすること。
- 2D用と3D用のworld／camera／transformを分けること。
- camera layer、group camera、shot切替、別preview cameraをv1/M5既定へ持ち込むこと。
- `FrameDesc`不一致をepsilon、stretch、crop、letterboxで黙って吸収すること。
- camera基盤完了をU1f/U2dやM5-P3の完了として数えること。
- native／Reactという表示技術やfirst／third-party分類からcamera意味を決めること。

## 8. 固定歴史出典とcoverage

camera decision初版`22591362`とruntime thaw初版`d300e4aa`を全文で読み、差分`22591362..0da61253`、`0da61253..c6f4ce34`、`d300e4aa..082dcbdc`を確認した。処分した5 unique blob（67,137 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04i-m2-camera-contracts.tsv`を正本とする。cutoff総数1,797のうち処分済みは319、未処分は1,478である。
