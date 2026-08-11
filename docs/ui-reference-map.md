# M3 UI参照地図

更新日: 2026-08-07

M3 UIを調べる時は、資料の新旧ではなく次の層で参照先を決める。会話履歴、スクリーンショット、旧HTML、React prototypeのいずれも、単独では製品仕様にならない。

## 画面名と実装状態

固有名、成果物種別、結合段階は[UI成果物・実装状態の用語](ui-artifact-terminology.md)を正本とする。

- **Motolii Studio**: 利用者へ届けるReact Native + native canvas desktop製品。
- **Motolii Studio Mock**: `docs/mocks-ui/`で外部browserに表示する開発モック。
- **Motolii Studio Preview**: 定義済みの製品surfaceを通常経路で一つのnative desktop実行ファイルへ結合した
  preview build。現時点では未実装である。

既存`motolii_ui_shell`とwinit/WebView/direct-wgpu/Vello製品routeは**Legacy Product Route / Migration Oracle**、個別`g0-9-*` / `g0-10-*`と2026-08-06〜07のRN/Skia検証は**Native Surface Spike**と呼ぶ。source asset、旧product route、isolated spikeの成立を、新しいRN product-integratedまたはpreview-runnableへ繰り上げない。

## 参照順位

| 層 | 役割 | 正本／入口 | 変更時の規則 |
|---|---|---|---|
| 規範 | 状態所有、Undo、入力、意味、受け入れ条件 | [M3仕様](specs/M3-ui-integration.md)、[UI操作言語](ui-interaction-language.md)、[UI視覚言語](ui-visual-language.md)、[UI境界規律](reviews/2026-07-14-m3-ui-boundary-prevention.md) | prototypeや会話から直接上書きせず、仕様・決定台帳を先に改訂する |
| 現行prototype / React source asset | 現在browserで比較する操作・構成と、RNへ移すconcept、component boundary、state、test oracle | `docs/mocks-ui/README.md`と固定commit `56c318ed`、[runtime再基線決定](reviews/2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md) | DOM/CSSのbyte同一移管を要求せず、意味・情報階層・状態・操作をRN primitivesとnative component contractへ変換する |
| 製品実装先例 | 高密度shell、時間面、GPU viewport、selection、component、試験を成立させた実装資産 | [UI runtime責任境界](ui-runtime-architecture.md)、[Rerun先例調査](reviews/2026-07-20-rerun-prior-art-survey.md)、[Rerun学習・転移計画](reviews/2026-07-20-rerun-learning-transfer-plan.md) | Rerunの画面・語彙・schemaを模倣しない。React/native所有は正本に従い、toolkit横断patternだけを比較入力とする。egui固有assetは製品へ依存・vendoring・移植しない |
| 採否台帳 | 先例、観察、未決、棄却、停止線 | `reviews/`の対象別decision／observation ledger | 出典、Motoliiへの翻訳、反映先を分ける |
| 移行互換 | React移行中の視覚parityと未置換領域 | [旧HTMLモック台帳](mocks/README.md)、`mocks-ui/src/legacy/` | 新しい判断を追加しない。React-native置換後に参照専用へ縮退する |
| 証拠 | ユーザー撮影画像、golden、操作記録 | `reviews/evidence/`、Playwright結果 | 版、OS、fixture、viewport、操作列をmanifest化する |
| 履歴 | Codexタスク、git履歴 | Codexタスク一覧、git log | 決定の探索にだけ使い、現行仕様として引用しない |

## Reactモック、Rerun、Motolii正本の役割

三者を競合するUI正本として扱わない。

| 資料 | 答える問い | 答えない問い |
|---|---|---|
| Reactモック（Motolii Studio Mock） | Motoliiで何を見せ、どう操作させたいか。React所有面のcomponent、fixture、Storybook、Playwright、stable IDは直接所有移管する製品source asset | React state、DOM event、CSS px、仮JSONをDocument/公開契約へ昇格すること。browserモックをnative製品Previewと呼ぶこと |
| Rerun | 高密度な製品shell、時間面、GPU viewport、selection、component、試験をどう成立させたか | Motoliiの作品意味、編集command、clip/keyframe操作。egui固有assetを製品へ`DEPEND/VENDOR/PORT`すること |
| Motolii規範・仕様 | 状態の持ち場、Undo、公開契約、受け入れ条件 | 具体token値や未採択component実装 |

React資産のRN移行は、固定source assetのcomponent boundary、情報階層、labels、状態、interaction testをproduct ownerへ移し、DOM element、CSS、browser event、WebView bridgeをRN primitives、StyleSheet、typed native component contractへ変換する。見た目のbyte同一やDOM互換を目的にせず、縮約版でconceptを落とさない。Rerunに存在することだけを理由に機能を足さず、Reactモックに存在する表示だけを理由に未決のDocument意味を実装しない。

## React web資産からReact Nativeへの移行状態

既存の「React product-owned」はReact web資産のowner確立を意味し、RN product routeへの移植完了ではない。`#plugin-browser-candidate`は全体構成を人間が確認するmock consumer route、`ui/motolii-web/source-provenance.json`はweb source closureのauthorityである。RN側の新しいsource authorityはM3-R0/R1で作り、旧closureを直接release runtimeと呼ばない。

| fixture／領域 | 実装状態 | 現在の用途 |
|---|---|---|
| `#plugin-browser-candidate`のBrowser | React product-owned。2026-08-09に`ui/motolii-rn/src/browser/BrowserPanel.tsx`へ着地した実装はhistorical oracleとして保持するが、同targetはread-only。current接続targetは`spikes/motolii-rn-probe/App.tsx` | `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`のproduct closureをsemantic oracleとして使い、probe内の既存BrowserへHost入力を接続する。別panelへcopyしない |
| `#plugin-browser-candidate`のEasing入口 / popup | triggerはReact product-owned（R2B完了）、popupはnative oracle | trigger source authorityは`ui/motolii-web/src/candidates/EasingTriggerCandidate.jsx`。object・channelとpressed/disabledのaccessible stateまでを維持する。visible summary chromeは未決。popup frame/preset/form/curve/modelはnative oracleとして維持し、active interval／Interp接続は別粒 |
| `#plugin-browser-candidate`の`KEYS / LAYERS` | React product-owned tool panel（R3B完了） | source authorityは`ui/motolii-web/src/candidates/KeyToolsCandidate.jsx`。mockの`TimelineCandidate.jsx`はconsumerであり、native time surfaceをReactへ移さない。各operationのHost接続は別粒 |
| `#plugin-browser-candidate`のInspector | React product-owned。`ui/motolii-rn/src/inspector/InspectorInitialReadPanel.tsx`はhistorical oracleとして保持するが、current接続targetは`spikes/motolii-rn-probe/App.tsx` | source authorityは`ui/motolii-web/src/candidates/InspectorCandidate.jsx`。probe内の既存Inspectorへbounded readを接続し、未接続parameter/easingを画面から推測しない |
| `#plugin-browser-candidate`のStage / Timeline time surface / Settings | legacy bridgeまたはReact比較candidate | native製品面のoracle／周辺文脈。React製品runtimeへ直接持ち込まない |
| `#archive/all-surfaces`等 | legacy HTMLをparseしたarchive bridge | 旧画面との視覚parity回帰。通常catalogへ出さない |
| `#skeleton` | React-native分解骨格 | component責務と組立境界の確認。視覚正本でも、Inspector等の代替製品実装でもない |

したがって、React上で表示されるだけではproduct-connectedと判定しない。未移管領域が
`src/legacy/LegacyHostBoundaryScreen.jsx`またはraw HTML由来のDOM／scriptへ依存する場合は、旧仕様を増やさず、
固定モック内で同形Reactへ抽出してから製品ownerへ移す。移管済み領域は固定mock commitへ戻らず、provenance
manifestのcurrent product closureを読む。正しいsourceが無いことを理由に、製品packageへ縮約版を先に作らない。

### G0-6H人間審判入力routeの裁定（2026-07-28）
- 現行route `#plugin-browser-candidate`（固定commit `56c318ed`）がG0-6Hのforward-lookingなhuman-judgment入力routeとして単独採択される。
- 旧`#reference/*` generation `u0e2-08f96cbd7754-85c0fc529ab1`は不変の再現/派生回帰証拠として残し、required human-judgment inputには用いない。
- route裁定はvisual parityを主張せず、正本は[G0-6H-S](reviews/2026-07-28-g0-6h-s-human-judgment-input-route-decision.md)。

### 座標描画面の機械監査（2026-07-22）

固定commit `56c318ed`のReact sourceを`canvas` / SVG / pointer / absolute-positionで走査した結果、literal
`<canvas>`と`getContext()`は0件だった。Canvas相当の座標描画面はEasing SVG、Multi-key Graph View SVG、
Timeline time/Z projectionであり、Easing、通常Timeline、Multi-key Graph View、Depth Railはnative isolated core
fixtureまで到達した。Graph ViewとDepth Railはいずれも外観・headless基本操作までで、AX/D2と
製品input接続を後続する。Stage gizmoはモック移植でなくnative presentation overlayの製品実装として別に扱う。

詳細なsource別分類、進行順、非目標は
[React coordinate surface機械監査](reviews/2026-07-22-m3-react-coordinate-surface-audit.md)を正とする。
`KEYS / LAYERS`、Align / Stagger / Stretch、小さなStagger説明SVGはReact所有のままで、nativeへ複製しない。
Multi-key Graph Viewの具体的な操作トポロジー、Blender先例の利用範囲、GPL停止線は
[native Multi-key Graph View受入契約](reviews/2026-07-22-m3-native-multi-key-graph-view-acceptance.md)に従う。

## 統合モックの面 → 実装レーン対応(2026-07-19操作確認スナップショット)

`#plugin-browser-candidate`は名前に反してBrowser単体ではなく、M3後半までを含む統合モックである。2026-07-19の操作確認時点の対応(現在地は日付時点のスナップショットであり、正本は[M3仕様](specs/M3-ui-integration.md)のタスク表):

| モック内の面 | 対応する実装領域 | 2026-07-19時点 |
|---|---|---|
| UI shell・可変panel・Stage | U1a/U1b/U1f | G0-9完了までtoolkit固有実装停止。fixture、境界、比較spikeだけ可 |
| Effect Inspector・自動parameter panel | U4a | 基盤依存待ち |
| packed Timeline・Group展開・選択 | U3a/U3b/U2h | 後続 |
| Automation展開・Key Tools | P56/P60+U3系 | 一部prototype判断のまま |
| 区間Easing・multi-key Graph View | U4b／U4e | 区間EasingはU4b。multi-key Graph Viewは後続のnative受入でU4eへ採択され、isolated core fixture合格。正式snapshot、U2h、D2、AX、製品dock接続は後続 |
| Effects Browser | U4d | 正式タスク化済み |
| Media Browser・folder・Tag・複数選択 | U6 | 正式タスク化済み |
| Create Browser・provider・generator | U9/Vism/Create境界 | 一部未統一 |
| Depth Rail・分配 | ui-score/M5系 | M3だけでは閉じない |
| readiness・rendering・stale表示 | U3f | M4 provider待ち |

**発見入口だけが存在する面**: `Type Pulse`はEffects/Createの両面へカードとして出るが、選択してもStage/Inspectorは`Echo Bloom`のまま。つまり現行モックが持つのはText Motionの**発見入口**までで、`適用先preflight → Live Text生成/Animator追加 → Inspector切替 → Character Score展開 → Stage文字選択`のhandoffは未モックである。この接続は[TM翻訳](reviews/2026-07-19-m3-text-motion-task-translation.md)の後続であり、TM第1弾はBrowser非依存(通常のObject作成経路)で進める。

三面構成(`Media / Create / Effects`)は下表のとおり**P41未統一のまま**であり、そのまま選択中のUI runtime/製品へ写さない。

## 既知の未統一

現時点で次を一つの「現行仕様」として読んではならない。

| 論点 | 規範／記録 | React prototype | 扱い |
|---|---|---|---|
| Browser一次分類 | [UI操作言語](ui-interaction-language.md)には`Media / Plugins`が残る | `Media / Create / Effects` | **未統一**。prototype台帳P41で比較中。名称をDocument型、package kind、公開APIへ焼かない |
| Easing Graph | M3 U4bは区間中心GraphとBezier編集を要求。高度型も2026-07-10に区間補間として採用済み | Reactはtriggerとobject・channel・pressed/disabledのaccessible state、native popupはframe/preset/user library/form/curve/grid/handleを所有する。visible summary chromeは未決。固定React viewはpopup全体のvisual/interaction oracle。高度型の適用前後でkeyframe構造不変。区間導出とcurve状態はlegacy fixture adapterが残る | [native Easing popup受入契約](reviews/2026-07-22-m3-native-easing-popup-acceptance.md)と[AM観察台帳](reviews/2026-07-19-am-keyframe-graph-observation.md)で残差追跡。state adapter撤去と製品`Interp`接続を分ける |
| React移行完了の意味 | 旧READMEは旧HTMLを「現行参照」と表現していた | Vite上では動くがBrowser以外の主要surfaceはbridge | 実行基盤のReact化とsurface所有のReact-native化を別々に記録する |

この表の未統一項目は、画面が動いていることや会話の新しさだけで解消しない。採否を決めたら、規範文書、prototype台帳、React fixture、試験を同じ変更単位で更新して本表から外す。

## 現行fixture

開発サーバーを`docs/mocks-ui`から起動し、次を使う。

```sh
npm ci
npm run dev -- --host 127.0.0.1
```

- 全体回帰（archive）: `http://127.0.0.1:5173/#archive/all-surfaces`
- Browser候補: `http://127.0.0.1:5173/#plugin-browser-candidate`
- 分解骨格: `http://127.0.0.1:5173/#skeleton`
- Storybookと試験: [mocks-ui README](mocks-ui/README.md)

## 更新チェック

1. 変更する意味と状態所有が規範層にあるか確認する。
2. 現行prototypeの対象hashと、React-native／bridgeの所有境界を特定する。
3. 先例やスクリーンショットは観察台帳へ記録し、観察と採用を分ける。
4. Rerunを参照する場合は監査commit、対象crate/file、転移分類、持ち込まない意味を記録する。
5. React candidate、操作試験、component map、採否台帳を同じ判断単位で更新する。
6. 旧HTMLへ新機能を追加しない。parity維持に必要な変更だけを許し、置換後は削除候補にする。

会話中に論点が広がった場合も、この順序を適用する。新しい用語、用途、状態所有、操作、配布単位、既存決定との矛盾が出た時点で、コード変更を続ける前に観察／比較中／決定／棄却／停止のどれかを台帳へ記録する。まだ雑談の範囲で実装判断へ影響しない案は記録を強制しない。
