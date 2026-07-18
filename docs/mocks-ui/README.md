# M3モック分解台帳

このディレクトリは、既存HTMLモックを並行して分解するための**モック専用台帳**である。[component-map.json](component-map.json)が安定ID、層、出典selector、直接関係するM3チケット、明示済みの状態所有、eguiでの投影先を対応付ける。

台帳はRust公開API、永続形式、製品component名、JSX props、製品token値を定めない。HTMLのDOM境界をegui境界へそのまま写す指示でもない。具体的な色、px、radiusは`U0e-2`の比較材料に留め、`G0-6H`の人間審判より前に`U0e-3`の製品値へ昇格させない。

## 忠実再現bridge

`src/legacy/LegacyHostBoundaryScreen.jsx`は、現行`m3-vism-host-boundary.html`をVite raw importし、`html-react-parser`でReact treeへ変換する。元CSS、class、ID、子DOMと、リポジトリ同梱の固定scriptを維持しながら、Browser、Color Book、Stage、Inspector、Timeline、Recovery、Settingsをnamed wrapperへ昇格する。

- `#all-surfaces`ほか既存hashはparser-backedな現行参照を表示する。
- `#skeleton`は分解境界だけを確認する簡略版であり、視覚正本ではない。
- bridgeが評価するscriptは静的raw importしたリポジトリ同梱fixtureだけである。外部HTMLや入力文字列を渡さない。
- surfaceを本実装へ置き換える時は、該当wrapperだけを変更し、旧HTMLとのPlaywright画像比較を維持する。

```sh
npm ci
npm run dev -- --host 127.0.0.1
npm run storybook
npm run test:visual
```

Storybookは現行参照とSkeletonを別階層に置く。Playwrightは旧HTMLとparser版を同じChrome・1440×900で撮影し、画像差分、主要surface座標、accessible landmarkを審判する。

## 参照順位

1. `docs/mocks/m3-vism-host-boundary.html`を現在の統合分解元とする。特に`#all-surfaces`はBrowserのProject/Pluginsを同時表示しないまま、Stage、Inspector、Timeline、Depth Rail、Color Bookを一画面で審判する。
2. `docs/mocks/m3-main-ui-v3-monochrome.html`は現在の構造基準であり、区画、密度、因果、Z/Group意味の比較に使う。色値は製品tokenの根拠にしない。
3. v4、v5、`m3-plugin-boundary-learning.html`は専門的な現行比較候補である。統合元を黙って上書きしない。
4. v1、v2、timeline v0、interaction v0、dynamics v1は履歴比較である。採用済みの観察は現行統合モックから取り、撤回済みの固定Track/Lane、重複Inspector、講義文、正常時の状態語を復活させない。

より新しいHTMLという理由だけで参照順位を変えない。順位変更は`docs/mocks/README.md`側の採否記録を先に更新してから台帳へ反映する。

## IDと層

IDは見た目の名前ではなく、モック内の責務に対して安定させる。

- `primitive.*`: 単一の操作・表示単位。単独でDocument意味を所有しない。
- `pattern.*`: 複数primitiveからなる反復文法。棚、parameter行、診断投影など。
- `surface.*`: Browser、Stage、Inspector、Timelineなど、明示的な作業面。
- `screen.*`: hash fixtureから再現する審判画面。

DOM classや表示文言が変わっても責務が同じならIDを変えない。責務を分割・統合する場合は既存IDを別の意味へ使い回さず、台帳変更で理由をレビューする。

`stateOwner`は既存文書または現行モックREADMEで明示された項目だけを書く。未記載は「Documentではない」と推測せず、フィールドなしのまま`U0b-1`へ送る。複数ownerがある項目は、正本、Transient preview、commit後のDocument投影を分けて記述する。

## 並行所有ルール

並行作業は次の境界で分ける。

- **shared担当**: `primitive.*`と`pattern.*`だけを所有する。surface固有の配置やfixture状態を変更しない。
- **surface担当**: 割り当てられた`surface.*`と対応fixtureだけを所有する。shared部品を直接変更せず、必要な差分を台帳ID付きでshared担当へ返す。
- **screen担当**: `screen.*`の組み立てとhash入場、golden生成条件だけを所有する。個別surface内部をforkしない。
- **reference担当**: 現行参照と履歴比較の分類を管理する。比較案から現行へ採用する時は、出典と撤回事項を残す。

同じ変更でshared部品と複数surfaceを横断しない。まずshared部品を独立させ、その固定後に各surfaceが取り込む。surfaceごとに似たButton、PanelHeader、ParameterRow、Diagnosticを複製しない。

JSX等へ移す場合も、1担当の変更範囲は台帳ID単位にする。ファイル名やexport名は実装上の都合であり、Rust/eguiの公開名へ伝播させない。React state、DOM event、CSS px、browser storageをDocument、domain intent、User settings形式の根拠にしない。

## 変更手順

1. 対象ID、source selector、hash fixtureを選ぶ。
2. 現行HTMLと同じ状態を出せる分解だけを行い、同じ変更でデザインを改訂しない。
3. shared部品の不足はsurface内へ複製せず、台帳へ追加して別変更に分ける。
4. hash fixtureとgolden生成条件を維持して視覚差分を確認する。
5. 意味、状態所有、製品token、公開境界を変えたくなったらモック実装を止め、対応する仕様・チケットへ戻す。

`component-map.json`は通常のJSONとしてparseできることを必須とする。台帳のticketは、チケット分解文書に直接対応がある場合だけ記載し、画面に存在するという理由で将来チケットを推測しない。
