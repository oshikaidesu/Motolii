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
//! 次切片)の**行の形そのもの**になるよう作ってある — 今回は表と `runs` 分割を建てず、
//! 1行(`TextDocument::style`)だけを持つ。

use serde::{Deserialize, Serialize};

use motolii_core::RationalTime;

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

/// 文字のスタイル。**将来のスタイル表(裁定85: `styles` + `runs`)の行の形そのもの**。
/// この切片では表・`runs` を建てず、`TextDocument::style` として1行(裁定98: 既定行
/// index 0)だけを持つ。範囲ごとに複数行へ分けるのは `text-style`/`text-value-run` 束
/// (次切片)の仕事。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentStyle {
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
    /// `text-document of`(Stroke Over Fill)。縁取りと字面の重ね順。
    pub stroke_over_fill: bool,
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
    /// `text-document f/s/fc/lh/tr/sc/sw/of` の既定行(裁定98)。
    pub style: TextDocumentStyle,
    /// `animated-text-document sid`(Slot ID)。歌詞テンプレートの差し替え口。
    /// **slots(`slot` 発注単位、未着手)と同じ口に乗せる** — 第二の差し替え機構を
    /// 作らない(地図の note どおり)。slots 機構自体がまだ store に無いので、
    /// ここでは参照識別子だけを持つ(解決は slots が生えた日の engine 側の仕事)。
    pub slot_id: Option<String>,
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
