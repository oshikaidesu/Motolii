# Vism A3外部表現・LayerSource lineageの価値回収（Unit 9D、2026-07-23）

状態: **処分完了**（16 historical blob、現行進捗表現の補正を含む）

対象: `vism-a3-external-expression-survey` 2版、`vism-a3d-radial-repeater-decision` 2版、`vism-a3s-layersource-lowering-spec` 12版。

関連: [A3R調査](2026-07-18-vism-a3-external-expression-survey.md)、[A3D採択](2026-07-18-vism-a3d-radial-repeater-decision.md)、[A3S lowering](2026-07-18-vism-a3s-layersource-lowering-spec.md)、[Vism計画](2026-07-17-vism-implementation-plan.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

このlineageの価値はRadial Repeater単体より、その手前で「plugin」という市場語を責任境界へ分解したことにある。

| 外部で同じplugin棚へ入る能力 | Motoliiの持ち場 | A3での処分 |
|---|---|---|
| propertyの時間関数・別値参照 | `ParamDriver`／型付きvalue source／Interpolation | A2/B2/UIへ。汎用文字列Expressionにしない |
| parameterだけから画を生成 | 0-input `LayerSource` | **A3採択・実装済み** |
| N textureのrole付き合成 | `Composite` | 入力roleとDocument graphの実証まで延期 |
| layer、camera、key等を変更 | Authoring Tool／Kit materialize／Host command | atomic batchまで延期 |
| 前frame状態を使う物理 | Simulation／StateTrack／Bake | A6へ延期 |
| 外部DCC操作・交換 | 明示import/export bridge | loss表を伴う独立spikeへ延期 |

Expressionから残すのは「parameterの文脈で値の決まり方を試し、即時previewし、固定値やkeyframeへ戻せる」連続性である。layer名やindexを文字列走査するglobal script、project open時の任意code、Document自由変更と独自Undoを標準作法へしない。

## 2. A3R二枝の処分

初版`ede4db47`は現行の簡潔な調査本文である。別枝`adcd7165`は同じ結論を保ったまま、AE Effect／AEGP、Blender Driver／Geometry Nodes／Simulation／Add-on、市場製品の責任差、bridgeのloss候補、反対側質問を詳述した。

詳述枝から独立した製品仕様は生まれていない。価値は次へ縮約して保持する。

- 市場の棚分類をHost trait分類として輸入しない。
- 専用panelの存在や製品人気を成功因果と推定しない。
- Blender Add-onをexecutor型紙にせず、将来bridgeはcamera、geometry、depth/matte、bake済みsequence等のlossを明示する。
- Parameter Panelを表現のホームにしても、path editor等の必要な大面積操作まで禁止したことにはしない。

現行A3Rは調査正本として維持し、採択がA3Dで既に完了した現在時点だけ補記した。詳述枝を丸ごと戻して外部製品の2026-07-23現在仕様を保証したことにはしない。

## 3. A3Dで採択した最小表現

`core.layer_source.radial_repeater` v1は、0-input、純関数、GPU生成の2D LayerSourceである。

- parameter閉集合は`count`、`radius`、`dot_radius`、`phase`、`angular_speed`、`color`の6件。
- `count`はinteger `1..=64`だが、このplugin versionのexpression domainでありHost全球上限ではない。
- 正準空間は原点中央、Y-up、高さ1.0。phase 0は+X、正angular speedはCCW。
- 円群はSDFのunionとしてcoverageを一度だけ掛け、重なりalphaを加算しない。
- straight sRGB入力を`[r*a*C, g*a*C, b*a*C, a*C]`へpremultiplyする。
- 同一`(t, params, FrameDesc)`は再生順やwall clockに依存せず同一結果を返す。

不採択の`seed`、Particle identity、path、Composite、physics、Expression runtime、Blender bridge、custom UIを同じv1へ戻さない。A3Dの後続版`24281dac`はSlint停止表現をU4a未着手へ更新しただけで、意味変更ではない。

## 4. A3S十二版の進行と現在の契約

| blob段階 | 追加された事実 | 判定 |
|---|---|---|
| `167d8f1d` | A3Dの9質問を閉じた初期lowering仕様 | 現行契約の基礎 |
| `46cbbdff` | A3-1a prepared LayerSource配線完了 | 実装進捗 |
| `4024ee26` | A3-1b built-in rect reserved ID拒否完了 | 実装進捗 |
| `fd40b59d` | A3-1c ID allowlistなし一般lowering完了 | 実装進捗 |
| `79679f0d` | 0-input+uniform64 Host cache欠落を発見しA3-0を先行挿入 | **重要な停止・再分割記録** |
| `0c8b1d6b` | A3-0固定cache定型完了 | 実装進捗 |
| `b4f491c9` | 外部Radial Repeater crate／first-party登録完了 | 実装進捗 |
| `2340c8f7` | 独立CPU oracle、union/premul/zero-size負例完了 | 実装進捗 |
| `acc95164` | 非UI Contract列挙を含むA3全件完了 | 実装完了 |
| `987a6371` | 古い「現状GAP」を解消記録へ訂正 | 時点補正 |
| `7fd415b1` | UI gateをSlintからegui入場待ちへ更新 | 歴史的進捗 |
| `0d746a32` | egui U0a入場済み、U4a個別依存へ更新 | 現行版 |

現在の拘束は次である。

1. `GraphBuilder`は`PreparedDocumentPlugins`を受け、catalog-backed LayerSourceのprepared paramsだけを評価する。raw recipe、revision、Undoを変更しない。
2. 登録済み0-input LayerSourceをexecutor kindで一般loweringし、`clear`や新IDのallowlistを置かない。
3. `doc.layer_source.rect`はDocument built-inのままで、catalog／registry登録をtyped拒否する。
4. contract欠落、executor欠落、kind不一致、future versionをA0Sの型付き分類で拒否する。
5. 0-input shader用cacheはbinding 0の`[f32;16]`に閉じた定型で、generic raw allocation APIへ拡張しない。
6. Contract列挙はU4aへのhandoffに足りるが、A3完了を製品Parameter Panel適合完了と呼ばない。

現行A3Dに実装完了を反映し、A3Sの`ORDER: READY`と未充足形の統合gateを完了状態へ補正した。仕様化ticket当時の非目標と検証記録は歴史証拠として残す。

## 5. 復活させない実装短絡

- `core.layer_source.clear`またはRadial RepeaterのID分岐を増やして一般loweringを迂回しない。
- raw `clip.params`をcatalog-backed LayerSourceへ渡さない。
- built-in rectを見かけ上registry pluginへ変換しない。
- LayerSourceへ架空の`Quality`、CPU readback、別preview経路を追加しない。
- Filter用texture+sampler+uniform4 cacheを0-input LayerSourceへ流用しない。
- A3-0からgeneric bind layout、任意uniform size、compute抽象、raw GPU resource APIを公開しない。
- UI不足を`ParamDef`の未決field、ID特例、Document schemaで先焼きしない。
- A3実装済みをVism package、third-party install/load、Kit、U4a完成の証明に使わない。

## 6. 固定歴史出典とcoverage

各pathの初版を全文で読み、A3Rの詳述枝、A3DのUI時点差分、A3Sの全11遷移を確認した。同一内容を別branchへ載せたcommitはblob単位で重複countしていない。

処分した16 blobの完全SHAは`evidence/historical-value-recovery/disposition-receipts/09d-vism-a3-expression-layersource.tsv`を正本とする。cutoff総数1,797のうち処分済みは217、未処分は1,580である。install/load/trust残余はUnit 9Eへ残し、本Unitの完了でVism全歴史完了とはしない。
