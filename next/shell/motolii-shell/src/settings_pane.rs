//! Settings パネル(タスク#18: Stage 背景色・市松・ui_scale)。ヘッダの歯車
//! ボタン(`Message::ToggleSettingsPanel`)でトグルする。
//!
//! **Inspector と同じ列グリッド/tokens 流儀**(発注書)を踏襲する — 見出し帯は
//! `inspector_pane::section_header` をそのまま再利用し、数値欄の枠色ロールは
//! `inspector_pane::value_input_style` を、ボタンの意味色ロールは
//! `crate::button_style` を共有する(2箇所で別の意匠を発明しない)。
//!
//! **書ける物を持たない**(他 pane と同じ制約): [`view`] は `&Composition`・
//! 下書き・`bool`・`Dimensions`・`Colors` しか受け取らない。書き口は
//! `crate::Shell::update` の該当 arm だけ。
//!
//! ## 3項目の置き場(発注書どおり)
//! - **Stage 背景色**: `Composition.background` そのもの(Document、undo が効く)。
//!   `crate::Shell::apply_background_preset`/`commit_background_channel` が
//!   read-modify-write で `Intent::SetComposition` を出す。
//! - **市松**: **表示専用**、Document には一切乗らない。仮の置き場は `Shell` 自身の
//!   フィールド(`crate::Shell::checkerboard`) — 発注書は「Workspace 側」と指示して
//!   いるが、`tokens.rs::Dimensions::ui_scale` の doc comment と同じ理由
//!   (Workspace 永続機構がまだ無い、裁定127/128)で仮置きする。
//! - **ui_scale(%)**: 既存 `tokens::Dimensions::ui_scale` をそのまま読み書きする —
//!   ここでは新しい置き場を作らない。書き戻しは [`crate::tokens::save_ui_scale`]。
//!
//! ## 市松 = AE型の透明可視化モード(裁定141)
//! 市松 ON の間、Stage プレビューは `Composition.background` を敷かずに合成した
//! 結果(`motolii_engine::Engine::render_frame_without_background`)へ市松を敷く
//! — 既定の不透明黒背景のままでもトグルは実際に画面を変える。Document・undo・
//! 書き出しは不変(プレビュー専用の入力差分、`motolii-engine` 側の裁定141 doc
//! 参照)。結線は `crate::Shell::refresh_frame`。旧仕様(「フレームの alpha<1 の
//! 画素にだけ乗るので不透明背景では無反応」)が実機で不合格になった経緯と、
//! その場しのぎだった `checkerboard_invisible_reason` の理由文が今回撤去された
//! 経緯は `next/DECISIONS.md` 裁定141 参照。[`BackgroundPreset::Transparent`] は
//! 背景を実際に透明で書き出したい人向けとして引き続き残す。

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use motolii_store::Composition;

use crate::inspector_pane::{section_header, value_input_style};
use crate::tokens::{Colors, Dimensions};
use crate::{button_style, Message};

// ---------------------------------------------------------------------------
// 背景色
// ---------------------------------------------------------------------------

/// `Composition.background` の編集対象チャンネル。R/G/B/A の4本、常にどれか1つ
/// だけ編集中(`inspector_pane::FieldDraft` と同じ「同時に1つ」の形)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BackgroundChannel {
    R,
    G,
    B,
    A,
}

impl BackgroundChannel {
    /// `Composition::background`(`[f32; 4]`)内の添字。
    pub fn index(self) -> usize {
        match self {
            Self::R => 0,
            Self::G => 1,
            Self::B => 2,
            Self::A => 3,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::R => "R",
            Self::G => "G",
            Self::B => "B",
            Self::A => "A",
        }
    }
}

/// 背景 RGBA 数値欄、編集中の下書き。**Document ではない**
/// (`inspector_pane::FieldDraft` と同じ形 — Enter まで store に触らない)。
#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundFieldDraft {
    pub channel: BackgroundChannel,
    pub text: String,
}

/// プリセット4種。押した瞬間に確定する(下書きを経由しない、Inspector の
/// M glyph トグルと同じ「即時1 Intent」の形)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BackgroundPreset {
    Black,
    White,
    /// 写真の「18%グレー」という呼び名を 0..255 の8bit値へそのまま当てた値
    /// (46 ≈ 255*0.18)。反射率換算のガンマ補正はしない — store の既存 solid
    /// rgba(`fixture.rs::LayerSpec::rgba` 等)と同じ「素の8bit値」慣習を保つ。
    Gray18,
    /// alpha=0 の1クリック経路。背景そのものを実際に透明で書き出したい人向け
    /// (裁定141以降、市松を見るための必須経路ではない — 市松は不透明背景の
    /// ままでも `render_frame_without_background` 経由で見える)。
    Transparent,
}

/// プリセットの実値。**丸め込み先はここ1箇所だけ**
/// (`crate::Shell::apply_background_preset` は呼ぶだけ)。
pub fn preset_rgba(preset: BackgroundPreset) -> [f32; 4] {
    let channel = |v: u8| v as f32 / 255.0;
    match preset {
        BackgroundPreset::Black => [channel(0), channel(0), channel(0), 1.0],
        BackgroundPreset::White => [channel(255), channel(255), channel(255), 1.0],
        BackgroundPreset::Gray18 => [channel(46), channel(46), channel(46), 1.0],
        BackgroundPreset::Transparent => [channel(0), channel(0), channel(0), 0.0],
    }
}

/// 数値入力欄の文字列 → 0..255 にクランプした値。読めなければ `None`
/// (`crate::Shell::commit_background_channel` が status 帯へ理由を出す)。
pub fn parse_channel_u8(text: &str) -> Option<f32> {
    crate::inspector_pane::parse_number(text).map(|value| value.clamp(0.0, 255.0) as f32)
}

// ---------------------------------------------------------------------------
// ui_scale
// ---------------------------------------------------------------------------

/// 入力文字列(%) → 50..200 にクランプ・1%刻みに丸めた `ui_scale`(1.00基準)。
/// 読めなければ `None`。
pub fn parse_ui_scale_percent(text: &str) -> Option<f32> {
    crate::inspector_pane::parse_number(text).map(|value| {
        let clamped_percent = value.clamp(50.0, 200.0).round();
        (clamped_percent / 100.0) as f32
    })
}

// ---------------------------------------------------------------------------
// 市松(表示専用 — 書き出しに乗らない)
// ---------------------------------------------------------------------------

/// 市松タイル1マスの一辺(px、`ui_scale` は掛けない — 罫線と同じ「装飾は
/// 物理px床」扱い。Handle は既に `stage_handle_rgba` で画面解像度へ縮められた
/// 後の複製なので、ここでの px は表示ピクセルそのもの)。
const CHECKERBOARD_TILE_PX: u32 = 8;

/// Stage 画像の下に敷く市松。**フレームの実 rgba(engine の premultiplied alpha
/// 出力そのもの、`../reference/KNOWN.md`「実 alpha が乗る」)を上書きする** —
/// 呼び出し側(`crate::Shell::refresh_frame`)は screenshot/export 用の生 rgba
/// (`RenderedFrame::rgba`)には触らせず、表示用 Handle の複製にだけ適用する
/// (「合成器が出せる」と「書き出しが吐く」は別問題)。
///
/// 完全不透明(alpha=255)の画素はそのまま — 市松は「透明の可視化」だけが仕事。
/// タイル色は `Colors::surface_raised`/`Colors::surface_panel`(意味色ロール
/// 経由、raw 値の直書き禁止)。
///
/// **`pub`(crate 内限定ではない)**: `screenshot.rs` が「実際に画面へ出る絵」を
/// 再現するのに同じ組み合わせを使う(`lib.rs::build_stage_handle` と同じ形)のに
/// 加えて、`tests/settings_drive.rs` が容疑2(このロジック自体のバグ)を
/// `frame_rgba()` 生値に対して直接固定するのにも使う — integration test は
/// 別クレート扱いなので `pub(crate)` では届かない。
pub fn composite_checkerboard(width: u32, height: u32, rgba: &mut [u8], colors: Colors) {
    if width == 0 || height == 0 {
        return;
    }
    let light = color_u8(colors.surface_raised);
    let dark = color_u8(colors.surface_panel);

    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            if i + 3 >= rgba.len() {
                continue;
            }
            let alpha = rgba[i + 3];
            if alpha == 255 {
                continue;
            }
            let tile = if (x / CHECKERBOARD_TILE_PX + y / CHECKERBOARD_TILE_PX).is_multiple_of(2) {
                light
            } else {
                dark
            };
            let inv_alpha = 255u32 - alpha as u32;
            for c in 0..3 {
                // premultiplied alpha の over 演算: src はすでに alpha 込みなので、
                // dst(市松タイル、不透明)側だけ `(1-alpha)` を掛けて足す
                // (`engine/motolii-compositor` の `Premultiplied` 経路と同じ式)。
                let src = rgba[i + c] as u32;
                let dst = tile[c] as u32;
                rgba[i + c] = (src + (dst * inv_alpha) / 255).min(255) as u8;
            }
            rgba[i + 3] = 255; // 市松で埋めた画素は表示上は不透明になる。
        }
    }
}

fn color_u8(color: iced::Color) -> [u8; 3] {
    [
        (color.r * 255.0).round().clamp(0.0, 255.0) as u8,
        (color.g * 255.0).round().clamp(0.0, 255.0) as u8,
        (color.b * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

// ---------------------------------------------------------------------------
// view
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn view(
    composition: Option<&Composition>,
    background_draft: Option<&BackgroundFieldDraft>,
    ui_scale: f32,
    ui_scale_draft: Option<&str>,
    checkerboard: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let body: Element<'static, Message> = match composition {
        None => container(
            text("comp が無い — 背景を編集できない")
                .size(dims.caption_text)
                .color(colors.text_muted),
        )
        .padding([0.0, dims.spacing_m])
        .into(),
        Some(composition) => {
            // 各行を hairline で区切る(裁定139「面色の塗り分けで区切っている
            // 残余を全部 hairline へ置換する」を Settings へ展開)。旧実装は
            // 行同士の区切りを何も持たず(面色でも線でもない、無地のまま
            // 積んでいた)、Inspector の `.prow` と同格の情報行として同じ
            // 弱い hairline ロール([`inspector_pane::border_hairline_weak`]、
            // `tokens::Colors` 経由)を延長する — 新しい視覚言語の発明ではない。
            let rows: Vec<Element<'static, Message>> = vec![
                hairline_bottom(background_row(composition, background_draft, dims, colors), dims, colors),
                hairline_bottom(preset_row(dims, colors), dims, colors),
                hairline_bottom(hint_row("書き出しにもこの背景色が乗ります", dims, colors), dims, colors),
                hairline_bottom(checkerboard_row(checkerboard, dims, colors), dims, colors),
                hairline_bottom(
                    hint_row("市松は表示だけ — 書き出しには乗りません", dims, colors),
                    dims,
                    colors,
                ),
                hairline_bottom(ui_scale_row(ui_scale, ui_scale_draft, dims, colors), dims, colors),
            ];
            column(rows).into()
        }
    };

    container(column![section_header("SETTINGS", dims, colors), body])
        .width(Length::Fill)
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

/// 行の下 hairline(裁定139)。`inspector_pane.rs::bordered_row` と同じ
/// trade-off(`iced_core::Border` は per-edge API が無く4辺一律にしかできない —
/// border-bottom の近似として4辺を使う、同 doc 参照)。**`.width(Length::Fill)`
/// をここで明示するのが本体**: `content` 側(row!)が自分の幅を宣言していなくても、
/// 外側 container が Fill を持てば hairline は pane 全幅に伸びる(Inspector の
/// `header`/`hint_row` と同じ実修正パターン)。
fn hairline_bottom(
    content: Element<'static, Message>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    container(content)
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            border: iced::Border {
                color: colors.border_hairline_weak,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn background_row(
    composition: &Composition,
    draft: Option<&BackgroundFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let channels = [
        BackgroundChannel::R,
        BackgroundChannel::G,
        BackgroundChannel::B,
        BackgroundChannel::A,
    ];
    let cells: Vec<Element<'static, Message>> = channels
        .into_iter()
        .map(|channel| channel_cell(channel, composition, draft, dims, colors))
        .collect();

    row![
        text("Background")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        row(cells).spacing(dims.spacing_xs),
    ]
    .spacing(dims.spacing_xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m])
    .into()
}

fn channel_cell(
    channel: BackgroundChannel,
    composition: &Composition,
    draft: Option<&BackgroundFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let current_u8 = (composition.background[channel.index()] * 255.0)
        .round()
        .clamp(0.0, 255.0) as u32;
    let displayed = draft
        .filter(|draft| draft.channel == channel)
        .map(|draft| draft.text.clone())
        .unwrap_or_else(|| current_u8.to_string());

    column![
        text(channel.label())
            .size(dims.caption_text)
            .color(colors.text_muted)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Fixed(dims.inspector_value_width)),
        text_input("", &displayed)
            .on_input(move |text| Message::SettingsBackgroundChannelInput(channel, text))
            .on_submit(Message::SettingsBackgroundChannelSubmit(channel))
            .size(dims.body_text)
            .width(Length::Fixed(dims.inspector_value_width))
            .padding(0.0)
            .align_x(iced::alignment::Horizontal::Center)
            .style(move |_theme, status| value_input_style(dims, colors, status)),
    ]
    .spacing(0.0)
    .into()
}

fn preset_row(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    row![
        preset_button("Black", BackgroundPreset::Black, dims, colors),
        preset_button("White", BackgroundPreset::White, dims, colors),
        preset_button("Gray 18%", BackgroundPreset::Gray18, dims, colors),
        // 背景を実際に透明で書き出したい人向け(裁定141以降、市松を見るための
        // 必須経路ではない — `BackgroundPreset::Transparent` の doc 参照)。
        preset_button("Transparent", BackgroundPreset::Transparent, dims, colors),
    ]
    .spacing(dims.spacing_xs)
    .padding([0.0, dims.spacing_m])
    .into()
}

fn preset_button(
    label: &'static str,
    preset: BackgroundPreset,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    button(text(label).size(dims.caption_text))
        .on_press(Message::SettingsBackgroundPreset(preset))
        .style(move |_theme, status| button_style(dims, colors, status))
        .into()
}

/// **表示専用**トグル。M glyph(`inspector_pane::mute_glyph`)と同じ「押すたび
/// 即トグル」の形だが、glyph ではなく文言ボタン(専用の意味色ロールを新設せず
/// 既存 `crate::button_style` を再利用する — 新しい状態色を1個のためだけに
/// 発明しない)。
fn checkerboard_row(checkerboard: bool, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let label = if checkerboard { "Checkerboard: On" } else { "Checkerboard: Off" };
    row![
        text("市松(透明の可視化)")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        button(text(label).size(dims.caption_text))
            .on_press(Message::ToggleCheckerboard)
            .style(move |_theme, status| button_style(dims, colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m])
    .into()
}

fn ui_scale_row(
    ui_scale: f32,
    draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let current_percent = (ui_scale * 100.0).round() as i64;
    let displayed = draft
        .map(|text| text.to_owned())
        .unwrap_or_else(|| current_percent.to_string());

    row![
        text("UI Scale (%)")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        text_input("", &displayed)
            .on_input(Message::UiScaleInput)
            .on_submit(Message::UiScaleSubmit)
            .size(dims.body_text)
            .width(Length::Fixed(dims.inspector_value_width))
            .padding(0.0)
            .align_x(iced::alignment::Horizontal::Center)
            .style(move |_theme, status| value_input_style(dims, colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m])
    .into()
}

fn hint_row(message: &'static str, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    container(
        text(message)
            .size(dims.caption_text)
            .color(colors.text_muted),
    )
    .padding([dims.spacing_xs, dims.spacing_m])
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_rgba_values_match_black_white_and_18_percent_gray() {
        assert_eq!(preset_rgba(BackgroundPreset::Black), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(preset_rgba(BackgroundPreset::White), [1.0, 1.0, 1.0, 1.0]);
        let gray = preset_rgba(BackgroundPreset::Gray18);
        assert!((gray[0] - 46.0 / 255.0).abs() < 0.0001, "Gray18 の値: {gray:?}");
        assert_eq!(gray[0], gray[1]);
        assert_eq!(gray[1], gray[2]);
        assert_eq!(gray[3], 1.0, "Black/White/Gray18 は常に不透明のはず");
    }

    /// 背景を実際に透明で書き出したい人向けのプリセット。他3プリセットと違い、
    /// これだけが alpha=0 を出す(裁定141以降、市松を見るための必須経路では
    /// ない — `render_frame_without_background` が不透明背景でも市松を見せる)。
    #[test]
    fn transparent_preset_is_fully_transparent() {
        assert_eq!(preset_rgba(BackgroundPreset::Transparent), [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn parse_channel_u8_clamps_into_0_255() {
        assert_eq!(parse_channel_u8("-10"), Some(0.0));
        assert_eq!(parse_channel_u8("999"), Some(255.0));
        assert_eq!(parse_channel_u8("128"), Some(128.0));
        assert_eq!(parse_channel_u8("not a number"), None);
    }

    #[test]
    fn parse_ui_scale_percent_clamps_into_50_to_200_and_rounds_to_whole_percent() {
        assert_eq!(parse_ui_scale_percent("100"), Some(1.0));
        assert_eq!(parse_ui_scale_percent("10"), Some(0.5), "50%未満は50%へクランプ");
        assert_eq!(parse_ui_scale_percent("500"), Some(2.0), "200%超えは200%へクランプ");
        assert_eq!(parse_ui_scale_percent("123.6"), Some(1.24), "1%刻みに丸まっていない");
        assert_eq!(parse_ui_scale_percent("not a number"), None);
    }

    /// **市松トグルの表示分岐、本命**: 不透明画素はそのまま・透明画素は市松タイル色
    /// そのものになる。単なる塗りつぶしと違い、この2分岐が両方とも実際に起きることを
    /// 見る(「表示専用トグルが実際に絵を分岐させる」ことの直接証拠)。
    #[test]
    fn composite_checkerboard_leaves_opaque_pixels_untouched_and_fills_transparent_ones_with_the_tile()
    {
        let colors = Colors::default();
        let mut rgba = vec![
            10, 20, 30, 255, // (0,0) 不透明 — 変わらないはず
            0, 0, 0, 0, // (1,0) 完全透明 — 市松タイル色そのものになるはず
        ];
        composite_checkerboard(2, 1, &mut rgba, colors);

        assert_eq!(&rgba[0..4], &[10, 20, 30, 255], "不透明画素が変わってしまった");

        let light = color_u8(colors.surface_raised);
        let dark = color_u8(colors.surface_panel);
        let got = [rgba[4], rgba[5], rgba[6]];
        assert!(
            got == light || got == dark,
            "透明画素が市松タイル色になっていない: {got:?}(light={light:?}, dark={dark:?})"
        );
        assert_eq!(rgba[7], 255, "市松で埋めた画素は不透明になるはず");
    }

    /// タイル境界(`CHECKERBOARD_TILE_PX`)で実際に色が切り替わること — 単なる
    /// 塗りつぶしとの区別(「市松」を名乗るなら格子であるはず)。
    #[test]
    fn composite_checkerboard_alternates_tiles_across_the_grid() {
        let colors = Colors::default();
        let width = CHECKERBOARD_TILE_PX * 2;
        let mut rgba = vec![0u8; (width * 4) as usize];
        composite_checkerboard(width, 1, &mut rgba, colors);

        let pixel = |x: u32| -> [u8; 3] {
            let i = (x * 4) as usize;
            [rgba[i], rgba[i + 1], rgba[i + 2]]
        };
        assert_ne!(
            pixel(0),
            pixel(CHECKERBOARD_TILE_PX),
            "タイル境界で市松の色が切り替わっていない — 単なる塗りつぶしになっている"
        );
    }

    #[test]
    fn composite_checkerboard_is_a_no_op_on_fully_opaque_frames() {
        let colors = Colors::default();
        let mut rgba = vec![1, 2, 3, 255, 4, 5, 6, 255];
        let original = rgba.clone();
        composite_checkerboard(2, 1, &mut rgba, colors);
        assert_eq!(rgba, original, "全画素不透明なら市松は何も変えないはず");
    }
}
