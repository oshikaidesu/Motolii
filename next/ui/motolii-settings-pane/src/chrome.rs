//! pane 横断で共有されるスタイル/描画ヘルパ(裁定160 切片5、pane split survey
//! `docs/reviews/2026-08-21-pane-split-survey.md` §2.4/§6)。
//!
//! `settings_pane`(この crate 自身)は `inspector_pane::{section_header,
//! value_input_style, parse_number}` と `crate::button_style`(旧 `lib.rs` 定義)を
//! 再利用していた——いずれも `tokens::{Colors, Dimensions}`(または生の `&str`)
//! だけを読む純関数(`Session`/`Document` 非依存)で、pane 間の import 障壁の
//! 実体だった(survey §2.4)。ここへ吸い上げることで `settings_pane → inspector_pane`
//! の import をゼロにする。
//!
//! **純粋な再配置・挙動ゼロ変更**: 以下4関数はいずれも移設元から本体を無改変で
//! 移した(シグネチャ・実装とも1文字も変えていない、`section_header` の
//! `Message` 型パラメータの generic 化だけが例外 — 下記 doc 参照)。
//!
//! ## 裁定160 切片9: crate 抽出に伴う再配置
//! `settings_pane` を独立 crate(`motolii-settings-pane`)へ抽出するにあたり、
//! `chrome` はここへ一緒に移した(4関数全部を `settings_pane` 自身が必要と
//! していたため — survey §2.4「必要最小の共有」を満たす選択)。`motolii-shell`
//! (root)側の消費者(`inspector_pane.rs`・`lib.rs` 自身)は
//! `pub(crate) use motolii_settings_pane::chrome;`(`motolii-shell/src/lib.rs`)
//! 経由の re-export で `crate::chrome::X` を無改修のまま読み続ける —
//! `motolii-shell` は assembler として `motolii-settings-pane` に依存する側
//! なので、これは新しい循環を作らない(root → pane の一方向のまま)。
//!
//! `section_header` だけ `pub(crate) fn` → `pub fn` に加えて `Message` 型引数を
//! generic化した(`Element<'static, Message>` の `Message` を呼び出し側が
//! 選べるように) — `inspector_pane.rs`(root `Message` を使う)と
//! `settings_pane::view`(pane ローカル `Message` を使う)の**両方**から
//! 呼べる必要があるため。中身は `text`/`container` だけで `Message` を
//! 生成する widget が無く、型パラメータの束縛を追加せずに generic化できる
//! (呼び出し側の戻り値の型から単一化される)。他3関数は最初から `Message` に
//! 触れていないため無改変。
use iced::widget::{button, container, text, text_input};
use iced::{Element, Length};

use motolii_tokens_rs::{Colors, Dimensions, Ink};

/// header の3ボタン共通スタイル。**意味色ロール経由**(raw 値の直書き禁止) —
/// hover/pressed/disabled をそれぞれ別ロールで塗り分ける(状態: hover・選択・無効)。
/// `pub`: `settings_pane` のプリセット/市松トグルボタンに加え、`motolii-shell`
/// root 側(header の Undo/Redo/+Layer/Settings ボタン)も同じ意味色ロールを
/// 使う — 状態ごとに専用の色を新設しない。
///
/// 線化 D5(裁定179 文法1/4、`docs/reviews/2026-08-22-chrome-grammar-audit.md`):
/// 常時輪郭は廃止 — 区別は面の明度段(`surface_raised` 地 vs `surface_panel`
/// パネル地)が担い、border は**透明のまま幅だけ残す**(幾何不変の透明 border
/// 方式、`chip_outline_fence` と同型)。フェンス=
/// `tests/chrome_line_fence.rs`。
pub fn button_style(dims: Dimensions, colors: Colors, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => colors.surface_hover,
        button::Status::Pressed => colors.state_selected,
        button::Status::Disabled => colors.surface_panel,
        button::Status::Active => colors.surface_raised,
    };
    let text_color = if status == button::Status::Disabled {
        colors.state_disabled
    } else {
        colors.text_primary
    };
    button::Style {
        background: Some(iced::Background::Color(background)),
        text_color,
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
}

/// pane 容器(Settings/Inspector の root container)の共通スタイル。
///
/// 線化 D5(裁定179 文法1「枠は内容と同族色、段差は明度1段だけ」):
/// パネルは `surface_panel` の面で app 地(`surface_app`、theme 経由の窓地)
/// から**明度1段**浮く — 容器の輪郭線は描かない(透明 border で幅だけ残す=
/// 幾何不変)。区切りは pane_grid の隙間(`spacing_m`、app 地が覗く)と
/// この明度段が担う(AE=暗い隙間 / Figma=明度差のみ、と同文法)。
/// `pub`: `settings_pane`(この crate)と `inspector_pane` が同じ容器意匠を
/// 再利用する — 2箇所で別の意匠を発明しない。フェンス=
/// `tests/chrome_line_fence.rs`。
pub fn panel_container_style(dims: Dimensions, colors: Colors) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(colors.surface_panel)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// `pub`: `settings_pane`(この crate)と `inspector_pane`(root 側)が同じ
/// 見出し帯(パネルタイトル/section 見出し共通トークン)を再利用する — 2箇所で
/// 別の意匠を発明しない。`Message` は generic(上記 doc 参照)。
pub fn section_header<Message: 'static>(
    label: &'static str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    // `.width(Length::Fill)`: `header` と同じ理由(柵で発見) — mock の `.sec` も
    // block 要素で pane 全幅の帯(実測: 修正前は幅 65〜68px)。
    //
    // **背景は塗らない**(裁定137/139、2026-08-21 更正): mock `.sec` は
    // `background`/`border` のどちらも持たない — 見出しは letter-spacing +
    // ink3(`text_muted`)+ 行高だけで区別する(旧実装は `surface_app` で塗って
    // 「面色の塗り分けで区切る」を犯していた — TRANSFORM/APPEARANCE/ATTRS の
    // 帯が周囲の `.prow` 行と違う沈んだ色の箱に見えていたのが実体)。
    container(
        text(label)
            .size(dims.caption_text)
            .color(Ink::Muted.resolve(&colors)),
    )
    .width(Length::Fill)
    .height(Length::Fixed(dims.inspector_section_header_height))
    .padding([0.0, dims.spacing_m])
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

/// `pub`: `settings_pane`(この crate)と `inspector_pane`(root 側)の数値欄
/// (背景RGBA・ui_scale%・Transform 行)も同じ枠色ロールを使う — 2箇所で
/// 別の意匠を発明しない。
pub fn value_input_style(
    dims: Dimensions,
    colors: Colors,
    status: text_input::Status,
) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => colors.action_active,
        _ => colors.border_default,
    };
    text_input::Style {
        background: iced::Background::Color(colors.surface_app),
        border: iced::Border {
            color: border_color,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        // 裁定170 M01: fork(0.15.0-dev)で `text_input::Style` から `icon`
        // フィールドが消えた。この crate は `text_input(...)` に `.icon(..)`
        // を一度も呼んでいない(usage 実測、icon glyph 自体が未設定)ため、
        // 0.14 でもこの色は何も描いていなかった — 行削除は見た目不変。
        placeholder: colors.text_muted,
        value: colors.text_primary,
        selection: colors.action_active,
    }
}

/// 入力文字列 → 数値。mock(`inspector-library.html`)は負号に `−`(U+2212)を使うので
/// 両対応する。`settings_pane`(背景RGBA・ui_scale%)・`inspector_pane`
/// (Transform 行)も同じパーサを使う(2箇所で別の意匠を発明しない)。
pub fn parse_number(text: &str) -> Option<f64> {
    text.trim().replace('\u{2212}', "-").parse::<f64>().ok()
}
