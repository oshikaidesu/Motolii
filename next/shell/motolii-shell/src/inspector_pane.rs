//! Inspector pane(第1波: Transform行 + Attrs行)。
//!
//! **視覚正本は `docs/mocks-ui/public/inspector-library.html` + `.css` そのもの**
//! (発注書 CANON)。旧 `crates/` 側の egui/iced 実装は手本にしない — `next/` は
//! 移植元ではなく成果を作る側(`../GOALS.md` 冒頭の規律どおり)。
//!
//! [`project`] が `StoreView`/[`crate::Session`] から**Document の写しではない、
//! 使い捨ての投影**(`timeline_pane::rows` と同じ形、裁定5)を作る。[`view`] は
//! それを iced widget へ描くだけで、投影の中身を判断しない。
//!
//! **canvas を使わない**(`timeline_pane` と違う選択)。KNOWN.md の実測
//! (「canvas と slider は Simulator から構造的に不可視」)どおり、canvas に
//! text 入力を乗せると Q0 横断柵からも iced_test からも見えなくなる。Inspector の
//! 行は値の型入力・改名・トグルが主役なので、`text_input`/`button` の標準 widget で
//! 組み、柵がそのまま効く形を選んだ(視覚正本と食い違う点として終了報告に書く)。
//!
//! **編集の確定方式**: 打鍵のたびに `Intent` を出すと1文字ごとに undo が割れる
//! (`ui-quality-bar` Q2)。[`crate::Shell`] は [`FieldDraft`]/name 下書きという
//! **Document ではない一時状態**(`crate::Shell::pending_drops` と同じ形)を持ち、
//! `on_submit`(Enter)で初めて1回の `Intent::SetTrack`/`SetAttrs` を出す — 1 gesture
//! = 1 undo。**静的値の編集は `SetTrack` に1キー `Hold`** で書く([`single_hold_track`])
//! — 発注書がその流儀を名指ししている。
//!
//! **型別 editor**(rerun `re_component_ui::create_component_ui_registry` の型→editor
//! 登録表と同じ考え方、コードは引かず型だけ写す): この第1波で使うのは
//! `motolii_eval::Value` のうち `F64`(数値行)と `Vec2`(2連 = X/Y 相当)だけ。
//! `Bool` は Attrs の hidden トグル(`Value` 経由ではなく `LayerAttrs::hidden` だが、
//! 型としては同格の on/off editor)。`Color`/`Enum`/`Path`/`LayerId` は Effect 束
//! (第1波の範囲外)の仕事。

use motolii_core::RationalTime;
use motolii_store::{
    property, Interp, Keyframe, KeyframeTrack, LayerId, PropertyId, StoreError, StoreView, Value,
};

use crate::tokens::{Colors, Dimensions};
use crate::{Message, Session};

// ---------------------------------------------------------------------------
// 型別 editor の対象 field
// ---------------------------------------------------------------------------

/// Transform 行が動かす field の識別。**`LayerId` を持たない** — 対象は常に
/// `Session::selection`(commit 時に読む)。選択が edit の合間に変わる稀なケースは
/// 「そのまま捨てる」で安全側に倒す(`crate::Shell::commit_inspector_field` 参照)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransformField {
    PositionX,
    PositionY,
    PositionZ,
    ScaleX,
    ScaleY,
    Rotation,
    Opacity,
    AnchorX,
    AnchorY,
}

impl TransformField {
    fn property_name(self) -> &'static str {
        match self {
            Self::PositionX | Self::PositionY => property::POSITION,
            Self::PositionZ => property::POSITION_Z,
            Self::ScaleX | Self::ScaleY => property::SCALE,
            Self::Rotation => property::ROTATION,
            Self::Opacity => property::OPACITY,
            Self::AnchorX | Self::AnchorY => property::ANCHOR,
        }
    }
}

/// この field の store 上の property。標準 property は予約語でも空でもないので
/// 失敗し得ない — `crate::Shell` はこの `Result` を「コードの誤り」として扱ってよい。
pub fn property_id(field: TransformField) -> Result<PropertyId, StoreError> {
    PropertyId::new(field.property_name())
}

/// 入力欄の下書き。**Document ではない** — commit(Enter)まで store に触らない。
#[derive(Clone, Debug, PartialEq)]
pub struct FieldDraft {
    pub field: TransformField,
    pub text: String,
}

/// `field` を編集した結果の新しい `Value`。Vec2 系(Position/Scale/Anchor の X/Y)は
/// **現在値の他成分を保つ** — X だけ書き換えて Y を 0 に潰す事故を防ぐ
/// (`current_vec2` は commit 側が `value_at` で読んだ今の値、無ければ [`default_vec2`])。
pub fn next_value(field: TransformField, input: f64, current_vec2: [f64; 2]) -> Value {
    match field {
        TransformField::PositionX | TransformField::ScaleX | TransformField::AnchorX => {
            Value::Vec2([input, current_vec2[1]])
        }
        TransformField::PositionY | TransformField::ScaleY | TransformField::AnchorY => {
            Value::Vec2([current_vec2[0], input])
        }
        TransformField::PositionZ | TransformField::Rotation => Value::F64(input),
        // 表示は % だが store は 0..1 の比(`property::OPACITY` の既定と同じ単位)。
        TransformField::Opacity => Value::F64((input / 100.0).clamp(0.0, 1.0)),
    }
}

/// track が無い(まだキーを打っていない)Vec2 property の既定値。Scale だけ等倍(1.0)、
/// 他は 0(`view.rs::resolve` の既定と同じ、裁定20 の応用)。
pub fn default_vec2(field: TransformField) -> [f64; 2] {
    match field {
        TransformField::ScaleX | TransformField::ScaleY => [1.0, 1.0],
        _ => [0.0, 0.0],
    }
}

/// 静的値を書く唯一の形。**1キー `Hold`**(発注書が名指しした流儀)。時刻は
/// `RationalTime::ZERO` — 1キーだけの track は `KeyframeTrack::eval` がどの時刻でも
/// 同じ値を返す(`t <= keys[0].t` と `t >= keys[last].t` が同じキーに落ちる、
/// `motolii-eval` の実装どおり)ので、時刻自体に意味は無い。
pub fn single_hold_track(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

/// 入力文字列 → 数値。mock(`inspector-library.html`)は負号に `−`(U+2212)を使うので
/// 両対応する。
pub fn parse_number(text: &str) -> Option<f64> {
    text.trim().replace('\u{2212}', "-").parse::<f64>().ok()
}

pub fn format_number(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

// ---------------------------------------------------------------------------
// 投影 — Document の写しではなく、1度描くための使い捨て値(`timeline_pane::rows` と同じ形)
// ---------------------------------------------------------------------------

/// 1成分(X/Y/Z、または scalar 1個)の投影。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentSlot {
    pub axis: &'static str,
    /// store の意味モデルにこの軸があるか。無ければ表示は `—`(mock の
    /// `emptyComponent` と同じ — Rotation の X/Y、Scale/Anchor の Z は Motolii の
    /// 2.5D モデル(裁定113)に無いので `false`)。
    pub present: bool,
    /// 表示単位での値(Opacity だけ % — store は 0..1)。`present=false` なら無意味。
    pub value: f64,
    /// track が 0〜1 キー(裁定20「キーを打っていない property は静止値」の範囲)なら
    /// 編集可。2キー以上(animated)は**この第1波では表示のみ**(発注書の指示 —
    /// 理由つきdisabledではなく、そもそも編集用 control を出さない)。
    pub editable: bool,
    /// この成分が編集される時に動く field。`present=false` なら `None`。
    pub field: Option<TransformField>,
}

fn absent_component(axis: &'static str) -> ComponentSlot {
    ComponentSlot {
        axis,
        present: false,
        value: 0.0,
        editable: false,
        field: None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RowValue {
    Vector([ComponentSlot; 3]),
    Scalar(ComponentSlot),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformRowProjection {
    pub label: &'static str,
    pub value: RowValue,
    pub decimals: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttrsProjection {
    pub name: String,
    pub hidden: bool,
    /// **表示のみ**(KNOWN.md: 対応 mode が Normal だけ — 既知の穴であって新発見では
    /// ない)。`BlendMode` の `Debug` 表示をそのまま使う(`Normal`/`Multiply`/…)。
    pub blend_mode: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionProjection {
    pub layer: LayerId,
    pub transform: Vec<TransformRowProjection>,
    pub attrs: AttrsProjection,
}

fn scalar_component(
    store: &StoreView<'_>,
    layer: LayerId,
    name: &str,
    axis: &'static str,
    field: TransformField,
    t: RationalTime,
    default: f64,
) -> Result<ComponentSlot, StoreError> {
    let property = PropertyId::new(name)?;
    let track = store.track(layer, &property)?;
    let editable = track.as_ref().map(|tr| tr.keys().len() <= 1).unwrap_or(true);
    let value = match store.value_at(layer, &property, t)? {
        Some(Value::F64(v)) => v,
        _ => default,
    };
    Ok(ComponentSlot {
        axis,
        present: true,
        value,
        editable,
        field: Some(field),
    })
}

fn vec2_components(
    store: &StoreView<'_>,
    layer: LayerId,
    name: &str,
    field_x: TransformField,
    field_y: TransformField,
    t: RationalTime,
    default: [f64; 2],
) -> Result<[ComponentSlot; 2], StoreError> {
    let property = PropertyId::new(name)?;
    let track = store.track(layer, &property)?;
    let editable = track.as_ref().map(|tr| tr.keys().len() <= 1).unwrap_or(true);
    let [x, y] = match store.value_at(layer, &property, t)? {
        Some(Value::Vec2(v)) => [v[0], v[1]],
        _ => default,
    };
    Ok([
        ComponentSlot {
            axis: "X",
            present: true,
            value: x,
            editable,
            field: Some(field_x),
        },
        ComponentSlot {
            axis: "Y",
            present: true,
            value: y,
            editable,
            field: Some(field_y),
        },
    ])
}

/// `store`/`session` から選択層の Inspector 投影を組み立てる。**読むだけ**。
/// 選択なし・選択層が削除済み(present でない)・comp が無い、のいずれも `Ok(None)`
/// (M13: 壊れているのではなく「まだ映す物が無い」)。
pub fn project(
    store: &StoreView<'_>,
    session: &Session,
) -> Result<Option<SelectionProjection>, StoreError> {
    let Some(layer) = session.selection else {
        return Ok(None);
    };
    if !store.has_layer(layer) {
        return Ok(None);
    }
    let Some(composition) = store.composition()? else {
        return Ok(None);
    };
    let t = RationalTime::try_from_frame(session.playhead, composition.fps)
        .unwrap_or(RationalTime::ZERO);

    let position_xy = vec2_components(
        store,
        layer,
        property::POSITION,
        TransformField::PositionX,
        TransformField::PositionY,
        t,
        [0.0, 0.0],
    )?;
    let position_z = scalar_component(
        store,
        layer,
        property::POSITION_Z,
        "Z",
        TransformField::PositionZ,
        t,
        0.0,
    )?;
    let position_row = TransformRowProjection {
        label: "Position",
        value: RowValue::Vector([position_xy[0], position_xy[1], position_z]),
        decimals: 3,
    };

    let scale_xy = vec2_components(
        store,
        layer,
        property::SCALE,
        TransformField::ScaleX,
        TransformField::ScaleY,
        t,
        [1.0, 1.0],
    )?;
    let scale_row = TransformRowProjection {
        label: "Scale",
        value: RowValue::Vector([scale_xy[0], scale_xy[1], absent_component("Z")]),
        decimals: 3,
    };

    let rotation_z = scalar_component(
        store,
        layer,
        property::ROTATION,
        "Z",
        TransformField::Rotation,
        t,
        0.0,
    )?;
    let rotation_row = TransformRowProjection {
        label: "Rotation",
        value: RowValue::Vector([absent_component("X"), absent_component("Y"), rotation_z]),
        decimals: 1,
    };

    let mut opacity = scalar_component(
        store,
        layer,
        property::OPACITY,
        "Opacity",
        TransformField::Opacity,
        t,
        1.0,
    )?;
    opacity.value *= 100.0; // store は 0..1、表示は %。
    let opacity_row = TransformRowProjection {
        label: "Opacity",
        value: RowValue::Scalar(opacity),
        decimals: 0,
    };

    let anchor_xy = vec2_components(
        store,
        layer,
        property::ANCHOR,
        TransformField::AnchorX,
        TransformField::AnchorY,
        t,
        [0.0, 0.0],
    )?;
    let anchor_row = TransformRowProjection {
        label: "Anchor",
        value: RowValue::Vector([anchor_xy[0], anchor_xy[1], absent_component("Z")]),
        decimals: 3,
    };

    let attrs = store.attrs(layer)?.unwrap_or_default();
    let attrs_projection = AttrsProjection {
        name: attrs.name,
        hidden: attrs.hidden,
        blend_mode: format!("{:?}", attrs.blend_mode),
    };

    Ok(Some(SelectionProjection {
        layer,
        transform: vec![
            position_row,
            scale_row,
            rotation_row,
            opacity_row,
            anchor_row,
        ],
        attrs: attrs_projection,
    }))
}

// ---------------------------------------------------------------------------
// view — StoreView の投影(SelectionProjection)と下書きだけを受け取る。書けない。
// ---------------------------------------------------------------------------

use iced::widget::{button, column, container, row as row_widget, scrollable, text, text_input};
use iced::{Element, Length};

pub fn view(
    projection: Option<&SelectionProjection>,
    field_draft: Option<&FieldDraft>,
    name_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let header = container(
        text("Inspector")
            .size(dims.title_text)
            .color(colors.text_primary),
    )
    .height(Length::Fixed(dims.panel_header_height))
    .padding([0.0, dims.spacing_m])
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_theme| container::Style {
        border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    });

    let body: Element<'static, Message> = match projection {
        None => empty_state(dims, colors),
        Some(selection) => selected_body(selection, field_draft, name_draft, dims, colors),
    };

    container(column![header, body])
        .width(Length::Fixed(dims.inspector_panel_width))
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_panel)),
            border: iced::Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// **Q0**: 選択なし時は死に chrome を出さない(効かない行を並べない) — 文言1つだけ。
fn empty_state(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    container(
        text("選択なし — layer を選ぶと Transform / Attrs が並ぶ")
            .size(dims.caption_text)
            .color(colors.text_muted),
    )
    .padding(dims.spacing_m)
    .into()
}

fn selected_body(
    selection: &SelectionProjection,
    field_draft: Option<&FieldDraft>,
    name_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let name_label = if selection.attrs.name.is_empty() {
        format!("layer {}", selection.layer.0)
    } else {
        selection.attrs.name.clone()
    };
    let summary = container(
        text(name_label)
            .size(dims.body_text)
            .color(colors.text_primary),
    )
    .height(Length::Fixed(dims.inspector_summary_height))
    .padding(dims.spacing_m)
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(colors.surface_raised)),
        border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    });

    let mut rows = column![summary, section_header("TRANSFORM", dims, colors)];
    for row_projection in &selection.transform {
        rows = rows.push(transform_row(row_projection, field_draft, dims, colors));
    }
    rows = rows.push(attrs_section(&selection.attrs, name_draft, dims, colors));

    scrollable(rows).height(Length::Fill).into()
}

fn section_header(label: &'static str, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    container(
        text(label)
            .size(dims.micro_text)
            .color(colors.text_muted),
    )
    .height(Length::Fixed(dims.inspector_section_header_height))
    .padding([0.0, dims.spacing_m])
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(colors.surface_app)),
        ..container::Style::default()
    })
    .into()
}

fn transform_row(
    row_projection: &TransformRowProjection,
    field_draft: Option<&FieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let label = text(row_projection.label)
        .size(dims.caption_text)
        .color(colors.text_primary)
        .width(Length::Fixed(dims.inspector_label_width));

    let cells: Vec<Element<'static, Message>> = match &row_projection.value {
        RowValue::Vector(components) => components
            .iter()
            .map(|slot| component_cell(slot, field_draft, row_projection.decimals, dims, colors))
            .collect(),
        RowValue::Scalar(slot) => {
            vec![component_cell(slot, field_draft, row_projection.decimals, dims, colors)]
        }
    };

    row_widget![label, row_widget(cells).spacing(dims.spacing_xs)]
        .spacing(dims.spacing_s)
        .height(Length::Fixed(dims.inspector_row_height))
        .align_y(iced::alignment::Vertical::Center)
        .padding([0.0, dims.spacing_m])
        .into()
}

fn component_cell(
    slot: &ComponentSlot,
    field_draft: Option<&FieldDraft>,
    decimals: usize,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    if !slot.present {
        // mock の `emptyComponent`(Turbulent Displace Offset の Z)と同じ意味 —
        // このモデルに無い軸だと明示する(空欄ではなく `—`)。
        return text("—")
            .size(dims.caption_text)
            .color(colors.text_muted)
            .width(Length::Fixed(dims.inspector_value_width))
            .into();
    }

    let displayed = match (slot.field, field_draft) {
        (Some(field), Some(draft)) if draft.field == field => draft.text.clone(),
        _ => format_number(slot.value, decimals),
    };

    let value_widget: Element<'static, Message> = match (slot.editable, slot.field) {
        (true, Some(field)) => text_input("", &displayed)
            .on_input(move |text| Message::InspectorFieldInput(field, text))
            .on_submit(Message::InspectorFieldSubmit(field))
            .size(dims.caption_text)
            .width(Length::Fill)
            .into(),
        // animated(2キー以上) — **表示のみと明示**(理由つきdisabledではなく、
        // そもそも編集 control を出さない。accent 色で「動いている値」と分かる形)。
        _ => text(displayed)
            .size(dims.caption_text)
            .color(colors.action_active)
            .width(Length::Fill)
            .into(),
    };

    row_widget![
        text(slot.axis)
            .size(dims.micro_text)
            .color(colors.text_muted),
        value_widget,
    ]
    .spacing(dims.spacing_xs)
    .width(Length::Fixed(dims.inspector_value_width))
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

fn attrs_section(
    attrs: &AttrsProjection,
    name_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let name_text = name_draft
        .map(|draft| draft.to_owned())
        .unwrap_or_else(|| attrs.name.clone());

    let name_row = row_widget![
        text("Name")
            .size(dims.caption_text)
            .color(colors.text_primary)
            .width(Length::Fixed(dims.inspector_label_width)),
        text_input("", &name_text)
            .on_input(Message::InspectorNameInput)
            .on_submit(Message::InspectorNameSubmit)
            .size(dims.caption_text),
    ]
    .spacing(dims.spacing_s)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m]);

    // `hidden` を先に値として取り出す — closure が `attrs`(呼び出し元の借用)を
    // そのまま move すると `Element<'static, _>` を返せなくなる(bool は Copy なので
    // 値だけ取り出せば closure は借用を持たない)。
    let hidden = attrs.hidden;
    let hidden_row = row_widget![
        text("Hidden")
            .size(dims.caption_text)
            .color(colors.text_primary)
            .width(Length::Fixed(dims.inspector_label_width)),
        button(
            text(if hidden { "On" } else { "Off" }).size(dims.caption_text)
        )
        .on_press(Message::InspectorToggleHidden)
        .style(move |_theme, status| toggle_button_style(dims, colors, status, hidden)),
    ]
    .spacing(dims.spacing_s)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m]);

    // blend: **表示のみ**(対応 mode が Normal だけなのは KNOWN の既知の穴)。
    let blend_row = row_widget![
        text("Blend")
            .size(dims.caption_text)
            .color(colors.text_primary)
            .width(Length::Fixed(dims.inspector_label_width)),
        text(attrs.blend_mode.clone())
            .size(dims.caption_text)
            .color(colors.text_muted),
    ]
    .spacing(dims.spacing_s)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m]);

    column![
        section_header("ATTRS", dims, colors),
        name_row,
        hidden_row,
        blend_row,
    ]
    .into()
}

fn toggle_button_style(
    dims: Dimensions,
    colors: Colors,
    status: button::Status,
    active: bool,
) -> button::Style {
    let background = if active {
        colors.state_selected
    } else {
        match status {
            button::Status::Hovered => colors.surface_hover,
            _ => colors.surface_raised,
        }
    };
    button::Style {
        background: Some(iced::Background::Color(background)),
        text_color: colors.text_primary,
        border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number_accepts_the_mock_minus_sign() {
        assert_eq!(parse_number("−0.075"), Some(-0.075));
        assert_eq!(parse_number("12.5"), Some(12.5));
        assert_eq!(parse_number("  3  "), Some(3.0));
        assert_eq!(parse_number("not a number"), None);
    }

    #[test]
    fn format_number_respects_decimals() {
        assert_eq!(format_number(1.0, 3), "1.000");
        assert_eq!(format_number(24.0, 1), "24.0");
        assert_eq!(format_number(100.0, 0), "100");
    }

    #[test]
    fn next_value_preserves_the_other_vec2_component() {
        assert_eq!(
            next_value(TransformField::PositionX, 5.0, [1.0, 2.0]),
            Value::Vec2([5.0, 2.0])
        );
        assert_eq!(
            next_value(TransformField::PositionY, 5.0, [1.0, 2.0]),
            Value::Vec2([1.0, 5.0])
        );
    }

    #[test]
    fn next_value_converts_opacity_percent_to_the_stored_fraction() {
        assert_eq!(next_value(TransformField::Opacity, 50.0, [0.0, 0.0]), Value::F64(0.5));
        // クランプ: 100 を超える入力・負の入力は store の 0..1 に収める。
        assert_eq!(next_value(TransformField::Opacity, 150.0, [0.0, 0.0]), Value::F64(1.0));
        assert_eq!(next_value(TransformField::Opacity, -10.0, [0.0, 0.0]), Value::F64(0.0));
    }

    #[test]
    fn single_hold_track_has_exactly_one_hold_keyframe() {
        let track = single_hold_track(Value::F64(2.5));
        assert_eq!(track.keys().len(), 1, "静的値は1キーのはず");
        assert_eq!(track.keys()[0].value, Value::F64(2.5));
        assert!(matches!(track.keys()[0].interp, Interp::Hold));
    }

    #[test]
    fn default_vec2_is_identity_scale_and_zero_elsewhere() {
        assert_eq!(default_vec2(TransformField::ScaleX), [1.0, 1.0]);
        assert_eq!(default_vec2(TransformField::PositionX), [0.0, 0.0]);
        assert_eq!(default_vec2(TransformField::AnchorY), [0.0, 0.0]);
    }
}
