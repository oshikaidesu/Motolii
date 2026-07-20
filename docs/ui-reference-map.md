# M3 UI参照地図

更新日: 2026-07-19

M3 UIを調べる時は、資料の新旧ではなく次の層で参照先を決める。会話履歴、スクリーンショット、旧HTML、React prototypeのいずれも、単独では製品仕様にならない。

## 参照順位

| 層 | 役割 | 正本／入口 | 変更時の規則 |
|---|---|---|---|
| 規範 | 状態所有、Undo、入力、意味、受け入れ条件 | [M3仕様](specs/M3-ui-integration.md)、[UI操作言語](ui-interaction-language.md)、[UI視覚言語](ui-visual-language.md)、[UI境界規律](reviews/2026-07-14-m3-ui-boundary-prevention.md) | prototypeや会話から直接上書きせず、仕様・決定台帳を先に改訂する |
| 現行prototype | 現在ブラウザで比較する操作・構成 | [React/Viteモック](mocks-ui/README.md) | hash fixture、操作試験、比較台帳を一緒に更新する。React/CSS値を製品契約へ焼かない |
| 採否台帳 | 先例、観察、未決、棄却、停止線 | `reviews/`の対象別decision／observation ledger | 出典、Motoliiへの翻訳、反映先を分ける |
| 移行互換 | React移行中の視覚parityと未置換領域 | [旧HTMLモック台帳](mocks/README.md)、`mocks-ui/src/legacy/` | 新しい判断を追加しない。React-native置換後に参照専用へ縮退する |
| 証拠 | ユーザー撮影画像、golden、操作記録 | `reviews/evidence/`、Playwright結果 | 版、OS、fixture、viewport、操作列をmanifest化する |
| 履歴 | Codexタスク、git履歴 | Codexタスク一覧、git log | 決定の探索にだけ使い、現行仕様として引用しない |

## React移行の実状態

「Reactへ移行済み」は実行基盤については正しいが、全surfaceがReact-nativeになったという意味ではない。

| fixture／領域 | 実装状態 | 現在の用途 |
|---|---|---|
| `#plugin-browser-candidate`のBrowser | React-native candidate | Discovery Browserの比較対象 |
| `#plugin-browser-candidate`のInterval Easing Editor | React-native candidate＋legacy state adapter | AM差分の1区間操作比較。区間導出とcurve状態はまだfixture adapter |
| `#plugin-browser-candidate`のTimeline dock Graph View／`#graph-view-candidate`の操作fixture | React-native candidate | Apple Motion構成、Cinema 4D tangent操作、Maya状態語彙、Blender navigationの比較。共通mock color roleへ接続するが製品state/APIではない |
| `#plugin-browser-candidate`のTimeline本体 / Key Tools | React-native candidate＋legacy state adapter | 一枚のpacking時間面、無名帯アクションrail、Object bar内M/S、帯単位の一括M/Sと表示高調整、Automation済みchannelのAE型縦一覧、Timeline右端dockで`KEYS / LAYERS`を切り替えるKeystone 3型操作面の比較対象 |
| `#plugin-browser-candidate`のStage / Inspector / Settings | legacy HTMLをparseしたbridge | React候補と同じ画面で周辺文脈を保つ移行互換層 |
| `#archive/all-surfaces`等 | legacy HTMLをparseしたarchive bridge | 旧画面との視覚parity回帰。通常catalogへ出さない |
| `#skeleton` | React-native分解骨格 | component責務と組立境界の確認。視覚正本ではない |

したがって、React上で表示されるだけではReact-native所有へ移ったと判定しない。`src/legacy/LegacyHostBoundaryScreen.jsx`またはraw HTML由来のDOM／scriptへ依存する領域は、旧仕様を増やさず、置換対象として台帳へ残す。

## 既知の未統一

現時点で次を一つの「現行仕様」として読んではならない。

| 論点 | 規範／記録 | React prototype | 扱い |
|---|---|---|---|
| Browser一次分類 | [UI操作言語](ui-interaction-language.md)には`Media / Plugins`が残る | `Media / Create / Effects` | **未統一**。prototype台帳P41で比較中。名称をDocument型、package kind、公開APIへ焼かない |
| Interval Easing Editor / Graph View | M3 U4bは1区間editor、U4eは実時間×実値のmulti-key Graph View。両者は別surface | U4bはAM差分と高度区間補間を操作試験化。U4eはTimeline dock統合と独立操作fixtureの双方でApple Motion構成＋Cinema 4D操作を比較 | [AM観察台帳](reviews/2026-07-19-am-keyframe-graph-observation.md)と[Graph View参照・比較決定](reviews/2026-07-19-graph-view-reference-decision.md)を分けて参照し、React JSX／SVGをeguiへ写経しない |
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
4. React candidate、操作試験、component map、採否台帳を同じ判断単位で更新する。
5. 旧HTMLへ新機能を追加しない。parity維持に必要な変更だけを許し、置換後は削除候補にする。

会話中に論点が広がった場合も、この順序を適用する。新しい用語、用途、状態所有、操作、配布単位、既存決定との矛盾が出た時点で、コード変更を続ける前に観察／比較中／決定／棄却／停止のどれかを台帳へ記録する。まだ雑談の範囲で実装判断へ影響しない案は記録を強制しない。
