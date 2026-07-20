# M3 / Rerun 実装後半発覚プレモーテム（2026-07-20）

状態: **決定／実装前ガード**。M3初回UシリーズとRerun転移で、実装担当が着手後に
初めて気づくと手戻りが大きい接合問題を、現行仕様・コード・固定Rerun sourceから
先回りして処分する。本書は新しい公開API、Document field、plugin契約、Rerun依存を
許可しない。製品意味は[M3仕様](../specs/M3-ui-integration.md)、Rerun転移権限は
[学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)が優先する。

## 1. 監査した現在地

- mainではU0a、U0b-1、U0b-2まで完了し、初回Uシリーズは1枝番ずつ直列実行する。
- `motolii-ui`の製品依存はegui 0.35 / egui-wgpu 0.35 / egui_tiles 0.16 /
  wgpu 29へ統一済みだが、製品shell、native texture preview、panel投影は未実装である。
- Rerunは固定main commit `954bf95a`と安定release 0.34.1を監査済みである。
  `re_ui`一括`DEPEND`は棄却候補で、個別assetの最終転移裁定は未完了である。
- 旧night 3分岐の並行統合は廃止され、U0b→入力→gesture→shell→worker→編集E2Eの
  直列順へ変更済みである。

## 2. 後半発覚候補と処分

| ID | 後から発覚しやすい事象 | 早期証拠 | 処分 |
|---|---|---|---|
| LD-1 | 複数の初期U枝番が同じ`motolii-ui`と意味境界を並行変更し、どの差分が正本か分からなくなる | 旧nightのU0b/U0e/U1aが同じcrate・仕様入口へ触れていた | **解決済み**。初回Uシリーズはファイル競合の有無にかかわらず1枝番ずつ直列実行し、旧night差分は証拠・抽出元に限定する |
| LD-2 | React fixture、Rust fixture、snapshotが別々の入力を持ち、見た目が一致しても別の製品状態を検査する | Reactは安定IDを持つが、DOM/CSSは製品契約でなく、U0e-2の共有fixture所有が未明示だった | **決定**。1審判につきtoolkit非依存のfixture manifestを一つだけ正とし、content ID・component安定ID・state・locale・viewport条件を持つ。React/eguiはadapterで同じmanifestを読む。DOM、CSS、snapshot、Rerun fixtureから意味を逆生成しない |
| LD-3 | immediate modeの初回／2 frame layoutとfont loadingによりsnapshotが揺れ、期待画像の更新で隠される | Rerun `list_item::scope`は前frame統計を次frameの列揃えに使う。OS font fallbackは環境差を持つ | **決定**。component snapshotは有界なlayout安定化手順を持ち、連続するlayout hashが一致する前のframeを採択しない。試験は再配布可能なtest-only fontを固定し、製品CJK font決定とは分離する。安定化しない場合はsnapshot更新で通さず失敗する |
| LD-4 | native texture表示で色変換、sRGB解釈、premultiplied alphaをUI側が重ね、Exportと見た目がずれる | 現行は`Rgba8Unorm` viewをeguiへ登録する方針だが、静止表示の色・alpha負例がU1a-1枝番に明示されていなかった | **決定**。UIはrendererが作った表示textureを一度sampleするだけで色変換・premultiply/unpremultiplyを追加しない。U1a-1にopaque、半透明edge、color rampの固定fixtureを置き、double transform、double premultiply、channel swapを拒否する |
| LD-5 | resize、pool再作成、panel detach時に`TextureId`とtexture slotの寿命がずれ、tearing、古いframe、GPU resource leakが後から出る | 「一度だけnative texture登録」は既決だが、retire/unregisterとsampling中slot再利用の審判が不足していた | **決定**。display slotはUI frameが参照中に再利用しない。登録はpool生成時一回、解除はpool retirement時だけとし、frame更新や通常resizeで登録し直さない。U1a-1でresize/minimize/restoreとpool交代を反復し、古いextent表示、同時slot書込、resource数単調増加を拒否する |
| LD-6 | panel名、翻訳文、挿入順からruntime IDを作り、rename／locale変更でlayout・active tab・selectionが失われる | Rerun BlueprintとReact stable IDは参考になるが、U1a-2のID由来が未明示だった | **決定**。panel/runtime投影のidentityはMotolii所有の安定catalog IDから作る。表示名、locale、並び順、`egui_tiles::TileId`をidentityにしない。同じlayout modelの再投影、表示名変更、locale変更でactive tabとsplit関係が不変であることをU1a-2で固定する |
| LD-7 | generationが新しくても、旧pool extent／旧request条件の結果を表示してしまう | U1b-2は完了順反転だけを要求していた | **決定**。generationは時刻だけでなく一つの完全なrender requestを識別する。UIはgenerationからsize、quality、document snapshot等を推測して混成せず、結果に対応するdisplay-pool generationが現行でない場合も破棄する。extent変更を挟んだ逆順完了をU1b-2の負例へ加える |
| LD-8 | Rerunのdensity表示は成立しても、semantic zoom境界で個別keyのhit targetと選択が消える | Rerun Time Panelは閲覧面で、Motoliiの編集hit-testを証明しない | **決定**。U3aは遠景density・中景cluster・近景individualを同じ時間rangeとstable ID projectionから作る。zoom境界前後で選択identity、playhead、visible rangeを保ち、density pixelをDocument object identityに使わない |
| LD-9 | Rerun調査全体がUチケットの前提に見え、無関係なRRレーン待ちで実装が停止する。または逆に、未裁定assetを「参考」として混入する | RR-1〜8は調査routeであり、個別Uタスクとの必要範囲が曖昧に読めた | **決定**。Rerun転移はUチケットごとのjust-in-time packetとする。Rerunを使わない発注はMotolii仕様とoracleだけで閉じ、RR全完了を待たない。Rerunを一度でも根拠・再利用箇所・変更案に含める場合だけ、対象file/API単位の6ラベルと転移裁定を必須にする |
| LD-10 | Rerun由来の数十行を移植した後で、license・由来・上流版・依存closureが追跡不能になる | `re_ui`はcode、font、iconでlicense面が異なり、mainと0.34.1の差分も大きい | **決定**。`DEPEND/VENDOR/PORT`の差分は監査commit、対象file/API、license、改変範囲、依存差分を同じPRへ記録する。由来記録なしの転記と、未監査font/icon/shaderの便乗をCI/レビューで拒否する |

## 3. fixtureの単一正本

LD-2を満たすfixture manifestは製品公開形式ではない。保存場所やcodecを本書では固定しないが、
一つの審判内で次を満たす。

1. fixture自体がMotoliiの意味を持ち、React DOM、egui widget、Rerun型を含まない。
2. `fixture_id`、内容のversionまたはhash、安定component ID、state、locale、viewport条件を
   adapter入力として列挙できる。
3. Reactとeguiの出力には、使用した同じfixture identityを証跡へ残す。
4. adapterが未対応stateを黙ってdefaultへ落とさず、型付きまたは検査可能な失敗にする。
5. snapshotはmanifestの意味を置換しない。画像更新だけでfixture変更を隠さない。

## 4. GPU表示境界の負例

U1a-1/U1b-2へ次の負例を割り当てる。

- 半透明edgeで二重premultiplyまたはunpremultiplyが起きる
- color rampへUI側の追加色変換がかかる
- frame更新ごとにnative texture登録数が増える
- UIがsample中のslotをworkerが次frame用に上書きする
- resize後に旧extentのresultが新poolへ表示される
- pool retirement後も`TextureId`またはGPU resource数が単調増加する
- minimize中のzero-size surfaceを通常frameとして扱う

試験oracleのためのreadbackはtest境界に限ってよい。製品のpreview経路へCPU pixel bridgeを
追加してはならない。

## 5. 実装順への反映

本監査は新しい横断タスク列を増やさない。既存枝番へ次のように吸収する。

| 枝番 | 追加する審判 |
|---|---|
| U0e-2 | 単一fixture manifest、React/egui adapter同一identity、test-only font、layout安定化 |
| U1a-1 | 色・alpha負例、native texture/pool寿命、resize/minimize/restoreのresource一定性 |
| U1a-2 | stable panel catalog ID、rename/locale後のlayout identity不変 |
| U1b-2 | extent/pool generationを跨ぐ逆順完了の破棄 |
| U3a | semantic zoom境界でselection/playhead/range不変、density pixelのidentity化拒否 |
| 全Rerun参照枝番 | just-in-time transfer packet、由来/license/dependency closure |

## 6. STOP

- fixture manifestを公開API、Document、plugin契約へ昇格したくなった
- 製品fontをtest-only fontへ合わせるためG0-6Hを省略したくなった
- native textureの色・alpha不一致を既存golden更新またはUI用別render経路で通したくなった
- `egui_tiles`、表示名、翻訳文、Rerun Entity/Blueprintから安定identityを作りたくなった
- stale resultを救うため、別generationのrequest条件をUI側で混成したくなった
- Rerun全レーン完了を無関係なUタスクの一括ゲートにしたくなった
- 未裁定Rerun assetを「数行だけ」「testだけ」として由来記録なしで持ち込みたくなった

いずれかが起きた場合は対象枝番を停止し、公開契約を広げずに本書と対象specへ戻す。
