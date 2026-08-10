# M3／M5 Stage Heroとprojection rootのPR前決定

- 日付: 2026-08-11
- 状態: **決定 / PR前の根本整理**
- 対象: M3 RN製品route、M5 Rerun Spatial Viewer、Stage／Timeline／Inspector、VS-1 Rectangle

## 1. 結論

M3の主役surfaceをStageとし、M5から採択したRerun Spatial ViewerをそのHero consumerとする。ここでHeroとは、作品意味を最も豊かに現像し、後続surfaceへ必要な投影を具体化する第一consumerを指す。M5やRerunをDocument、Undo、selection、playheadのownerにする語ではない。

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
M5-derived Stage Hero   bounded side projections
Rerun spatial/render     Timeline = time/location
                        Inspector = meaning/control
```

## 2. Authorityとtranslation root

永続意味の正本は`motolii-doc::Document`、変更ownerはD2 single writer、Undo/Redoは既存Command／journal routeのままとする。Rerun entity path、RN component state、Timeline bar、Inspector fieldを正本にしない。

翻訳根は公開schemaを新設せず、現行の次の値を同じaccepted generationとして扱う論理envelopeである。

- `PublishedDocument.snapshot: Arc<Document>`
- Document `revision`
- `primary: Option<LayerId>`
- `projection_generation`
- `RnProductHost.current_time`相当のHost-owned evaluation time

現行では時刻が`PublishedDocument`のfieldではないため、この決定から新しい共通structやwire schemaを発明しない。最初のPR compileでexisting ownerとcall siteを数え、同一generationとして渡せる最小private seamを選ぶ。

## 3. surface別の意味

| surface | 同じidentityから投影する意味 | 所有しないもの |
|---|---|---|
| Stage Hero | 評価済み空間、shape、transform、style、layer order、selection用spatial address。Rerun entity pathは`LayerId`から導出する表示address | Document、history、selection正本、独自playhead |
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
| Stage編集幾何 | `ClipSource::Vector`をtyped `Unavailable`にする | 既存plugin Rectの幾何routeを無断流用せず、exact gapとして維持 |
| Rerun Stage | Path2D custom visualizerと固定path表示はprobe済み | accepted Document snapshot／time／LayerIdからの製品projectionが残る |

Draft PR #470の固定5点pathはRerun表示機構のprobeであり、このtranslation rootを通らない。M3 Rectangle完成、Timeline／Inspectorの意味owner、Undo/Redo接続の証拠にはしない。

## 5. PRへcompileする順序

最初にcompileする候補は、VS-1 Rectangleのaccepted snapshot／time／`LayerId`を既存Rerun Stage入力へ写す一契約である。IssueやPRはまだ開かない。次をcurrent mainで閉じてから`READY_TO_OPEN`へ上げる。

1. `StandardShape::Rect`の製品call siteと`#[cfg(test)]`境界を数える。
2. 既存Vector Rect→`OverlayRect` loweringとRerun Path2D visualizerの採否を比較する。
3. exact input、output、owner、entity address導出、stale rejection、failure returnを固定する。
4. `R1-GPU-BINDING`の残余とwrite setの交差を測り、第二device／surface／rendererを負例にする。
5. automated oracleを、same `LayerId`／revision／time、固定bool 0、Document write 0として閉じる。
6. 実RN StageでBrowser Place後にRectangleが見えることをexternal visual gateとする。

このStage契約の入力が固定された後、file allowlistが交差しないTimeline product projectionとInspector bounded read projectionは並列化できる。Undo/Redoで三面から消失／復帰する`R1-E2E`は実装PRでなく、各consumer着地後の統合受入とする。

## 6. 禁止するまとめ方

- 全surface用の巨大な`SceneObject`、汎用projection framework、第二scene graphを先に作る。
- Rerun store／Blueprint／entity pathをDocument、Undo、selection、playheadのauthorityにする。
- TimelineやInspectorからRerun entityを逆引きして製品意味を作る。
- Stage Heroを理由にTimeline／Inspector／D2を一PRへ束ねる。
- #470のbool／固定座標を製品snapshot接続へ拡張し続ける。
- Circle、overlap、Path editをRectangleのaccepted snapshot接続より先に製品発注する。

## 7. 発注前handoff

PR候補は[M3-R1-STAGE-RECTANGLE](../pr-seat-candidate-catalog.md#現在の候補)の一行へ戻す。closed orderには`BASE / AUTHORITY / CURRENT STATE / OWNER / EXACT TARGET / ALLOWLIST / READ SET / POSITIVE AND NEGATIVE ORACLES / NON-GOALS / RETURN`を揃える。exact targetが複数ownerへ割れる場合は一発でまとめず、最初の見えるStage出口を保ったまま一契約へ`REDUCE`する。

