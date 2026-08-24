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
//! - ボタンの絵は SVG アイコン(裁定186 で文字グリフ |◀ ◂ ▶ ▸ ▶| から置換 —
//!   `motolii-icons` の Material Symbols)。**常時輪郭なし**(素のアイコン)。
//!   hover で面が浮く(`surface_hover`)。押下 ink の詳細は
//!   [`transport_button`] の doc(svg は button の text_color を継がない)。
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

use iced::widget::{button, container, row, text, text_input, Space};
use iced::{Background, Element, Length};

use motolii_icons::{frame_px_for_glyph_px, Icon};
use motolii_store::Fps;

use super::TimelinePane;
use crate::tokens::{Colors, Dimensions};
use crate::Message;

/* motolii-component
id = "timeline.transport_navigation"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["transport_spec_with_frame_draft", "commit_frame_input", "jump_to_keyframe"]
meaning = ["FrameInput", "JumpToNextKeyframe", "JumpToPreviousKeyframe"]
evaluation = ["parse_frame_input", "property_key_points", "nearest_meaning_point"]
render = ["transport_spec", "view"]
observable = ["committing_a_frame_input_moves_the_playhead", "jumping_to_a_property_key_moves_only_the_playhead"]
*/

/// transport の1ボタンぶんの宣言(絵と意味の対応表)。[`view`] はこの spec を
/// widget へ写すだけ — テストは widget 木ではなくこの spec を検分する
/// (`lane_bar` の比率純関数と同じ「検証可能な継ぎ目」の型)。
#[derive(Debug, Clone)]
pub struct TransportButton {
    /// SVG アイコン(裁定186 — 文字グリフ |◀ ◂ ▶ ▸ ▶| からの置換)。step は
    /// Play の大三角と同じ ▶ を2つの動詞に使わないための1段差を保つ:
    /// Play=`PlayArrow`(大三角)、step=`ArrowLeft`/`ArrowRight`(小三角 —
    /// 旧 ◂/▸ の直訳。chevron 系は別動詞のための在庫)。
    pub icon: Icon,
    /// 押下で発する pane-local Message(shell が既存の再生系 Message へ写す)。
    pub message: Message,
    /// 状態の器(裁定179)としての「今 on か」。Play‖Pause だけが持つ
    /// (再生中 = accent ink)。他の4ボタンは押し口であって器ではない。
    pub active: bool,
}

/// transport 帯の宣言全体: 5ボタン(S0 順)+ ループトグル + Timecode(1138)。
#[derive(Debug, Clone)]
pub struct TransportSpec {
    /// S0: 先頭 / 1コマ戻 / Play‖Pause / 1コマ進 / 末尾 の業界慣習固定順。
    pub buttons: [TransportButton; 5],
    /// ループ再生トグル(map 1082/1083、B21 第1切片)。5ボタン(瞬間の押し口)
    /// とは別身分の**状態の器**(裁定179 — Play‖Pause の active と同じ accent
    /// ink 文法)なので、`buttons` 配列(S0 の慣習順)には混ぜない。
    ///
    /// アイコンは [`Icon::Repeat`](Material `repeat`)。B21 実装時は repeat
    /// 不在で [`Icon::Redo`] を暫定の顔にしていたが、第5波 shell 結線で
    /// vendoring が届いたのでここ1箇所を差し替えた(RETURN 要求の消化)。
    pub loop_button: TransportButton,
    /// 表示中 property の次/前キーへ渡る2つの押し口。先頭/末尾とは別の
    /// 意味なので、既存の5ボタン配列へ混ぜずに分ける。
    pub keyframe_buttons: [TransportButton; 2],
    /// Timecode のフレーム番号部(等幅で描く数字部)。
    pub frames: String,
    /// Timecode の秒部(小数2桁)。comp が無く fps を引けない時は `None` —
    /// 黙って誤った値を描くより描かない([`TimelinePane::marker_frame`] の
    /// M13 と同じ姿勢)。
    pub seconds: Option<String>,
}

/// 絵と意味の対応表を組む純関数([`view`] とテストの共通の正本)。
/// `looping` はループ on/off(`PaneState::loop_enabled()` — B21 第1切片で追加)。
pub fn transport_spec(playhead: i64, fps: Option<Fps>, playing: bool, looping: bool) -> TransportSpec {
    TransportSpec {
        buttons: [
            TransportButton {
                icon: Icon::SkipPrevious, // 旧 |◀ 先頭
                message: Message::JumpPlayheadToStart,
                active: false,
            },
            TransportButton {
                icon: Icon::ArrowLeft, // 旧 ◂ 1コマ戻(小三角)
                message: Message::StepPlayhead(-1),
                active: false,
            },
            TransportButton {
                // 「次に何が起きるか」を示す慣習(shell 旧 Play バーと同じ):
                // 停止中=▶(押すと再生)、再生中=⏸(押すと停止)。
                icon: if playing { Icon::Pause } else { Icon::PlayArrow },
                message: Message::TogglePlayback,
                active: playing,
            },
            TransportButton {
                icon: Icon::ArrowRight, // 旧 ▸ 1コマ進(小三角)
                message: Message::StepPlayhead(1),
                active: false,
            },
            TransportButton {
                icon: Icon::SkipNext, // 旧 ▶| 末尾
                message: Message::JumpPlayheadToEnd,
                active: false,
            },
        ],
        loop_button: TransportButton {
            icon: Icon::Repeat, // 第5波結線で vendoring 済み(`TransportSpec::loop_button` doc 参照)
            message: Message::ToggleLoop,
            active: looping,
        },
        keyframe_buttons: [
            TransportButton {
                icon: Icon::ChevronLeft,
                message: Message::JumpToPreviousKeyframe,
                active: false,
            },
            TransportButton {
                icon: Icon::ChevronRight,
                message: Message::JumpToNextKeyframe,
                active: false,
            },
        ],
        frames: playhead.to_string(),
        seconds: seconds_label(playhead, fps),
    }
}

/// 直接入力中だけ Timecode のフレーム文字列を下書きへ差し替える。
/// 秒表示はまだ確定 playhead の値を使うため、入力途中に別の時刻を描かない。
pub fn transport_spec_with_frame_draft(
    playhead: i64,
    fps: Option<Fps>,
    playing: bool,
    looping: bool,
    draft: Option<&str>,
) -> TransportSpec {
    let mut spec = transport_spec(playhead, fps, playing, looping);
    if let Some(draft) = draft {
        spec.frames = draft.to_owned();
    }
    spec
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
    let spec = transport_spec_with_frame_draft(
        pane.playhead,
        pane.fps,
        pane.playing,
        pane.loop_enabled,
        pane.frame_draft.as_deref(),
    );

    let mut children: Vec<Element<'static, Message>> =
        Vec::with_capacity(spec.buttons.len() + spec.keyframe_buttons.len() + 10);
    for spec_button in spec.buttons {
        children.push(transport_button(spec_button, dims, colors));
    }
    // ループトグル(B21)。瞬間の押し口5つとは別の束(状態の器)なので、
    // Timecode と同じ「兄弟 gap より1段広い既存段」で区切る(線を引かない —
    // 裁定179)。
    children.push(Space::new().width(Length::Fixed(dims.spacing_m)).into());
    children.push(transport_button(spec.loop_button, dims, colors));
    children.push(Space::new().width(Length::Fixed(dims.spacing_m)).into());
    for spec_button in spec.keyframe_buttons {
        children.push(transport_button(spec_button, dims, colors));
    }
    children.push(
        button(text("Graph").size(dims.caption_text))
            .on_press(Message::ToggleGraphEditor)
            .padding([0.0, dims.spacing_xs])
            .into(),
    );
    // Timecode(1138)。同じく `spacing_m` で「別の束」であることを示す。
    children.push(Space::new().width(Length::Fixed(dims.spacing_m)).into());
    children.push(
        text_input("frame", spec.frames)
            .on_input(Message::FrameInput)
            .on_submit(Message::FrameCommit)
            .size(dims.body_text)
            .font(iced::Font::MONOSPACE)
            .width(Length::Fixed(dims.inspector_value_width))
            .padding([0.0, dims.spacing_xs])
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

/// transport の1ボタン。常時輪郭なし・hover で面が浮く(裁定179)。踏面は
/// 帯高いっぱい×`timeline_transport_button_width`(S1: アイコンより大きい)。
///
/// ## アイコン化の寸法・ink(裁定186)
/// - 枠寸は旧文字グリフの字寸(`body_text`)を [`frame_px_for_glyph_px`]
///   (Material live area 比 24/20)で写した視覚同等寸 — 踏面・帯高は
///   dimensions.json の `timeline_transport` 節のまま無改変。
/// - ink は svg の tint(`motolii_icons::icon` へ渡す色)で塗る。svg は
///   button の `text_color` を継がないため、**押下瞬間の accent ink は
///   アイコンには効かない**(svg の style は Pressed 状態を持たない — fork
///   `widget/src/svg.rs` の `Status` は Idle/Hovered のみ)。器としての
///   accent(Play‖Pause の再生中)は spec 由来の tint で従来どおり立つ。
///   hover の面浮きも従来どおり(押下フィードバックは面が担う)。
fn transport_button(
    spec: TransportButton,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    // 状態の器(Play‖Pause の再生中)は rail の M/S/L active と同じ accent ink。
    let idle_ink = if spec.active { colors.action_active } else { colors.text_secondary };
    let glyph = motolii_icons::icon(
        spec.icon,
        frame_px_for_glyph_px(dims.body_text),
        idle_ink,
    );
    button(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(dims.timeline_transport_button_width))
    .height(Length::Fill)
    .padding(0)
    .on_press(spec.message)
    .style(move |_theme, status| {
        let background = match status {
            // 押下・hover: 面が浮く(輪郭は出さない — 裁定179)。
            button::Status::Pressed | button::Status::Hovered => {
                Some(Background::Color(colors.surface_hover))
            }
            // 常時: 素のアイコン(輪郭なし・面なし)。
            _ => None,
        };
        button::Style {
            background,
            // svg には効かない(tint が正)が、契約として ink を宣言しておく。
            text_color: idle_ink,
            ..button::Style::default()
        }
    })
    .into()
}
