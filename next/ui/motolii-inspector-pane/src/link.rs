//! LINK section ── 2026-08-22 発注「レイヤーを指す」文法、第3号。
//!
//! **背景**: `PropertySource::Link`(型付き link、裁定206)は本日 store へ
//! 着地したが `source_layer` を選ぶ口が無く、`Intent::SetPropertyLink` の
//! 呼び手がゼロだった。この module がその初めての呼び手 ── **plugin_id は
//! 最小限(恒等)で構わない、器が通ることが目的**(発注書どおり)。
//!
//! **持つ**: [`LinkTarget`](v1 の link 対象 ── 標準 property 5種のみ。
//! mask/effect param のような id 付き property は対象外、「器を通す」範囲の
//! 絞り方)・投影の構成要素([`LinkSourceCandidate`]/[`LinkRowProjection`] ──
//! 定義はここ、組み立ては [`crate::projection::project`])・書き口
//! ([`commit_inspector_link`]/[`clear_inspector_link`])・LINK section の
//! view([`link_section`])。
//!
//! **持たない**: 候補の絞り込み(自分自身を選べない・循環しない)は
//! [`crate::projection::project`] が `&StoreView` を持つ時点で行う
//! (`link_would_cycle` ── store 側の書き込み時循環拒否
//! `validate_no_link_cycle` と同じロジックの UI 側複製、`projection.rs` 冒頭
//! doc 参照)。ここは絞り込み済みの投影を描くだけ。

use motolii_core::RationalTime;
use motolii_store::{
    property, Document, Intent, LayerId, PropertyId, PropertyLink, PropertySource, Value,
};

use motolii_settings_pane::chrome::section_header;
use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{button, column, pick_list, row as row_widget, text};
use iced::{Element, Length};

use crate::chrome::{bordered_row, flat_button_style, pick_list_style, value_cell_padding};
use crate::projection::LayerCandidate;
use crate::transform::single_hold_track;
use crate::Message;

/// v1 の link 対象(発注書「器が通ることが目的」の範囲 ── 標準 property 5種
/// のみ、mask/effect param のような id 付き property は対象外)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LinkTarget {
    Position,
    Scale,
    Rotation,
    Opacity,
    Anchor,
}

impl LinkTarget {
    pub const ALL: [LinkTarget; 5] = [
        LinkTarget::Position,
        LinkTarget::Scale,
        LinkTarget::Rotation,
        LinkTarget::Opacity,
        LinkTarget::Anchor,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LinkTarget::Position => "Position",
            LinkTarget::Scale => "Scale",
            LinkTarget::Rotation => "Rotation",
            LinkTarget::Opacity => "Opacity",
            LinkTarget::Anchor => "Anchor",
        }
    }

    fn property_name(self) -> &'static str {
        match self {
            LinkTarget::Position => property::POSITION,
            LinkTarget::Scale => property::SCALE,
            LinkTarget::Rotation => property::ROTATION,
            LinkTarget::Opacity => property::OPACITY,
            LinkTarget::Anchor => property::ANCHOR,
        }
    }

    /// 標準 property 名は予約語でも空でもないので構築は失敗し得ない
    /// (`crate::transform::property_id` と同じ `expect` の理由)。
    pub fn property_id(self) -> PropertyId {
        PropertyId::new(self.property_name()).expect("標準 property 名は予約語でも空でもない")
    }

    /// [`Self::property_id`] の逆写像 ── `PropertySource::Link` を読んだ時に
    /// 「その参照先が5種のどれか」を引き直すために要る(手打ちや将来の別
    /// 経路で5種以外の property へ link された Document は `None` ── 捏造
    /// しない、M13)。
    pub fn from_property_name(name: &str) -> Option<LinkTarget> {
        Self::ALL.into_iter().find(|target| target.property_name() == name)
    }

    /// unlink 時に戻す既定値(`crate::transform::default_vec2`/
    /// `scalar_component` の既定と同じ値 ── 発明しない)。
    fn default_value(self) -> Value {
        match self {
            LinkTarget::Position | LinkTarget::Anchor => Value::Vec2([0.0, 0.0]),
            LinkTarget::Scale => Value::Vec2([1.0, 1.0]),
            LinkTarget::Rotation => Value::F64(0.0),
            LinkTarget::Opacity => Value::F64(1.0),
        }
    }
}

/// 「別レイヤーの別 property」1候補(link 元の pick_list が示す選択肢)。
/// **id で区別できる**表示にする発注どおり ── `layer.label` が既に id を
/// 含む([`LayerCandidate`] doc 参照)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkSourceCandidate {
    pub layer: LayerCandidate,
    pub property: LinkTarget,
}

impl LinkSourceCandidate {
    fn display_label(&self) -> String {
        format!("{} · {}", self.layer.label, self.property.label())
    }
}

/// LINK section の1行(標準 property 1種ぶん)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkRowProjection {
    pub target: LinkTarget,
    /// 現在この property が link されているなら、その参照先。**参照先が
    /// 5種の外**(手打ち等で作られた Document)なら `None` ── 「link されて
    /// いない」と区別できないが、この切片の範囲外(M13: 無い意味を有るふりで
    /// 出さない、捏造しない側に倒す)。
    pub current: Option<LinkSourceCandidate>,
    /// 選べる (layer, property) の組(`project` が自己参照/循環を除外済み)。
    pub candidates: Vec<LinkSourceCandidate>,
}

/// LINK 元を選ぶ(pick_list からの選択)。**即1回の `Intent::SetPropertyLink`**
/// (`PickFont`/`PickMatteSource` と同じ即時操作の形)。`plugin_id` は
/// `"motolii.link.identity"` 固定(発注書「plugin_id は最小限で構わない」)・
/// `time_offset` は 0・`params` は空 ── 器を通すのが目的で、係数 UI は次切片。
/// 選択なしは黙って no-op。
pub fn commit_inspector_link(
    doc: &mut Document,
    selection: Option<LayerId>,
    target: LinkTarget,
    source: LinkSourceCandidate,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let link = PropertyLink {
        source_layer: source.layer.id,
        source_property: source.property.property_id(),
        time_offset: RationalTime::ZERO,
        plugin_id: "motolii.link.identity".to_owned(),
        params: Vec::new(),
    };
    doc.apply(Intent::SetPropertyLink {
        layer,
        property: target.property_id(),
        link,
    })
    .map_err(|error| format!("property link を書けない: {error}"))
}

/// LINK を外す ── **`Intent::SetTrack` を再び投げるだけ**(`crate::slot`
/// doc「`SetTrack`/`SetPropertySlot` を再び投げれば普通の track/slot へ
/// 戻せる」を link にもそのまま適用、専用の「解除」variant は要らない)。
/// 戻す値は `t=0` で評価した**現在の見た目**(link が動いていた場合、それを
/// 保つ ── `text_document_content` が `RationalTime::ZERO` 固定で読むのと
/// 同じ「複雑な時間評価を持ち込まない」判断)。読めなければ
/// [`LinkTarget::default_value`] へ落ちる(壊れているのではなく無いだけ、
/// M13)。既に link でなければ no-op(決定7)。
pub fn clear_inspector_link(
    doc: &mut Document,
    selection: Option<LayerId>,
    target: LinkTarget,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let property = target.property_id();
    match doc
        .view()
        .property_source(layer, &property)
        .map_err(|error| format!("property の出処を読めない: {error}"))?
    {
        Some(PropertySource::Link(_)) => {}
        _ => return Ok(()), // link でない(stale click)── 黙って捨てる。
    }
    let value = doc
        .view()
        .value_at(layer, &property, RationalTime::ZERO)
        .map_err(|error| format!("値を読めない: {error}"))?
        .unwrap_or_else(|| target.default_value());
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track: single_hold_track(value),
    })
    .map_err(|error| format!("track を書けない: {error}"))
}

/// LINK section: 標準 property 5種を固定の行で並べる(mask/effect と違い
/// 「無ければ出さない」の Q0 判断は適用しない ── どの layer でも他 layer を
/// 指せて良いはずなので常に現れる)。
pub(crate) fn link_section(
    links: &[LinkRowProjection],
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut section = column![section_header("LINK", dims, colors)];
    for row in links {
        section = section.push(link_row(row, dims, colors));
    }
    section.into()
}

/// LINK 行1本。pick_list で参照先を選び(`row.candidates` ── 自己参照/循環を
/// 除いた候補、`project` が絞り込み済み)、Clear は常に出す(Speed Reset と
/// 同じ「無反応ゼロより一貫」判断 ── link でない時は no-op)。
fn link_row(row: &LinkRowProjection, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let target = row.target;
    let selected = row.current.clone();
    let options = row.candidates.clone();

    let picker = pick_list(selected, options, |candidate: &LinkSourceCandidate| {
        candidate.display_label()
    })
    .on_select(move |candidate: LinkSourceCandidate| Message::PickLinkSource(target, candidate))
    .text_size(dims.body_text)
    .width(Length::Fixed(dims.inspector_value_width))
    .padding(value_cell_padding(dims))
    .placeholder("Pick…")
    .style(move |_theme, status| pick_list_style(dims, colors, status));

    let content = row_widget![
        text(target.label())
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        picker,
        button(text("Clear").size(dims.caption_text))
            .on_press(Message::ClearLink(target))
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}
