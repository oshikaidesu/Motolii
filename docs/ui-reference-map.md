# M3 UI参照地図

更新日: 2026-07-19

M3 UIを調べる時は、資料の新旧ではなく次の層で参照先を決める。会話履歴、スクリーンショット、旧HTML、React prototypeのいずれも、単独では製品仕様にならない。

## 参照順位

| 層 | 役割 | 正本／入口 | 変更時の規則 |
|---|---|---|---|
| 規範 | 状態所有、Undo、入力、意味、受け入れ条件 | [M3仕様](specs/M3-ui-integration.md)、[UI操作言語](ui-interaction-language.md)、[UI視覚言語](ui-visual-language.md)、[UI境界規律](reviews/2026-07-14-m3-ui-boundary-prevention.md) | prototypeや会話から直接上書きせず、仕様・決定台帳を先に改訂する |
| 現行prototype | 現在ブラウザで比較する操作・構成 | `docs/mocks-ui/README.md`(React/Viteモック。`codex/m3-mock-components`ブランチ側に実体) | hash fixture、操作試験、比較台帳を一緒に更新する。React/CSS値を製品契約へ焼かない |
| 採否台帳 | 先例、観察、未決、棄却、停止線 | `reviews/`の対象別decision／observation ledger | 出典、Motoliiへの翻訳、反映先を分ける |
| 移行互換 | React移行中の視覚parityと未置換領域 | [旧HTMLモック台帳](mocks/README.md)、`mocks-ui/src/legacy/` | 新しい判断を追加しない。React-native置換後に参照専用へ縮退する |
| 証拠 | ユーザー撮影画像、golden、操作記録 | `reviews/evidence/`、Playwright結果 | 版、OS、fixture、viewport、操作列をmanifest化する |
| 履歴 | Codexタスク、git履歴 | Codexタスク一覧、git log | 決定の探索にだけ使い、現行仕様として引用しない |

## React移行の実状態

「Reactへ移行済み」は実行基盤については正しいが、全surfaceがReact-nativeになったという意味ではない。

| fixture／領域 | 実装状態 | 現在の用途 |
|---|---|---|
| `#plugin-browser-candidate`のBrowser | React-native candidate | Discovery Browserの比較対象 |
| `#plugin-browser-candidate`のEasing Graph view | React-native candidate＋legacy state adapter | AM差分の操作比較。区間導出とcurve状態はまだfixture adapter |
| `#plugin-browser-candidate`のStage / Inspector / Timeline本体 / Settings | legacy HTMLをparseしたbridge | React候補と同じ画面で周辺文脈を保つ移行互換層 |
| `#archive/all-surfaces`等 | legacy HTMLをparseしたarchive bridge | 旧画面との視覚parity回帰。通常catalogへ出さない |
| `#skeleton` | React-native分解骨格 | component責務と組立境界の確認。視覚正本ではない |

したがって、React上で表示されるだけではReact-native所有へ移ったと判定しない。`src/legacy/LegacyHostBoundaryScreen.jsx`またはraw HTML由来のDOM／scriptへ依存する領域は、旧仕様を増やさず、置換対象として台帳へ残す。

## 統合モックの面 → 実装レーン対応(2026-07-19操作確認スナップショット)

`#plugin-browser-candidate`は名前に反してBrowser単体ではなく、M3後半までを含む統合モックである。2026-07-19の操作確認時点の対応(現在地は日付時点のスナップショットであり、正本は[M3仕様](specs/M3-ui-integration.md)のタスク表):

| モック内の面 | 対応する実装領域 | 2026-07-19時点 |
|---|---|---|
| egui shell・可変panel・Stage | U1a/U1b/U1f | U1a-1から着手可能 |
| Effect Inspector・自動parameter panel | U4a | 基盤依存待ち |
| packed Timeline・Group展開・選択 | U3a/U3b/U2h | 後続 |
| Automation展開・Key Tools | P56/P60+U3系 | 一部prototype判断のまま |
| Interval Easing・multi-key Graph View | U4b/U4e | 正式タスク化済み、後続 |
| Effects Browser | U4d | 正式タスク化済み |
| Media Browser・folder・Tag・複数選択 | U6 | 正式タスク化済み |
| Create Browser・provider・generator | U9/Vism/Create境界 | 一部未統一 |
| Depth Rail・分配 | ui-score/M5系 | M3だけでは閉じない |
| readiness・rendering・stale表示 | U3f | M4 provider待ち |

**発見入口だけが存在する面**: `Type Pulse`はEffects/Createの両面へカードとして出るが、選択してもStage/Inspectorは`Echo Bloom`のまま。つまり現行モックが持つのはText Motionの**発見入口**までで、`適用先preflight → Live Text生成/Animator追加 → Inspector切替 → Character Score展開 → Stage文字選択`のhandoffは未モックである。この接続は[TM翻訳](reviews/2026-07-19-m3-text-motion-task-translation.md)の後続であり、TM第1弾はBrowser非依存(通常のObject作成経路)で進める。

三面構成(`Media / Create / Effects`)は下表のとおり**P41未統一のまま**であり、そのままegui/製品へ写さない。

## 既知の未統一

現時点で次を一つの「現行仕様」として読んではならない。

| 論点 | 規範／記録 | React prototype | 扱い |
|---|---|---|---|
| Browser一次分類 | [UI操作言語](ui-interaction-language.md)には`Media / Plugins`が残る | `Media / Create / Effects` | **未統一**。prototype台帳P41で比較中。名称をDocument型、package kind、公開APIへ焼かない |
| Easing Graph | M3 U4bは区間中心GraphとBezier編集を要求。高度型も2026-07-10に区間補間として採用済み | viewはReact候補へ置換し、AM差分とBounce / Elastic / Cyclic(Sine) / Random / Steps / Elastic Stepsを操作試験化。高度型の適用前後でkeyframe構造不変。区間導出とcurve状態はlegacy fixture adapterが残る | [AM観察台帳](reviews/2026-07-19-am-keyframe-graph-observation.md)で残差追跡。state adapter撤去と製品`Interp`接続を分ける |
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
- Storybookと試験: `docs/mocks-ui/README.md`(`codex/m3-mock-components`ブランチ側に実体)

## 更新チェック

1. 変更する意味と状態所有が規範層にあるか確認する。
2. 現行prototypeの対象hashと、React-native／bridgeの所有境界を特定する。
3. 先例やスクリーンショットは観察台帳へ記録し、観察と採用を分ける。
4. React candidate、操作試験、component map、採否台帳を同じ判断単位で更新する。
5. 旧HTMLへ新機能を追加しない。parity維持に必要な変更だけを許し、置換後は削除候補にする。

会話中に論点が広がった場合も、この順序を適用する。新しい用語、用途、状態所有、操作、配布単位、既存決定との矛盾が出た時点で、コード変更を続ける前に観察／比較中／決定／棄却／停止のどれかを台帳へ記録する。まだ雑談の範囲で実装判断へ影響しない案は記録を強制しない。
