//! Inspector pane の絵(M-4b)。**モデルを書く道はここに無い** — 受け取った
//! [`InspectorSeat`] を並べ、押された事実を [`InspectorEvent`] として返すだけ。
//!
//! 構成は 2026-08-13 裁定の採択案 A(4 section 常設)+ 同日の key add UX 決定:
//!
//! 1. identity 行(名前・種別・M / S)。押下状態は Document 由来の model から
//! 2. TRANSFORM: Position / Scale(Vec2 = 1 header + 1 key ボタン + X/Y 値行)、
//!    Rotation / Opacity(値行に key ボタン)。key ボタンは3状態
//! 3. EFFECTS: 共有 FX の一覧 + ON/OFF。param は F64 / Vec2 / Vec3 / Color だけ行を出す
//! 4. Audio: **出さない** — gain のエディタ操作 API がまだ無く、接続できない
//!    chrome は置かない(Q0。座席が立ったら section ごと足す)
//!
//! 配色は iced 既定のまま(theme レーン別走)。独自 hex は置かない。
//! スクラブ部品と key ボタンは widgets レーンの契約
//! ([`crate::widgets_stub`] — INTEGRATION: swap to widgets module)。

use iced::widget::{button, column, container, row, space, text};
use iced::{Center, Element, Fill};

use motolii_ui::blitz_shell::UiEditParam;

use crate::inspector_model::{
    arity, param_label, EffectParamValue, EffectSection, InspectorModel, InspectorSeat, ParamRow,
};
use crate::message::Message;
use crate::widgets_stub::{key_button, scrub_value, ScrubEvent, ScrubSpec};

/// pane の見出し。
pub const TITLE: &str = "Inspector";
/// 選択なしの空状態(egui 版 `seat_live` と同じ文言)。
pub const NO_SELECTION: &str = "No selection \u{2014} pick a layer in the Timeline";
/// TRANSFORM section の見出し。
pub const TRANSFORM: &str = "TRANSFORM";
/// EFFECTS section の見出し。
pub const EFFECTS: &str = "EFFECTS";
/// 共有 FX が1つも無いときの正直な一言(幽霊行を出さない — Q7)。
pub const NO_EFFECTS: &str = "No shared FX";

/// Inspector で押された事実。「では何をするか」(intent 化)は
/// `Shell::update` が決める(M-1 と同じ層の分け方)。
#[derive(Debug, Clone, PartialEq)]
pub enum InspectorEvent {
    /// M を押した。押下状態は Document が持つので反転要求だけ。
    ToggleMute,
    /// S を押した。同上。
    ToggleSolo,
    /// ◇ / ◆ を押した(playhead へキーを打つ / 値を更新する)。
    KeyPressed(UiEditParam),
    /// スクラブ部品の1事象。
    Scrub {
        param: UiEditParam,
        component: usize,
        event: ScrubEvent,
    },
    /// 共有 FX の ON/OFF。`enabled` は**これから成るべき状態**。
    SetEffectEnabled { definition_id: u64, enabled: bool },
}

/// pane ぜんたい。`seat` は accepted snapshot から導出済みの投影で、
/// この関数は状態を持たない(毎フレーム作り直してよい)。
pub fn inspector<'a>(seat: InspectorSeat, editor_status: Option<String>) -> Element<'a, Message> {
    let mut body = column![text(TITLE).size(15)].spacing(10);
    body = match seat {
        InspectorSeat::NoSelection => body.push(text(NO_SELECTION).size(12)),
        InspectorSeat::Multi(count) => body.push(
            text(format!("{count} items selected \u{2014} pick one to edit")).size(12),
        ),
        InspectorSeat::Unreadable(reason) => body.push(text(reason).size(12)),
        InspectorSeat::Ready(model) => body
            .push(identity(&model))
            .push(transform(&model))
            .push(effects(model.effects)),
    };
    // 断り・確定の一言(エディタの status 行)。無ければ席ごと出さない。
    if let Some(status) = editor_status.filter(|status| !status.is_empty()) {
        body = body.push(text(status).size(11));
    }
    container(body).width(Fill).padding(10).into()
}

/// 1. identity 行。名前・種別と、M / S(押下状態は Document 由来)。
fn identity<'a>(model: &InspectorModel) -> Element<'a, Message> {
    let meta = match model.child_count {
        Some(children) => format!(
            "{} \u{b7} {} children \u{b7} {} shared FX",
            model.item_kind,
            children,
            model.effects.len()
        ),
        None => format!(
            "{} \u{b7} {} shared FX",
            model.item_kind,
            model.effects.len()
        ),
    };
    row![
        column![
            text(model.layer_name.clone()).size(14),
            text(meta).size(11)
        ]
        .spacing(2),
        space().width(Fill),
        flag_button("M", model.muted, Message::Inspector(InspectorEvent::ToggleMute)),
        flag_button("S", model.solo, Message::Inspector(InspectorEvent::ToggleSolo)),
    ]
    .spacing(6)
    .align_y(Center)
    .into()
}

/// M / S の1枚。押下状態で iced 既定 style を切り替えるだけ(独自色なし)。
fn flag_button<'a>(label: &'a str, pressed: bool, message: Message) -> Element<'a, Message> {
    button(text(label).size(12))
        .padding([2, 8])
        .style(if pressed {
            button::primary as fn(&iced::Theme, button::Status) -> button::Style
        } else {
            button::secondary
        })
        .on_press(message)
        .into()
}

/// 2. TRANSFORM section。
fn transform<'a>(model: &InspectorModel) -> Element<'a, Message> {
    let mut section = column![text(TRANSFORM).size(11)].spacing(6);
    for param_row in &model.transform {
        section = section.push(transform_rows(param_row));
    }
    section.into()
}

/// Transform 1 param ぶんの行。Vec2 は 1 header + 1 key ボタン + X/Y 値行、
/// scalar は 1 行に値と key ボタン(2026-08-13 mapping)。
fn transform_rows<'a>(param_row: &ParamRow) -> Element<'a, Message> {
    let param = param_row.param;
    let label = param_label(param);
    if !param_row.editable {
        // 閉じていない DocParam 種。値は出すが、触れる部品を置かない(Q0)。
        return row![
            text(label).size(12),
            space().width(Fill),
            text(format_components(&param_row.components)).size(12),
        ]
        .spacing(6)
        .align_y(Center)
        .into();
    }
    let key = key_button(
        param_row.key_state,
        Message::Inspector(InspectorEvent::KeyPressed(param)),
    );
    if arity(param) == 2 {
        // header 行(名前 + key ボタン)。X / Y は独立キーに見せない。
        let mut rows = column![row![text(label).size(12), space().width(Fill), key]
            .spacing(6)
            .align_y(Center)]
        .spacing(3);
        for (component, axis) in ["X", "Y"].into_iter().enumerate() {
            let value = param_row.components.get(component).copied().unwrap_or(0.0);
            rows = rows.push(
                row![
                    text(axis).size(11),
                    space().width(Fill),
                    scrub(param, component, value),
                ]
                .padding(iced::padding::left(12))
                .spacing(6)
                .align_y(Center),
            );
        }
        rows.into()
    } else {
        let value = param_row.components.first().copied().unwrap_or(0.0);
        row![
            text(label).size(12),
            space().width(Fill),
            scrub(param, 0, value),
            key,
        ]
        .spacing(6)
        .align_y(Center)
        .into()
    }
}

/// 値行のスクラブ部品1枚。仕様(範囲・刻み)は param ごとにここで決める。
fn scrub<'a>(param: UiEditParam, component: usize, value: f64) -> Element<'a, Message> {
    let spec = ScrubSpec {
        value,
        decimals: 3,
        // Opacity だけ 0..1 に閉じる(範囲外の要求は入口で塞ぐ — Q3)。
        min: matches!(param, UiEditParam::Opacity).then_some(0.0),
        max: matches!(param, UiEditParam::Opacity).then_some(1.0),
        // 正準座標(原点中央・高さ1.0)も不透明度も 0..1 桁の世界(egui 版
        // DragValue の speed 0.005 と同じ判断で、刻みは細かく)。
        step: 0.01,
        integer: false,
    };
    scrub_value(spec, move |event| {
        Message::Inspector(InspectorEvent::Scrub {
            param,
            component,
            event,
        })
    })
}

/// 3. EFFECTS section。
fn effects<'a>(effects: Vec<EffectSection>) -> Element<'a, Message> {
    let mut section = column![text(EFFECTS).size(11)].spacing(6);
    if effects.is_empty() {
        return section.push(text(NO_EFFECTS).size(12)).into();
    }
    for effect in effects {
        let toggle = button(text(if effect.enabled { "ON" } else { "OFF" }).size(11))
            .padding([2, 8])
            .style(if effect.enabled {
                button::primary as fn(&iced::Theme, button::Status) -> button::Style
            } else {
                button::secondary
            })
            .on_press(Message::Inspector(InspectorEvent::SetEffectEnabled {
                definition_id: effect.definition_id,
                enabled: !effect.enabled,
            }));
        section = section.push(
            row![
                text(effect.plugin_id.clone()).size(12),
                space().width(Fill),
                toggle,
            ]
            .spacing(6)
            .align_y(Center),
        );
        for param in effect.params {
            section = section.push(
                row![
                    text(param.id).size(11),
                    space().width(Fill),
                    text(format_effect_value(param.value)).size(11),
                ]
                .padding(iced::padding::left(12))
                .spacing(6)
                .align_y(Center),
            );
        }
    }
    section.into()
}

fn format_components(components: &[f64]) -> String {
    components
        .iter()
        .map(|value| format!("{value:.3}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// FX param の値の見せ方。Color は swatch の代わりに 16進(独自色 UI を発明しない)。
fn format_effect_value(value: EffectParamValue) -> String {
    match value {
        EffectParamValue::F64(v) => format!("{v:.3}"),
        EffectParamValue::Vec2([x, y]) => format!("{x:.3}, {y:.3}"),
        EffectParamValue::Vec3([x, y, z]) => format!("{x:.3}, {y:.3}, {z:.3}"),
        EffectParamValue::Color([r, g, b, _]) => format!(
            "#{:02X}{:02X}{:02X}",
            (r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (b.clamp(0.0, 1.0) * 255.0).round() as u8
        ),
    }
}
