//! text-layer の**静的組版**の意味 — content・スタイル既定値・フォント参照。
//!
//! `text` 発注単位(75行)の第1切片。**アニメータ・range-selector・text-modifier(rive)・
//! follow-path は建てない**(次切片) — ここに置くのは Lottie の `text-document` /
//! `animated-text-document` / `font` / `font-list` / `text-data`(`d` のみ)と、
//! それに対応する Rive `text`(box の alignValue/width/height)だけ。ラスタライズ
//! (字形を実際に描く経路)もこの切片ではやらない。
//!
//! **裁定98**: line-height / tracking の正本が3つ(document 基底 / スパン / アニメーター)
//! になっていた穴を塞ぐ決定。Rive には document 基底が存在せず、`Text` はフォントも
//! サイズも持たず全部 `TextStyle` にある。裁定85(「表 + 分割が content を隙間なく覆う」)
//! が既に決めている以上、Lottie の `text-document f/s/fc/lh/tr/sc/sw/of` は独立した
//! document 値ではなく**「スタイル表の既定行(index 0)」**と読み直す。そのため
//! [`TextDocumentStyle`] は将来の範囲スタイル表(裁定85、`text-style`/`text-value-run` 束、
//! 次切片)の**行の形そのもの**になるよう作ってある — この第1切片の時点では表と `runs`
//! 分割を建てず、1行だけを持っていた(表 `styles`/分割 `runs` は第3切片が実際に建てた —
//! 下の節参照)。
//!
//! ## 第2切片: range selector とアニメータの Document 意味(裁定75/76)
//!
//! ここに足すのは Lottie の `text-data`(`a`=Ranges/`m`=Alignment)・`text-range`・
//! `text-range-selector`・`text-style`(アニメーター側の5フィールド: fc/ls/sc/sw/t)。
//! **Rive の `text-modifier-group`/`text-modifier-range`(グリフの位置・回転・拡大・
//! 不透明度)・`text-value-run`・`style_spans`・字形ラスタライズは触らない**(次切片)。
//!
//! - [`TextRange`] は AE のアニメーター1個。**動かす property(`text-range a` Style)を
//!   フィールドとして持たない** — マスク([`crate::MaskId`])/effect([`crate::EffectId`])
//!   と同じ理由で、動く量は [`crate::PropertyId::text_range_fill_color`] 等の平坦な
//!   `KeyframeTrack` に乗る。**track の有無自体が「この animator がその属性を触るか」を
//!   表す**(裁定20 の応用)。position/scale/rotation/opacity(`helpers/transform` 継承)は
//!   当時は Rive の `text-modifier-group` が正本を持つまで持ち越しだったが、第3切片が
//!   `PropertyId::text_range_position` 等として実装した(下の節参照)。**`skew` だけは
//!   持ち越し先が無い** — Rive の `text-modifier-group` は skew を持たない(地図が
//!   確認した実物の欠)ので、text 束としては未解決のまま残る。
//! - [`TextRangeSelector`] も同じ理由で **動く部分(Start/End/Offset/Max Amount)を
//!   持たない** — `PropertyId::text_range_selector_offset` 等に乗る。「カラオケワイプは
//!   Offset を時間駆動するだけに畳める」という地図の note どおり、offset は本質的に
//!   時間変化する値なので静止フィールドでは表現できない。
//! - `text-range-selector rn`(Randomize)は Lottie の int-boolean のままでは
//!   Preview=Export(M15/D1)と両立しない(裁定75/101)。[`TextRandomize`] で seed を
//!   必ず持たせる — 先例が無いのでここで自前設計する。
//! - 複数アニメーターの合成規則(旧 text-model.md の骨格: 並び順=逐次適用、
//!   各アニメーターは全 property を加算)は `TextDocument::ranges`(`Vec<TextRange>`)の
//!   **並びと `id` の同一性**だけを固定する。加算そのものは評価器(engine、未着手)の仕事。
//!
//! ## 第3切片: Rive 由来の modifier / style span(裁定76/82/85/92/93/95/97/98/99)
//!
//! `text` 発注単位(44行)の最終切片。地図の残り28行(`source=rive`)を閉じる —
//! `text-modifier-group` / `text-modifier-range` / `text-style` / `text-style-axis` /
//! `text-style-feature` / `text-style-paint` / `text-value-run` / `text-variation-modifier`。
//! **字形ラスタライズはやらない**(意味だけ。shaping は別束)。
//!
//! - **`text-modifier-range` の6行(modifyFrom/modifyTo/strength/unitsValue/typeValue/
//!   offset)は新しいコードを要らない** — 地図の note どおり、Lottie `text-range-selector`
//!   の `s`/`e`/`a`/`r`/`b`/`o` と**同じ値の Rive 側の呼び名**でしかない(裁定90)。
//!   第2切片が建てた [`TextRangeSelector::based_on`]/[`TextRangeSelector::range_units`]、
//!   [`crate::PropertyId::text_range_selector_start`] 等をそのまま指す。
//! - **`text-style`(rive)の6行(fontSize/lineHeight/letterSpacing/fontAssetId/
//!   familyName/styleName)も新しいコードを要らない** — 裁定98 が「Lottie の
//!   `text-document f/s/fc/lh/tr/sc/sw/of` はスタイル表の既定行(index 0)」と読み直した
//!   とおり、[`TextDocumentStyle`] の `size`/`line_height`/`tracking`/[`FontRef`] が
//!   そのまま Rive `text-style` の形である。**この切片が変えるのは器**(`TextDocument`
//!   が今までの1行だけの `style` ではなく [`TextDocumentStyle::id`] つきの `styles`
//!   (表)を持つようになる)。
//! - **`text-modifier-group`(8行、グリフの位置・回転・拡大・不透明度)** は
//!   [`crate::PropertyId::text_range_origin`]/[`crate::PropertyId::text_range_position`]/
//!   [`crate::PropertyId::text_range_rotation`]/[`crate::PropertyId::text_range_scale`]/
//!   [`crate::PropertyId::text_range_opacity`] の平坦な `KeyframeTrack` に乗る —
//!   `text-range-selector`/`text-style` と同じ「track の有無=触るか」(裁定20)。
//!   `x`/`y` と `scaleX`/`scaleY` は Rive の `"group"` 注記どおり Vec2 1個にまとめる
//!   (裁定61 と衝突しない、地図の note どおり)。**`modifierFlags` は不採用** —
//!   Motolii は track の有無自体が有効フラグなので第二の帳簿を持たない。
//! - **`text-style-axis`/`text-value-run`/`text-style-feature`/`text-variation-modifier`**
//!   が [`TextDocumentStyle`]/[`TextRange`] を実際の表・分割へ広げる新規部分:
//!   - [`TextStyleId`] でスタイル表 `TextDocument::styles` の各行に安定 id を持たせる
//!     (裁定85「スパンに値を直書きしない」— 添字だと削除/並べ替えで別の行へ付き直る)
//!   - [`TextRun`] が `TextDocument::runs`(`Vec<TextRun>`)で content を隙間なく
//!     重なりなく覆う分割を表す(裁定85/87/88/89)。`styleId` は `TextRun::style`
//!   - [`TextStyleAxis`] が `TextDocumentStyle::axes` で可変フォント軸への束縛
//!     (どのタグに乗るか)を持つ。**値(axisValue)は [`crate::PropertyId::text_style_axis`]
//!     の track**(裁定92 の唯一の例外 — 軸だけはスタイル層でアニメ可、裁定93)
//!   - [`TextStyleFeature`] が `TextDocumentStyle::features` で OpenType feature の
//!     タグと値を静止設定として持つ(裁定99。v1 はキーフレーム化しない、裁定92のまま
//!     — Rive の `featureValue` は `animates: true` だが例外リストに入っていない)
//!   - [`TextVariationAxis`] が `TextRange::variation_axes` で「再シェープする層」
//!     (裁定76 の3層目)のアニメーターが動かす軸タグを持つ。値(Δ)は
//!     [`crate::PropertyId::text_range_variation_axis`] の track — スパン側の
//!     axisValue(絶対値)とは層が違うので二重帳簿ではない(地図の note どおり)
//! - **`text-style-paint`(@class)も新しいコードを要らない** — 「fill/stroke をスタイル表
//!   の1行に平坦に持ち、重ね順は `of` の bool 1個」は [`TextDocumentStyle::stroke_over_fill`]
//!   がすでに体現している

use serde::{Deserialize, Serialize};

use motolii_core::RationalTime;

use crate::SlotId;

/// フォント参照。実体は path + 指紋(**素材と同じ形**、裁定79/97 — Lottie の名前参照は
/// 採らない)。family/style は解決キー(編集時だけ要る、Rive `runtime: false` の裏、裁定97)。
///
/// **font-list は第二の素材台帳を作らない** — 表を別に持たず、参照する側
/// (`TextDocumentStyle::font`)が直接この型を持つ(裁定79「表 + refId は1枚 JSON を
/// 自己完結させる配信の都合であって編集器の Document の意味ではない」と同じ理由)。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontRef {
    /// フォント実体の在処(`font fPath`)。
    pub path: String,
    /// 内容識別。無くても描ける(`LayerSource::Media.fingerprint` と同じ理由)。
    pub fingerprint: Option<String>,
    /// family 名(`font fFamily`)。実体解決のキー。
    pub family: String,
    /// face 選択キー(`font fStyle`)。
    pub style: String,
}

/// 水平揃え(`text-document j`)。Rive `text.alignValue` が同じ意味を持つと地図の note が
/// 明示している(Rive text.alignValue) — Rive も animates を持たない = 静止設定という点まで
/// 一致するので、ここでは普通の enum(静止設定)として持つ。両端揃え4種は行分割器が要るので
/// 後回し(3値のみ)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextJustify {
    Left,
    Right,
    Center,
}

/// text-document `t`(Text)の1ホールドキー。**i/o(補間ハンドル)を持たない** —
/// Lottie 自身が構造でホールドを保証しており(`animated-text-document k`)、一般の
/// `KeyframeTrack`(`motolii_eval::Value`)には乗らない — `Value` に文字列バリアントが
/// 無い(裁定78 が意図的に除外したまま。次に要る日まで足さない、軸4)。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentKeyframe {
    pub t: RationalTime,
    pub content: String,
}

/// content の時間変化。**時刻昇順を型で保証する**(`motolii_eval::KeyframeTrack` と
/// 同じ形 — `insert` が二分探索で挿入位置を決め、同時刻は置き換える)。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentTrack {
    keys: Vec<ContentKeyframe>,
}

impl ContentTrack {
    pub fn new() -> Self {
        Self::default()
    }

    /// キーを挿入する。同時刻のキーが既にあれば置き換える。
    pub fn insert(&mut self, key: ContentKeyframe) {
        match self.keys.binary_search_by(|k| k.t.cmp(&key.t)) {
            Ok(i) => self.keys[i] = key,
            Err(i) => self.keys.insert(i, key),
        }
    }

    pub fn keys(&self) -> &[ContentKeyframe] {
        &self.keys
    }

    /// **Hold 評価**(`animated-text-document k`)。t 以前で最後に打たれたキーの内容を
    /// そのまま返す — 線形補間もイージングも無い(文字列に「中間」は無い)。
    /// キーが1つも無ければ空文字列。
    pub fn eval(&self, t: RationalTime) -> &str {
        let keys = &self.keys;
        let Some(first) = keys.first() else {
            return "";
        };
        if t <= first.t {
            return &first.content;
        }
        let last = keys.len() - 1;
        if t >= keys[last].t {
            return &keys[last].content;
        }
        let i = match keys.binary_search_by(|k| k.t.cmp(&t)) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        &keys[i].content
    }
}

/// スタイル表([`TextDocument::styles`])の安定 ID。**添字ではない** —
/// [`TextRangeId`]/[`crate::MaskId`] と同型の理由(裁定85「スパンに値を直書きしない」)。
/// 添字だと `styleId` で参照する [`TextRun`] や `text_style.{id}.axis.{tag}` の track が、
/// 表の削除/並べ替えで別の行へ黙って付き直る。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextStyleId(pub u32);

impl std::fmt::Display for TextStyleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 可変フォント軸への束縛(`text-style-axis`)。**裁定92 の唯一の例外** — 他のスタイル
/// 属性は v1 で静止(裁定92)だが、軸値はシェーピングの入力そのものなので P6「軸だけは
/// スタイル層でアニメ可」に当たる(裁定93)。ここに持つのは**どのタグに乗るか**だけで、
/// 値は [`crate::PropertyId::text_style_axis`] の `KeyframeTrack`(track の有無=このスタイルが
/// その軸を触るか、裁定20 の応用 — タグは開放集合なので固定 property のように暗黙列挙できず、
/// 明示的な一覧が要る)。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextStyleAxis {
    /// `text-style-axis tag`。可変フォントの軸タグ(例: `"wght"`/`"wdth"`)。
    pub tag: String,
}

/// OpenType feature の切替(`text-style-feature`)。裁定99 — `text-document ca`
/// (Small Caps)不採用が「正本は OpenType feature」と書いたのに、その置き場が地図に
/// 無かった穴を塞ぐ。日本語歌詞は `palt`(プロポーショナルメトリクス)の有無で詰め方が
/// 別物になる。**v1 は静止**(裁定92のまま — Rive の `featureValue` は `animates: true`
/// だが、裁定93 の例外リストは軸値だけで feature は含まない)。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextStyleFeature {
    /// `text-style-feature tag`。OpenType feature タグ(`"smcp"`/`"liga"`/`"palt"` 等)。
    pub tag: String,
    /// `text-style-feature featureValue`。
    pub value: u32,
}

/// 文字のスタイル。**スタイル表([`TextDocument::styles`])の1行**(裁定85: `styles` +
/// `runs` の `styles` 側)。裁定98 により、旧来の Lottie `text-document f/s/fc/lh/tr/
/// sc/sw/of` は「独立した document 値」ではなくこの表の**既定行(id 0)**として読み直す。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentStyle {
    /// 表の中の安定 id。[`TextRun::style`] がこれを指す。
    pub id: TextStyleId,
    /// `text-document f`(Font Family)。
    pub font: FontRef,
    /// `text-document s`(Font Size)。組版のサイズ。animator の scale とは別物。
    pub size: f32,
    /// `text-document fc`(Fill Color)。字面色。`motolii_eval::Value::Color` と同じ
    /// RGBA・非線形sRGB・straight-alpha・各成分0.0–1.0。
    pub fill: [f64; 4],
    /// `text-document lh`(Line Height)。行送り。`None` = 未指定(フォントのメトリクスから)。
    pub line_height: Option<f32>,
    /// `text-document tr`(Tracking)。基底トラッキング。
    pub tracking: f32,
    /// `text-document sc`(Stroke Color)。縁取り色。歌詞では必須級。
    pub stroke_color: Option<[f64; 4]>,
    /// `text-document sw`(Stroke Width)。縁取り幅。
    pub stroke_width: f32,
    /// `text-document of`(Stroke Over Fill)。縁取りと字面の重ね順。**`text-style-paint`
    /// の`@class`(fill/stroke を持つ形)を子ではなくこの bool 1個で平坦に持つ**
    /// (裁定85/86 — Rive の子の並び順が重ね順になる形は採らない)。
    pub stroke_over_fill: bool,
    /// `text-style-axis` の列。可変フォント軸への束縛(タグのみ、値は track)。
    pub axes: Vec<TextStyleAxis>,
    /// `text-style-feature` の列。OpenType feature の静止設定。
    pub features: Vec<TextStyleFeature>,
}

/// アニメーターの安定 ID。マスク([`crate::MaskId`])/effect([`crate::EffectId`])と
/// 同型の理由 — 添字だと並べ替え/削除で別アニメーターの property track へ付き直る
/// (裁定65/85 と同型の問題)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextRangeId(pub u32);

impl std::fmt::Display for TextRangeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 範囲選択器の粒度(`constants/text-based`)。AE 由来で発明の余地が無い。
/// **「文字」はクラスタに読み替える**(裁定75 — 単語分割器を回避する安い代替)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextBasedOn {
    Characters,
    CharactersExcludingSpaces,
    Words,
    Lines,
}

/// 範囲値の単位(`constants/text-range-units`)。歌詞は Index、テンプレートは Percent。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextRangeUnits {
    Percent,
    Index,
}

/// 重みカーブの形(`constants/text-shape`)。text-model §6「shape の初期セット」の
/// 未決がこれで埋まる(裁定75)。`text-range-selector sh` が使う。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextShape {
    Square,
    RampUp,
    RampDown,
    Triangle,
    Round,
    Smooth,
}

/// グループ内アンカーの粒度(`constants/text-grouping`)。`text-alignment-options g`
/// が使う。text-model §6 の未決がここで埋まる(裁定75)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextGrouping {
    Characters,
    Word,
    Line,
    All,
}

/// **seed 付き randomize**(裁定75/101)。Lottie の `text-range-selector rn` は
/// int-boolean で seed を持たず、再生器ごとに結果が変わって Preview=Export(M15/D1)と
/// 両立しない。有効/無効は `Option` の有無で表し(番兵を採らない、裁定94 と同じ形)、
/// 有効なら必ず seed を伴う形にする。**先例が無いのでここで自前設計する**(裁定101)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRandomize {
    pub seed: u64,
}

/// `text-range-selector` の**静止部分**だけ。動く部分(`s`/`e`/`o`/`a` — Start/End/
/// Offset/Max Amount)はマスクの形状・不透明度と同じ理由で property track に乗る
/// ([`crate::PropertyId::text_range_selector_start`] 等)。**カラオケワイプは
/// `o`(Offset)を時間駆動するだけに畳める**という地図の note どおり、offset は
/// 本質的に時間変化する値なので静止フィールドでは表現できない。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRangeSelector {
    /// `text-range-selector b`(Based On)。
    pub based_on: TextBasedOn,
    /// `text-range-selector r`(Range Units)。
    pub range_units: TextRangeUnits,
    /// `text-range-selector sh`(Shape)。
    pub shape: TextShape,
    /// `text-range-selector rn`(Randomize)。`None` = 無効。
    pub randomize: Option<TextRandomize>,
}

/// 可変フォント軸への束縛(`text-variation-modifier`)。裁定76 の3層のうち
/// 「**再シェープする層**」の唯一の住人 — アニメーターがセレクタの重みに応じて軸の
/// **Δ値**を動かす(スパン側の [`TextStyleAxis`] が持つ絶対値とは層が違うので二重帳簿
/// ではない、地図の note どおり)。ここに持つのは**どのタグを動かすか**だけで、Δ値は
/// [`crate::PropertyId::text_range_variation_axis`] の `KeyframeTrack`。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextVariationAxis {
    /// `text-variation-modifier axisTag`。
    pub tag: String,
}

/// AE のアニメーター1個。**1アニメーター=セレクタ1個**(AE の複数セレクタ交差は
/// 採らない、裁定75 — 軸4「拡張の口は trait 1本」と同じ形の単純化)。
///
/// `text-range a`(Style、アニメーターが動かす property の束)は**フィールドとして
/// 持たない** — マスクの形状・effect の param と同じ理由で、動く property は
/// [`crate::PropertyId::text_range_fill_color`] 等(この id が名前空間を作る)の
/// 平坦な `KeyframeTrack` に乗る。**track が存在するかどうか自体が「この animator が
/// その属性を触るか」を表す**(裁定20「キーを打っていない property は静止値」の応用)
/// — 触らない属性のために空の値を持たせる必要が無い。position/scale/rotation/opacity
/// (Rive `text-modifier-group`)も同じ理由で [`crate::PropertyId::text_range_position`]
/// 等の track に乗り、フィールドとしては持たない。
///
/// 複数アニメーターの合成規則(旧 text-model.md の骨格: 並び順=逐次適用、各
/// アニメーターは全 property を加算)は `Vec<TextRange>` の並びと `id` の同一性だけを
/// 固定すれば表現できる — 加算そのものは評価器(engine、未着手)の仕事。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextRange {
    pub id: TextRangeId,
    /// `text-range nm`(Name)。利用者が付ける名前。並び順=適用順を明示する UI に要る。
    pub name: String,
    /// `text-range s`(Selector)。
    pub selector: TextRangeSelector,
    /// `text-variation-modifier` の列(このアニメーターが動かす可変フォント軸)。
    /// タグのみ(値は track) — [`TextStyleAxis`] と同じ理由(裁定76 の3層目)。
    pub variation_axes: Vec<TextVariationAxis>,
}

/// 同じ id のアニメーターが2枚あると、selector/style の property track
/// (`text_range.{id}.…`)の持ち主が決まらない。[`crate::mask::validate_unique_ids`]と
/// 同型の検査 — [`validate`] の中で `document.ranges` に対して呼ぶ。
fn validate_unique_range_ids(ranges: &[TextRange]) -> Result<(), crate::StoreError> {
    for (i, range) in ranges.iter().enumerate() {
        if ranges[..i].iter().any(|other| other.id == range.id) {
            return Err(crate::StoreError::Property(format!(
                "text-range id {} が2枚ある",
                range.id
            )));
        }
    }
    Ok(())
}

/// 同じタグの軸束縛が1つの行/アニメーターに2枚あると、
/// `text_style.{id}.axis.{tag}`/`text_range.{id}.variation.{tag}` の track が
/// どちらの束縛の物か区別できない(値そのものは同じ track を指すので壊れはしないが、
/// 「2つの束縛が同じ track を共有している」という無意味な重複を型で拒む)。
fn duplicate_tag<'a>(mut tags: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let mut seen = std::collections::HashSet::new();
    tags.find(|tag| !seen.insert(*tag))
}

/// `text-alignment-options`(`text-data m`)。グループ内アンカーのオフセットと粒度 —
/// text-model §6 の未決がここで埋まる(裁定75)。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextAlignmentOptions {
    /// `text-alignment-options a`(Alignment)。グループ内アンカーのオフセット
    /// (Lottie と同じくパーセント)。**意味は採り既定だけ変える** — 既定
    /// `[0.0, 0.0]` は「字面中心」を指す(Lottie の既定=左寄せ原点とは異なる)。
    pub anchor_offset: [f32; 2],
    /// `text-alignment-options g`(Grouping)。アンカー点の粒度。
    pub grouping: TextGrouping,
}

impl Default for TextAlignmentOptions {
    fn default() -> Self {
        Self {
            anchor_offset: [0.0, 0.0],
            grouping: TextGrouping::Characters,
        }
    }
}

/// `text-value-run`。本文(現在時刻の `content`)を隙間なく重なりなく覆う分割
/// (裁定85「表 + 分割」の分割側)。**絶対オフセットではなく長さ**で持つ(裁定87 —
/// 本文編集で以降が自動的にずれる。AE の絶対オフセットは「長さが戻ると再び有効に
/// なりうる」反面事例)。並びが `content` 上の出現順そのもの — 添字ではなく `Vec` の
/// 順序が位置を表すので、`len` の合計だけがずれの原因になり、それは [`validate`] が
/// 検査する(隙間/重なりを別に表現する型を持たない、裁定88「重なりは編集の動詞であって
/// 保存形ではない」)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRun {
    /// このランが覆う長さ(裁定90: 拡張書記素クラスタ数 — shaping 前に計算できる単位、
    /// クラスタは selector の単位と揃えてある)。
    pub len: u32,
    /// `text-value-run styleId`。裁定85 の `runs: [(len, style_index)]` の
    /// `style_index` そのもの — [`TextDocument::styles`] の中の [`TextStyleId`] を指す。
    pub style: TextStyleId,
}

/// text-layer の中身。**`Layer:text` component 1個**(素材と同じ JSON 経路)。
/// `LayerSource::Text` の中身の正本(裁定112(k) の後継 — 素の文字列1本だった所を
/// content track・スタイル既定値・フォント参照まで持つ形へ広げる)。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextDocument {
    /// `text-document t` / `animated-text-document k`。**時間変化しうる唯一のフィールド**
    /// (Hold のみ、裁定92「v1でスパン style はキーフレーム化しない」の裏返し —
    /// 動くのは中身の文字列だけで、組版そのものは静止する)。
    pub content: ContentTrack,
    /// `text-document j`(Justify)。Rive `text.alignValue` と同じ意味(text.alignValue)。
    pub justify: TextJustify,
    /// `text-document sz`(Wrap Size)。`None` = point text(折返し無し)。
    /// Rive の `text.width`(Rive text.width)/`text.height`(Rive text.height)が sz の2成分に対応する
    /// (箱幅を動かすと毎フレーム行分割が要るので、Lottie も Rive も静止設定)。
    pub wrap_size: Option<[f32; 2]>,
    /// スタイル表(裁定85: `styles` + `runs` の `styles` 側)。**`id` に同一性を持つ**
    /// ([`TextStyleId`])。裁定98 により、Lottie の `text-document f/s/fc/lh/tr/sc/sw/of`
    /// はこの表の**既定行(`id 0`)**として読み直す — 独立した document 値ではない。
    pub styles: Vec<TextDocumentStyle>,
    /// `animated-text-document sid`(Slot ID)。歌詞テンプレートの差し替え口。
    /// **slots(`slot` 発注単位)と同じ口に乗る** — `slot` 束が実装した
    /// [`crate::SlotId`]/[`crate::Slot`]/[`crate::PropertySource`] をそのまま指す
    /// (第二の差し替え機構を作らない、地図の note どおり)。値の解決自体は
    /// comp の Slots 表を引く evaluator(engine、未着手)の仕事のままだが、参照の型は
    /// もう素の `String` ではない — スロット表の行と1つの型で結ばれている。
    pub slot_id: Option<SlotId>,
    /// `text-data a`(Ranges)。アニメーターの列 — MV タイポグラフィの核心(裁定75)。
    /// 並び=適用順。動く量は `id` で名前空間が決まる `PropertyId` 経由で別途乗る
    /// ([`TextRange`] のドキュメント参照)。
    pub ranges: Vec<TextRange>,
    /// `text-data m`(Alignment、`text-alignment-options`)。グループ内アンカーの
    /// オフセットと粒度。
    pub alignment: TextAlignmentOptions,
    /// `text-value-run` の列(裁定85: `styles` + `runs` の `runs` 側)。空 = `styles` の
    /// 最初の行(既定行)が本文全体を覆う(`ranges` が空 = 無加工と同じ、裁定20 の応用)。
    pub runs: Vec<TextRun>,
}

/// [`TextDocument`] を書く唯一の道([`crate::Intent::SetTextDocument`])が守る不変条件を
/// 1箇所にまとめる。個々の検査はそれぞれ**なぜ要るか**が異なるので関数を分けたが、
/// 呼び出し口は1つにして `document.rs` を薄く保つ。
pub(crate) fn validate(document: &TextDocument) -> Result<(), crate::StoreError> {
    validate_unique_range_ids(&document.ranges)?;
    validate_unique_style_ids(&document.styles)?;
    for range in &document.ranges {
        if let Some(tag) = duplicate_tag(range.variation_axes.iter().map(|a| a.tag.as_str())) {
            return Err(crate::StoreError::Property(format!(
                "text-range {} の variation axis タグ「{tag}」が2枚ある",
                range.id
            )));
        }
    }
    for style in &document.styles {
        if let Some(tag) = duplicate_tag(style.axes.iter().map(|a| a.tag.as_str())) {
            return Err(crate::StoreError::Property(format!(
                "text-style {} の axis タグ「{tag}」が2枚ある",
                style.id
            )));
        }
        if let Some(tag) = duplicate_tag(style.features.iter().map(|f| f.tag.as_str())) {
            return Err(crate::StoreError::Property(format!(
                "text-style {} の feature タグ「{tag}」が2枚ある",
                style.id
            )));
        }
    }
    validate_runs(&document.styles, &document.runs)?;
    Ok(())
}

/// 同じ id のスタイル行が2枚あると、`text_style.{id}.axis.{tag}` の track や
/// [`TextRun::style`] の参照先が決まらない。[`validate_unique_range_ids`] と同型。
fn validate_unique_style_ids(styles: &[TextDocumentStyle]) -> Result<(), crate::StoreError> {
    for (i, style) in styles.iter().enumerate() {
        if styles[..i].iter().any(|other| other.id == style.id) {
            return Err(crate::StoreError::Property(format!(
                "text-style id {} が2枚ある",
                style.id
            )));
        }
    }
    Ok(())
}

/// `runs` が[`TextDocument::styles`]を隙間なく重なりなく覆う分割になっているかを検査する
/// (裁定85/87/88)。**空は常に妥当**(既定行が全体を覆う、上の doc コメント参照)。
///
/// - `len == 0` の run は本文を1文字も覆わない空の分割で、意味を持たない
/// - `style` が `styles` に存在しない run は、どのスタイルも指さない宙ぶらりんの
///   参照になる(裁定37「無いと読めないを区別する」— 書けてしまうと後で気付けない)
/// - **隣接する2つの run が同じ `style` を指してはいけない**(裁定89: 併合は最適化
///   ではなく正しさの契約 — 併合しないと合字/カーニングがスパン境界で切れ、
///   意味が同じ2つの Document が違う画を出す)
fn validate_runs(styles: &[TextDocumentStyle], runs: &[TextRun]) -> Result<(), crate::StoreError> {
    for (i, run) in runs.iter().enumerate() {
        if run.len == 0 {
            return Err(crate::StoreError::Property(format!(
                "text-value-run[{i}] の len が 0(本文を1文字も覆わない run は無意味)"
            )));
        }
        if !styles.iter().any(|style| style.id == run.style) {
            return Err(crate::StoreError::Property(format!(
                "text-value-run[{i}] が styleId {} を指すが、その id のスタイル行が無い",
                run.style
            )));
        }
        if i > 0 && runs[i - 1].style == run.style {
            return Err(crate::StoreError::Property(format!(
                "text-value-run[{}] と [{i}] が同じ styleId {} を指して隣接している。\
                 隣接同値ランは併合すること(裁定89)",
                i - 1,
                run.style
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(num: i64, den: i64) -> RationalTime {
        RationalTime::try_new(num, den).unwrap()
    }

    #[test]
    fn content_track_holds_until_the_next_key() {
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe {
            t: t(0, 1),
            content: "1番".to_owned(),
        });
        track.insert(ContentKeyframe {
            t: t(2, 1),
            content: "2番".to_owned(),
        });

        assert_eq!(track.eval(t(0, 1)), "1番");
        assert_eq!(
            track.eval(t(1, 1)),
            "1番",
            "次のキーまで前の内容を保持する(Hold)"
        );
        assert_eq!(track.eval(t(2, 1)), "2番");
        assert_eq!(track.eval(t(100, 1)), "2番", "末尾はクランプ");
    }

    #[test]
    fn empty_content_track_evaluates_to_empty_string() {
        assert_eq!(ContentTrack::new().eval(t(0, 1)), "");
    }

    #[test]
    fn inserting_the_same_time_replaces_not_duplicates() {
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe {
            t: t(0, 1),
            content: "a".to_owned(),
        });
        track.insert(ContentKeyframe {
            t: t(0, 1),
            content: "b".to_owned(),
        });
        assert_eq!(track.keys().len(), 1);
        assert_eq!(track.eval(t(0, 1)), "b");
    }
}
