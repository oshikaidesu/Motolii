# M3／M5 Stage Heroとprojection rootのPR前決定

- 日付: 2026-08-11
- 状態: **決定 / PR前の根本整理**
- 対象: M3 RN製品route、M5 Rerun Spatial Viewer、Stage／Timeline／Inspector、VS-1 Rectangle

## 1. 結論

M3の主役surfaceをStageとし、固定Rerun Spatial Viewerをそのspatial runtimeとする。MotoliiはRerunを包むcreator-facing wrapperであり、作品意味をDocument／D2からRerun入力へ翻訳する。M5やRerunをDocument、Undo、selection、playheadのownerにはしないが、Rerunのscene／view／query／visualizer／camera／picking／rendererをMotolii内で作り直さない。

従来はStageの製品像が弱く、Rectangleの意味がDocument、render graph、Timeline、Inspector、probe rendererへ局所的に現れたまま、どの製品出口へ揃えるかが見えにくかった。以後は次の向きで一つずつ回収する。

```text
Document + D2 single writer + Undo/Redo
                  |
                  v
accepted snapshot / revision / primary LayerId
projection generation / Host-owned evaluation time
                  |
          +-------+-------+
          |               |
          v               v
Rerun Spatial Viewer    bounded side projections
Stage spatial runtime    Timeline = time/location
                        Inspector = meaning/control
```

## 2. Authorityとtranslation root

永続意味の正本は`motolii-doc::Document`、変更ownerはD2 single writer、Undo/Redoは既存Command／journal routeのままとする。Rerun entity path、RN component state、Timeline bar、Inspector fieldを正本にしない。

翻訳根は公開schemaやshape別frameを新設せず、現行の次の値を同じaccepted generationとしてRerunの既存入力へ渡す論理envelopeである。

- `PublishedDocument.snapshot: Arc<Document>`
- Document `revision`
- `primary: Option<LayerId>`
- `projection_generation`
- `RnProductHost.current_time`相当のHost-owned evaluation time

現行では時刻が`PublishedDocument`のfieldではないため、この決定から新しい共通structやwire schemaを発明しない。最初のPR compileでexisting ownerとcall siteを数え、同一generationとして渡せる最小private seamを選ぶ。

## 3. surface別の意味

| surface | 同じidentityから投影する意味 | 所有しないもの |
|---|---|---|
| Stage Hero | Rerun Spatial Viewerが評価・表示する空間、shape、transform、style、layer order、selection用spatial address。Rerun entity pathは`LayerId`から導出する表示address | Document、history、selection正本、独自playhead、第二spatial engine |
| Timeline | Clip interval、track／band、key、現在時刻。Stageで見える対象の時間上の所在 | shape schema、spatial scene、Document clone |
| Inspector | primary対象のtype、既存parameter、diagnostic、利用可能なtyped operation | mock state、汎用parameter framework、直接Document write |

三surfaceは同じrevision／identity／evaluation timeを読むが、同時pixel更新barrierを作らない。各consumerがstale generationを拒否する。surface別snapshot、selection store、history、Rerun storeを第二authorityとして持たない。

## 4. Rectangleで確認できる現在地

| edge | current main | 処分 |
|---|---|---|
| Browser→D2 Place→Undo/Redo | `PlaceRectangleRequest`から`Command::AddTrackItem`へ接続済み | `REUSE / LANDED` |
| 永続shape | `LayerId`付き`ClipSource::Vector / VectorRecipe / StandardShape::Rect` | `REUSE / LANDED` |
| Preview／Export評価 | Vector Rectを不透明白`OverlayRect`へlower済み | `REUSE / LANDED` |
| Timeline | Clipから`TimelineBar { layer, start, end, band }`へ投影済み | `REUSE`。RN製品component接続は別edge |
| RN Inspector | revision、time、primary、`layer_id / display_name`まで | Rect width／height／positionのbounded projectionが残る |
| Stage編集幾何 | modifierなしの`StandardShape::Rect`を同じ`LayerId`／size／world／cameraで投影 | `REUSE / LANDED`。他のVectorとmodifier付きRectはtyped `Unavailable`を維持 |
| Rerun Stage | Path2D custom visualizerと固定path表示はprobe済み | accepted Document snapshot／time／LayerIdからの製品projectionが残る |

Draft PR #470の固定5点pathはRerun表示機構のprobeであり、このtranslation rootを通らない。M3 Rectangle完成、Timeline／Inspectorの意味owner、Undo/Redo接続の証拠にはしない。

## 5. PRへcompileする順序

最初の契約は、VS-1 Rectangleを特別扱いせず、accepted snapshot／time／`LayerId`をRerun Spatial Viewerの既存入力へ翻訳する一本である。

1. Documentの評価結果をRerun entity／component入力へ写す。
2. scene query、View、visualizer、camera、picking、drawはRerunへ任せる。
3. 結果を`ui/motolii-rn/`の既存Stageへ載せ、確定操作だけD2へ戻す。

自動oracleはsame `LayerId`／revision／time、stale拒否、Document write 0。外部gateはBrowser Place後に同じRN StageでRectangleが見えることとする。これが通ればartifactをcopyせず`PRODUCT_SOURCE`へ繰り上げる。Rerun subsystem自体の再比較や、固定fixtureを意味authorityへ昇格することはcompile項目にしない。

このStage契約の入力が固定された後、file allowlistが交差しないTimeline product projectionとInspector bounded read projectionは並列化できる。Undo/Redoで三面から消失／復帰する`R1-E2E`は実装PRでなく、各consumer着地後の統合受入とする。

## 6. 禁止するまとめ方

- 全surface用の巨大な`SceneObject`、汎用projection framework、第二scene graphを先に作る。
- Rerun store／Blueprint／entity pathをDocument、Undo、selection、playheadのauthorityにする。
- TimelineやInspectorからRerun entityを逆引きして製品意味を作る。
- Stage Heroを理由にTimeline／Inspector／D2を一PRへ束ねる。
- #470のbool／固定座標を製品snapshot接続へ拡張し続ける。
- `RerunStageFrame`等のshape別中間scene、direct `re_renderer` draw route、第二camera／pickingを作る。
- probeの`encode_rerun_stage_shapes`をDocument意味の正本にする。artifact内の一時fixtureとしては、実入力に置換されるまで保持できる。
- Circle、overlap、Path editをRectangleのaccepted snapshot接続より先に製品発注する。

## 7. 発注前handoff

PR候補は[M3-R1-STAGE-RECTANGLE](../pr-seat-candidate-catalog.md#現在の候補)の一行へ戻す。closed orderには`BASE / AUTHORITY / CURRENT STATE / OWNER / EXACT TARGET / ALLOWLIST / READ SET / POSITIVE AND NEGATIVE ORACLES / NON-GOALS / RETURN`を揃える。exact targetが複数ownerへ割れる場合は一発でまとめず、最初の見えるStage出口を保ったまま一契約へ`REDUCE`する。
