//! Transport 帯 — 普通のタイムラインプレイヤーの顔(発注 2026-08-22
//! 「タイムラインに再生ボタンが無いのは謎」。map 採用済 1041 Play/Pause・
//! 1042/1043 Step ±1・1044/1045 Go to End/Start、採用予定→採用済 1138 Timecode)。
//!
//! ## 置き場(S 文法から導出した1案 — 朝の合否で直す型、先例 = TP)
//! Timeline pane の**上端・全幅**(ルーラーの上)。S6(常設可視 — 隠れて
//! いないから読める)と S0(先頭/戻/再生/進/末尾の業界慣習順は横一列でしか
//! 成立しない)から、左端の縦積みではなく上端の横帯を採る。モックは transport
//! について沈黙しているので、これは実測転写ではなく導出 — 実窓で見てから直す。
//!
//! ## 枠の文法(裁定179 — 箱は状態の器)
//! - ボタンは**常時輪郭なし**(素のグリフ)。hover で面が浮き
//!   (`surface_hover`)、押下で ink が accent(`action_active`)。
//! - 区切り線を引かない — 帯はクリップ面(地)との明度1段
//!   (`surface_panel`、rail と同じ面)だけで分かれる。
//! - グリフの ink は内容より1段静か(`text_secondary`)。Timecode の数字
//!   だけは内容そのもの(playhead の値)なので `text_primary`、単位グリフ
//!   (f/s)はさらに1段静か(`text_muted`)。
//! - Play‖Pause は状態の器(rail の M/S/L と同型): 再生中は accent ink。
//!
//! ## 意味は既存 λ のまま(発注書「奪わない・重複定義しない」)
//! 各ボタンは pane-local [`Message`] の transport 4腕を発するだけで、意味
//! (`Shell::toggle_playback`/`nav::step_playhead`/`nav::comp_end_frame`)には
//! 触らない。Space/矢印等の既存ショートカットも無改変(shell 側の λ 解決)。
//! shell は `Message::Timeline(..)` の先取りで既存腕へ写す(supervisor 統合)。
//!
//! ## 寸法(裁定178: Rust 直書き禁止)
//! 帯高・踏面・間隔はすべて `tokens/dimensions.json` の `timeline_transport`
//! 節(導出根拠は同 JSON の `_note_*` — 帯高 = Ableton Control Bar 実測30の
//! 同源転写、gap = 裁定167 梯子下段 0.075×帯高)。

use iced::widget::{button, container, row, text, Space};
use iced::{Background, Element, Length};

use motolii_store::Fps;

use super::TimelinePane;
use crate::tokens::{Colors, Dimensions};
use crate::Message;

/// transport の1ボタンぶんの宣言(絵と意味の対応表)。[`view`] はこの spec を
/// widget へ写すだけ — テストは widget 木ではなくこの spec を検分する
/// (`lane_bar` の比率純関数と同じ「検証可能な継ぎ目」の型)。
#[derive(Debug, Clone)]
pub struct TransportButton {
    /// グリフ文字(アイコンフォント新規導入禁止 — 発注書)。step は fold
    /// 三角と同族の小三角(◂/▸)、Play は大三角(▶)— 同じ ▶ を2つの動詞に
    /// 使わないための1段差(発注書例示 |◀ ◀ ▶ ▶ ▶| からの明示的な逸脱、
    /// RETURN に記載)。
    pub glyph: &'static str,
    /// 押下で発する pane-local Message(shell が既存の再生系 Message へ写す)。
    pub message: Message,
    /// 状態の器(裁定179)としての「今 on か」。Play‖Pause だけが持つ
    /// (再生中 = accent ink)。他の4ボタンは押し口であって器ではない。
    pub active: bool,
}

/// transport 帯の宣言全体: 5ボタン(S0 順)+ Timecode(1138)。
#[derive(Debug, Clone)]
pub struct TransportSpec {
    /// S0: 先頭 / 1コマ戻 / Play‖Pause / 1コマ進 / 末尾 の業界慣習固定順。
    pub buttons: [TransportButton; 5],
    /// Timecode のフレーム番号部(等幅で描く数字部)。
    pub frames: String,
    /// Timecode の秒部(小数2桁)。comp が無く fps を引けない時は `None` —
    /// 黙って誤った値を描くより描かない([`TimelinePane::marker_frame`] の
    /// M13 と同じ姿勢)。
    pub seconds: Option<String>,
}

/// 絵と意味の対応表を組む純関数([`view`] とテストの共通の正本)。
pub fn transport_spec(playhead: i64, fps: Option<Fps>, playing: bool) -> TransportSpec {
    TransportSpec {
        buttons: [
            TransportButton {
                glyph: "|\u{25C0}", // |◀ 先頭
                message: Message::JumpPlayheadToStart,
                active: false,
            },
            TransportButton {
                glyph: "\u{25C2}", // ◂ 1コマ戻
                message: Message::StepPlayhead(-1),
                active: false,
            },
            TransportButton {
                // 「次に何が起きるか」を示す慣習(shell 旧 Play バーと同じ):
                // 停止中=▶(押すと再生)、再生中=‖(押すと停止)。
                glyph: if playing { "\u{2016}" } else { "\u{25B6}" }, // ‖ / ▶
                message: Message::TogglePlayback,
                active: playing,
            },
            TransportButton {
                glyph: "\u{25B8}", // ▸ 1コマ進
                message: Message::StepPlayhead(1),
                active: false,
            },
            TransportButton {
                glyph: "\u{25B6}|", // ▶| 末尾
                message: Message::JumpPlayheadToEnd,
                active: false,
            },
        ],
        frames: playhead.to_string(),
        seconds: seconds_label(playhead, fps),
    }
}

/// playhead の秒表示(小数2桁)。有理 fps(`num/den`)から
/// `frame × den / num` — 29.97(30000/1001)でも正しい。
fn seconds_label(playhead: i64, fps: Option<Fps>) -> Option<String> {
    let fps = fps?;
    Some(format!("{:.2}", playhead as f64 / fps.as_f64()))
}

/// transport 帯の widget。[`TimelinePane::view_with_transport`] だけが呼ぶ。
pub(crate) fn view(pane: &TimelinePane) -> Element<'static, Message> {
    let dims = pane.dims;
    let colors = pane.colors;
    let spec = transport_spec(pane.playhead, pane.fps, pane.playing);

    let mut children: Vec<Element<'static, Message>> =
        Vec::with_capacity(spec.buttons.len() + 6);
    for spec_button in spec.buttons {
        children.push(transport_button(spec_button, dims, colors));
    }
    // Timecode(1138)。ボタン群との間は兄弟 gap より1段広い既存段
    // (`spacing_m`)で「別の束」であることを示す(線を引かない — 裁定179)。
    children.push(Space::new().width(Length::Fixed(dims.spacing_m)).into());
    children.push(
        text(spec.frames)
            .size(dims.body_text)
            .font(iced::Font::MONOSPACE)
            .color(colors.text_primary)
            .into(),
    );
    children.push(unit_glyph("f", dims, colors));
    if let Some(seconds) = spec.seconds {
        children.push(Space::new().width(Length::Fixed(dims.spacing_s)).into());
        children.push(
            text(seconds)
                .size(dims.body_text)
                .font(iced::Font::MONOSPACE)
                .color(colors.text_primary)
                .into(),
        );
        children.push(unit_glyph("s", dims, colors));
    }

    container(
        row(children)
            .spacing(dims.timeline_transport_gap)
            .align_y(iced::alignment::Vertical::Center)
            .padding([0.0, dims.spacing_s]),
    )
    .width(Length::Fill)
    .height(Length::Fixed(dims.timeline_transport_height))
    .align_y(iced::alignment::Vertical::Center)
    // 区切り線なし(明度1段) — rail と同じ面(`surface_panel`)がクリップ面
    // (地)との差を作る。border は描かない(裁定179)。
    .style(move |_theme| container::Style {
        background: Some(Background::Color(colors.surface_panel)),
        ..container::Style::default()
    })
    .into()
}

/// Timecode の単位グリフ(f/s)。数字ではないので等幅にしない(発注書
/// 「等幅は数字部のみ許可」)。ink は数字よりさらに1段静か。
fn unit_glyph(unit: &'static str, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    text(unit).size(dims.caption_text).color(colors.text_muted).into()
}

/// transport の1ボタン。常時輪郭なし・hover で面が浮く・押下で accent
/// (裁定179)。踏面は帯高いっぱい×`timeline_transport_button_width`
/// (S1: グリフより大きい)。
fn transport_button(
    spec: TransportButton,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    // 状態の器(Play‖Pause の再生中)は rail の M/S/L active と同じ accent ink。
    let idle_ink = if spec.active { colors.action_active } else { colors.text_secondary };
    button(
        text(spec.glyph)
            .size(dims.body_text)
            .color(idle_ink)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fixed(dims.timeline_transport_button_width))
    .height(Length::Fill)
    .padding(0)
    .on_press(spec.message)
    .style(move |_theme, status| {
        let (background, text_color) = match status {
            // 押下 = active: ink が accent(面は hover のまま — 器が2重に
            // 語らない)。
            button::Status::Pressed => {
                (Some(Background::Color(colors.surface_hover)), colors.action_active)
            }
            // hover: 面が浮く(輪郭は出さない)。
            button::Status::Hovered => {
                (Some(Background::Color(colors.surface_hover)), idle_ink)
            }
            // 常時: 素のグリフ(輪郭なし・面なし)。
            _ => (None, idle_ink),
        };
        button::Style {
            background,
            text_color,
            ..button::Style::default()
        }
    })
    .into()
}
