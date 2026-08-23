# 残作業の割り振り(機械導出)

`scripts/plan_backlog.py` が生成。**手で編集しない。**

**「普通の動画ソフト」の定義は `normal-map.tsv` が持っている。** 残っているのは
`verdict` が `採用予定`/`結線待ち` の行で、どの pane が持つかは `bundle` →
`intent-bundles.tsv` の `home` から引ける。順番は `freq`(4製品中いくつに在るか)降順。

- 残 983件 / crate 9本 / 家が未決 269件

## crate ごと(重い順)— **crate が違えば同時に走れる**

| crate | 残 | freq≥2 | 束 |
|---|---|---|---|
| `ui/motolii-timeline-pane` | 234 | 2 | B15 キーフレーム束, B18 作業範囲/イン・アウト点束(裁定177 headline例), B19 マーカー/フラグ束 ほか5 |
| `ui/motolii-inspector-pane` | 137 | 1 | B01 ブレンドモード束, B02 マスク/マット束, B03 ラベル色束 ほか4 |
| `ui/motolii-stage-pane` | 98 | 0 | B17 カメラ/3Dビュー束, B22 ガイド/グリッド表示束, B23 プレビュー解像度/画質束 ほか2 |
| `shell/motolii-shell` | 73 | 3 | B25 パネル可視性/フォーカス束(最大統合) |
| `ui/motolii-settings-pane` | 56 | 1 | B12 環境設定束 |
| `ui/motolii-menubar` | 42 | 3 | B06 ヘルプ/診断束, B11 File束(MB-1延長), B33 クリップボード/履歴束(MB-0延長) |
| `ui/motolii-browser-pane` | 41 | 2 | B08 素材取り込み束 |
| `ui/motolii-keymap` | 21 | 0 | B45 微調整(ナッジ)束 |
| `ui/motolii-export-pane` | 12 | 0 | B09 書き出し束 |

## 最優先(freq≥2 — 4製品中2つ以上が持つ = 普通度が高い)

| crate | id | freq | 項目 | 意味 |
|---|---|---|---|---|
| `ui/motolii-timeline-pane` | 163 | 3 | Split / Razor (clip at playhead) | プレイヘッド位置でクリップを分割 |
| `shell/motolii-shell` | 981 | 3 | Tools / Toolbar panel | 編集ツール格納パネル |
| `(家が未決)` | 1441 | 3 | Zoom In | ズームイン |
| `(家が未決)` | 1442 | 3 | Zoom Out | ズームアウト |
| `ui/motolii-timeline-pane` | 1315 | 2 | New Sequence / Timeline / Composition | 新規シーケンス・タイムライン・コンポジションを作成 |
| `ui/motolii-browser-pane` | 166 | 2 | Make Subclip | サブクリップを作成 |
| `ui/motolii-browser-pane` | 167 | 2 | Replace (footage/clip) | 素材・クリップを置換 |
| `ui/motolii-settings-pane` | 1145 | 2 | Keyboard Shortcuts (editor) | キーボードショートカットのカスタマイズ画面 |
| `shell/motolii-shell` | 982 | 2 | Effects / Effects & Presets panel | エフェクト一覧パネル |
| `shell/motolii-shell` | 983 | 2 | Info panel | 選択項目の情報パネル |
| `ui/motolii-inspector-pane` | 169 | 2 | Speed / Duration | クリップの速度・長さを変更 |
| `ui/motolii-menubar` | 438 | 2 | Edit Original | 元アプリで編集 |
| `ui/motolii-menubar` | 439 | 2 | Find | 検索 |
| `ui/motolii-menubar` | 440 | 2 | Paste Attributes | 属性・プロパティを貼り付け |
| `(家が未決)` | 164 | 2 | Apply Audio Transition (default, at edit point) | 既定のオーディオトランジションを編集点に適用 |
| `(家が未決)` | 165 | 2 | Apply Video Transition (default, at edit point) | 既定のビデオトランジションを編集点に適用 |

## 家が未決 269件(束が `home` を持たない/語で引けない)

**推測で割り当てていない。** 束の `home` を決めるのが先。

- B26 ワークスペースレイアウト束: 36件
- B32 ロック/可視性/リンク束: 27件
- B46 テキストプロパティ/アニメータ束: 27件
- B40 ソース参照/マルチカム束: 26件
- B24 ズーム束(MA L4例示): 24件
- B36 新規コンテンツ作成束: 20件
- B29 ツール切替束: 18件
- B10 プロジェクト整理束: 18件
- B48 テキストカーソル/選択束(MA L4例示): 15件
- B42 音声内容整形束: 13件
