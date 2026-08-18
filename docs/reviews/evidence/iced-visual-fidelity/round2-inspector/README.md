# iced Inspector 視覚再現 — round 2(2026-08-19)証拠

## 前提

round 1(`docs/reviews/evidence/iced-visual-fidelity/README.md`)後、利用者が
実機で Inspector を目視して5点の残差を報告(この round のブリーフ本文)。
基準は変わらず `docs/mocks-ui/public/inspector-library.css` +
`inspector-library.html`(egui 側 `inspector_panel/theme.rs` は同じ css の
写しなので対応表として読んだだけ、コードは写していない)。

## 並び絵

| 面 | ファイル |
|---|---|
| 基準(mocks-ui) | `mock-inspector-reference.png`(利用者支給、`/tmp/motolii-design-reference/inspector-reference.png` の写し) |
| iced 実物(round2 後、`--screenshot` 実測) | `iced-inspector-seated.png`(1960×1300 = 980×650 論理 ×2) |
| 同、Inspector 部分を切り出し | `iced-inspector-crop.png` |

撮り方:
```
MOTOLII_SELECT_LAYER=0 cargo run -p motolii-shell-iced -j 5 -- \
  --project /tmp/verify-proj.json --screenshot /tmp/insp-round2.png 150
```

## 潰した差(5点の指示ぜんぶに対応)

### 1. property 行の左帯 + kind icon が無かった

**原因は実装済みの帯が高さ0で潰れていた不具合。** `inspector_pane.rs` の
`property_row` / `effect_header` は最初から `--property-color` の帯を
`color_bar(ROW_BAND_W, 0.0, band)` で描こうとしていたが、第2引数(高さ)を
固定 `0.0` で渡していた(panel header の accent bar 用に書いた同じ関数を
使い回した名残)。行の実高さは可変なので、固定 0 は「高さ0の帯」= 見えない、
になっていた。修正は `row_band()`(新関数、`height(Fill)`)に差し替え —
`inspector-library.css:291 .propertyRow::before` の写し。

kind icon は新規追加。`inspector-library.css:313-326 .propertyName i`
(15×15、border と字が両方 `--property-color`)を単一グリフへ簡約:

| param | glyph | 出所 |
|---|---|---|
| Position | `+` | css の crosshair アイコンの意味を1字に簡約 |
| Rotation | `↻`(U+21BB) | css の回転アイコンの意味を1字に簡約 |
| Scale | `↔`(U+2194) | css の対角矢印アイコンの意味を1字に簡約 |
| Opacity | `◐`(U+25D0) | css の半月アイコンとほぼ同じ字そのもの |

Fill は口が無い(intent が無い)ので割り当てていない(Q0、ブリーフの指示
どおり)。角丸は css 側にも無いので足していない(Ableton 風フラット、
2026-07-14 裁定)。

### 2. section 見出しが弱い

`TRANSFORM`/`APPEARANCE`/`EFFECTS` の文字自体は元から全大文字
(`pub const TRANSFORM: &str = "TRANSFORM"` 等)・ExtraBold だったが、
下罫線が無く、他の帯と地続きに見えていた。`section_heading` に
`rule::horizontal(1)`(`crate::theme::style::separator`)を足して、
inspector-library.css:141-148 の `border-bottom` を写した(`panel_header` /
`identity` / `column_header` も同様に不足していた下罫線を足した — コード
コメントには「呼び出し側が積む」と書いてあったが実際には積んでいなかった)。

**足さなかったもの**: 折り畳み `˅` chevron。read-model
(`InspectorModel`)は section の折り畳み状態を持たず、押しても何も起きない
ボタンは Q0(触れそうで触れない物は不合格)違反になる。round1 からの
既存判断を継続。字間(letter-spacing)も iced のこの fork には無い口で、
文字間隔を空白文字で誤魔化すのも不正確なので入れていない(残差として
下に明記)。

### 3. 値が枠付き入力欄に見える

`widgets/scrub_value.rs` の `draw()` が常に `border width 1.0 radius 3.0`
の枠付きボックスを描いていた(iced widget の既定的な「ボタンらしさ」)。
inspector-library.css:369-392 `.valueCell` は角丸も枠も無く、地の色も
`surface_app` 側へわずかに寄せた沈んだ色(74% mix)で、hover で
`--property-color` を9%だけ混ぜ、drag/編集中だけ `box-shadow: inset 0 0 0
1px var(--property-color)` が立つ、という設計。`draw()` をこの状態機械へ
書き直し、`radius: 0.0`・idle/hover は border 無し・drag/編集だけ
`--property-color`(行ごとに違う、`ScrubSpec` に `accent: Color` を新規に
追加して呼び出し側の帯色をそのまま渡す)の 1px 枠、に変えた。

X/Y の字(`.valueCell > b`, css:410)も `text_muted` 固定だったのを
`--property-color` へ変更(行ごとに色が違う、reference のとおり)。

列そろえ: `column_header` の padding が `left(11).right(6)` で
`property_row` の content padding `[4,8]` と1〜3px ずれていたのを、両方
`left(8).right(8)` へそろえた(X/Y 見出しの右端が値セルの右端の真上に
来る)。

**直さなかった差**: 基準は Position/Scale が **1行に X と Y が並ぶ**
(`.propertyRow` の grid が `Property | X | Y | Z | key` の5列)。この
iced pane はband width 300px(基準 496px)しか無く、round1 の判断で
「header 行 + X 行 + Y 行」の縦積みに崩してある(`transform_rows` の
コメントに理由が残っている)。この round のブリーフはこの構造差を
指示していないので触っていない — 残差として明記する。

### 4. APPEARANCE section が無い

`inspector_pane.rs` に `appearance()` を新設。`model.transform` から
`UiEditParam::Opacity` の行だけを抜いて `APPEARANCE` 見出し付きの
section として TRANSFORM の下・EFFECTS の上に置いた(inspector-library.
html:29-60 の並びと同じ)。**モデルは一切変えていない** —
`InspectorModel.transform` の中身・順序・`ParamRow` は既存のまま、
`inspector_pane.rs`(view 層)でどの section へ描くかを分けただけ。
Fill は intent が無いので出していない(Q0、ブリーフの指示どおり)。

### 5. 列見出しの字間・色・下罫線

下罫線は #2 で直した(`column_header` にも `rule::horizontal(1)` を追加)。
色は元から `text_muted` で css の `--mock-role-text-muted` と一致していた
(変更不要)。字間は #2 と同じ理由で見送り(iced のこの fork に
letter-spacing の口が無い)。

## 直さなかった差(理由つき)

- **letter-spacing 全般**(section 見出し・column header・FX badge 等):
  iced のこの fork の `text` widget に letter-spacing の API が無い。
  round1 から持ち越しの残差(`inspector_pane.rs:224-232` の
  コード comment にも明記されている)。
- **X/Y が1行に並ばず縦積み**: 上記#3 参照。pane 幅 300px の既存制約で、
  この round の指示範囲外。
- **fold chevron 無し**: 上記#2 参照。read-model が折り畳み状態を持たない
  ため、Q0(死に chrome 禁止)により意図的に足していない。
- **Rotation / Opacity / APPEARANCE / EFFECTS が既定スクロール位置で
  見えない**: round1 から持ち越しの制約(pane の可視高さに対して行が
  多い、`.tableScroller{overflow:auto}` の写しである `scrollable` に
  包んである)。今回のスクリーンショットも既定スクロール位置(先頭)を
  撮ったため、Position/Scale までしか写っていない。**別の検証**として
  `tests/inspector_drive.rs` に一時テスト(`scratch_round2_visual_probe`、
  検収後に revert 済み・コミットには含まれない)を足し、
  `ui.find(inspector_pane::APPEARANCE)` / `ui.find("◐")` / `ui.find("↻")`
  が全て見つかることを確認した — APPEARANCE section と Rotation/Opacity
  の kind icon は構築時に panic せず、ツリーに実在する(座標までは
  この方法では見えないが、Position/Scale と同じ `transform_rows` /
  `kind_icon` 経路を通るので構造的に同じ結果になる)。
- **FX param 行(Turbulent Displace 等)の kind icon / bracket 罫線**:
  この round の指示は host TRANSFORM/APPEARANCE(Position/Rotation/
  Scale/Fill/Opacity)だけに絞っていたので、FX param 行(`effect_param_row`)
  には触っていない。

## token へ追加した値

**0件。** 新しい色は全部 [`crate::theme::Tokens`] 既存 role の再利用
(`Tokens::DARK` の該当 field は round1 と同じ出所 —
`inspector-library.html:29-60` の `--property-color` 割当)。scrub_value の
新しい状態(sunken / tinted)も `theme::mix()` で既存2色から導いただけで、
新しい hex は1つも書いていない。

## テスト集計

`cargo test -p motolii-shell-iced -j 5` — 25 test binary(unittests含む)、
**127 tests、全 green**(既存分含め1件も落としていない)。
`widgets_scrub_value.rs` は `ScrubSpec` に `accent` field を足した分だけ
3箇所の struct literal を更新した(語彙・assertion は無改変)。

## 変更ファイル

- `crates/motolii-shell-iced/src/inspector_pane.rs`(行の帯修正・kind icon・
  APPEARANCE section・下罫線・列そろえ)
- `crates/motolii-shell-iced/src/widgets/scrub_value.rs`(`ScrubSpec::accent`
  追加、`draw()` を css の状態機械へ書き直し)
- `crates/motolii-shell-iced/tests/widgets_scrub_value.rs`(新 field ぶんの
  struct literal 更新、語彙は無改変)
