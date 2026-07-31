# CU-205S Opacity Direct通常製品route 分割決定

- 日付: 2026-07-31
- 状態: **決定 / DONE**
- 親: CU-205 / U4a-2

## 1. 結論

`CU-205`を次の正常系4粒とE2Eへ分割する。

1. `CU-205B PRODUCT-ASSET`: first-party `core.filter.opacity`を既存Browser
   `PluginCard`へtyped catalog projectionで接続する。source consumer親`CU-205B1`を
   provenance guard `CU-205B1G`とsource実装`CU-205B1I`へ分け、その後
   shipped Host/bundle `CU-205B2`へ進む
2. `CU-205T PRODUCT`: Browserのtyped attach intentを、現在のprimary layerに対する
   既存`DocumentWriter::prepare_create_effect`とjournal-first D2 routeへ接続する
3. `CU-205P PRODUCT`: 選択中Effect Useの`NodeDesc` / `ParamDef`を既存
   `map_parameter_control`へ渡し、Inspector Hostの生成controlへ投影する
4. `CU-205W PRODUCT`: `amount` gestureをnonblocking latest previewと、release時の
   1 gesture = 1 Undoへ接続する
5. `CU-205E E2E`: 通常製品windowでRectangle配置→Opacity追加→amount変更→
   preview→Undo→Redoを完走する

正常系4粒は`CU-204P`の実在診断triggerを前提にしない。`CU-204P`は`WAIT`を維持し、
`CU-205`親と`U4a-2`は、正常系と既存のinvalid/read-only共通診断表示が両方閉じるまで
`SPLIT / WAIT`とする。到達不能な診断をfixtureやunknown command注入で偽装しない。

実装順は`CU-205B1G → CU-205B1I → CU-205B2 → CU-205T → CU-205P → CU-205W → CU-205E`で固定する。
presentation、Document attach、Inspector projection、gesture write、E2Eを一粒へ束ねない。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [M3 U4a](../specs/M3-ui-integration.md)、[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)、[CU-G09 Browser catalog projection](2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md)、[U4a-1 parameter control](../decision-index.md)、M2 D1l effect prepare / D2 single writer |
| `INTERNAL TARGET` | `opacity_contract()` / `core.filter.opacity` v1 / `amount`、product `PluginCard`、`BrowserHostRuntime`、`DocumentWriter::prepare_create_effect`、`DocumentEditRuntime`、`map_parameter_control`、`InspectorHostRuntime` |
| `OWNER` | Effect Definition / Use / parameter値はDocument。primary layerとactive Effect UseはHost Transient。Browser / Inspectorのhover・focus・開閉はlocal presentation。first-party catalog projectionはDocument外Host read model |
| `WRITE ROUTE` | Browser typed intent→Host coordinator→current primary検証→`prepare_create_effect`→既存journal-first D2→publish。parameterは既存`SetProperty(EffectParam)` routeへ入り、ReactはDocument writerを持たない |
| `GAP` | first-party catalogからEffects Browserへの通常projection、attach intent、active Effect Use投影、生成control、preview gesture接続が無い。現行Browser HostはRectangle 1件、Inspector Hostは`nodes: []` |
| `RESOLUTION ROUTE` | 既存`PluginCard`、CU-G09 private catalog shape、first-party catalog、effect prepare、parameter control、Inspector Hostを`REUSE`し、B1G/B1I/B2/T/P/W/Eへ`REDUCE`する |
| `DISPOSITION` | 正常系は`PASS`。到達可能sourceの無い`CU-204P`だけを`WAIT`に残す |

## 3. 実在sourceと固定値

正常系の最初の対象は次だけとする。

- plugin id: `core.filter.opacity`
- version: `1`
- kind: `Filter`
- display name: `Opacity`
- category: `Color`
- parameter: `amount`
- value type: `F64`
- default: `1.0`
- domain: `[0.0, 1.0]`
- inputs: `1..=1`

これらは`plugins/motolii-plugin-opacity/src/lib.rs::opacity_contract()`と
`motolii_plugins_firstparty::first_party_catalog()`から読む。`Echo Bloom`、
`Type Pulse`、`Fold Field`、thumbnail token、表示label、配列位置をOpacityへ
special-case対応させない。

新規追加は既存Definition共有ではないため、`prepare_link_effect_use`ではなく
`prepare_create_effect`を使う。初期parameterはcontractのdefaultから
`EffectDefinitionDraft`へ明示保存する。空paramsをruntime defaultへ任せない。
既存`plugin_resolution.rs`のprivate `Value → DocValue`変換を再利用し、Opacity専用または
新しい汎用converterを重複実装しない。UI側で別のdefault、Effect Definition共有、
stable ID、Undo意味を発明しない。

## 4. CU-205B Browser source

### REACT AUTHORITY

- 対象面: product-owned Effects Browser
- 契約: [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)
- UI runtime境界: `ui/motolii-web` / product Browser Host
- spec ID: M3 `U4a-2` / `CU-205B`

### SOURCE ASSET

- 固定commit: `56c318edcddab7cf95d263cc2f7dd2b4e6791134`
- 旧path: `docs/mocks-ui/src/candidates/DiscoveryBrowserCandidate.jsx`
- product path: `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`
- export/component: `DiscoveryBrowserCandidate` / existing `PluginCard`
- closure: `discovery-browser-candidate.css`、Browser ownership/provenance guard、
  現行visual / interaction oracle

### PRESERVE

既存Effects BrowserのDOM順、class、stable ID、ARIA、tab/search/result layout、
`PluginCard`のdrag interaction、CSS、visual state、threshold、goldenを維持する。

### REPLACE

固定3 card literalをOpacityへ読み替えない。通常製品routeだけで、Hostから受け取る
strict CU-G09 catalog itemを既存`PluginCard` instanceへ渡すprivate seamを追加する。
fixture/mockの既存3 cardとlocal tagging/selectionはoracle consumerとして残し、製品の
plugin identity正本にしない。

Opacityの最小製品表示写像はHost read modelで次へ固定する。

- `catalog_revision`: `1`
- catalog `scope_ref`: `first-party-effects`
- `item_id`: plugin idのexact string
- `display_name`: `NodeDesc.display_name`
- taxonomy refs: `PluginKind::Filter`→`effect`、`NodeDesc.category == "Color"`→`color`
- provider ref: first-party bundled provenance→`built-in`
- install state ref: current first-party catalogへの登録成功→`installed`
- preview kind: motion previewを供給しないため`poster`
- pack / impact / tags: `null`または空。fixture値を補完しない
- kind glyph / generic thumbnail decoration: `PluginCard`役割固定のlocal presentation

本snapshotのprivate vocabulary entryもexactに次へ固定する。

- scopes: `{ id: "first-party-effects", label: "First-party effects", scope_ref: null }`
- taxonomies:
  `{ id: "effect", label: "Effect", scope_ref: "first-party-effects" }`と
  `{ id: "color", label: "Color", scope_ref: "first-party-effects" }`
- providers:
  `{ id: "built-in", label: "Built-in", scope_ref: "first-party-effects" }`
- install states:
  `{ id: "installed", label: "Installed", scope_ref: "first-party-effects" }`
- packs / impact units / tags: 空配列

この写像はprivate Host read modelであり、plugin ABI、manifest、Document、community
catalogの公開語彙を増やさない。

### STATE OWNER

catalog identityと登録状態はDocument外Host read model。選択、hover、search、
tag visibilityはTransient / local presentation。Document、Undo、stable ID counterは
Reactに置かない。

### DIAGNOSTIC ROUTE

通常Effects BrowserへOpacity cardを表示する。`#diagnostics/*`、fixture-only route、
mock standaloneを製品成果にしない。

### NEGATIVE ORACLE

二重component copy、legacy/mock runtime import、`Echo Bloom`等からのidentity推測、
bare item idだけのHost受理、欠落field fallback、React semantic store、
threshold/golden変更を拒否する。

### STOP

既存`PluginCard`を捨てた縮約leaf、公開catalog/plugin契約の新設、未決metadataの捏造、
Document/Undo ownerのReact移動、通常route以外だけの成立が必要なら停止する。

初回`CU-205B1`施工はGrok `REJECT`となり、差分を採用しなかった。固定source hashを
直接列挙する`browser-catalog-decoder.test.mjs`が、既に存在するappend-only
`source-provenance.json::postPromotionChanges` authorityと再結合されておらず、
正当なpost-promotion source変更を無条件に拒否したためである。期待hashを実装ごとに
同じ差分で書き換えず、先に`CU-205B1G ORACLE-GUARD`を行う。

`CU-205B1G`はtest-only provenance guardだけを所有する。

- `browser-ownership.test.mjs`に既存する`validatePostPromotionChanges`を、product codeへ
  importされないguard-tests内の共有moduleへ移す
- Browser固定SHA、index 0固定entry、exact key、同一file、task一意、非空reason、
  hash chain、last current hashとlive source一致を一つも弱めない
- `browser-catalog-decoder.test.mjs`はBrowser JSXと`source-provenance.json`の
  その時点のhash literalを期待値更新せず、同じ共有validatorでlive chainを検証する
- `inspector-read-model-decoder.test.mjs`はInspector JSXの固定hashを維持したまま、
  Browser変更でも変化する`source-provenance.json`全体hashだけを固定authorityから外す
- decoder、React、provenance JSON、fixture、製品runtimeを変更しない
- helper重複、単なるcurrent hash受理、空chain受理、index 0緩和を拒否する

`CU-205B1I`はproduct Web sourceとprovenance appendだけを所有する。

- `browserHostCodec.js`でstrict catalog snapshotを既存`browserCatalogDecoder`へ渡す
- `host/main.jsx`からdecode済みcatalogを`DiscoveryBrowserCandidate`へ渡す
- `CandidatePluginBrowser`はcatalog入力がある通常Hostだけで既存`PluginCard`を
  projected item consumerにし、入力なしのmock/standalone 3 cardを変更しない
- Host fixtureによるcodec/component/ownership試験を追加する
- `source-provenance.json::postPromotionChanges`へ`CU-205B1I`の旧末尾hash→新live hashを
  一件appendし、共有validatorを通す
- generated-host、Rust、Document、intent、CSS、visual oracleを変更しない

projected `PluginCard`の値は次だけとし、fallbackを置かない。

- `itemId` / `name`: exact `item_id` / non-null `display_name`
- `category` / `subtype`: exact taxonomy refs `effect` / `color`のid + label
- `mode`: exact install-state ref `installed`から`installed`
- `folder` / `labels`: taxonomy labelを順序どおり空白結合した導出値
- `search`: display name、item id、taxonomy label、provider label、install-state labelの順序付き結合
- `thumbnail`: role-fixed local presentation `poster`
- `kind`: role-fixed local presentation `FX`
- `state` / `identity` / `impact`: `undefined`
- `pack`: `null`
- `motion`: `false`
- `tags`: 空配列、`tagVisible`: `true`

present catalogでscope、item、vocabulary ref、exact labelが一致しなければcodecで拒否する。
Candidate側で`catalogs[0]`、static 3 card、label、item idへfallbackしない。
Hookは常に同じ順で呼び、catalog専用Contextを新設せず既存prop経路で渡す。Results countは
projected item 1件時に`1`、catalog absent時に既存`3`とする。

`CU-205B2`はshipped Host接続だけを所有する。

- Rust `BrowserHostSession`へexact private first-party snapshotを追加する
- `opacity_contract()`を直接複製せず`first_party_catalog()`の登録済みcontractを読む
- Host bundleを正規buildで再生成し、manifestとRust `include_bytes!`を新hashへ接続する
- 通常製品windowでOpacity cardが見えることをshipped bundleから確認する
- Web source意味、React DOM/CSS、intent、Document、selection、Undoを変更しない

CU-205B1Gだけ、またはCU-205B1Iだけを通常製品接続の完成と数えない。
B1G/B1I/B2が揃って親CU-205Bを`DONE`とする。
CU-205B全体はcard表示とtyped sourceだけを所有し、attach、D2、Inspector、previewを実装しない。

## 5. CU-205T attach

CU-205Tの最初のCommit入口は、[UI interaction language](../ui-interaction-language.md)の
共通Browser文法どおり`double click`とkeyboard `Enter`とする。single clickは従来どおり
card選択/previewだけで、Documentを変更しない。Commit時点で現在のprimary layerが実在し、
sourceの`(scope_ref, item_id)`がcurrent catalog snapshotに一致する場合だけ、新しい
Opacity Definition + Useを末尾へ一度追加する。

既存drag payloadもbare `itemId`からtyped source carrierへ交換するが、drag/drop commitは
明示的な対象hit-testが別粒で閉じるまでDocument write 0とする。現在のprimaryへ黙って
dropしたことにしたり、Stage全体を対象layerと解釈したりしない。

- primaryなし、stale/unknown source、非Filter、contract/default検証失敗はwrite 0
- Reactはtarget layer、insert index、Effect ID、Definition IDを決めない
- Hostはcurrent published primaryとDocument snapshotからtarget / indexを決める
- `prepare_create_effect`成功後だけ既存journal-first routeへ渡す
- 1 attach = 1 journal macro = 1 Undo
- Apply / Undo / Redo後は既存publish / reconcileを通す
- attach成功時の新Effect UseをHost Transientのactive Effect Useにする
- primary変更、対象Use消滅、UndoでUse消滅、project再openでactive Effect Useをclearする

Rectangle PlaceのStage hit-testやcanonical positionをEffect attachへ流用しない。
Opacityの初回Direct操作はdouble click / Enterで現在のprimary layerへ適用し、対象が無い時に
中央配置、自動Rectangle生成、最終primaryへの暗黙fallbackをしない。

## 6. CU-205P Inspector projection

CU-205Pはcurrent primary layerとactive Effect Useから、Effect Definitionのplugin id /
version / paramsを読み、同じfirst-party catalogのcontractへ照合する。各保存parameterを
`map_parameter_control`へ渡し、既存Inspector Host snapshotの`nodes`へexact identityと
control specを投影する。

- `amount`のcontrolは`F64` domain `[0,1]`
- control identityはEffect Use ID + exact param idで、labelやDOM順から逆算しない
- Document snapshotはread-only、ReactにEffect/selection cloneを作らない
- missing plugin/version、missing definition/use、unknown param、unsupported value typeは
  もっともらしいcontrolへ縮退しない
- normal first-party catalogの全保存parameterが`map_parameter_control`成功することを
  invariant testで固定する

unsupported型を持つ架空first-party plugin、fixture Effect、diagnostic-only routeを
`CU-204P`のtriggerとして追加しない。

## 7. CU-205W preview / gesture

CU-205Wは`amount`の1 gestureを次へ接続する。

1. updateはUI threadをblockせずlatest-wins preview requestへ送る
2. generationが古いcompleted previewを表示しない
3. drag中のpointer軌跡をDocument / journalへ保存しない
4. release時の確定値だけを既存`SetProperty(EffectParam)` routeへ渡す
5. 100 updatesでもUI thread wait 0、最終previewと確定Document値が一致する
6. release 1回でUndo 1回
7. 初回確定前のEscape / focus lossはDocument write 0

適用後Cancel、公開gesture lifecycle、第二preview renderer、CPU pixel経路、色変換経路、
plugin ABI、Document schemaは本粒で増やさない。

## 8. CU-205E E2E

通常製品windowだけを使い、次を同一Document identityで確認する。

1. empty projectからRectangleを配置しprimaryにする
2. Effects Browserにtyped first-party Opacity cardが見える
3. cardをsingle clickしてもDocument不変で、double clickまたはEnterによりprimary
   RectangleへOpacityを一度追加する
4. Inspectorに`amount` F64 control `[0,1]`が自動生成される
5. 100 update後のlatest previewとrelease値が一致する
6. Undoでparameter確定を一回戻し、次のUndoでEffect attachを一回戻す
7. RedoでEffect attachとparameter確定が同じEffect / parameter意味へ戻る
8. save/reopen後もDocumentのEffect Definition / Use / amountは保持されるが、
   active Effect UseというUI選択は安全な未選択へ戻る

mock、fixture-only Host、headless helperだけの完走をE2E合格にしない。

## 9. CU-204Pとの関係

first-party catalogの全保存parameterが正常にmapできることは、正常系の正例であり
`CU-204P`の診断sourceではない。現行5 `DiagnosticReasonCode`へ通常操作から確実に到達する
source、表示期間、replacement、clear時点が閉じるまで`CU-204P`は`WAIT`を維持する。

将来、実在する通常操作が既存reasonへ到達した時だけCU-204Pを再開する。新しいreasonを
本正常系の都合で追加せず、同じInspector Feedback surfaceを再利用する。

## 10. 非目標とSTOP

- `CU-204P`、U4c Advanced、U2c-2 Direct/Advanced conformanceを完了扱いにする
- custom plugin panel、third-party install/load、community catalogを実装する
- plugin ABI、Document schema、journal形式、公開command / catalog APIを変える
- `prepare_link_effect_use`で暗黙のshared Definition意味を作る
- first-party contract以外のdefault、domain、display name、categoryをUIで持つ
- raw label、thumbnail、配列位置、bare item idからidentityを推測する
- ReactにDocument、primary、active Effect Use、historyの正本を置く
- Browser、attach、Inspector、preview、E2Eを一つの実装粒へ束ねる
- test expectation、visual threshold、goldenを実装都合で変える
- diagnostic-only route、架空plugin、fixture Effectを通常製品完成証拠にする
