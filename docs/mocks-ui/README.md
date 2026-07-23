# M3 Reactモック台帳

このディレクトリが、ブラウザで操作する**現行M3比較prototypeの実行入口**である。参照順位とReact移行の実状態は[M3 UI参照地図](../ui-reference-map.md)を先に読む。[component-map.json](component-map.json)が安定ID、層、出典selector、直接関係するM3チケット、明示済みの状態所有、eguiでの投影先を対応付ける。

台帳はRust公開API、永続形式、製品component名、JSX props、製品token値を定めない。HTMLのDOM境界をegui境界へそのまま写す指示でもない。具体的な色、px、radiusは`U0e-2`の比較材料に留め、`G0-6H`の人間審判より前に`U0e-3`の製品値へ昇格させない。

## 現在の所有境界

React/Viteへの移行は実行基盤として完了しているが、全surfaceのReact-native化は完了していない。`src/legacy/LegacyHostBoundaryScreen.jsx`は、legacy `m3-vism-host-boundary.html`をVite raw importし、`html-react-parser`でReact treeへ変換する移行bridgeである。元CSS、class、ID、子DOMと、リポジトリ同梱の固定scriptを維持しながら、Browser、Color Book、Stage、Inspector、Timeline、Recovery、Settingsをnamed wrapperへ昇格する。

- 通常入場（hashなし）と`#catalog`はReact候補だけを表示する。旧fixtureは`#archive/catalog`と`#archive/<fixture>`に隔離し、`#all-surfaces`等の旧hashは未登録として扱う。
- registryの`catalogKind`は`candidate / reference / diagnostic / archive`の閉集合で、通常`#catalog`は`candidate`、`#archive/catalog`は`archive`だけを列挙する。referenceとdiagnosticは直URLだけで開き、一覧へ混ぜない。
- `#archive/all-surfaces`ほかarchive hashはparser-backedな**legacy parity参照**を表示する。新しい判断の実装先ではない。
- `#plugin-browser-candidate`はBrowser wrapper、Timeline、Interval Easing EditorをReact-native候補へ差し替える。multi-key Graph ViewはTimeline dock内の`譜面 / GRAPH`切替へ統合し、同じdock寸法と時間文脈を保つ。`#graph-view-candidate`は操作試験用の独立fixtureとして残す。Graph Viewと区間editorは同じsurface名・座標・状態所有へ統合しない。
- Graph Viewの時間／値rangeとcurve演算は`src/candidates/graph-view-model.js`、React表示とpointer取得は`GraphViewCandidate.jsx`が所有する。modelはmock内部の比較seamであり、公開plugin APIや永続形式ではない。dock resize時はviewBoxを実寸へ合わせて表示座標だけを再投影し、rangeのauto-fitや非等方stretchを行わない。
- Browser候補はMedia / Create / Effectsを`src/patterns/DiscoveryBrowser.jsx`の共通`Search / Sources / Collections / Results / View switch`構成で比較する。詳細な比較仮説は[操作prototype台帳](../reviews/2026-07-19-m3-interaction-prototype-decision-ledger.md)のP27〜P33、P41〜P46へ集約し、このREADMEへ仕様を重複させない。採択済みlayout操作の正本P48/P49はmain docs側を参照する。
- BrowserのSources / Registered folders / Collections / Packsを含む左階層railは、境界dragで横幅を変更し、左端までdragすると閉じる。閉じた後の再表示は現行の縦型`HIERARCHY`入口を使う。状態はWorkspace-session候補で、検索結果、選択、Document、Undoを変えない。
- Media / Effects / Createのtag表示はEffects型の左階層UIへ統一する。共通化するのはtagへitemをdropして分類し、tag行から結果を絞り込む操作文法だけで、tag名と割当はitem種別ごとに分ける。Host taxonomyやplugin manifestとは別で、DocumentとUndoを変えない。
- 既存Settingsの`Plugin thumbnail size`は重複controlを増やさず`Browser thumbnail size`へ一般化し、Media / Effects / Createの3面で同じ値を使う。Reactでは既存settingをBrowser rootの共通値へ接続し、egui移植時はBrowser共通表示設定から各結果gridへ投影する。
- Media / Effects / Createは共通View toggleで`thumbnail-only / thumbnail+name / list`を選べる。card内はitem名へ先に幅を割り当て、tagは残余幅だけでelideする。Effectsのcard高も共通thumbnail寸法へ揃え、旧Plugin専用寸法による空白を残さない。
- Browser内の見出し、階層label、tag、card名、状態文は1行固定で折り返さない。幅を超えた表示はelideし、詳しい説明は既存の下部tips / focus情報へ委ねる。
- Easing候補は`src/candidates/EasingGraphCandidate.jsx`で、current/context keyの複合識別、明示Overshoot、Copy、現在区間Paste、現在channel一括Paste、既決の高度区間補間を比較する。Bounce / Elastic / Cyclic(Sine) / Random / Steps / Elastic Stepsはkeyframeを増やさず、選択区間のoutgoing interpolationだけを変更する。グラフの座標写像はcurve内容で動的フィットせず、Overshoot OFFは標準固定範囲、ONは最初から最大可動域を収める固定範囲を使う。モードは説明文ではなく、枠内curve／上限越えcurveの専用ピクトグラムとpressed状態で示す。[AM観察台帳](../reviews/2026-07-19-am-keyframe-graph-observation.md)の差分とPlaywright非破壊試験を同時に更新する。
- Timeline候補は`src/candidates/TimelineCandidate.jsx`で、左端のInbox、名称欄相当の細い無名帯アクションrail、右側の一枚のpacking時間面を維持し、見やすいBeat ruler / major-minor tick / 横scroll、所有者ではない水平packing guide、Object bar内S/Mを比較する。rail右端と時間面左端、rail header下端とBeat ruler下端、各rail行下端と対応guide下端は同じ座標を使う。guide間隔とbar高は設定toolbarや全帯共通controlを作らず、rail各帯の下辺に統合した小さな二本線gripを上下dragし、触った帯だけ変更する。railのS/MはLane状態ではなく、押下時に同じ帯へ載る全Objectの既存S/Mへ一括適用し、全ON / 全OFF / 混在をObject集合から導出する。固定Object rail、固定名列、1項目1横行、Lane所有状態は追加しない。packing表示密度はWorkspace-session候補、S/Mは未決のObject状態投影で、React stateやpx値からDocument、Undo、評価意味、保存形式を定義しない。Easing / Curve Shelfは移行中の既存操作を保つ。
- Depth Rail候補も`TimelineCandidate.jsx`が所有する。2D Objectの初期`z=0`はhover／選択で扇状展開せず`0 × N`の件数付きstackとして示し、Timelineの選択stable IDだけをfocus同期する。通常bar clickは閉じたRailを開かず、各barまたは譜面headerのDepth iconが明示入口になる。同じparentの複数選択では常設`Layer Order Distribute` iconから奥端・手前端をpreviewし、authoring order、Reverse、Apply、Cancelを比較する。親側はGroup自身を1 marker、child選択時は`ROOT / Group`のparent-local scopeへ切り替え、rootと子を同じ配布集合へ混ぜない。React stateのZ値、区間、scope、px位置はprototype fixtureであり、Document schema、専用Depth field、D2 command型を定義しない。
- Automation候補はObject bar内の`◆ n`（0件時は`◇＋`）からAE型の縦一覧として展開し、実際にAutomationを持つchannel行だけを表示する。複数channelのkeyを同時選択するとTimeline右端へ既定dockした独立`Key Tools`でKeystone 3型のmodular操作を行える。`KEYS / LAYERS`は排他的に切り替わり、KEYSは`Object別 / Channel別 / 全選択`を明示したAlign / Stagger / Stretch、LAYERSは選択ObjectだけのAlign / Stagger / Shiftを表示する。両modeのsectionを同時表示せず、説明文は置かず対象名・section名・数値・iconだけを使う。未使用channelは末尾の`＋`から検索する。BrowserのPreset棚とInspectorのparameter面を占有せず、将来は共通panel systemで移動・split・tab化する候補。操作結果は比較用React stateであり、Document schema、D2 command、default key、Undo意味を定義しない。
- `#skeleton`は分解境界だけを確認する簡略版であり、視覚正本ではない。
- bridgeが評価するscriptは静的raw importしたリポジトリ同梱fixtureだけである。外部HTMLや入力文字列を渡さない。
- surfaceをReact-nativeへ置き換える時は、該当wrapperだけを変更する。parityが必要な移行中だけ旧HTMLとのPlaywright画像比較を維持し、置換後の新機能を旧HTMLへ逆実装しない。

```sh
npm ci
npm run dev -- --host 127.0.0.1
npm run storybook
npm run test:reference-guard
npm run test:visual
```

Storybookは現行参照、改善候補、Skeletonを分離する。Playwrightは旧HTMLとparser版を同じChrome・1440×900で撮影して参照bridgeの画像差分を審判し、改善候補は別の操作試験で共通Browser文法と逸脱状態だけの表示を確認する。

`scripts/reference-guard.mjs`はGR-R1/R2のheadless guardである。U0e-2のprovenance manifestはreference leaf、固定source assetとtest evidenceのpath/export/SHA-256 closure、`reference/*` route、`document / scenes / tokens`の順の三層probe、固定normal capture renderer moduleを宣言する。Babel ASTとPostCSSで実importと無条件JSX合成、legacy/archive runtime、自己登録、copy、生色値、fixture load結果からsource component propへの到達を検査する。`verifyFixtureCausality`はmanifestでhash固定したrendererだけをloadし、同じ三pathへ各状態を二つの順序で再生して決定性と各層のnormal capture変化を検査する。semantic ID、画像類似、装飾import、path名やmutation hintだけではこのguardを通らない。

通常動線:

- 現行React候補: `http://127.0.0.1:5173/` または `#plugin-browser-candidate`
- multi-key Graph View比較: `#graph-view-candidate`
- 現行候補一覧: `#catalog`
- legacy archive一覧: `#archive/catalog`
- 個別parity参照: `#archive/all-surfaces`等

## Legacy参照順位

1. `docs/mocks/m3-vism-host-boundary.html`はReact bridgeのparity sourceであり、製品または現行interactionの正本ではない。
2. `docs/mocks/m3-main-ui-v3-monochrome.html`は区画、密度、因果、Z/Group意味の履歴比較に限る。色値は製品tokenの根拠にしない。
3. v4、v5、`m3-plugin-boundary-learning.html`は専門的な履歴比較候補である。React候補を黙って上書きしない。
4. v1、v2、timeline v0、interaction v0、dynamics v1は履歴比較である。撤回済みの固定Track/Lane、重複Inspector、講義文、正常時の状態語を復活させない。

より新しいHTMLという理由だけで参照順位を変えない。現行prototypeの変更は、規範文書または比較台帳の採否を先に更新してReact fixtureへ反映する。

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
