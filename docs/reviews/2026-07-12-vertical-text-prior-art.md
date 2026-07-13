# 先例調査: 縦書き(日本語縦組み)テキストレイアウトの既存実装分解(2026-07-12)

ステータス: **調査メモ。反対側レビュー前。設計根拠にしないこと**([規律6点](README.md)の1・2・6項)。本文の所見は「一次資料で確認できた範囲」と「仮説と整合する事例」を区別して書く。「裏付けられた」とは書かない。判定語(採用/縮小/延期/棄却)は反対側レビュー後に付す。

## これは何か

P6(motolii-text: fontique + harfrust + Vello `draw_glyphs`、ラン単位純関数3点、組版はプラグイン側 = F-6分界)を前提に、**縦書きを既存実装がどう分解しているか**を調べた調査メモ。目的は「P6のコア契約(shapeに方向パラメータ)が縦書きプラグインに足りるか、足りない口はどこか」の候補列挙であり、縦書きを実装する/しないの判定はしない([M5仕様の未決事項](../specs/M5-3d-and-post.md)「縦書き対応の時期とスコープ」と実装ガード9「回転で実装しない」は据え置き)。

調査方法: 公式仕様(W3C/Unicode/Microsoft OpenType)・公式リポジトリのソースとissue・バグトラッカを直接参照した。harfrust はソースをローカルに取得して該当コードを目視確認した(commit `92d5853`、2026-07-03時点)。二次資料(個人ブログ・Medium等)は根拠に使っていない。

---

## 問い1: 既存実装は縦書きをどう分解しているか

### V-1. 主要ブラウザ2系統(Blink/Gecko)の分解は同型 —「向き分割はレイアウト層、縦字形と縦メトリクスはシェーピング層」

- **Blink**: テキストはシェーピング前に `RunSegmenter` が「スクリプト(ScriptRunIterator)・**向き(OrientationIterator)**・絵文字表現」で分割される。縦組み日本語にラテン文字が混在する例では、ラテン部分が `OrientationRotateSideways`、かなが `OrientationKeep` に分類され、**別ランとして分離されてから** HarfBuzz に渡る。出典(一次資料で確認): [Blink's Text Stack README](https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/blink/renderer/platform/fonts/README.md)
- **Gecko**: 直立(upright)ランは HarfBuzz に垂直方向(`HB_DIRECTION_IS_VERTICAL`)で渡してシェーピングし、sideways ランは**横書きとしてシェーピングして描画時に90°回転**する。textrun の vertical フラグは「グリフ位置データの x/y を入れ替えて、単純グリフレコードが縦送りを持つ」形で実装。向きの決定は UTR50(UAX#50)ベースで、`text-orientation: mixed` を支えるために向きフラグは textrun 内の GlyphRun 単位で持つ。出典(一次資料で確認): [Bugzilla 902762 "support vertical text run construction"](https://bugzilla.mozilla.org/show_bug.cgi?id=902762)(実装者 jfkthame のコメント)、メタバグ [145503](https://bugzilla.mozilla.org/show_bug.cgi?id=145503)
- **抽出できる境界線(両者共通)**: (a) UAX#50 による直立/横倒しの**ラン分割はシェーピングの外**(レイアウト層)。(b) 分割済みランに対する `vert` 適用・縦送り幅・縦原点は**シェーピング層**(HarfBuzz)。(c) sideways ランの90°回転は**描画側の変換**。— これは P6 の「コアはシェーピング原語、組版はプラグイン」という分界仮説と**整合する事例**

### V-2. WebKit は writing-mode 対応済みだが、内部分解は今回未確認

- WebKit も `writing-mode`/`text-orientation` を実装済みで、公式ブログでフォームコントロールの縦書き対応(2024)の設計記事がある。出典: [Implementing Vertical Form Controls](https://webkit.org/blog/15190/implementing-vertical-form-controls/)。ベースライン選択(mixed 時に alphabetic を使う)のバグ報告: [WebKit Bug 208824](https://bugs.webkit.org/show_bug.cgi?id=208824)
- ただしシェーピング層/レイアウト層の分解点のソースレベル確認は**未確認**(未読)。V-1 の2実装と同型かは断定しない

### V-3. CoreText は「属性+フレーム進行方向」のAPI面で縦組みを閉じ込め、縦原点変換の関数を公開している

- CoreText には縦字形を有効化する文字列属性 `kCTVerticalFormsAttributeName` と、グリフ配列に対する「(横の)デフォルト原点→縦原点へのオフセット」を返す `CTFontGetVerticalTranslationsForGlyphs`、フォント方向を指定する `CTFontOrientation.vertical`、行を右から左へ積む `CTFrameProgression` が存在する。出典(APIの実在はApple公式ドキュメントのページで確認): [kCTVerticalFormsAttributeName](https://developer.apple.com/documentation/coretext/kctverticalformsattributename) / [CTFontOrientation.vertical](https://developer.apple.com/documentation/coretext/ctfontorientation/vertical) / [CTFrameProgression](https://developer.apple.com/documentation/coretext/ctframeprogression) / [iOS 8.1 API diffs(CTFontGetVerticalTranslationsForGlyphs の存在確認)](https://developer.apple.com/library/archive/releasenotes/General/iOS81APIDiffs/modules/CoreText.html)
- 注: Apple 開発者ドキュメントはJSレンダリングのため本文の逐語確認ができなかった(ページ実在とAPI名・概要は確認)。詳細挙動(vert/vrt2 のどちらを踏むか等)は**未確認**
- **抽出できる境界線**: 「縦原点への変換」をフォントAPI(`CTFontGetVerticalTranslationsForGlyphs`)として**単体公開**している点が特徴。シェーピング結果と別口で原点変換だけ取れる設計は、P6 が原点情報をどう出すかの参考になる

### V-4. Skia/Flutter 系は縦書きを持たず、公式回答が「RotatedBox(回転)で代用」— 実装ガード9の反面事例

- Skia のパラグラフレイアウト(SkParagraph)の公開APIに縦書き方向の指定は無い(ヘッダを目視: [Paragraph.h](https://github.com/google/skia/blob/main/modules/skparagraph/include/Paragraph.h) に writing-mode 相当なし)。Flutter の `writing-mode` 要望 issue はクローズされ、コメント上の帰結は「`RotatedBox` で横倒しテキストには足りる」(モンゴル文字ユーザーが「エンジンには実装されないと理解した」と記録)。出典(一次資料で確認): [flutter/flutter#14262](https://github.com/flutter/flutter/issues/14262)、関連 [#35994](https://github.com/flutter/flutter/issues/35994)(カスタムテキストレイアウトの口の要求)、[#147728](https://github.com/flutter/flutter/issues/147728)
- **当てはめ**: 「基盤が縦書きの口を持たないと、エコシステムの答えが『回転』に退化する」事例。M5実装ガード9(回転による簡易縦書きを出すのが最悪)と**整合する事例**。P6 が shape に方向パラメータを持つこと自体が、この退化への予防線という読み方ができる(判定はレビュー後)

### V-5. libass は VSFilter 互換の「@フォント」方式 = シェーピングは横のまま `vert`+`vkna` を焚き、送り幅だけ縦advanceに差し替える

- libass のソースでは、`@` 前置フォント名(`desc.vertical`)のランに対して HarfBuzz フィーチャ `vert` と `vkna` を有効化し([ass_shaper.c L158-181](https://github.com/libass/libass/blob/f9fd3d20dff1cd84b7c74c8ae7f79711ad7736fa/libass/ass_shaper.c#L158-L181))、コードポイント U+02F1 以上のグリフについて `horiAdvance = vertAdvance` に差し替える([同 L220, L270-271](https://github.com/libass/libass/blob/f9fd3d20dff1cd84b7c74c8ae7f79711ad7736fa/libass/ass_shaper.c#L220))。行としては横組みのまま、回転は利用者側(スクリプトの回転タグ)に委ねる GDI/VSFilter 互換モデル
- **当てはめ**: 「シェーピング原語(フィーチャ有効化+縦メトリクス取得)だけで、行組みモデルを変えずに縦書き相当を焚く」最小実装の実例。ただし UAX#50 による混在向き制御は無く、品質はフル縦組みに達しない。P6 コアが出す原語の最小セットを考える材料

### V-6. Vivliostyle は自前レンダラを持たず、ブラウザの writing-mode 実装に全面依存する(=縦組みをレイアウト層より上で再実装しない先例)

- Vivliostyle.js は「Web標準技術ベースの組版システム」で、ページ組版(ページ分割・マージンボックス等)を独自に行うが、テキストレンダリング自体はブラウザに委ねる(EPUB Adaptive Layout 実装由来)。出典(一次資料で確認): [vivliostyle/vivliostyle.js README](https://github.com/vivliostyle/vivliostyle.js)
- **当てはめ**: 「縦書きの下回り(シェーピング+向き分割)は既存基盤に任せ、上位組版だけ実装する」ことが成立している事例 — F-6「組版はプラグイン側」の分界仮説と**整合する事例**(ただし Vivliostyle が依存している「下回り」はブラウザのレイアウト層まで含むので、境界の高さは P6 より上にある点に注意)

### V-7. resvg(Rust) は「横シェーピング+行ごと90°回転+UAX#50で直立文字だけ逆回転」— vert を踏まないため約物字形は置換されない

- resvg(usvg のテキストレイアウト)は SVG 1.1 の `writing-mode: tb` を、(a) テキスト全体を90°回転、(b) `unicode-vo` クレート(UAX#50)で `Orientation::Upright` と判定されたクラスタだけ逆回転して直立、という**クラスタ変換ベース**で実装している。コード中に「Could not find a spec that explains this, but this is how other applications are shifting the "rotated" characters」というコメントが残る。TTB シェーピングも `vert` 適用も行わない。出典(一次資料で確認): [usvg/src/text/layout.rs の apply_writing_mode](https://github.com/linebender/resvg/blob/adc94f76ee381cce8fe92477b39d4b58638f583c/crates/usvg/src/text/layout.rs#L1041-L1075)
- **当てはめ**: Rust圏の出荷済み縦書きは現状これが最有力例だが、モデルは「回転ベース」(約物の縦字形置換なし)。実装ガード9が禁じた品質帯の実例として参照できる

---

## 問い2: HarfBuzz本家の縦書きサポートの正確な範囲と、harfrust の移植状況

### V-8. HarfBuzz の縦書き既定は「TTB方向で `vert` のみ自動適用」— 2013年に vrt2/valt/vpal/vkrn を既定から外す決定が残っている

- 2013年の John Daggett(Mozilla)の調査とパッチにより、垂直ランの既定フィーチャは `{valt, vert, vkrn, vpal, vrt2}` から **`vert` のみ**に縮小された。理由: (a) OpenType には `vert` モデルと `vrt2` モデル(横倒しグリフをフォント内蔵)の**排他な2モデル**があり、一般的なのは「レイアウトエンジンが向き決定を持ち、`vert` だけ焚く」モデル、(b) Adobe のフィーチャ登録者自身が valt/vpal/vkrn は既定オフであるべきと回答。出典(一次資料で確認): [harfbuzz ML: default features for vertical text runs](https://lists.freedesktop.org/archives/harfbuzz/2013-August/003491.html)
- OpenType 公式レジストリも同じ分解を明文化: `vert`(直立用字形変換)と `vrtr`(横倒し用字形変換)は「**グリフを図形的に回転するレイアウトエンジン(UTR#50準拠)向け**」、`vrt2` は「フォント側の回転済みグリフに依存するエンジン向け」で、`vert` と `vrt2` は併用しない。出典(一次資料で確認): [Microsoft OpenType feature registry u-z(vert/vrtr/vrt2 の Feature interaction 節)](https://learn.microsoft.com/en-us/typography/opentype/spec/features_uz)

### V-9. HarfBuzz は TTB シェーピング時に「縦送り幅(vmtx+VVAR)・縦原点(VORG→bbox+tsb→ascent の3段フォールバック)」を出力座標へ焼き込む。それより上(向き分割・行組)はスコープ外と自己宣言

- HarfBuzz マニュアルは「bidi・フォント混在の分割・改行/ハイフネーション/ジャスティフィケーションはやらない」「1本の水平(または垂直)な線の上のシェーピングだけ」と明記。出典: [What HarfBuzz doesn't do](https://harfbuzz.github.io/what-harfbuzz-doesnt-do.html)
- 縦原点について、メンテナ(behdad)は「**VORG が(CFF系での正確な縦原点に)必要**。無ければ横 ascent を縦原点の代用にする」「シェーピング結果の位置をそのまま信じよ(利用者側で vmtx/tsb を再適用するな)」と回答。出典(一次資料で確認): [harfbuzz/harfbuzz Discussion #3294](https://github.com/harfbuzz/harfbuzz/discussions/3294)
- フォントに `vert` が無い場合、Unicode の縦書き用互換文字(縦組み用約物の presentation forms)へ写像するフォールバックが本家に入っている(Firefox の挙動を取り込んだもの)。出典: [harfbuzz/harfbuzz#355 "Fallback vertical shaping"](https://github.com/harfbuzz/harfbuzz/issues/355)
- `vert` はフォントが妙な script/langsys の下に登録している例が多く、HarfBuzz は**script横断のグローバル検索**で `vert` を探す。出典: [harfbuzz/harfbuzz#63](https://github.com/harfbuzz/harfbuzz/issues/63)(harfrust ソースコメントからの参照で確認)

### V-10. harfrust は上記の縦書き経路を移植済み(TTB・vert・vmtx/vhea/VORG/VVAR・縦presentation-formsフォールバック)。既知の穴は「縦メトリクステーブルの無いフォントの縦原点」1点

一次資料(ソース目視、commit `92d5853`)で確認した範囲:

- **TTB方向**: `Direction::TopToBottom` が存在し、垂直判定・反転が実装されている([common.rs L24-69](https://github.com/harfbuzz/harfrust/blob/92d585399b638f77dd743b8e80d271b88b7c3cff/harfrust/src/hb/common.rs#L24))
- **vert 自動適用(グローバル検索込み)**: 垂直方向では `vert` を `F_GLOBAL_SEARCH` で有効化。本家の2013年決定と #63 をコメントで引用([ot_shape.rs L152-167](https://github.com/harfbuzz/harfrust/blob/92d585399b638f77dd743b8e80d271b88b7c3cff/harfrust/src/hb/ot_shape.rs#L152-L167))
- **縦presentation-formsフォールバック**: `vert` がフォントに無い場合、縦書き互換文字への写像を試す([ot_shape.rs L780-788](https://github.com/harfbuzz/harfrust/blob/92d585399b638f77dd743b8e80d271b88b7c3cff/harfrust/src/hb/ot_shape.rs#L780))
- **縦ポジショニング**: 垂直方向では `y_advance = advance_height`(vmtx、VVAR/phantom deltas 込み)を設定し、縦原点を x_offset/y_offset から減算して**出力座標へ焼き込む**([ot_shape.rs L444-460](https://github.com/harfbuzz/harfrust/blob/92d585399b638f77dd743b8e80d271b88b7c3cff/harfrust/src/hb/ot_shape.rs#L444))
- **縦原点の3段フォールバック**: VORG(+VVAR delta)→ glyf bbox + tsb → ascent/descent 代用、vmtx 欠落時の縦送りは `ascent - descent` 代用([glyph_metrics.rs L215-300](https://github.com/harfbuzz/harfrust/blob/92d585399b638f77dd743b8e80d271b88b7c3cff/harfrust/src/hb/glyph_metrics.rs#L215))
- **既知の穴**: 本家テストとの縦書き不一致は「**縦メトリクステーブルを持たないフォント**」で残る。behdad: 「HB/FTと完全一致させるにはグリフ bbox が要り、それは(harfrustでなく)skrifa の仕事。実現困難として受け入れを提案」→ 大半の縦テストは有効化され、`vertical_015`/`vertical_017` の2件のみブロックリスト。出典(一次資料で確認): [harfbuzz/harfrust#50](https://github.com/harfbuzz/harfrust/issues/50)、[gen-shaping-tests.py L33-35](https://github.com/harfbuzz/harfrust/blob/92d585399b638f77dd743b8e80d271b88b7c3cff/scripts/gen-shaping-tests.py#L33)
- **系譜の整理**: harfrust は rustybuzz のフォーク(ttf-parser → read-fonts 移行が動機)で、HarfBuzz **v13.0.0** 相当。rustybuzz は「port is finished」の同期保守モードで HarfBuzz v10.1.0 相当。出典: [harfrust README](https://github.com/harfbuzz/harfrust) / [rustybuzz README](https://github.com/harfbuzz/rustybuzz)
- **注意(harfrust全般の制約)**: AAT `mort` 非対応・外部ライブラリ統合なし(README記載)。AAT 経由でしか縦字形を持たない旧macOSフォントの挙動は**未確認**

---

## 問い3: シェーピングだけでは足りない部分の先例

### V-11. 約物の縦字形は「`vert` で置換」が本則、フォント非対応時の受け皿は「Unicode縦書き互換文字への写像」まで既にシェーパ内にある。ただし品質保証は無い

- OpenType レジストリ上、`vert` の機能は「縦組みで直立表示するのに適した字形への変換」(例: U+FF08 → 回転形 U+FE35、小書き仮名「ぁ」の右上寄せ)で、**縦組みでは既定オン**とされる。出典: [features_uz#vert](https://learn.microsoft.com/en-us/typography/opentype/spec/features_uz)
- `vert` 非搭載フォントへのフォールバック(V-9/V-10 の presentation forms 写像)は「そのフォントが縦書き互換文字のグリフを持つ場合」しか効かない。**CJKフォントの `vert` 対応率の定量データは公開恒久文書で見つけられなかった(未確認)**。「同梱 Noto Sans CJK は `vert` を持つ」ことも本調査では逐語確認していない(未確認 — P6完了条件に足す場合は実フォントで検査するのが筋)
- 句読点の詰め(JLREQ級の行組をしないエンジン向け)には `vchw`/`vhal` 等の**フィーチャ側の代替**が登録されており、レジストリは「JLREQ級のレイアウトを自前でやるエンジンはこれらを使うな、やらないエンジンは縦CJKで常時適用せよ」と両モデルを明文化している。出典: [features_uz#vchw](https://learn.microsoft.com/en-us/typography/opentype/spec/features_uz)

### V-12. Latin混在時の直立/横倒し判定(UAX#50)は「シェーパの外」— どの実装もレイアウト層(=Motolii分界ではプラグイン)が持つ

- UAX#50 は文字ごとの `Vertical_Orientation` プロパティ(U/R/Tu/Tr)を定義し、「フォントテーブルや上位プロトコルに依存しない安定既定向き」を提供する。出典: [UAX #50](https://www.unicode.org/reports/tr50/)
- CSS は `text-orientation: mixed` の規範動作として「U/Tu/Tr は直立、R は90°時計回りに横倒し」で組むことを UA に要求し、直立組版では「縦組み用フィーチャ(例: OpenType `vert`)を有効化しなければならない」「**縦メトリクスを持たないフォントでは UA が縦メトリクスを合成しなければならない**」と規定する。出典(一次資料で確認): [CSS Writing Modes Level 4 §text-orientation](https://www.w3.org/TR/css-writing-modes-4/#text-orientation)
- 実装も一致: Blink は OrientationIterator(V-1)、Gecko は GlyphRun 単位の向きフラグ(V-1)、resvg は unicode-vo(V-7)。HarfBuzz 自身は向き分割をしない(V-9)
- Rust でこの判定に使える公開部品: [unicode-vo クレート](https://crates.io/crates/unicode-vo)(UAX#50 実装、resvg が使用)、[icu_properties の VerticalOrientation](https://unicode-org.github.io/icu4x/rustdoc/icu_properties/index.html)(ICU4X)。**プラグイン側で完結可能**な材料が存在することは確認

### V-13. 縦中横(tate-chu-yoko)は「横シェーピングした塊を1em枠に押し込む」— 幅圧縮は OpenType 幅フィーチャ(hwid/twid/qwid)経由

- CSS `text-combine-upright` の規範動作: 結合対象は水平に組み、**1文字分の縦枠(1em)** に収める。圧縮には OpenType の `hwid`(半角)/`twid`(1/3角)/`qwid`(1/4角)フィーチャの適用を要求し、組んだ結果は単一の代替文字(U+FFFC相当)として扱う。出典(一次資料で確認): [CSS Writing Modes Level 3 §text-combine-upright](https://www.w3.org/TR/css-writing-modes-3/#text-combine-upright)
- **含意**: 縦中横は「横方向 shape の再利用+フィーチャ指定+配置」で組める = シェーピング原語で成立する組版。ただし**フィーチャ指定の口が shape 入力に要る**(P6影響候補C-2)

### V-14. 禁則・ぶら下げはシェーピングと独立の行分割問題 — 仕様体系(JLREQ/UAX#14/CSS Text)はプラグイン実装で参照可能だが、ぶら下げは主要ブラウザすら未出荷

- 禁則(行頭/行末禁則・分離禁則)の要件は [JLREQ(W3C 日本語組版処理の要件)](https://www.w3.org/TR/jlreq/) が定義し、行分割機会の基盤は [UAX #14 Line Breaking](https://www.unicode.org/reports/tr14/)、厳しさの切替は [CSS Text Level 3 の line-break プロパティ](https://www.w3.org/TR/css-text-3/#line-break-property)(strict/normal/loose の対象文字クラスを列挙)が対応する
- ぶら下げ(hanging punctuation)は [CSS Text Level 3 の hanging-punctuation](https://www.w3.org/TR/css-text-3/#hanging-punctuation-property) にあるが、[MDN の互換表](https://developer.mozilla.org/en-US/docs/Web/CSS/hanging-punctuation)で「Baseline ではない(広く使われるブラウザで動かない)」= 実装は Safari 系に限られる状態(2026-07 時点)
- **含意**: 禁則/ぶら下げはコア口を増やさずプラグインで組める(UAX#14系クレートは cosmic-text/parley も内部利用)。ぶら下げの実装優先度が低い先例は「v1スコープから外しても差別化を失わない」ことと**整合する事例**(判定はしない)

---

## 問い4: Rustエコシステムの現在地

### V-15. cosmic-text / parley / swash はいずれも縦書き未対応(2026-07-12時点)

- **cosmic-text**: 「Vertical text」issue が2022年から**オープンのまま**(本文は writing-mode 相当の要望、コメントは JLREQ/CLREQ への参照のみで実装進捗なし)。一次資料: [pop-os/cosmic-text#11](https://github.com/pop-os/cosmic-text/issues/11)
- **parley**: リポジトリ issue 検索で縦書き(writing-mode)実装 issue は見つからず(vertical と付く issue はインラインボックスの縦位置合わせの話)。East Asian line breaking の議論はある([linebender/parley#301](https://github.com/linebender/parley/issues/301))。「未対応」は消極的証拠(issue不在+機能一覧に記載なし)による — 強い主張はしない
- **swash**: issue 検索で vertical 該当ゼロ。shape API に方向の口があるかは**未確認**
- **エコシステム内の出荷済み実装**は resvg の回転ベース実装(V-7)が確認できた範囲の最有力。TTB シェーピング(vert 適用込み)を使った出荷済み Rust 組版エンジンは今回**見つけられなかった(未確認 — 「存在しない」とは言えない)**

### V-16. typst は縦書きRFCがオープン(未実装)で、要求分解が Motolii の想定分界と同型

- typst の縦書き RFC(2025-)は「シェーピングエンジンは横書き前提が典型。縦組みでは多くのグリフに90°回転が要る。CJK は回転不要。約物の正しい表示には `vert` 有効化が要る(Unicode に回転済み約物のコードポイントもある)」と技術課題を整理し、writing-mode 属性・論理軸API・縦中横・約物配置を要件化している。ステータスはオープン(実装マージなし)。一次資料: [typst/typst#5908 RFC: Vertical Writing Mode](https://github.com/typst/typst/issues/5908)
- **含意**: 「harfrust(=HB系)を使う Rust 組版エンジンでも、縦書きの主作業はシェーパの外(向き分割・回転・行組)」という認識が第三者プロジェクトでも同型 — V-1 の境界線抽出と**整合する事例**

---

## 問い5に代えて: P6契約への影響候補の列挙(判定はしない)

P6 API契約(shape(ラン, フォント, **方向**, 言語, 軸座標) → グリフ列+送り幅+クラスタ対応表)を上の所見に当てると、縦書きプラグインが必要とする口の候補は以下。**採用判定は反対側レビュー後**。

- **C-1. shape 出力の「送り幅」を方向中立の語彙にする**: harfrust は TTB 時に `y_advance`(縦送り)と、**縦原点減算済みの x_offset/y_offset** を返す(V-10)。契約の出力型が「横の送り幅」前提だと TTB の結果を運べない。候補: 出力を「主軸advance+交差軸オフセット(x_offset/y_offset)」として明文化(横書き経路では今のゴールデンと同値)
- **C-2. OpenType フィーチャ指定の入力口が shape に無い**: `vert` は方向から自動だが(V-8/V-10)、縦組み品質に関わる `vkna`・`vpal`/`vchw`/`vhal`・`vkrn`、縦中横の `hwid`/`twid`/`qwid`(V-13)、横倒しラン用の `vrtr` はいずれも**呼び出し側指定のフィーチャ**。現契約の入力(ラン, フォント, 方向, 言語, 軸座標)にはフィーチャリストが無い。歌詞プラグイン以外(スモールキャップ等)にも波及する汎用の穴の可能性
- **C-3. 向き分割(UAX#50)を前提とした「ラン再分割の自由」の保証**: 縦書きプラグインは itemize 結果をさらに「直立ラン/横倒しラン」に分割してから shape を呼ぶ必要がある(V-1/V-12)。UAX#50 判定自体は unicode-vo / icu_properties でプラグイン側完結が可能(V-12)。コアに要るのは判定機能ではなく、「**itemize 結果の部分範囲を任意に切って shape に渡してよい**(結果が全体shapeと乖離しうる点の明文化含む)」という契約上の保証
- **C-4. 縦組みの行送りメトリクス(vhea ascender/descender 等)の公開口**: shape はグリフ単位を返すが、行を右から左へ積む行送り・横倒しランのベースライン合わせには**フォントレベルの縦メトリクス**が要る(CSS は「無ければ UA が合成せよ」とまで規定 = V-12)。コアが横系メトリクス(hhea/OS2)しか公開しない設計だと足りない可能性
- **C-5. 縦メトリクス欠落フォントの診断**: harfrust の既知の穴は「縦テーブルの無いフォントの縦原点」(V-10)で、フォールバック時は ascent 代用等の劣化になる(V-9)。P6 の豆腐診断(実装ガード8)の縦版として「vmtx/VORG/vert 欠落の診断出力」を足すかは検討候補
- **C-6. sideways 描画の材料**: 横倒しランは「横 shape+90°回転」(V-1)で、`glyph_transform` 貫通(P6既定)で描けるが、回転の基準点(ベースライン位置)の計算に C-4 のメトリクスが要る
- **C-7. フォールバック解決(fontique)と縦字形の相互作用**: `vert` 非搭載フォントへの presentation-forms フォールバック(V-9)は「同フォントに縦書き互換文字グリフがある」場合のみ有効。フォールバック選定が縦書き適性(vmtx/vert の有無)を考慮しないと、縦組みで劣化フォントに解決される可能性 — 挙動は**未確認**、論点としてのみ記録

なお、**禁則・ぶら下げ・行組(V-14)・縦中横の配置(V-13)はコア口の追加なしにプラグインで組める**見込みが立つ、というのが本調査の範囲での読み(これも判定はレビュー後)。

## 未確認事項リスト

1. WebKit 内部のシェーピング/レイアウト分解点(V-2)— ソース未読
2. CoreText の vert/vrt2 適用詳細・縦原点の計算式(V-3)— Apple ドキュメント本文が取得不能、逐語未確認
3. CJK フォントの `vert` フィーチャ対応率の定量データ(V-11)— 公開恒久文書を発見できず
4. 同梱予定 Noto Sans CJK の vert/vmtx/VORG 搭載状況(V-11)— 実フォント検査未実施
5. swash の shape API における方向指定の有無(V-15)
6. 「TTBシェーピングを使った出荷済み Rust 組版エンジンが存在しない」ことの確定(V-15)— 消極的証拠のみ
7. fontique のフォールバック選定が縦書き適性を考慮するか(C-7)
8. harfrust の AAT(morx)経由でしか縦字形を持たないフォントの挙動(V-10 注意)
9. skrifa / read-fonts が vhea/vmtx のフォントレベルメトリクスを公開APIでどこまで出しているか(C-4 の実装可否)— harfrust 内部では read-fonts 経由で読んでいることのみ確認
10. 青空文庫系ビューア等、和文特化アプリの内部実装 — 再確認可能な公開恒久文書(公式リポジトリ)を今回特定できず、調査対象から除外した

## 出典一覧(主要)

- Blink: [Blink's Text Stack](https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/blink/renderer/platform/fonts/README.md)
- Gecko: [Bugzilla 902762](https://bugzilla.mozilla.org/show_bug.cgi?id=902762) / [145503](https://bugzilla.mozilla.org/show_bug.cgi?id=145503)
- WebKit: [Vertical Form Controls](https://webkit.org/blog/15190/implementing-vertical-form-controls/) / [Bug 208824](https://bugs.webkit.org/show_bug.cgi?id=208824)
- CoreText: [kCTVerticalFormsAttributeName](https://developer.apple.com/documentation/coretext/kctverticalformsattributename) / [CTFrameProgression](https://developer.apple.com/documentation/coretext/ctframeprogression) / [iOS 8.1 API diffs](https://developer.apple.com/library/archive/releasenotes/General/iOS81APIDiffs/modules/CoreText.html)
- Skia/Flutter: [Paragraph.h](https://github.com/google/skia/blob/main/modules/skparagraph/include/Paragraph.h) / [flutter#14262](https://github.com/flutter/flutter/issues/14262)
- libass: [ass_shaper.c@f9fd3d2](https://github.com/libass/libass/blob/f9fd3d20dff1cd84b7c74c8ae7f79711ad7736fa/libass/ass_shaper.c)
- Vivliostyle: [vivliostyle.js README](https://github.com/vivliostyle/vivliostyle.js)
- resvg: [usvg text layout.rs@adc94f7](https://github.com/linebender/resvg/blob/adc94f76ee381cce8fe92477b39d4b58638f583c/crates/usvg/src/text/layout.rs#L1041)
- HarfBuzz: [what-harfbuzz-doesnt-do](https://harfbuzz.github.io/what-harfbuzz-doesnt-do.html) / [ML 2013-08 default vertical features](https://lists.freedesktop.org/archives/harfbuzz/2013-August/003491.html) / [Discussion #3294](https://github.com/harfbuzz/harfbuzz/discussions/3294) / [#355](https://github.com/harfbuzz/harfbuzz/issues/355) / [#63](https://github.com/harfbuzz/harfbuzz/issues/63)
- harfrust: [README](https://github.com/harfbuzz/harfrust) / [#50](https://github.com/harfbuzz/harfrust/issues/50) / ソース(commit [92d5853](https://github.com/harfbuzz/harfrust/tree/92d585399b638f77dd743b8e80d271b88b7c3cff): ot_shape.rs, glyph_metrics.rs, common.rs, gen-shaping-tests.py)
- rustybuzz: [README](https://github.com/harfbuzz/rustybuzz)
- Unicode/W3C: [UAX #50](https://www.unicode.org/reports/tr50/) / [UAX #14](https://www.unicode.org/reports/tr14/) / [CSS Writing Modes 4 §text-orientation](https://www.w3.org/TR/css-writing-modes-4/#text-orientation) / [CSS Writing Modes 3 §text-combine-upright](https://www.w3.org/TR/css-writing-modes-3/#text-combine-upright) / [CSS Text 3 line-break](https://www.w3.org/TR/css-text-3/#line-break-property) / [同 hanging-punctuation](https://www.w3.org/TR/css-text-3/#hanging-punctuation-property) / [JLREQ](https://www.w3.org/TR/jlreq/)
- OpenType: [feature registry u-z (vert/vrtr/vrt2/vkna/vpal/vchw/vhal/vkrn)](https://learn.microsoft.com/en-us/typography/opentype/spec/features_uz)
- Rust圏: [cosmic-text#11](https://github.com/pop-os/cosmic-text/issues/11) / [parley#301](https://github.com/linebender/parley/issues/301) / [typst#5908](https://github.com/typst/typst/issues/5908) / [unicode-vo](https://crates.io/crates/unicode-vo) / [icu_properties](https://unicode-org.github.io/icu4x/rustdoc/icu_properties/index.html) / MDN: [hanging-punctuation](https://developer.mozilla.org/en-US/docs/Web/CSS/hanging-punctuation)
