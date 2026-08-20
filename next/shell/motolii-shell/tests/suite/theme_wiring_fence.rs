//! 柵: 実窓の地色欠陥(`main.rs` の `iced::application(...)` に `.theme(...)`
//! 結線が無かった)の再発を防ぐ。
//!
//! **実態**(`next/` workspace の `iced` は crates.io 版 0.14.0 そのもの、
//! fork/patch なし — `cargo metadata` で確認済み): `.theme()` を呼ばないと
//! `Program::theme` の既定実装(`iced_program-0.14.0/src/lib.rs` 100-106行、
//! `fn theme(...) -> Option<Self::Theme> { None }`)が常に `None` を返し、
//! winit 側は `<Theme as theme::Base>::default(system_theme)`
//! (`iced_winit-0.14.0/src/window/state.rs` 60行)へフォールバックする。
//! `Theme::default`(`iced_core-0.14.0/src/theme.rs` 257-281行)は
//! `Mode::None | Mode::Light => Self::Light` — OS 外観取得に失敗すれば常に
//! iced 組み込みの `Theme::Light`、取得できても組み込みの `Theme::Dark` で
//! あって、どちらも `tokens::Colors`(このリポジトリの色トークン正本)
//! 由来ではない。
//!
//! `main.rs` は binary crate で公開関数を持たない(`Shell` を組み立てて
//! `iced::application` へ渡すだけの「窓を開けるだけ」の薄い口 — ファイル冒頭の
//! doc comment 参照)ので、この柵は `.theme()` の**呼び出し漏れ**をソース走査で
//! 固定する(KNOWN「柵は実装しやすい方で1本」)。`theme_from_colors` 自体が
//! tokens から正しく Theme を組んでいることは下の3本で別途確かめる。

const MAIN_RS: &str = include_str!("../../src/main.rs");

/// **本命**: `iced::application(...)` から `.run()` までの builder chain に
/// `.theme(` が含まれること。無いと `Program::theme` の既定実装(常に `None`)
/// のまま — 上のコメントの欠陥そのものへ逆戻りする。
#[test]
fn iced_application_builder_wires_theme_before_run() {
    let app_start = MAIN_RS
        .find("iced::application(")
        .expect("main.rs から iced::application(...) の呼び出しが消えている");
    let after_app = &MAIN_RS[app_start..];
    let run_pos = after_app
        .find(".run()")
        .expect(".run() が見つからない — builder chain の終端を検出できない");
    let builder_chain = &after_app[..run_pos];

    assert!(
        builder_chain.contains(".theme("),
        "iced::application(...) の組み立てに .theme(...) が結線されていない。\
         未結線だと Program::theme の既定実装が常に None を返し、winit は \
         OS 設定依存の組み込み Light/Dark(tokens と無関係)へフォールバックする \
         — このファイル冒頭のコメント、tokens::theme_from_colors の doc comment 参照"
    );
}

/// `theme_from_colors` が返す `Theme` の窓の地色(`theme::Base::base` の
/// `background_color` — 実窓が clear に使う色そのもの)が `Colors::surface_app`
/// と一致すること。`Extended::generate` は背景色を変換せずそのまま
/// `Background::base.color` へ通す(`iced_core-0.14.0/src/theme/palette.rs`)
/// ので厳密一致。
#[test]
fn theme_from_colors_uses_the_app_surface_as_the_window_base_color() {
    use iced::theme::Base;
    use motolii_shell::tokens::{theme_from_colors, Colors};

    let colors = Colors::default();
    let theme = theme_from_colors(&colors);
    let style = theme.base();

    assert_eq!(
        style.background_color, colors.surface_app,
        "Theme::base().background_color が tokens の surface_app と一致しない — \
         実窓の地色が tokens 経由になっていない疑い"
    );
}

/// tokens の既定色(`Colors::default` = ダーク系、`surface_app` は暗い灰)で
/// 組んだ `Theme` が `Mode::Dark` を名乗ること。組み込み `Theme::Light` への
/// フォールバック(欠陥時の実際の挙動)ではこの assert は必ず落ちる。
#[test]
fn theme_from_colors_reports_dark_mode_not_the_builtin_light_fallback() {
    use iced::theme::Base;
    use motolii_shell::tokens::{theme_from_colors, Colors};

    let theme = theme_from_colors(&Colors::default());
    assert_eq!(
        theme.mode(),
        iced::theme::Mode::Dark,
        "tokens の暗い surface_app から作った Theme が Dark を名乗っていない"
    );
}

/// `theme_from_colors` は `Colors` の純粋関数(ハードコードした固定パレットを
/// 返すのではない) — `surface_app` を変えれば `Theme` の地色も追随する。
/// これは `tokens::watch_subscription`(debug のホットリロード)経由で
/// `Shell.tokens` が差し替わったときに `.theme(|shell| ...)` が次の
/// `synchronize` で新しい色を拾える、という主張の直接の裏付け。
#[test]
fn theme_from_colors_tracks_token_values_instead_of_a_hardcoded_palette() {
    use iced::theme::Base;
    use motolii_shell::tokens::{theme_from_colors, Colors};

    let default_colors = Colors::default();
    let mut changed_colors = default_colors;
    changed_colors.surface_app = iced::Color::from_rgb(0.9, 0.9, 0.9);

    let default_theme = theme_from_colors(&default_colors);
    let changed_theme = theme_from_colors(&changed_colors);

    assert_ne!(
        default_theme.base().background_color,
        changed_theme.base().background_color,
        "surface_app を変えても Theme の地色が変わらない — 固定パレットに \
         戻っている疑い(watch 再読込に追随できなくなる)"
    );
    assert_eq!(changed_theme.base().background_color, changed_colors.surface_app);
}
