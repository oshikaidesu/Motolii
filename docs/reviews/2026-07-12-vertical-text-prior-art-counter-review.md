# 反対側レビュー: 縦書き先例調査の再判定 — 今決めるのは語彙と口だけ(2026-07-12)

ステータス: **独立批判レビュー**(元調査メモの記述を信用せず、一次資料を自分で再取得して判定した。元調査の完遂・検証を引き継がない)

対象: [縦書き先例調査メモ](2026-07-12-vertical-text-prior-art.md)の所見V-1〜V-16、P6契約への影響候補C-1〜C-7、未確認事項10点

## 結論

元メモは規律6点制定後の最初の調査メモとして書かれており、**事実面の精度は高い**。ソース行番号つきの主張(harfrust・resvg・libass)は全点、pinned commitの原文で再現した。GitHub issue/discussionの引用(harfrust#50、harfbuzz#3294、flutter#14262、cosmic-text#11、typst#5908)も原文と一致し、仕様引用(OpenType registry、CSS Writing Modes 3/4)も逐語確認できた。「未確認」の自己申告も適切に付いている。

それでも判定が割れた点は4つある(詳細は「元メモとの不一致点」):

1. **V-13の「縦中横にはフィーチャ適用を要求」は過大** — CSS仕様は幅バリアントが揃わない場合の幾何スケーリングを明示的に許容しており、フィーチャ口なしでも劣化版の縦中横は組める。C-2の必然性を一段弱める
2. **V-15のparley「縦書きissue不在」は事実として不正確** — [linebender/parley#634](https://github.com/linebender/parley/issues/634)(2026-06-01起票、オープン)が `parley_core` 提案に vertical writing modes を明記している。結論(未出荷)は不変だが、「シェーピング側に縦を、行組は外に」という**同一の分界を同一エコシステム(linebender)が引いている追加傍証**であり、watch対象
3. **V-8の出典URL(freedesktop MLアーカイブ)は再確認不能(403)** — ただし決定自体はHarfBuzz本体コミット [d71c0df](https://github.com/harfbuzz/harfbuzz/commit/d71c0df2d17f4590d5611239577a6cb532c26528)("Remove vrt2, vkrn, vpal, and valt from default vertical features" / "See thread by John Dagget on the list")で確認できる。恒久出典はコミットに差し替えるべき
4. **V-4の事実は元メモより強い** — Flutter創設メンバーHixieの実言は「We do not intend to add vertical text support」「Properly supporting vertical text requires a significantly more elaborate architecture **from the beginning** and we concluded that the additional cost to the project was too high」。ただし因果の向きには注記が要る(後述)

C-1〜C-7への推奨判定(最終判定はユーザー):

| 候補 | 推奨 | 一言根拠 |
|---|---|---|
| C-1 出力語彙の方向中立化 | **採用(文言のみの縮小形)** | HB系の自然出力そのもの。motolii-text未着工の今なら費用≈0 |
| C-2 フィーチャ透過口 | **採用(弱い採用・最小透過口)** | プラグイン側で吸収不能な唯一の口。ただし延期しても致命ではない |
| C-3 ラン再分割の自由の明文化 | **採用(文言のみ)** | 既存契約の暗黙の約束を1文にするだけ。新規口なし |
| C-4 縦行送りメトリクス口 | **延期** | 純追加APIで後付け可能。一方通行扉ではない |
| C-5 縦メトリクス欠落診断 | **延期** | 縦書きプラグイン着手時の完了条件へ |
| C-6 sideways描画の材料 | **棄却(新規作業として)** | `glyph_transform`貫通で材料は現契約に既にある |
| C-7 fallback選定の縦適性 | **延期(論点記録のみ)** | 挙動未確認。同梱フォント下限保証(ガード8a)が実質の緩和策 |

縦書き機能そのものは「より小さい対策=延期」で足りる。**v1は横書きのみのスコープ宣言+実装ガード9(回転禁止)据え置き**を推奨する。今やる価値があるのは契約の語彙と口の形だけであり、それも「不可逆だから」ではなく「未着工の今が最安だから」である(後述の一方通行扉分析)。

## 判定方法

各所見・各候補に対し、次の順で反対尋問した。

1. **事実**: 出典URLを自分で開き(GitHub rawソースはpinned commitでダウンロードして目視、issueは`gh api`で本文取得)、メモの要約が原文から言えるか
2. **転移条件**: ブラウザ実装の分界がMVエディタ(動的リフロー稀・選択/編集なし・アクセシビリティ要件なし・歌詞=短行)に転移するか
3. **因果**: 「口が無い→回転に退化」等の帰属に別の説明(需要不足・優先度)が残っていないか
4. **より小さい対策**: プラグイン側吸収・縦書き自体の延期・文言だけの予約で足りないか。判断基準は「ユーザーデータまたは公開契約へ不可逆に焼くかどうか。焼かない選択が可能ならv1では小さい方を選ぶ」

## 所見ごとの再判定

### 問い1: 既存実装の分解

**V-1(Blink/Gecko同型分解)— 支持**。Blink READMEの原文確認: RunSegmenterがScriptRunIterator・OrientationIterator・emoji表現で分割し、縦組み例でひらがな=OrientationKeep、ラテン=OrientationRotateSideways と分類されてからHarfBuzzへ渡る記述あり。Bugzilla 902762のjfkthameコメントも確認: sidewaysランは「horizontal font instanceでshapeし描画時に90°回転」、verticalフラグは「グリフ位置のx/y入れ替え」、text-orientation:mixedはGlyphRun単位の向きフラグ(UTR50ベース)。メモの要約は原文から言える。

転移条件の注記: ブラウザが向き分割をレイアウト層に置く理由には選択・編集・動的リフローというMVエディタに無い要件も混ざる。しかし境界線の根拠はブラウザ都合ではなく、(a) HarfBuzz自身のスコープ宣言(V-9で確認)、(b) OpenType registryの2モデル設計(V-8で確認)という**要件非依存の一次資料**で独立に支えられている。要件差は結論を変えない。むしろ「歌詞=短行・JLREQ級品質が初日から要らない」という差は、縦書き自体のv1延期を支持する方向に働く。

**V-2(WebKit未確認)— 支持(そのまま)**。メモ自身が「断定しない」と自己限定しており、適切。

**V-3(CoreText)— 縮小**。Apple開発者ドキュメントはJSレンダリングのため本レビューでも逐語確認できず、メモの自己申告(API実在まで)を超える確認はできなかった。「縦原点変換の単体公開が参考になる」という抽出は**設計参考に引用しない**ことを推奨する。API実在の記録としてのみ残す。

**V-4(Skia/Flutter=回転代用)— 支持(事実は増強、因果に注記)**。Paragraph.h(main)をダウンロードしgrep: `vertical`/`writing` に該当なし=公開APIに縦書き指定なしを確認。flutter#14262は`gh api`で全コメント取得: Hixieの「We do not intend to add vertical text support」(2018-01)、「significantly more elaborate architecture from the beginning … additional cost too high」(2018-02)、モンゴル文字ユーザーsuragchの「I realize this isn't going to get implemented in the Flutter engine」(2019-07)、rrousselGitの「`RotatedBox` is enough for sideway texts as said by @hixie」(2019-07)を確認。

因果の注記: メモの「基盤が口を持たないとエコシステムの答えが回転に退化する」は、一次資料上は**「優先度判断→口を作らない→回転が定番回答化」**という因果連鎖であり、需要不足・優先度という「別の説明」は排除されていない — むしろそれが起点である。Motoliiへの転移で使えるのはむしろHixieの実言の方: 縦書きの口は「from the beginning」でないと費用が跳ねる、とFlutter側が自認している点。ただしFlutterの費用はパラグラフレイアウト層のアーキテクチャの話で、P6のようなラン単位シェーピング原語に方向パラメータを置く費用とは桁が違う。「P6が方向パラメータを既に持つこと自体が予防線」という読みは、この限定つきで支持する。

**V-5(libass=横シェーピング+vert/vkna+送り差し替え)— 支持(行番号は概算と注記)**。pinned commit `f9fd3d2` のass_shaper.cをダウンロードして確認: `init_features`/`set_run_features`で`desc.vertical`ランに VERT+VKNA を有効化、`VERTICAL_LOWER_BOUND = 0x02f1`(ass_font.h L35)、`ass_glyph_metrics_construct`で`v->horiAdvance = v->vertAdvance`(L271)。すべて実在。メモの行番号(L158-181等)は数行ずれるが実質は正確。**注目点**: 出荷済みの最小実装が使った機構は「呼び出し側フィーチャ指定+メトリクス差し替え」であり、C-2(フィーチャ口)の実在需要の証拠になっている。

**V-6(Vivliostyle=ブラウザ依存)— 支持(弱い傍証として)**。README確認: 「Web標準技術ベースの組版システム」でHTML+CSS組版。テキストレンダリングをブラウザに委ねることはJS実装という性質から言えるが、READMEに明文はない。メモ自身の注記(依存する下回りはブラウザのレイアウト層まで含み、境界の高さはP6より上)が正しく、**F-6の傍証としては弱い**。格は「整合する事例」のまま維持。

**V-7(resvg=回転ベース)— 支持**。pinned commit `adc94f7` のlayout.rsをダウンロードして確認: `apply_writing_mode`が`unicode_vo::char_orientation`でUprightだけ逆回転、「Could not find a spec that explains this…」コメントも逐語で実在。`vert`適用なし・TTBシェーピングなしも確認(フィーチャ設定箇所に垂直分岐なし)。「実装ガード9が禁じた品質帯の実例」という当てはめは妥当 — 回転ベースで約物字形が置換されないのは機構上の必然(vertを踏まないため)であり、因果帰属も正しい。ただし「resvgが回転を選んだ」こと自体はSVG 1.1の`writing-mode: tb`という限定要件への対応であり、Rust圏の能力限界の証拠ではない点は注記。

### 問い2: HarfBuzz/harfrust

**V-8(既定はvertのみ・2013年決定)— 支持(出典差し替え要)**。freedesktop MLアーカイブ(003491/003490)はHTTP 403で**再確認不能**。ただし: (a) HarfBuzz本体コミットd71c0dfのメッセージが「Remove vrt2, vkrn, vpal, and valt from default vertical features / See thread by John Dagget on the list」で決定の実在を確認、(b) harfrustソース(ot_shape.rs L157-159)が同じMLスレッドとコミットをコメントで引用、(c) OpenType registry原文で `vert`=「should be active by default in vertical writing mode」、`vert`/`vrtr`=「layout engines that graphically rotate glyphs … such as those conforming to UTR#50」向け、`vrt2`=pre-rotated依存エンジン向け、「the 'vert' feature should never be used with 'vrt2'」、`vkna`=「should be off by default」、`vchw`=「If a layout engine supports advanced layout for CJK text as described in CLREQ/JLREQ/KLREQ, this feature should not be used. Otherwise, this feature should always be applied in vertical layout of CJK text」をすべて逐語確認。**運用注**: レビュー規律の「再確認可能な公開恒久文書」基準に照らし、出典一覧のML URLにはコミットd71c0dfを併記すべき。

**V-9(HBの縦出力とスコープ宣言)— 支持**。what-harfbuzz-doesnt-doの原文確認(bidi・行分割・ハイフネーション・ジャスティフィケーション非対応、「single horizontal (or vertical) line」)。Discussion #3294のbehdad発言を逐語確認: 「VORG is required. Finding glyph extents from CFF outlines is _REALLY_ expensive」「If VORG is missing, we use the horizontal ascent as glyph vertical origin」「Steps 2 to 5 should not be necessary」(=シェーピング結果を信じよ)。#355/#63は個別に開かなかったが、harfrustソースの該当コード(下記V-10)とコメントが内容を裏書きする。

**V-10(harfrustの移植状況)— 支持・全点一次確認**。commit `92d5853` の4ファイルをダウンロードして目視:

- common.rs L24-26: `TopToBottom`/`BottomToTop` 実在、L29-70に垂直判定・反転
- ot_shape.rs L152-167: 垂直方向で`vert`を`F_GLOBAL_SEARCH`有効化、2013年決定と#63をコメント引用 — 逐語一致
- ot_shape.rs L780-788: `is_vertical() && !has_vert`で縦presentation-forms写像(`as_codepoint().vertical()`) — 一致
- ot_shape.rs L444-461(`position_default`): 垂直で`pos.y_advance = advance_height(glyph)`、`vertical_origin`をx_offset/y_offsetから減算 — 一致(**C-1の事実基盤**)
- glyph_metrics.rs L214-239: vmtx欠落時は`ascent - descent`代用、VVAR/phantom deltas込み。L262以降(`v_origin`): VORG(+VVAR)→bbox+tsb→ascent/descent系代用の3段 — 一致
- gen-shaping-tests.py L33-35: `vertical_015`/`vertical_017`のみブロックリスト、PR#52参照 — 一致。PR#52は2025-05-14マージ済みを`gh api`で確認
- issue#50(クローズ済み)のbehdad発言を逐語確認: 「To match HB/FT completely, we require glyph bounding boxes, which is currently an Skrifa job. I suggest we accept as not feasible」「The mismatch only happens with fonts set vertically that don't have vertical metrics tables」

版・保守リスクの評価(タスク指示による追加確認): harfrustはharfbuzz org配下、最新リリース**0.12.0(2026-07-03)= 参照コミットと同時期**、HarfBuzz v13.0.0相当、活発に開発中。README記載(rustybuzzフォーク・read-fonts移行動機・AAT mort非対応・外部ライブラリ統合なし)も確認。リスクは (a) 0.x系でAPI形状が変わりうる、(b) 縦書き経路の同等性はHB本家テストスイート同期(gen-shaping-tests)で機械固定されているため挙動後退リスクは低い。緩和策は既存の実装ガード8(b)(シェーピングをtrait境界の裏に置きharfbuzz FFIへ差し替え可能)で足りており、**新規対策は不要**。記録上は「commit 92d5853」より「**harfrust ≥ 0.12(縦テスト有効化はPR#52、2025-05以降)**」と書く方が追跡可能。

### 問い3: シェーピングの外の先例

**V-11(vert本則+フォールバック+品質保証なし)— 支持**。OpenType registry原文で`vert`の機能・U+FF08→U+FE35例・「active by default in vertical writing mode」を確認。presentation-formsフォールバックはharfrustコード(V-10)で実在確認。未確認事項3・4(CJKフォントのvert対応率、Noto Sans CJKの搭載状況)は未確認のまま — ただしNoto検査は同梱時にフォントツール1行で済む話であり、**縦書き着手時の完了条件に送ればよい**(今の調査続行は不要)。

**V-12(UAX#50はシェーパの外)— 支持**。CSS Writing Modes 4原文確認: 「the UA must determine the orientation of each typographic character unit by its Vertical_Orientation property: … upright if U, Tu, or Tr; or … sideways (90° clockwise) if R」「(E.g. the OpenType vert feature must be enabled.)」「The UA must synthesize vertical font metrics for fonts that lack them.(合成ヒューリスティクスは未定義)」。unicode-voのresvg使用はV-7ソースのimportで確認。icu_propertiesのVerticalOrientationは未再確認(小)。

**V-13(縦中横)— 縮小**。CSS Writing Modes 3原文確認: 1em枠(「The effective size of the composition is assumed to be 1em square」)、単一グリフ扱い(「treated as a single glyph representing the Object Replacement Character U+FFFC」)は一致。ただし圧縮規則の原文は「OpenType implementations **must** use width-specific variants (hwid/twid/qwid) … **in cases where those variants are available for all typographic character units** in the composition. **Otherwise, the UA may use any means** to compress the text, including … **scaling the text geometrically**, or any combination thereof」。つまり**幾何スケーリングが仕様公認のフォールバック**であり、メモの「圧縮には…フィーチャの適用を要求」は片面だけ。仕様級品質にはフィーチャ口が要るが、「フィーチャ口が無いと縦中横が組めない」とまでは言えない。C-2の根拠を「必須」から「品質上限を上げる口」へ弱める。

**V-14(禁則/ぶら下げ)— 支持(部分確認)**。MDNの「This feature is not Baseline because it does not work in some of the most widely-used browsers」を確認。「Safari系に限られる」の互換表詳細は今回再確認できなかった(Baseline外であることは確認)。JLREQ/UAX#14/CSS Text 3の参照は仕様の実在として妥当。「v1スコープから外しても差別化を失わない」の読みに異論なし。

### 問い4: Rust現在地

**V-15(cosmic-text/parley/swash未対応)— 縮小/訂正**。cosmic-text#11がオープンのまま(2022-10起票)を確認。swashは全文検索1件のみで、それも「Add the ability to flip outlines」= 縦書きレイアウトissueではない(メモの「該当ゼロ」は実質正しい)。**訂正が要るのはparley**: [linebender/parley#634](https://github.com/linebender/parley/issues/634)「Implement `parley_core` (analysis, shaping, reshaping breaks, **vertical text**, inline objects)」(2026-06-01、オープン)が存在し、本文は「everything relevant to shaping should be in `parley_core` (including features like vertical writing modes), but it does not provide layout (stacking of lines, wrapping around floated boxes, alignment, etc.)」と、**Motoliiの分界仮説と同型の切り方を同一エコシステムが提案中**。結論「未対応(出荷済み実装なし)」は不変だが、(a) 「issue不在」の記述は差し替え、(b) parley_coreの行方はP6の設計妥当性・将来の部品調達の両面でwatch対象にする価値がある。

**V-16(typst RFC)— 支持**。typst#5908がオープンであること、要求分解(縦組みでの回転・CJK直立・vert有効化・縦中横)を本文で確認。

## C-1〜C-7の再判定

### 前提: 「凍結ゲート前に入れないと入れにくいか」の一方通行扉分析

タスクの核心質問に先に答える。P6は凍結ゲート(2026-07-10宣言)後の凍結対象タスクだが、**motolii-textクレートはまだ存在しない**(workspace確認済み: crates/にmotolii-text無し)。つまりP6契約は「文書上の決定」であって、コードもゴールデンも未着工。契約変更の費用は今なら**文言編集のみ**。着工後は解凍手続き(変更理由+migrate経路+ゴールデン更新の3点セット)と、第1号歌詞プラグインへの波及になる。

そのうえで、C-1〜C-7のどれも**ユーザーデータ・公開契約への不可逆な焼き込みではない**:

- C-1は`#[non_exhaustive]`な出力構造体にすれば後からフィールドを足せる(読む側は壊れない)
- C-2はtraitメソッドへの引数追加=破壊的変更だが、v1はworkspace内静的リンクで機械的に直せる。公開SDK(v2)前に入れれば外部影響ゼロ
- C-4/C-5は純追加API、C-6は既存契約の範囲、C-7は挙動調査マター

したがって「今決めないと詰む」ものは**無い**。推奨判定は不可逆性ではなく「未着工の今なら費用≈0のもの」と「実費が発生するもの」の分離で決めた。

### C-1. shape出力の方向中立語彙 — **採用(文言のみの縮小形)**

事実基盤は最強(V-10で`y_advance`と原点減算済みoffsetの焼き込みをソース確認)。ただし推奨するのは「縦書き対応」ではない: **harfrust/HarfBuzzの自然な出力形は元々グリフごとの(x_advance, y_advance, x_offset, y_offset)であり、横専用契約はそれを半分捨てる射影**である。P6契約文の「送り幅」を「グリフごとの2D advance+2D offset(横書き経路では既存ゴールデンと同値)」へ明文化し、出力型を`#[non_exhaustive]`にする。それだけ。縦ゴールデン・縦テストは足さない。費用は文言1〜2行、防げるのは「最初の実装者が`Vec<f32>`の横送り配列で型を切り、第1号プラグインがそれを読み始めた後の型変更波及」。

### C-2. OpenTypeフィーチャの透過口 — **採用(弱い採用・最小透過口)**

C-1〜C-7で唯一「プラグイン側で吸収する」が原理的に不可能な口(フィーチャはshape呼び出しの中を通るしかない)。事実基盤も確認済み: vertは方向から自動(V-8/V-10)だが、vkna(既定オフ・裁量)・vpal/vchw/vhal・hwid/twid/qwid・vrtrは呼び出し側指定(OpenType registry逐語確認)。出荷済み最小実装のlibassも呼び出し側指定で組んでいる(V-5)。さらにこの口は縦書き専用ではない — 横書きでもpalt/tnum/smcp/ss01等はモーショングラフィックスの現実的な要求。

ただし正直に縮小要素を並べる: (a) V-13の縮小により「無いと縦中横不能」は言えなくなった(幾何スケーリングが仕様公認の代替)、(b) 第1号歌詞プラグイン(横書き)の必須要件ではない、(c) 後から足す費用もworkspace内なら機械的。よって「今入れないと詰む」ではなく「**shaperネイティブの入力を透過するだけで発明ゼロ・既定空リストなら既存ゴールデン不変**」という費用対効果での弱い採用。形は `features: &[(tag, value, range)]` の透過のみとし、**コアはフィーチャの意味解釈・既定制御を一切持たない**(vchw既定オン等の判断はプラグイン責務)。

### C-3. ラン部分範囲shapeの保証の明文化 — **採用(文言のみ)**

新規口ゼロ。P6契約の「プラグイン側で組める例: 文字ごとサイズ(ラン分割)」が既にこの自由を暗黙に約束しており、text-model.mdのstyle_spans設計も同じ前提に立っている。やることは「itemize結果の部分範囲を任意に切ってshapeへ渡してよい。境界をまたぐシェーピング効果(合字・カーニング等)は失われうる」の2文を契約に足すだけ。縦書きが来ても来なくても要る文言(style_spans分割が既に使う)。

### C-4. 縦行送りメトリクスの公開口 — **延期**

CSS WM4の「UA must synthesize vertical metrics」要求(逐語確認済み)は、縦組版実装者に合成責務があることの証拠であって、**今コアに口を掘る理由にはならない**。`vertical_metrics(font) -> Option<...>`は純追加APIで、縦書きプラグイン着手時に足せば一切の波及がない。未確認事項9(skrifa/read-fontsのvhea/vmtx公開範囲)もその時点の実装調査で足りる。今やる価値があるとすれば「横系メトリクス口をhhea専用の名前にしない」程度の命名配慮だが、これも必須とは言えない。

### C-5. 縦メトリクス欠落フォントの診断 — **延期**

harfrustの既知の穴(V-10、「vertical metrics tablesの無いフォントのみ」とbehdadが明言)は実在するが、この診断は縦書き機能が存在して初めて意味を持つ。豆腐診断(ガード8c)の縦版として、**縦書きプラグイン着手時の完了条件に1行足す**ことを予約するのみ。

### C-6. sideways描画の材料 — **棄却(新規作業として)**

横shape+90°回転はGeckoの実証済みモデル(V-1確認済み)で、P6の`glyph_transform`貫通(契約3項)で既に描ける。足りないのは回転基準点の計算材料=C-4のメトリクスであり、C-4に完全従属。独立の契約変更候補として立てる必要がない。

### C-7. fontiqueフォールバックの縦書き適性 — **延期(論点記録のみ)**

挙動未確認(メモ自身の申告どおり)のまま契約に触るべきでない。かつ実害の受け皿は既にある: ガード8(a)の「同梱Noto Sans CJKだけで日本語完全描画」の下限保証は、フォールバック選定が縦適性を無視しても同梱フォントで受け止める構図。縦書き着手時に「同梱フォントのvert/vmtx/VORG検査(未確認事項4)」と一緒に調べる。

## 元メモとの不一致点(明示)

1. **V-13(縮小)**: 「圧縮には hwid/twid/qwid の適用を要求」→ 正しくは「全文字分の幅バリアントがある場合は必須、無ければ幾何スケーリング等の任意手段を許容」。C-2の根拠を「必須の口」から「品質上限の口」へ格下げ
2. **V-15(訂正)**: parleyに縦書きを明記したオープンissue #634(parley_core提案、2026-06-01)が存在する。「issue不在」の消極的証拠は差し替え。ただし「未出荷」の結論は不変で、むしろ分界仮説の同型傍証が1件増えた
3. **V-8(出典差し替え)**: freedesktop MLアーカイブは403で再確認可能性の基準を満たさない。HarfBuzzコミットd71c0dfを恒久出典として併記すること
4. **V-3(縮小)**: CoreTextの「縦原点変換の単体公開はP6の参考になる」という抽出は、逐語確認不能のAPI面に依拠するため設計参考から外す(API実在の記録のみ残す)
5. **V-4(因果の再帰属)**: 「口が無い→回転に退化」は「優先度判断→口を作らない→回転が定番化」と読み直す。ただしHixieの「from the beginning」発言により、「方向パラメータを最初から持つP6の予防線」という含意はむしろ強まった
6. **V-5(軽微)**: 引用行番号は数行ずれ(実質への影響なし)。VERTICAL_LOWER_BOUNDの定義はass_font.h L35
7. **V-10(補記)**: 「commit 92d5853時点」の所見はharfrust 0.12.0(2026-07-03)と同時期であり、記録は「≥0.12、縦テスト有効化はPR#52(2025-05マージ)以降」と版で書き直すのが追跡可能

## 「今決める必要があるもの」の最小リスト

P6(motolii-text)着工前に、契約文書に対して:

1. **スコープ宣言の維持**: v1は横書きのみ。実装ガード9(回転による簡易縦書き禁止)据え置き。縦書き機能自体は延期 — これが「より小さい対策」の本体
2. **C-1(文言)**: shape出力を「グリフごとの2D advance+2D offset」と明文化し、出力型を`#[non_exhaustive]`とする。横書きゴールデンは同値・縦テストは足さない
3. **C-3(文言)**: 「itemize結果の部分範囲を任意にshapeへ渡してよい/境界をまたぐシェーピング効果は失われうる」の2文を追加
4. **C-2(採否の決定のみ)**: 最小透過フィーチャ口(既定空・コアは意味解釈しない)を入れるか、この1点だけP6着工前に決める。本レビューの推奨は弱い採用

それ以外(C-4〜C-7、Noto検査、skrifa調査、CJK vert対応率)はすべて「縦書きプラグイン着手時」へ送る。着工トリガーが来たら、未確認事項4(同梱フォント検査)→9(skrifaメトリクス)→7(fontique適性)の順で1日以内に消化できる粒度である。

## 未確認として残すもの

- freedesktop MLアーカイブ原文(403)。決定の実在はコミットd71c0dfで代替確認済み
- MDN hanging-punctuationの「Safari系限定」の互換表詳細(Baseline外であることのみ確認)
- 元メモの未確認事項1〜10は全点そのまま有効(本レビューで新たに解消したものは無し。うち5=swashは「縦書きレイアウトissue不在」まで確認を進めた)
- parley_core(#634)の行方 — 分界仮説の外部検証として半年単位でwatch

## このレビューの使い方

元メモの所見をP6契約・ゲートへ採用する時は、本レビューの判定語を併記する(規律6)。C-1/C-2/C-3は「不可逆だから今やる」のではなく「未着工の今が最安だから文言で先に締める」という費用判断であり、ユーザーが縦書きの優先度を下げるならC-2の延期も整合的な選択である。縦書き機能の実装判定そのものは、本レビューの対象外(M5未決事項のまま)。
