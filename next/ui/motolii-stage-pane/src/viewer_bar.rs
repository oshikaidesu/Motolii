//! **Viewer の家**(発注 2026-08-22)— 3束を1つの部品にまとめる:
//! **視点タブ**(B17 カメラ/3Dビュー束)・**画質**(B23 プレビュー解像度/画質束)・
//! **チャンネル表示**(B05 色/アルファ表示束の Viewer 側)。
//! `docs/reviews/2026-08-22-intent-bundles-draft.md` の bundle 表がこの3行の
//! 住所を揃って「Viewer」と指す(intent-flow-map.md の家表も同じ)。
//!
//! ## 背景 — 何が既存で何がここの新規分か
//!
//! - **画質(B23)の cap 巡回自体**(`Auto → ½ → ¼`)と**市松**は
//!   [`crate::state_band_view`] が Stage 下縁状態帯として**既に実装済み**
//!   (裁定163)。ここは重複させない — [`crate::PreviewResolutionCap`]/
//!   [`crate::effective_preview_scale`] をそのまま読むだけ(正本は据え置き、
//!   `lib.rs` は読み専用)。この module が足すのは「クリックで直接選ぶ」
//!   メニュー型の選択 UI([`resolution_quality_view`]) — 巡回1本槍だった
//!   既存動線の**拡張**であって置換ではない(発注書「その拡張と整理」)。
//! - **視点タブ(B17)は新規**。既存の観測カメラ⇄レンダリングカメラ切替
//!   (裁定157・[`crate::Message::Observe`]/[`crate::Message::
//!   ResetToRenderCamera`])は「戻る」操作とホイール/中ボタンドラッグでの
//!   「入る」操作しか持たず、**タブとして両方向を並べて見せる顔が無かった**。
//!   mock `next/reference/mocks/stage-semantics.html`(v4/v5)の `.vtabs`
//!   (`◉ Camera` / `✥ User View`、上縁=identity、S5)がその顔の正本。
//!   [`view_tab_for_observation`] は既存の `Option<ObservationCamera>` から
//!   タブの見た目状態を導くだけの純写像(裁定157 の意味を複製しない)。
//! - **チャンネル表示(B05 Viewer 側)は新規**。engine 側の実能力を確認済み:
//!   [`motolii_engine::Engine::render_frame_without_background`](裁定141・
//!   市松の入力源)が既に**真の(背景と未合成の)RGBA8**を返す — 市松がまさに
//!   この alpha を可視化する機能なので、engine は既にチャンネル分の情報を
//!   持っている。よって chnnale 単離は**表示専用のピクセル変換**
//!   ([`apply_channel_display`])だけで足り、**engine への新規要求は無い**
//!   (発注書 4の「無ければ見送り」条件に該当しない — 消化した)。
//!
//! ## この module の契約(発注書「純関数+部品として完結」)
//!
//! - **`Engine`/`motolii-shell` に触れない**。`Engine` を可変で取る関数も
//!   ゼロ(`crate::observation_preview_source` のような書き口はここには無い —
//!   `tests/viewer_bar_fence.rs` がソーステキストで縛る)。**この行自体が
//!   禁止パターンの文字列を書くと柵の grep が自己参照で誤爆する
//!   (実測 2026-08-22、既存赤テスト回収発注)ため、ここでは記号を割って
//!   表現する — 意味は上と同じ「`&` + `mut` + ` Engine`」。**
//! - **`Message` は pane 全体([`crate::Message`])から独立**
//!   ([`gizmo::GizmoDrag`] と同じ理由: 既存 `crate::Message` へ variant を
//!   足すと `motolii-shell` 側の exhaustive match が壊れる — このレーンの
//!   write-set 外)。supervisor が `.map(...)` して実配線へ翻訳する。
//!   想定される翻訳(実装はしない、契約だけ明記):
//!   - `Message::SelectViewTab(ViewTab::Camera)` → `crate::Message::
//!     ResetToRenderCamera`
//!   - `Message::SelectViewTab(ViewTab::UserView)` → `crate::Message::
//!     Observe(ObservationCamera::default())`(観測へ既定値で入る —
//!     具体的な初期倍率は mock 未決事項「User View の初期倍率」のまま)
//!   - `Message::SelectResolutionCap(cap)` → 現在値との差分ぶん
//!     `crate::Message::CycleResolutionCap` を supervisor 側で繰り返す
//!     か、直接セットする経路を今後 `lib.rs` 側に足す(cycle-only な現状の
//!     `crate::Message` を書き換えるのはこのレーンの write-set 外)
//!   - `Message::SelectChannelDisplay(mode)` → 表示専用状態の更新+
//!     [`apply_channel_display`] を display source の合流点(市松と同格の
//!     `DisplaySource`)へ挿す(`lib.rs` 冒頭 doc の「`frame_rgba`/
//!     `refresh_frame` の本体は assembler 側」と同じ理由でここには置けない)
//! - **意匠は裁定179(枠の文法)+裁定187(icon-first)**: 選択の器は輪郭では
//!   なく[`crate::Colors::state_selected`](既存の「状態: 選択」ロール、
//!   新色ゼロ)の面塗り、非選択は透明地+`text_muted`。**icon+tooltip は
//!   見送った** — `motolii-icons` の在庫37個(裁定186/187)にカメラ/画質/
//!   チャンネル向けの適合グリフが無い(`videocam`/`hd`/`rgb` 系は未 vendoring)。
//!   新規 SVG の vendoring はこのレーンの EXACT TARGET 外(`motolii-icons/
//!   assets/` は write-set に無い)なので、既存の状態帯(`crate::
//!   state_band_item`)と同じ「S4 text 段」の文字ラベルで代替した
//!   (飾りではなく機能は完全 — RETURN にアイコン在庫の穴として記録する)。

use iced::widget::{button, row, text};
use iced::{Background, Border};

use motolii_engine::ObservationCamera;
use motolii_tokens_rs::{Colors, Dimensions};

use crate::{effective_preview_scale, PreviewResolutionCap};

// ---------------------------------------------------------------------------
// pane ローカル Message(module 冒頭 doc「契約」参照 — `crate::Message` とは
// 独立、`gizmo::GizmoDrag` と同型の理由)。
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Message {
    /// 視点タブのクリック(B17)。翻訳先は module 冒頭 doc 参照。
    SelectViewTab(ViewTab),
    /// 画質メニューの直接選択(B23)。既存の巡回専用
    /// [`crate::Message::CycleResolutionCap`] と違い、任意の値へ1クリックで
    /// 飛べる(発注書「選択 UI」— 巡回は探索、メニューは既知の値への直行)。
    SelectResolutionCap(PreviewResolutionCap),
    /// チャンネル表示の切替(B05 Viewer 側)。
    SelectChannelDisplay(ChannelDisplay),
}

type Element<'a> = iced::Element<'a, Message>;

// ---------------------------------------------------------------------------
// B17: 視点タブ — mock `.vtabs`(`◉ Camera` / `✥ User View`)。
// ---------------------------------------------------------------------------

/// タブの見た目状態。**Document には一切乗らない**(表示専用の分岐 —
/// `crate::Message::Observe`/`ResetToRenderCamera` が扱うのは
/// `Option<ObservationCamera>` そのもの、このタブはその読み替えでしかない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewTab {
    /// mock `◉ Camera` — レンダリングカメラの視点(書き出しと同一)。
    /// `observation == None` に対応。
    Camera,
    /// mock `✥ User View` — 世界を眺めるユーザーの視点(旧称: 観測/自由視点)。
    /// `observation == Some(_)` に対応。
    UserView,
}

const VIEW_TABS: [ViewTab; 2] = [ViewTab::Camera, ViewTab::UserView];

impl ViewTab {
    fn label(self) -> &'static str {
        match self {
            ViewTab::Camera => "Camera",
            ViewTab::UserView => "User View",
        }
    }
}

/// 現在の観測カメラ値からタブの見た目状態を導く純写像(裁定157 の
/// `Option<ObservationCamera>` を読み替えるだけ、意味の複製ではない)。
pub fn view_tab_for_observation(observation: Option<ObservationCamera>) -> ViewTab {
    match observation {
        None => ViewTab::Camera,
        Some(_) => ViewTab::UserView,
    }
}

/// 上縁タブそのもの(mock `.vtabs`)。呼び出し側(supervisor)が Stage pane
/// の上に積む — 配置そのものはこの関数の関知しない(発注書「部品として完結」)。
pub fn view_tabs_view(active: ViewTab, dims: Dimensions, colors: Colors) -> Element<'static> {
    let tabs = VIEW_TABS
        .into_iter()
        .map(|tab| segment_item(tab.label(), tab == active, Message::SelectViewTab(tab), dims, colors));
    row(tabs).spacing(dims.theme().space.xs).into()
}

// ---------------------------------------------------------------------------
// B23: 画質選択 UI — 値自体は `crate::PreviewResolutionCap`(正本は
// `lib.rs`、ここでは複製しない)。
// ---------------------------------------------------------------------------

const RESOLUTION_CHOICES: [PreviewResolutionCap; 3] =
    [PreviewResolutionCap::Auto, PreviewResolutionCap::Half, PreviewResolutionCap::Quarter];

/// `crate::state_band_view` のラベル文言と同一の文字列を返す(表現の一貫性
/// — 帯とメニューで同じ cap が違う文字に見えると混乱するため意図的に複製、
/// ただし計算(`effective_preview_scale`)は crate 正本を呼ぶだけで再実装しない)。
fn resolution_choice_label(choice: PreviewResolutionCap, auto_scale: f64) -> String {
    match choice {
        PreviewResolutionCap::Auto => {
            format!("Auto ({:.2}×)", effective_preview_scale(auto_scale, choice))
        }
        PreviewResolutionCap::Half => "½".to_owned(),
        PreviewResolutionCap::Quarter => "¼".to_owned(),
    }
}

/// 画質メニュー(発注書「選択 UI」)。3値を並べて**直接選べる**
/// (`crate::state_band_view` の cap 項目は巡回専用 — 両者は共存可能、
/// 発注書「その拡張と整理」の「拡張」側)。
pub fn resolution_quality_view(
    current: PreviewResolutionCap,
    auto_scale: f64,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static> {
    let items = RESOLUTION_CHOICES.into_iter().map(|choice| {
        segment_item(
            resolution_choice_label(choice, auto_scale),
            choice == current,
            Message::SelectResolutionCap(choice),
            dims,
            colors,
        )
    });
    row(items).spacing(dims.theme().space.xs).into()
}

// ---------------------------------------------------------------------------
// B05(Viewer 側): チャンネル表示 — RGB(既定)/Alpha/R/G/B 単独。
// ---------------------------------------------------------------------------

/// 表示チャンネルの選択。**Document には一切乗らない**(市松と同格の表示専用
/// 状態、`crate::Message::ToggleCheckerboard` と同じ「d 住所(Viewer/表示)」)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChannelDisplay {
    /// 素通し(色補正等の合成結果をそのまま見せる)。
    #[default]
    Rgb,
    /// alpha チャンネルをグレースケールで単離。
    Alpha,
    /// 赤チャンネルをグレースケールで単離。
    Red,
    /// 緑チャンネルをグレースケールで単離。
    Green,
    /// 青チャンネルをグレースケールで単離。
    Blue,
}

const CHANNEL_CHOICES: [ChannelDisplay; 5] = [
    ChannelDisplay::Rgb,
    ChannelDisplay::Alpha,
    ChannelDisplay::Red,
    ChannelDisplay::Green,
    ChannelDisplay::Blue,
];

impl ChannelDisplay {
    /// クリック巡回の次の値(5値の wrap、[`crate::PreviewResolutionCap::
    /// next`] と同型)。メニュー型([`channel_display_view`])が主だが、
    /// 将来ショートカット巡回を足す時のための純関数(発注書に反しない —
    /// 使われなくても副作用の無い純関数の追加は write-set を割らない)。
    pub fn next(self) -> Self {
        match self {
            Self::Rgb => Self::Alpha,
            Self::Alpha => Self::Red,
            Self::Red => Self::Green,
            Self::Green => Self::Blue,
            Self::Blue => Self::Rgb,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Rgb => "RGB",
            Self::Alpha => "Alpha",
            Self::Red => "R",
            Self::Green => "G",
            Self::Blue => "B",
        }
    }
}

/// チャンネル表示メニュー。
pub fn channel_display_view(current: ChannelDisplay, dims: Dimensions, colors: Colors) -> Element<'static> {
    let items = CHANNEL_CHOICES.into_iter().map(|choice| {
        segment_item(choice.label(), choice == current, Message::SelectChannelDisplay(choice), dims, colors)
    });
    row(items).spacing(dims.theme().space.xs).into()
}

/// **表示専用のピクセル変換**(純関数、module 冒頭 doc「engine 側の実能力」
/// 参照 — engine 変更なしで足りる)。`rgba` は `Engine::render_frame_without_
/// background` 相当の、背景と未合成の真の RGBA8(4byte/px)。`Rgb` は恒等
/// (`to_vec` の複製のみ、変換はしない)。他チャンネルは値をグレースケール化し
/// alpha を不透明(255)へ固定する — AE の "Show Channel" 系の慣習(単離
/// チャンネルは常にグレースケール表示、着色はしない、市松のような透明表現とは
/// 別の診断モード)。
///
/// `rgba.len()` が4の倍数でない(壊れたバッファ)場合、末尾の端数は
/// `chunks_exact` により黙って捨てられる(comp の RGBA8 バッファは常に
/// `width*height*4` — この前提が壊れている方が異常、パニックさせない方を選ぶ:
/// 表示専用のここでパニックすると Stage 全体が固まる)。
pub fn apply_channel_display(rgba: &[u8], mode: ChannelDisplay) -> Vec<u8> {
    if mode == ChannelDisplay::Rgb {
        return rgba.to_vec();
    }
    let channel_index = match mode {
        ChannelDisplay::Red => 0,
        ChannelDisplay::Green => 1,
        ChannelDisplay::Blue => 2,
        ChannelDisplay::Alpha => 3,
        ChannelDisplay::Rgb => unreachable!("Rgb は上で早期 return 済み"),
    };
    let mut out = Vec::with_capacity(rgba.len());
    for pixel in rgba.chunks_exact(4) {
        let value = pixel[channel_index];
        out.extend_from_slice(&[value, value, value, 255]);
    }
    out
}

// ---------------------------------------------------------------------------
// 集約: Viewer の家そのもの(発注書 EXACT TARGET)。
// ---------------------------------------------------------------------------

/// 3束を1本の部品として返す(発注書「純関数+部品として完結」)。配置
/// (Stage pane のどこに積むか — mock は上縁だが最終判断は supervisor)は
/// 呼び出し側の仕事。
pub fn viewer_bar_view(
    observation: Option<ObservationCamera>,
    resolution_cap: PreviewResolutionCap,
    auto_scale: f64,
    channel_display: ChannelDisplay,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static> {
    let tabs = view_tabs_view(view_tab_for_observation(observation), dims, colors);
    let quality = resolution_quality_view(resolution_cap, auto_scale, dims, colors);
    let channels = channel_display_view(channel_display, dims, colors);

    row![
        tabs,
        iced::widget::Space::new().width(iced::Length::Fixed(dims.theme().space.l)),
        quality,
        iced::widget::Space::new().width(iced::Length::Fixed(dims.theme().space.l)),
        channels,
    ]
    .spacing(dims.theme().space.m)
    .padding([dims.theme().space.xs, dims.theme().space.m])
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

// ---------------------------------------------------------------------------
// 部品共通: セグメント項目(裁定179「箱は状態の器」— 選択= 面塗り
// `state_selected`、非選択=透明地+`text_muted`、常時輪郭は持たない)。
// `crate::state_band_item` と同じ「text 段のクリック可能ラベル」だが、
// あちらは `crate::Message` 型・こちらは pane ローカル `Message` 型なので
// 型が違い共有できない(小さく複製 — `motolii-browser-pane::tab_style` と
// 同じ「pane crate 間の相互依存を作らない」判断)。
// ---------------------------------------------------------------------------

fn segment_item(
    label: impl Into<String>,
    active: bool,
    message: Message,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static> {
    let label = label.into();
    button(text(label).size(dims.theme().text.body))
        .on_press(message)
        .padding([dims.theme().space.xs, dims.theme().space.m])
        .style(move |_theme, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let background = if active {
                Some(Background::Color(colors.state_selected))
            } else if hovered {
                Some(Background::Color(colors.surface_hover))
            } else {
                None
            };
            button::Style {
                background,
                text_color: if active {
                    colors.text_primary
                } else if hovered {
                    colors.text_secondary
                } else {
                    colors.text_muted
                },
                border: Border { radius: 0.0.into(), ..Border::default() },
                ..button::Style::default()
            }
        })
        .into()
}

// ---------------------------------------------------------------------------
// テスト(発注書「テストを書く(実行しない)」— ORACLE 未実行明記、RETURN 参照)。
// 純関数だけを直接呼ぶ(module 冒頭 lib.rs と同じテスト方針 — canvas 系
// widget は iced_test で不可視、view builder 系はここでは「panic しない」の
// スモークのみ)。
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- B17: 視点タブ --------------------------------------------------

    #[test]
    fn view_tab_reflects_observation_presence() {
        assert_eq!(view_tab_for_observation(None), ViewTab::Camera);
        assert_eq!(
            view_tab_for_observation(Some(ObservationCamera::default())),
            ViewTab::UserView
        );
    }

    // -- B23: 画質選択 ----------------------------------------------------

    #[test]
    fn resolution_choice_label_matches_state_band_wording() {
        assert_eq!(resolution_choice_label(PreviewResolutionCap::Auto, 1.0), "Auto (1.00×)");
        assert_eq!(resolution_choice_label(PreviewResolutionCap::Half, 1.0), "½");
        assert_eq!(resolution_choice_label(PreviewResolutionCap::Quarter, 1.0), "¼");
    }

    // -- B05: チャンネル表示 -----------------------------------------------

    #[test]
    fn channel_display_defaults_to_rgb() {
        assert_eq!(ChannelDisplay::default(), ChannelDisplay::Rgb);
    }

    #[test]
    fn channel_display_next_cycles_through_five_values_and_wraps() {
        assert_eq!(ChannelDisplay::Rgb.next(), ChannelDisplay::Alpha);
        assert_eq!(ChannelDisplay::Alpha.next(), ChannelDisplay::Red);
        assert_eq!(ChannelDisplay::Red.next(), ChannelDisplay::Green);
        assert_eq!(ChannelDisplay::Green.next(), ChannelDisplay::Blue);
        assert_eq!(ChannelDisplay::Blue.next(), ChannelDisplay::Rgb);
    }

    #[test]
    fn apply_channel_display_rgb_is_identity() {
        let rgba = [10u8, 20, 30, 40, 50, 60, 70, 80];
        assert_eq!(apply_channel_display(&rgba, ChannelDisplay::Rgb), rgba.to_vec());
    }

    /// **本命**: 各チャンネルの値がグレースケール(3チャンネル同値)へ単離され、
    /// alpha は不透明(255)固定になる(module doc「AE の Show Channel 慣習」)。
    #[test]
    fn apply_channel_display_isolates_each_channel_to_grayscale_with_opaque_alpha() {
        let rgba = [10u8, 20, 30, 40, 200, 150, 100, 50];

        let red = apply_channel_display(&rgba, ChannelDisplay::Red);
        assert_eq!(red, vec![10, 10, 10, 255, 200, 200, 200, 255]);

        let green = apply_channel_display(&rgba, ChannelDisplay::Green);
        assert_eq!(green, vec![20, 20, 20, 255, 150, 150, 150, 255]);

        let blue = apply_channel_display(&rgba, ChannelDisplay::Blue);
        assert_eq!(blue, vec![30, 30, 30, 255, 100, 100, 100, 255]);

        let alpha = apply_channel_display(&rgba, ChannelDisplay::Alpha);
        assert_eq!(alpha, vec![40, 40, 40, 255, 50, 50, 50, 255]);
    }

    #[test]
    fn apply_channel_display_output_length_matches_input_for_whole_pixels() {
        let rgba = vec![1u8; 4 * 16];
        for mode in CHANNEL_CHOICES {
            assert_eq!(apply_channel_display(&rgba, mode).len(), rgba.len());
        }
    }

    // -- view builder 群: panic しないことのスモーク ------------------------

    #[test]
    fn view_builders_do_not_panic_for_every_state() {
        let dims = Dimensions::default();
        let colors = Colors::default();
        for tab in VIEW_TABS {
            let _ = view_tabs_view(tab, dims, colors);
        }
        for choice in RESOLUTION_CHOICES {
            let _ = resolution_quality_view(choice, 1.0, dims, colors);
        }
        for mode in CHANNEL_CHOICES {
            let _ = channel_display_view(mode, dims, colors);
        }
        let _ = viewer_bar_view(None, PreviewResolutionCap::Auto, 1.0, ChannelDisplay::Rgb, dims, colors);
        let _ = viewer_bar_view(
            Some(ObservationCamera::default()),
            PreviewResolutionCap::Half,
            0.5,
            ChannelDisplay::Alpha,
            dims,
            colors,
        );
    }
}
