# M3 Browser panelをReactモックからeguiへ翻訳する実験

日付: 2026-07-20

状態: **SPIKE／比較中。設計根拠・製品component・M3タスク完了として扱わない**

## 問い

ReactモックのBrowserをDOM/JSXごと変換せず、制限付きUI IRとFlex/Grid layoutだけを
共有する小さな変換パイプラインで、eguiへ現実的な工数で翻訳できるか。

## 実験範囲

- `#plugin-browser-candidate`にある3面で反復する
  `Search / Sources / Results / View switch / item card`だけを対象にする
- JSON fixtureを検証して非公開`BrowserPanelSpec`のRust定数へ生成し、eguiへ投影する
- `egui_taffy 0.13`のFlex/Gridで、カードがpanel幅に応じて折り返すことを確認する
- rendererは渡された矩形の内側だけを描き、window、split、tab、dockを所有しない
- 既存U1a-1のStage、Inspector、Timeline、native texture経路は変えない
- 同じfixtureを固定viewportでReactとeguiの両方に表示し、構造、密度、
  resize時の破綻、Dark/Lightでの破綻、実装差分を比較する

## 状態所有

この実験で追加するitem、検索文字列、選択tab、view modeはすべて
**SPIKE fixtureまたはTransient表示状態**である。Document、User settings、
Workspace-session形式、journal、Undoへ保存しない。

## 非目標／停止線

- React/JSX/CSSの汎用parserやHTML/CSS互換engineを作らない
- IRはBrowser fixtureのデータに限定し、任意widget、animation、gradient、mask、
  absolute positioningを表現しない
- `Media / Create / Effects`の一次分類を採択しない。P41未統一を解消しない
- Reactモック、`docs/mocks/`、Document、公開plugin API、公開Rust APIを変更しない
- CSSの色、px、radiusを製品tokenへ昇格しない。G0-6Hを完了扱いにしない
- color/theme値とpanel配置をIRへ含めない。製品化時の見た目は解決済みsemantic
  tokenだけを受け、外側layoutはMotolii所有model→`egui_tiles`投影へ任せる
- Browserの検索、D&D、適用handoff、生成、永続layoutを実装しない
- `egui_taffy`の型を`motolii-ui`外へ出さない

公開API、永続形式、Document意味、plugin契約の変更が必要になった時点で実験を止める。

## 合否

次を満たせば、`egui_taffy`をBrowser系のlayout補助として次の比較候補へ残す。

1. egui 0.35の既存shellへ依存競合なく組み込める
2. 1つの検証済みfixtureからRust定数を決定的に生成し、railとcard gridを描画できる
3. panel resizeでcard列が折り返し、文字が縦1文字ずつに崩れない
4. 同じrendererがDark/Lightと狭幅/広幅で動き、色値・panel位置をfixtureへ要求しない
5. toolkit依存が`motolii-ui`から漏れず、既存U1a試験が全緑
6. Reactモックとの差が「不足機能」と「layout翻訳限界」に分けて説明できる

次のいずれかなら不採用または限定採用とする。

- eguiの二重passやintrinsic sizingが既存shellで不安定
- card程度のlayoutにも大量の独自measure/absolute positioningが必要
- keyboard focus、scroll、accessibilityを標準egui widgetから外す必要がある
- crateの導入範囲が`motolii-ui`を越える

## パイプライン仮説

```text
React fixture / component props
        ↓ 明示export（DOM/CSS解析はしない）
制限付きUI IR (JSON)
        ↓ schema・ID・語彙を検証
生成済みprivate Rust定数
        ↓
共通egui renderer + egui_taffy
```

製品化する場合も、layoutと部品データを生成対象にし、interaction、Document command、
Stage、Timeline、custom paintは手書きadapter側に残す。未知のIR nodeは似た見た目へ
黙って変換せず、build時に拒否する。

パネル自身は配置を知らない。`Browser`という役割と内部componentだけを生成し、
組み込みpreset、任意split/tab/resize/hide/restore、将来の別window投影は
Motolii所有panel layout modelが決める。`egui_tiles::Tree`や`TileId`もIRへ保存しない。

## 外観履歴の意味への昇格候補

2026-07-20、ユーザーは`#plugin-browser-candidate`の外観を気に入っていたと明示した。
これはBrowser面に限る**G0-6H部分証拠**であり、5画面全体の審判完了ではない。
React/CSS値を保存するのではなく、実画面比較後に次を採否する。

- dark neutralを土台に、面全体を強い色で塗らない
- 左の階層railと右の結果面を、余白ではなく細い境界と明度差で分ける
- itemはthumbnail/glyphを主、名前とkindを1行の従とする
- 高密度でも検索、source、結果の順序と所在が常時見える
- cardを独立した広告面にせず、同じgrid語彙へ揃える
- 選択や領域識別の色は小面積で使い、icon、outline、位置と併用する

固定hex、shadow、radius、panel幅は昇格対象外である。Dark/Light/custom themeは同じ
semantic roleから解決し、外観の意味を保ったまま値だけを差し替えられることを要求する。

## 検証コマンド

```sh
cargo test -p motolii-ui
cargo tree -d
./scripts/check-docs.sh
git diff --check
```

目視比較はReactモックとnative eguiを同一Macで起動し、固定幅と狭幅の2条件で記録する。

## 実験結果

判定: **部分合格。限定パイプライン候補として継続し、製品採用は未決**

### 合格した範囲

- `egui_taffy 0.13`は既存egui 0.35 / wgpu 29のU1a-1 shellへ依存競合なく入った
- JSON fixtureをbuild時に検証し、非公開Rust定数へ生成して同じrendererへ渡せた
- Explorer / Plugins / Genの切替と検索をnative egui実画面で操作確認した
- `style`等の未知fieldを受ける逃げ道をgenerator側で拒否する形にした
- `egui_taffy` / `taffy`を公開型走査のtoolkit禁止語へ追加し、
  `motolii-ui`外へ型を出していない
- Reactモックの左rail＋右results、thumbnail/glyph優位、1行labelという構図を、
  色値をコピーせずeguiへ翻訳できた

### 実験で見つかった負例

最初のrail実装では`Generators`が1文字ずつ縦へ折れた。`egui_taffy`のintrinsic
text sizingへ任せると、モックの「全label 1行固定」が失われる。rendererの既定を
有限幅＋`TextWrapMode::Truncate`へ変更し、`Genera…`相当の1行表示へ修復した。

この負例から、text wrap、min width、overflowは各画面で調整する値ではなく、
IR node/component roleごとの共通変換規則にする必要がある。

### 未合格／この実験が証明していないこと

- React component propsからIRを明示exportする前半は未実装。今回のJSONは手で作った
  対応fixtureであり、React→egui自動変換の完了ではない
- 現IRはBrowserのitem/sourceデータだけで、Row / Column / Grid等の汎用layout
  node schemaはまだ持たない
- Darkでの構造比較は実施したが、U0e semantic token生成物を統合していないため、
  Light/custom theme適合は未審判
- 左panelのegui resize入口は残したが、Motolii所有layout model→`egui_tiles`による
  split/tab/dock/hide/restoreはU1a-2の責務であり未審判
- keyboard focus、scroll、AccessKit、pseudo-locale、CJK、長いprovider名は未審判
- Stage、Timeline、Graph、custom paintをこの方式で変換できるとは主張しない

### 次の分割候補

1. **Browser IR決定**: `Surface / Search / SourceRail / ResultGrid / ItemCard`の閉じた
   node集合、semantic token role、unknown拒否、custom renderer escapeを文書化する
2. **React exporter**: component propsと`component-map.json`からIRを明示exportし、
   DOM/CSS parserを使わない
3. **決定的compiler**: schema検証、生成hash、手編集拒否、負例fixtureを独立させる
4. **egui adapter**: allocateされた矩形へだけ描き、truncate、wrap、scroll、
   focus、accessibilityをrole既定として実装する
5. **U0e結合**: Dark / Light / custom themeを同じsemantic roleで投影する
6. **U1a-2結合**: 同じ生成panelをsplit/tab/resize/hide/restoreした時の不変条件を審判する

1〜4は一括の汎用UI frameworkとして発注せず、Browserの1 surfaceで閉じる。
5と6は既存タスク境界を再利用し、変換パイプライン側へtheme保存やdock正本を作らない。
