# UI作法の機械検証

「動画ソフトで普通」を、初期値の正解や主観スコアに変換しない。外部資料は候補を
集めるために使い、候補の採否は実装をブラックボックスで動かした観測に委ねる。

## 判定の境界

```text
外部4製品の語彙源
        ↓  候補列挙・製品数・矛盾検出
候補の収束(GREEN) / 同率(HOLD) / 矛盾(RED) / 問いの欠落(ORACLE_GAP)
        ↓  候補を仕様にし、実装を probe で操作
control → meaning → evaluation → render → observable
        ↓
実行結果(UNEXECUTED のままなら未検収)
```

`GREEN` は外部作法の候補が収束しただけで、Motolii の合格ではない。`HOLD` は
候補を一つに選べない。`ORACLE_GAP` は「初期値」「既定値」など、外部コーパスが
そもそも答えていない問いである。ここで人が中央や左上を選んで埋めてはいけない。

## 実行

```bash
python3 scripts/check_ux_conventions.py "$(git rev-parse --show-toplevel)" \
  --query anchor --question initial-default
python3 scripts/check_ux_conventions.py "$(git rev-parse --show-toplevel)" \
  --query 'Timeline panel' --question behavior
```

`normal-map.tsv` は4製品の外部語彙源を正規化した候補索引、`docs/reviews/2026-08-21-
normal-map-sources/` は原典行とURLである。検査器は台帳の `verdict` を合否へ使わず、
製品列・出典タグ・矛盾表記・原典行だけを読み取る。JSON出力は後続の black-box
probe の入力にできる。

## 今回のアンカー検証で言えること

アンカーには `Anchor Point`、`Center Anchor Point in Layer Content`、
`Center anchor point in visible content` という複数の作法があり、初期値を明示した
原典行はない。したがって検査器の答えは「中央」ではなく `ORACLE_GAP` になる。
既存コードにアンカーの操作面と純関数テスト名があることは別に観測できるが、最新の
実窓または black-box probe を通すまでは `UNEXECUTED` である。

## UIの読みやすさの一括検査

色と固定寸法は、各ペインが個別に採点するのではなく、共通JSONと共通検査器で判定する。
色の正本は `ui/motolii-tokens/sources/motolii-dark.json`、読みやすさの閾値は
`next/ui/motolii-tokens-rs/tokens/readability.json`、寸法の正本は
`next/ui/motolii-tokens-rs/tokens/dimensions.json` である。

```bash
python3 scripts/check_ui_readability.py "$(git rev-parse --show-toplevel)"
```

検査器は通常文字4.5:1、非テキスト3:1を色ロールの組み合わせへ適用し、UIソースに
非ゼロの `Length::Fixed` が残っていないかを検査する。ゼロ余白、作品データ色、意味を
表す値は対象外である。基準は [WCAG 2.2 Contrast Minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html)
と [Google Developersのコントラスト指針](https://developers.google.com/tech-writing/accessibility/self-study/sufficient-contrast)
に置き、数値の採点を人の手に戻さない。

## デザイン数値の抽出と反映

`check_ui_readability.py` は既に正本へ移された値の閾値を検査する柵であり、ソースに
残るデザイン数値を正本へ逆引きする器ではない。そのためUIを追加する時は、先に次の
生成器で数値とtoken参照を抽出する。

```bash
python3 scripts/derive_design_values.py "$(git rev-parse --show-toplevel)" --write
python3 scripts/derive_design_values.py "$(git rev-parse --show-toplevel)" --check
```

生成物 `next/reference/generated/design-values.tsv` は、デザイン文脈の数値ごとに
`source(file:line)`、文脈、値、`suggested_token`、正本照合結果を持つ。`dims.*`/
`colors.*` の参照は正本に存在することを緑で確認し、`dims.components.*` は
`dimensions.json` のコンポーネント名前空間まで照合する。raw literal は反映先が分かるまで
赤で残す。UIでない検証幅・波形サンプリング・オフスクリーン器具の値は `GREEN_POLICY` として
台帳に残すが、共通寸法へ混ぜない。フレーム数・時間・作品データの数値をデザイン値へ誤登録
しないよう、抽出対象はUIの寸法・文字サイズ・余白・境界・操作面のsinkに限定する。

この生成器はJSONへ意味不明なキーを自動追加しない。自動化する範囲は「抜き出す」「反映先を
示す」「実装が正本を読むことを検査する」までで、名前を決められない値は赤のまま次の
コンポーネント作成へ渡す。反映後はdebugのtoken watcherが同じJSONを再読込するので、
台帳・正本・実窓が別々の数字を持たない。

### Tailwindから採る根の構造

Tailwindの公式設計では、低層の設計判断をtheme variableへ置き、そこからutility APIを
生やす。utilityは既定の拘束されたスケールを選ぶ入口であり、component classは複雑な
構造をまとめる必要がある場合だけ使う。この関係は [Theme variables](https://tailwindcss.com/docs/theme)
と [Adding custom styles](https://tailwindcss.com/docs/adding-custom-styles) に明記されている。

MotoliiではCSS classの代わりに `Dimensions::theme()` を共通utility APIとする。

```text
JSON正本(Dimensions)
        ↓  ui_scale適用済み
theme.space / theme.text / theme.size / theme.stroke / theme.target
        ↓  paneが組み合わせて使う
Browser / Stage / Inspector / Timeline のコンポーネント例外
```

共通値は `dims.theme().space.m`、`dims.theme().text.body` のように意味名前空間から
読む。pane固有の比率、分割数、命中半径、グラフの制御寸法は `dims.components.*` に残す。
従って「新しいコンポーネントだから共通値を複製する」「共通に見えるから別paneの比率を
借りる」の両方を禁止する。Tailwindの arbitrary value に相当する一回限りの値は、raw
literalのまま赤で台帳に出し、責任を持つcomponentへ移してから緑にする。

utilityは値を二重化する場所ではない。`Dimensions::theme()` は既存JSONから名前付きの
読み口を組み立てるだけで、`ui_scale` の適用点も増やさない。これによりJSONをhot reload
した時に、共通utilityとpane例外が同じ一枚の正本から更新される。

## Motoliiの二層基準

配置と概念はAbleton Liveを基準にする。Browserを素材の入口、Stageを結果の面、Inspectorを
調整面、Timelineを線形の編集面として分け、Timelineのトラックを縦に積む。これは
[Ableton Arrangement View](https://www.ableton.com/en/manual/arrangement-view/) と
[Ableton Live Concepts](https://www.ableton.com/en/manual/live-concepts/) を隠さず出典にする。
実装上の初期配置・タブ順・比率は `presentation/presentation.json` に置き、起動時も同じJSONから
読む。Rust側に別の初期配置を持たない。

見やすさはGoogle DevelopersのAccessibilityを基準にする。左から右・上から下の読み順、
標準的な状態表現、キーボード到達性、色だけに依存しない状態表示を優先する。通常文字の
コントラスト、フォーカス境界、操作対象の最小面は `readability.json` と共通検査器で判定し、
ペインごとの主観スコアには戻さない。配置の出典と可読性の出典は混ぜず、変更時にどちらを
動かしたかをコードとJSONから追えるようにする。

色は Ableton Live を基準にする。正本は Ableton Live 12.1 の
`Default Dark Neutral High.ask` を参照した `ui/motolii-tokens/sources/motolii-dark.json` で、
暗い面、明るい文字、オレンジの操作状態、シアンのデータ色へ意味役割を写像する。
Ableton Live は Theme 設定で配色を選ぶ設計を公式マニュアルに記載している。
Google/Material は可読性と到達性の検査にだけ使い、色の値を混ぜない。

この配色方針から採る根は次の通りである。

- 面を画面の大部分にし、二次領域はまず余白で分ける。線や領域を増やして区切らない。
- 寸法は単独の値でなく、4/8系の基準グリッド、文字・アイコン・容器の階層として決める。
- 色は hex の似姿ではなく、`surface`/`on-surface`/`outline`/`primary`/`container` の
  意味役割とペアで決める。作品データ色とUI chromeの色を混ぜない。
- 色・大きさ・位置だけを状態の唯一の手掛かりにしない。文字、形、フォーカス、操作結果を
  別の契約として持つ。

このため、現在の `tokens/dimensions.json` の `ui_scale=1.18` は採用根拠のない暫定値であり、
空間を全寸法へ一律に掛ける実装を最終仕様にはしない。次の変更では、まず上の根を
`surface/spacing/type/state` の契約へ分け、その後に各比率を決める。

## 出典

- [UI-Design Driven Model-Based Testing](https://eceasst.org/index.php/eceasst/article/view/1609)
- [Model-Based Contract Testing of Graphical User Interfaces](https://www.jstage.jst.go.jp/article/transinf/E98.D/7/E98.D_2014EDP7364/_article/-char/en)
- [TOM: Model-Based Testing of Graphical User Interfaces](https://haslab.github.io/TRUST/papers/facs17.pdf)
- [Visual Testing of GUIs by Abstraction](https://arxiv.org/abs/2007.10419)
- [Material Design: Structure](https://m1.material.io/layout/structure.html)
- [Material Design: Metrics & keylines](https://m1.material.io/layout/metrics-keylines.html)
- [Android Developers: Color](https://developer.android.com/design/ui/mobile/guides/styles/color?hl=en)
- [Android Developers: Material 3 in Compose](https://developer.android.com/develop/ui/compose/designsystems/material3)
- [Google Developers: Accessibility](https://developers.google.com/style/accessibility)
- [Ableton: Arrangement View](https://www.ableton.com/en/manual/arrangement-view/)

これらは候補の収集・抽象状態・契約・black-box oracle の分離を支持する資料であり、
アンカーの初期値そのものを決める出典ではない。
