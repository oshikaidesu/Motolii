# テキスト字形描画ルート裁定 probe(R6)— AE 化の最大欠落を塞ぐ経路の実測

**verdict: 緑(採用ルート確定・切片割り付き)**

- 日付: 2026-08-22
- 発注: 「テキスト字形描画ルートの裁定 probe(AE 化の最大欠落)」— 前任レーンが
  API 上限で中断した続きを完走
- probe: `next/probes/r6-text-shaping/`(merge 前提ではない検証コード。製品 crate
  非接触 — 触ったのは workspace members への1行追加のみ)
- 実行: `cargo test --manifest-path next/Cargo.toml -p r6-text-shaping`(全緑・
  5 test)

## 0. 前提 — 今どこが空いているか(実測で確認)

`motolii-store::LayerSource::Text` は既にある(`layers/text-layer/ty`、`Layer:text`
component)。だが `motolii-engine` の `texture_for`
(`next/engine/motolii-engine/src/lib.rs:622`)は
`LayerSource::Null | LayerSource::Shape | LayerSource::Text | LayerSource::Group`
を同じ枝で「texture を焼かない」扱いにしている — **Text layer は今、1px も
描いていない**。これが発注文の「AE 化の最大欠落」の実体で、この probe はここへ
繋ぐ経路を決める。

## 1. 採用ルート

```
文字列 + TextDocumentStyle(store)
  → cosmic-text 0.19(FontSystem + Buffer, shaping/layout)
  → SwashCache::get_outline_commands(zeno::Command 列)
  → 次数上げ(Quad→Cubic)+ y-up→y-down 変換
  → motolii_vector::Contour/Vertex(頂点相対 cubic ハンドル、Lottie bezier と同型)
  → motolii_vector::PathSource::Bezier → Shape(fill/stroke は store から直写し)
  → motolii_vector::render() → premultiplied RGBA8
```

**依存増分ゼロ**: `cosmic-text 0.19.0`(→ swash 0.2.10 → zeno 0.3.3)は iced fork
(`iced_graphics`)が既に依存グラフへ引いている(`next/Cargo.lock` 実測、direct 化
しても同一 package に unify される)。harfrust/fontique の直組みは比較のため
API 実在を確認したが(下記 §4)、依存を増やす側であり、cosmic-text 経路で英字・
CJK とも合格したため不要と判定 — 発注文の判定軸「保守最低限(既存依存の
再利用 > 新依存)」がそのまま決め手になった。

## 2. 実測(テスト5本、全緑・`cargo test -p r6-text-shaping`)

| テスト | 検証内容 | 結果 |
|---|---|---|
| `a_string_becomes_outlines_then_pixels_then_a_png` | "Motolii" 96px → 輪郭8+(o の中マド・i の点込み)→ raster → 赤fillが画素へ到達 → PNG | 緑 |
| `line_height_is_the_baseline_delta_numerically` | lh=90/140 を指定 → 2行の baseline 差が指定値と ±0.01px で一致 | 緑 |
| `tracking_widens_glyph_advances_numerically` | tr=250(AE 1/1000em)、size=100 → glyph advance が `tr/1000×size`=25px 広がる(±0.05px)。行幅増分も 3glyph 分 ±0.15px で一致 | 緑 |
| `cjk_text_goes_through_the_same_route` | ヒラギノ角ゴシック W3(.ttc)で「字形が画素になる」8文字 → 8 glyph・輪郭8+・可視画素 4,000+ → PNG | 緑 |
| `shaping_is_deterministic_for_the_same_input` | 同一入力を2回組んで完全一致(器具自体の再現性) | 緑 |

生成 PNG を目視確認: `motolii-arial-96.png` は "Motolii" が滑らかな曲線で
描画され、o のカウンター・i の点まで正しく分離。`cjk-hiragino-80.png` は
「字形が画素になる」8文字が字形として正しく分離・接筆も自然
(スクリーンショットで確認済み)。

lh/tr が**画素を数えて「広がった気がする」ではなく layout の座標そのもの**で
固定されている点が、発注の合格線2/3を字義通り満たす。

## 3. 変換の要点(判定材料)

- **outline は swash 経由**: `SwashCache::get_outline_commands` が
  `zeno::Command`(MoveTo/LineTo/QuadTo/CurveTo/Close)を返す。zeno は y-up、
  motolii-vector の正準空間は y-down なので baseline 回りで y を反転する。
- **QuadTo(TrueType の2次)は3次へ次数上げ**(c1 = p0 + ⅔(q−p0)、
  c2 = p1 + ⅔(q−p1) — 厳密変換で近似ではない)。`Vertex` は Lottie `bezier` と
  同型の「頂点相対 cubic ハンドル」なので、そのまま `PathSource::Bezier` に載る。
- **tr(トラッキング)の単位**: store の `tracking` は Lottie `text-document tr`
  = AE の tracking(1/1000 em)。cosmic-text の `letter_spacing` は em 正規化
  された advance へ足される(`cosmic-text-0.19.0/src/shape.rs` 実測:
  `x_advance / font_scale + spacing`、layout で `* font_size`)ので、
  `tracking / 1000.0` の1行で写る — §2 のテストが数値で裏取り済み。
- **lh(行送り)**: `Metrics::line_height` がそのもの。`None`(フォントメトリクス
  準拠)は probe では `size * 1.2` 固定 — 実装切片では swash のフォント
  メトリクスから引く(§5 切片1)。
- **フォントは path で解決する**(`FontRef::path` の意味そのまま):
  `fontdb::Database::load_font_file` → `FontSystem::new_with_locale_and_db`。
  システムフォント走査をしないので試験が決定的で速い。

## 4. API 実在確認(この機械で確認・追加依存なしで足りる範囲)

- **rich text(range/runs)の受け口は実在する**: `cosmic-text-0.19.0/src/buffer.rs:1102`
  `Buffer::set_rich_text<'r,'s,I>(spans: I, default_attrs, shaping, alignment)`
  (`I: IntoIterator<Item = (&'s str, Attrs<'r>)>`)。store の
  `TextRun`(`styles`+`runs` 表、裁定85)を `(&str, Attrs)` の列へ写す口として
  そのまま使える見込み — この probe は使っていない(1スタイル1バッファのみ)。
- **可変フォント軸(fvar)は `wght` のみ公開経路**: `cosmic-text-0.19.0/src/swash.rs`
  (`:30`,`:96`,`:272`)を実測すると、cosmic-text 内部が触るのは
  `font.as_swash().variations()` へ **font-weight 由来の1軸だけ**渡す経路で、
  任意タグ(`text-style-axis.tag`)を bind する公開 API は無い。**gap として
  記録**(下記 §6)。
- harfrust 直組みは比較のため crate 存在(`harfrust 0.5.2`)を確認したのみ
  ——現状の依存グラフには既に skrifa/read-fonts 経由で入っている(re_renderer
  由来)が、cosmic-text の 1本道が両言語で通った時点で харfrust への切替は
  「保守最低限」に反するので不採用。

## 5. 切片割り(実装切片、絞め殺し方式)

- **切片1(依存)**: `cosmic-text` を `motolii-vector`(または新設
  `motolii-text` leaf crate — 判断は次のレーン)の direct dependency へ昇格。
  `probes/r6-text-shaping/src/lib.rs` の `shape_text`/`to_shape`/`rasterize`
  +`commands_to_contours` をほぼそのまま移植(この probe は移植元として
  書かれている — テストも一緒に持っていける)。lh の `None` 分岐だけ
  swash フォントメトリクス参照へ差し替える。
- **切片2(vector 出口)**: `Contour`/`Vertex` → `PathSource::Bezier` は
  probe で確定済み。ここは実質「そのまま昇格」で新規判断なし。
- **切片3(engine 統合)**: `next/engine/motolii-engine/src/lib.rs:622` の
  `texture_for` から `LayerSource::Text` を分離する枝を作り、`Layer:text`
  component(`content`/`styles`/`runs`)を切片1の関数へ渡して raster を得る。
  受入条件 = 「Text layer が Preview で1px でも見える」+ Preview=Export
  一致(export 経路も同じ関数を呼ぶ、分岐を作らない)。
- **切片4(Inspector TEXT section)**: `next/ui/motolii-inspector-pane/` は
  現状 TEXT 束を一切持たない(grep 0 件、実測済み)。`TextDocumentStyle` の
  `size`/`line_height`/`tracking`/`fill`/`stroke_*`/`font` を書き戻す新設
  section。書き込みは既存 pane 群と同じく Intent 経由のみ(既存不変量)。

各切片は「動く物が増える」形の絞め殺しで独立に PR 化できる(切片1〜2は
コードの引っ越しのみで挙動ゼロ変更、切片3で初めて画素が出る、切片4で編集可能に
なる)。

## 6. gap(この probe が意図的にやらない・記録だけする)

- **range-selector / animator の適用**: 文字単位の重み付け(裁定75/85)は
  この probe の範囲外。ただし §4 の `set_rich_text` 受け口が runs → styled spans
  の写像先として実在するので、切片3以降で塞げる見込みは立った。
- **可変フォント軸(fvar)**: cosmic-text 公開 API に `wght` 以外の軸を bind する
  口が無い(§4 実測)。`text-style-axis` の任意タグ対応が要る日が来たら、
  ここだけ harfrust 直組み or swash 低レベル API への部分的な迂回が要る
  ——**その日まで足さない**(軸4 の原則どおり)。
- **縦書き**: 未検証。日本語歌詞が縦書きを要求するかは製品判断待ち
  (cosmic-text 自体に縦書き layout は無い)。

## 7. 証跡

- probe コード: `next/probes/r6-text-shaping/`(Cargo.toml + src/lib.rs +
  tests/r6.rs、workspace members に1行追加)
- 実行ログ: `cargo test --manifest-path next/Cargo.toml -p r6-text-shaping` —
  5 passed; 0 failed(§2)
- 生成 PNG(目視確認済み): scratchpad 配下
  `r6/motolii-arial-96.png`(512×160)・`r6/cjk-hiragino-80.png`(704×128)
- ソース根拠: `~/.asdf/installs/rust/stable/registry/src/index.crates.io-*/cosmic-text-0.19.0/src/{buffer,shape,swash}.rs`
- `next/check.sh`: owns/wraps marker 全通過(`probes/r6-text-shaping/src/lib.rs`
  298行として集計)
