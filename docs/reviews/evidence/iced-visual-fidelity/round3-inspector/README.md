# iced Inspector 視覚再現 — round 3(2026-08-19)証拠

## 前提

`motolii-css-metrics`(Blitz を使った CSS 計算値の抽出器具)が main に着地し、
前レーンの突き合わせで Inspector の 11 定数は完全一致していた。本 round は
**残差の解消**が主眼(利用者の指示)。

作業途中で運転手から訂正が入った: **Inspector の見た目・構造の正本は
`docs/mocks-ui/public/inspector-library.html` + `inspector-library.css` その
ものであり、egui 版 `crates/motolii-ui/src/inspector_panel/` は「html/css →
egui 変換がうまくいかなかった側」なので手本にしない**。数値だけでなく
html の markup 構造(section の入れ子・class 名・`data-*` 属性・1行の内部
構造)を設計の意図として読み、そこから直す。egui を見てよいのは**既決の
振る舞い**(key ボタン3状態の意味・accepted からのみ導出・reject 時に進めない、
等)だけ。この README はその読み方に沿って書いてある。

## 並び絵

| 面 | ファイル |
|---|---|
| 基準(mocks-ui、利用者支給) | `/tmp/motolii-design-reference/inspector-reference.png` |
| iced 実物(round3 後、`--screenshot` 実測) | `iced-inspector-seated.png`(1960×1300 = 980×650 論理 ×2) |
| 同、Inspector 部分を切り出し | `iced-inspector-crop.png` |

撮り方(ブリーフ指定どおり、120 未満は誤診なので 150 で撮った):
```
MOTOLII_SELECT_LAYER=0 cargo run -p motolii-shell-iced -j 5 -- \
  --project /tmp/verify-proj.json --screenshot <out>.png 150
```

## 1. X / Y の列組み(内部矛盾の解消)

**判断: 1 行に詰める。pane を広げる側は選ばなかった。**

### 根拠(html/css の構造から読んだ意図)

`inspector-library.css:115-120`:
```css
.columnHeader,
.propertyRow {
  display: grid;
  grid-template-columns: minmax(132px, 1fr) repeat(3, 64px) 26px;
}
```
`.columnHeader` と `.propertyRow` は**同じ grid 定義**を共有している —
つまり html の設計意図は最初から「列見出しと値行が同じ列幅を持つ1段の
grid」であり、`<div class="propertyRow" data-control="Position">` の中身
(`inspector-library.html:29-35`)は `.propertyName`(icon+名前)・
`.valueCell`(X)・`.valueCell`(Y)・`.valueCell`(Z)・`.keyButton` を**同じ
行の5個の grid セルとして**並べている。DOM 上も1つの `<div>` で、
「header 行 + X 行 + Y 行」のような入れ子 row 構造は存在しない。

対して訂正前の `inspector_pane.rs` は Vec2 param を「header 行(icon+名前+
key ボタン)+ X 行 + Y 行」の3行へ縦積みにしていた(round1 の判断、round2
は指示範囲外として持ち越し)。一方で [`column_header`] は
`Property | X | Y` の3列だけを見出しに掲げていた。**見出しが3列を約束して
いるのに実物は3行を積む**という内部矛盾が実測(利用者の目視)で指摘された
残差1である。

html/css の grid はそもそも1行しか無いので、正しい直し方は列見出しどおり
**1行に詰める**ことだった。pane 幅(300px、`INSPECTOR_PANE_W`)を css 正本
(496px)に合わせて広げる案も検討したが、300px は他 wave(Browser/Stage/
Timeline の3分割)の既定値でこのレーンの都合だけで動かす理由が無く、
実測の結果 300px のままでも X/Y 2列(64px×2)+ icon(15px)+ label + key
(18px)が余裕を持って収まった(下記実測参照)ので widen しなかった。

### 変更

- [`transform_rows`](../../../../../crates/motolii-shell-iced/src/inspector_pane.rs)
  を全面書き直し。Vec2(Position/Scale)も scalar(Rotation/Opacity)も
  **1 param = 1 行**にし、`row![name, spacer, values, key]` の構成にした。
  Vec2 は `value_cell("X", ...)` / `value_cell("Y", ...)` を横に並べる
  (spacing 0 — grid セルが隙間無く接する css の意図の写し)。scalar は
  `dims::VALUE_COL_W * 2.0`(X+Y 2列ぶん)の1セルに値を出す —
  `inspector-library.css:415 .valueCell.scalar { grid-column: span 3 }`
  の写し(このパネルは Z が無いので2列ぶん)。
- [`column_header`] に **Key 見出し列を新設**。訂正前は Property|X|Y の
  3見出ししか無く、値行が実際に持つ4本目の列(key ボタン)を1本も
  約束していなかった — これが「3列だけ約束」側の内部矛盾だった。html の
  最後の見出しは ◇ グリフそのもの(`aria-label="Keyframe"`)だが、ここは
  **意図的に "Key" という字にした**(下記「単純化」参照)。

### 実測(`motolii-css-metrics -- inspector out.json` の実物)

```
header.columnHeader > span  →  Property 277px / X 64px / Y 64px / Z 64px / Key 26px
button.keyButton             →  26×25px(grid 解決後の実物)
```
`dims::VALUE_COL_W`(64px)は既存どおり css 実測と一致。Key 列は css だと
26px だが、iced 実物の `key_button` widget は
[`crate::widgets::key_button::KEY_BUTTON_SIZE`](../../../../../crates/motolii-shell-iced/src/widgets/key_button.rs)
= **18px**(既存の意味 — この round では触っていない)なので、見出し側の
Key 列幅もその実物に揃えた(26px にすると見出しと実物のボタンがずれる)。

### 単純化した点(既知の簡略化)

- **X/Y の小さいタグの重ね方**: html は `<b aria-hidden="true">X</b>` を
  `position:absolute` で値の上に重ねる(`inspector-library.css:410`)。
  この iced fork の `scrub_value` widget は絶対配置オーバーレイの口を
  持たず(widget 自体を書き換えるのはこのレーンの外)、`value_cell()`
  では字を値の**前に並べる**簡略形にした。数値は右詰めのままなので
  見た目の破綻は無い(実測画像参照)。
- **Key 見出しの字**: html は ◇ グリフそのものを見出しにも置くが、
  ここは "Key" という単語にした。理由は表示ではなく**既存テストの保護**
  — `key_button` widget は実物ボタンの表示文字列としてまさに同じ ◇/◆
  グリフ(`\u{25c7}` / `\u{25c6}`)を使っており、`tests/inspector_drive.rs`
  ・`tests/replay_oracle.rs` は両方とも
  `press(simulator, "\u{25c7}")`(depth-first の先頭 = Position の実物
  ボタン、という前提)でこれを掴んでいる。見出しにも同じグリフを置くと
  `iced_test::Simulator::click` が実物ボタンより先に見出しへ当たり、
  上記2テストを静かに壊す(既存の意味を壊さない柵に抵触する)。実測で
  green を確認済み(下記テスト集計)。

## 2. 疑似要素由来の装飾(`::before`/`::after` 洗い出し)

css を読み、モックにあって iced に無い帯・区切り・アクセントを洗い出した。
**この round で埋めたもの**(上記#1と関連、実在の機能に繋がる分だけ):

| 要素 | css 出所 | 対応 |
|---|---|---|
| `.valueCell > b`(X/Y タグ) | `inspector-library.css:410` | `value_cell()` に追加(簡略形、上記参照) |
| `.columnHeader span:last-child`(Key 見出しの amber 色) | `inspector-library.css:138` | `column_header()` に追加(`action_active`) |

**既に埋まっていたもの**(round2 までの成果、確認のみ・変更なし):
`.panelHeader::before`(accent bar)、`.propertyRow::before`(行帯)、
`.effectHeader` の inset box-shadow(帯として `row_band` で実装済み)。

**埋めなかったもの**(read-model にまだ無い機能に紐づく — Q0)は次節に
まとめる。

## 3. FX param 行の kind icon

前 round は host TRANSFORM/APPEARANCE(Position/Rotation/Scale/Opacity)行
だけに `kind_icon()` を足していた(`dims::KIND_ICON` のコメントに残差として
明記済み)。この round で `effect_param_row()` にも同じ部品を適用した
(`effect_kind_glyph()`)。

css の `data-param-kind` は scalar/vector/angle/integer/boolean/choice の
6種を持つが、`EffectParamValue`(`crate::inspector_model`、書き換え禁止の
read-model)は F64/Vec2/Vec3/Color の4種しか閉じていない。対応できるのは
そのうち scalar(F64 → `•`)・vector(Vec2/Vec3 → `↔`、host Scale と同じ
簡約)・color(Color → `■`)の3つだけで、**angle/integer/boolean/choice の
4種はこの round では表せない**(Q0: 無い語彙を発明しない — 次節に残差として
明記)。

## 4. 古いヒント文の撤去

Inspector の footer 直上に出る `editor_status`(`Shell::editor_status()` →
`TimelineState::status()`)の**既定値**が
`"space=play  L=loop  Cmd+G=group  Del=delete  drag name=reorder"` だった
— iced shell にまだ無い機構(再生・loop)を宣伝し、`Cmd+G`/`drag name` も
口が無い(`crate::shortcuts` の表を見ればどちらも `implemented: false`)。
近道キーレーンが発見して `spawn_task` 済みの実害。

**判断: Inspector からは撤去して status 帯側に一本化した。** 近道キーの
正本は [`crate::shortcuts`] で、実際に効く行だけを窓全体の status 帯
(`view.rs::status_band` → `shortcuts::legend_line()`)が**既に**出している
(2026-08-19 近道キー移植レーンの成果)。Inspector 側は個別行の undo/redo・
削除・拒否理由など「実際に起きた事実」を伝える役目なので、それらは残し、
この1つの固定文字列だけを黙らせた(`STALE_KEY_HINT` 定数で厳密一致フィルタ)。

`TimelineState` は egui 版とも共有する crate(`motolii-ui`)の型なので、
**既定値そのものはここでは書き換えていない**(egui shell 本体を変えない柵)
— 表示側(`inspector_pane.rs`)だけを直した。スクリーンショット
(`iced-inspector-crop.png`)で `"selected starter-still"` という実際の
status が出ており、固定ヒント文は跡形も無いことを確認した。

## 5. oracle の強化(両方向チェック)

前レーンが「`inspector_pane.rs` の private `dims` を `pub(crate)` へ上げれば
真の両方向になる」と提案を残していた。**実際に上げてみると `pub(crate)` では
足りないことが分かった** — `tests/css_metrics_oracle.rs` は
`motolii-shell-iced` の外側にある統合テスト crate で、`pub(crate)` はその
crate からは不可視(privacy checker がそこで止まる)。真に両方向にするには
`pub` が要る。

- `inspector_pane.rs` の `mod dims` → `pub mod dims`(11 定数はすべて元から
  `pub const` だったので中身は無改変)。
- `css_metrics_oracle.rs` の `inspector_dims_match_css_computed_values` を、
  11箇所すべて `motolii_shell_iced::inspector_pane::dims::*` を `use` して
  実物どうしを比較する形に書き換えた(以前は 2026-08-19 時点の実測値を
  literal で転記していただけ)。Timeline 側と同じ「どちらが変わっても
  落ちる」形になった。

## Q0: html にあるが機能が無いので置かなかったもの

運転手からの訂正指示どおり、html/css を読んで見つけた「対応する intent /
read-model がまだ無い」control をすべて列挙する。**どれも今回は繋がなかった**
(繋げられる読み口・書き口が無い):

- **モードタブ(Effect / Custom)**: `.modeTabs` — Custom 拡張(タグ付け・
  ノートのプレビュー機能)のレジストリが無い。常に Effect 相当の内容だけ出す。
- **Z 列**: 製品の transform が2D。3D 転換の予定は無い(round1 からの既存判断)。
- **Fill(色)編集**: 色ピッカー・recent colors。`InspectorModel` に Fill の
  読み書き intent が無い(round1 からの既存判断、継続)。
- **FX Stack toolbar**: Find(絞り込み)・Group(⌘G)ボタン。effect の
  選択・グルーピング機構が read-model に無い。
- **effect select checkbox**(グルーピング用の `○`)・**drag handle**
  (`⠿`、並べ替え)・**effect rename**(`✎`、F2)・**effect の右クリック
  context menu**(copy/paste/duplicate/delete): いずれも対応する `UiIntent`
  が無い(`SetEffectEnabled` 以外の FX intent は存在しない)。
- **ADVANCED 折り畳みサブグループ**とその tree-line 装飾
  (`.advancedToggle::before/::after`): read-model の `EffectParamRow` は
  advanced/basic を区別するフィールドを持たない(フラットな配列)。
- **choiceControl**(列挙値のドロップダウン)・**toggleControl**(On/Off
  boolean)・**seedRandomize**(整数 seed + "New" ボタン): `EffectParamValue`
  が Choice/Boolean/Integer を持たない(F64/Vec2/Vec3/Color の4種のみ)。
  #3 で明記した kind icon の残差と同じ根っこ。
- **FX param 行の key 列**(`.keyButton`/`.keyPlaceholder`): `EffectParamRow`
  は `key_state` を持たない — FX param をキーフレーム化する intent 自体が
  存在しない。host TRANSFORM/APPEARANCE の `KeyPressed(UiEditParam)` は
  host の4 param(Position/Rotation/Scale/Opacity)だけを受ける閉じた
  enum で、FX param の id では呼べない。
- **effect groups**(`.effectGroup`、Ungroup、group の enable/bypass)、
  **parameter の右クリック context menu**(Reset value): 対応する intent
  が無い。
- **section fold chevron**: round1/round2 からの既存判断を継続 — read-model
  が折り畳み状態を持たないので押しても何も起きないボタンは作らない。

## 直さなかった差(理由つき)

- **letter-spacing 全般**: round1/round2 からの持ち越し。iced のこの fork
  の `text` widget に letter-spacing の口が無い。
- **X/Y タグの絶対配置オーバーレイ**: 上記#1の「単純化した点」参照。
- **`.effectSection .keyButton` の property-color 背景tint**: FX param 行が
  そもそも key ボタンを持たない(上記 Q0 参照)ので該当なし。
- **Rotation/Opacity/APPEARANCE/EFFECTS が既定スクロール位置で見えない**:
  round1/round2 から持ち越しの制約(pane の可視高さに対して行数が多い)。
  今回のスクリーンショットも既定スクロール位置(先頭)で撮ったため
  Position/Scale/Rotation の頭までしか写っていない — `iced-inspector-crop.png`
  参照。`cargo test` は `common::scroll_then_click` 経由でスクロール後の
  状態も検証済み(`an_effect_toggle_writes_the_shared_definition`)。

## token へ追加した値

**0件。** Key 見出しの色は既存 `action_active`(round1/round2 と同じ
`--property-color` 割当の再利用)。新しい hex は1つも書いていない。

## テスト集計

`cargo test -p motolii-shell-iced -j 5` — **26 test binary(unittests含む)、
132 tests、全 green**(既存分含め1件も落としていない)。

内訳の主なもの:
- `tests/css_metrics_oracle.rs`: 3 passed(うち `inspector_dims_match_css_computed_values`
  が今回 pub 化した `dims` を直接 `use` して実物どうしを比較する形に変わった)
- `tests/inspector_drive.rs`: 6 passed(◇ の press が Position の実物ボタン
  を掴み続けていることを含め、Key 見出しの "Key" 化が既存の意味を壊して
  いないことを確認)
- `tests/replay_oracle.rs`: 3 passed(同じ ◇ press の別経路)
- `tests/widgets_key_button.rs` / `tests/widgets_scrub_value.rs`: 変更なし、
  green のまま(widget 自体は触っていない)

`red 先行` は本レーンでは失敗する単体テストの形では表現しづらい性質の作業
(縦積み→1行の構造修正は既存の自動テストが検知していない視覚残差だった)
なので、既存テストを green のまま保つこと + スクリーンショットによる
実物比較を受け入れ条件として扱った(下記画像参照。正直な残差は上に列挙した
とおり)。

## 変更ファイル

- `crates/motolii-shell-iced/src/inspector_pane.rs`(X/Y 列組みの1行化・
  Key 見出し新設・value_cell/key_placeholder 新設・FX param kind icon・
  古いヒント文フィルタ・`dims` を `pub` へ)
- `crates/motolii-shell-iced/tests/css_metrics_oracle.rs`(Inspector 側の
  11 assertion を `dims` の実物 `use` へ書き換え、両方向チェックに)
