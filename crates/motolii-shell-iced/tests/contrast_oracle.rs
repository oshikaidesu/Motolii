//! contrast oracle — トンマナ統一 campaign 第3波レーンH-c: 評価軸の定数化(第1弾)。
//!
//! 基準は `docs/ui-visual-language.md`: 通常文字 contrast **4.5:1** 以上、
//! 大文字/太字 3:1、意味を持つ UI 境界/icon **3:1**(WCAG 2.2 SC 1.4.11 借用)。
//! ここは `Tokens::DARK`(`crates/motolii-shell-iced/src/theme/mod.rs`)の
//! 21 role のうち、実際に「文字 role × 面 role」「境界 role × 面 role」として
//! 使われている組だけを検査する — 発明した組み合わせは置かない
//! (使用実態が確認できない組は末尾の `EVIDENCE_GAP` コメントへ)。
//!
//! ## WCAG 計算はこのファイル内に実装する(依存 crate を足さない)
//!
//! `iced::Color` の `r`/`g`/`b` は既に sRGB 0.0..=1.0 の値
//! (`Tokens::rgb` が `Color::from_rgb8` = `u8/255.0` を経由するだけ)なので、
//! そのまま WCAG の相対輝度式へ渡せる。
//!
//! ## 文字ペアの対応表(role × 面、使用箇所 file:line が根拠)
//!
//! | 文字 role | 面 role | 根拠(1つずつ) |
//! |---|---|---|
//! | text_primary | surface_raised | `theme/style.rs:24`(`action` button, Active 状態の地) |
//! | text_primary | surface_hover | `theme/style.rs:25`(`action` button, Hovered/Pressed の地) |
//! | text_primary | surface_panel | `inspector_pane.rs:434` + `timeline/structure.rs:84`(lock button hover: `(border_strong, surface_panel, text_primary)`)、`inspector_pane.rs:948` + `inspector_pane.rs:757`(`row_panel_style` = surface_panel) |
//! | text_primary | surface_app | `timeline/structure.rs:121`(rename box を surface_app で塗る)+ `:131`/`:143`(同じ box に text_primary で文字を描く) |
//! | text_secondary | surface_app | `view.rs:141`(TAGLINE, 座席が無い画面の既定地 = surface_app)、`inspector_pane.rs:567` + `inspector_pane.rs:524`(`section_band_style` = surface_app) |
//! | text_secondary | surface_panel | `view.rs:367` + `view.rs:45`(`status_band` = surface_panel)、`inspector_pane.rs:201`/`inspector_pane.rs:801` + `inspector_pane.rs:1014`(`panel_style` = surface_panel) |
//! | text_muted | surface_app | `view.rs:156`(DROP_HINT)、`timeline/structure.rs:86`(lock button idle 状態の fill+text)、`inspector_pane.rs:980`/`inspector_pane.rs:573` + `inspector_pane.rs:524`(`section_band_style`) |
//! | text_muted | surface_raised | `view.rs:170`(action_button の近道, 地は Active 状態の `surface_raised`)、`inspector_pane.rs:353` + `inspector_pane.rs:387`(`raised_panel_style` = surface_raised) |
//! | text_muted | surface_panel | `view.rs:289`/`view.rs:377` + `view.rs:45`(`status_band`)、`inspector_pane.rs:223` + `inspector_pane.rs:312`(`bottom_border_style`)、`inspector_pane.rs:715` + `inspector_pane.rs:757`(`row_panel_style`) |
//!
//! disabled 系(`button::Status::Disabled` = `theme/style.rs:26` の
//! `surface_panel`/`text_muted`)は対象外(意味的に低コントラストが正 —
//! 発注書の ALLOWLIST 節の指示どおり)。ここで検査する `text_muted` の面ペアは
//! すべて非 disabled の常設表示(idle 状態・添え物・ヒント行)から採った。
//!
//! ## 境界ペアの対応表(border role × 隣接面 role、border 自身の fill に対して)
//!
//! WCAG 1.4.11 は「意味を持つ UI 境界」を隣接色に対して測る。ここでは
//! `iced::Border` が塗りの外周に付く場所を、その塗り(面)に対して測った —
//! ボタンや canvas 矩形が「自分の地に対して縁取りが見えるか」という、
//! 発注書が言う "境界ペア" の最も直接的な読み方。
//!
//! | 境界 role | 面 role | 根拠 | 実測 | 3:1 判定 |
//! |---|---|---|---|---|
//! | border_strong | surface_panel | `inspector_pane.rs:434` + `timeline/structure.rs:84`(lock button hover の stroke/fill) | 3.12 | 満たす → assert |
//! | border_default | surface_raised | `theme/style.rs:24` + `:32-33`(action button Active の stroke/fill) | 1.42 | 満たさない → pin |
//! | border_default | surface_hover | `theme/style.rs:25` + `:32-33`(action button Hovered/Pressed の stroke/fill) | 1.25 | 満たさない → pin |
//! | border_default | surface_app | `timeline/structure.rs:86`(lock button idle の stroke/fill) | 1.64 | 満たさない → pin |
//! | border_default | surface_panel | `inspector_pane.rs:1011-1019`(`panel_style`, Inspector 全体の外枠) | 1.55 | 満たさない → pin |
//!
//! `border_default` は生成 token の中で最も暗い非背景色(`#3b3b3b`)なので、
//! どの面と組んでも 3:1 に届かない — 個々の使用箇所の欠陥ではなく token 自体の
//! 値の性質。`border_strong`(`#686868`)へ差し替えれば `surface_panel` は
//! 3.12 で越える(このテストの `known-pass` 行が実例)が、`surface_app`
//! (最も暗い面)相手では `border_strong` でも届かない可能性がある — この
//! 組は使用実態が無いので検査していない(EVIDENCE_GAP参照)。
//!
//! ## EVIDENCE_GAP(使用実態が確認できず検査しなかった組)
//!
//! - `focus` role: `theme/mod.rs:193` の `entries()` 一覧にしか出ず、
//!   実際に focus ring や輪郭として描画している箇所が `src/` に無い
//!   (`grep -rn "\.focus\b" src/` は `entries()` の1行のみ)。文字にも境界にも
//!   使われていないので、このテストでは検査しない。
//! - `border_strong` × `surface_app` / `surface_raised` / `surface_hover`:
//!   `border_strong` の実使用は hover 状態の lock button(面は常に
//!   `surface_panel`)だけで、他の面と組んで使われている箇所が無い。
//! - `border_default` × `surface_hover` 以外の hover 面(`surface_raised`
//!   自体の hover 遷移など)、および way_* / data / shape / status_* の
//!   accent 色を「文字として面に置く」使用(scrub 部品の accent 文字色など)
//!   は、面が呼び出し側ごとに動的に変わる(`accent: iced::Color` 引数)ため、
//!   「この面と組む」と1組に固定できる根拠 file:line が無い。固定の対応が
//!   増えたら追検査する。

use iced::Color;

use motolii_shell_iced::theme::Tokens;

/// sRGB 0.0..=1.0 の1成分を線形化する(WCAG 2.x 定義)。
fn linearize(c: f32) -> f64 {
    let c = c as f64;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 相対輝度。`iced::Color` の r/g/b は既に sRGB 0.0..=1.0
/// (`Tokens::rgb` = `Color::from_rgb8` = `u8/255.0` を経由するだけ)なので、
/// そのまま渡せる。
fn relative_luminance(c: Color) -> f64 {
    0.2126 * linearize(c.r) + 0.7152 * linearize(c.g) + 0.0722 * linearize(c.b)
}

/// WCAG コントラスト比 = (明るい方の輝度+0.05) / (暗い方の輝度+0.05)。
fn contrast_ratio(a: Color, b: Color) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (lighter, darker) = if la > lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

/// 通常文字の下限(`docs/ui-visual-language.md`)。
const NORMAL_TEXT_MIN: f64 = 4.5;
/// 添え物文字(muted)の下限 — 大文字/太字と同じ 3:1 を借用
/// (発注書 CONTEXT 節の指示どおり)。
const MUTED_TEXT_MIN: f64 = 3.0;
/// 意味を持つ UI 境界の下限(WCAG 2.2 SC 1.4.11 借用)。
const BORDER_MIN: f64 = 3.0;

/// primary/secondary — 4.5:1 以上を要求する文字ペア。
#[test]
fn primary_and_secondary_text_meet_normal_text_minimum() {
    let t = Tokens::DARK;
    let pairs: [(&str, Color, &str, Color); 6] = [
        ("text_primary", t.text_primary, "surface_raised", t.surface_raised),
        ("text_primary", t.text_primary, "surface_hover", t.surface_hover),
        ("text_primary", t.text_primary, "surface_panel", t.surface_panel),
        ("text_primary", t.text_primary, "surface_app", t.surface_app),
        ("text_secondary", t.text_secondary, "surface_app", t.surface_app),
        ("text_secondary", t.text_secondary, "surface_panel", t.surface_panel),
    ];
    for (text_name, text_color, surface_name, surface_color) in pairs {
        let ratio = contrast_ratio(text_color, surface_color);
        assert!(
            ratio >= NORMAL_TEXT_MIN,
            "{text_name} on {surface_name}: {ratio:.3} < {NORMAL_TEXT_MIN} (通常文字の下限)"
        );
    }
}

/// muted — 3:1 以上を要求する文字ペア(添え物文字。大文字/太字と同じ下限を借用)。
#[test]
fn muted_text_meets_the_borrowed_large_text_minimum() {
    let t = Tokens::DARK;
    let pairs: [(&str, Color, &str, Color); 3] = [
        ("text_muted", t.text_muted, "surface_app", t.surface_app),
        ("text_muted", t.text_muted, "surface_raised", t.surface_raised),
        ("text_muted", t.text_muted, "surface_panel", t.surface_panel),
    ];
    for (text_name, text_color, surface_name, surface_color) in pairs {
        let ratio = contrast_ratio(text_color, surface_color);
        assert!(
            ratio >= MUTED_TEXT_MIN,
            "{text_name} on {surface_name}: {ratio:.3} < {MUTED_TEXT_MIN} (muted の下限)"
        );
    }
}

/// 境界ペアのうち、現行値で 3:1 を満たす組(`border_strong` × `surface_panel`)。
#[test]
fn border_strong_on_surface_panel_meets_the_ui_boundary_minimum() {
    let t = Tokens::DARK;
    let ratio = contrast_ratio(t.border_strong, t.surface_panel);
    assert!(
        ratio >= BORDER_MIN,
        "border_strong on surface_panel: {ratio:.3} < {BORDER_MIN} (UI境界の下限)"
    );
}

/// 境界ペアのうち、現行値で 3:1 に届かない組(`border_default` は生成 token
/// 中で最も暗い非背景色 — token 自体の値の性質であって、個々の使用箇所の
/// 欠陥ではない)。**ここでは 3:1 を要求しない** — 現行の実測値を pin し、
/// 「下回る方向への変化」だけを fail させる(発注書 CONTEXT 節の指示どおり。
/// 改善のために閾値を先取りしない — 直すのは別レーンの仕事)。
///
/// 実測(この定数化レーンで測った値。数値自体が乖離の記録):
/// - border_default × surface_raised = 1.420
/// - border_default × surface_hover  = 1.247
/// - border_default × surface_app    = 1.645
/// - border_default × surface_panel  = 1.554
#[test]
fn border_default_known_deviation_does_not_regress_further() {
    let t = Tokens::DARK;
    // (面名, 面色, 実測時の下限pin — 実測値よりわずかに低く取って浮動小数の
    // 丸めで誤って fail しないようにする。3:1 を要求しない — 既知の乖離)。
    let pairs: [(&str, Color, f64); 4] = [
        ("surface_raised", t.surface_raised, 1.41),
        ("surface_hover", t.surface_hover, 1.24),
        ("surface_app", t.surface_app, 1.64),
        ("surface_panel", t.surface_panel, 1.55),
    ];
    for (surface_name, surface_color, pinned_min) in pairs {
        let ratio = contrast_ratio(t.border_default, surface_color);
        assert!(
            ratio >= pinned_min,
            "border_default on {surface_name} 退行: {ratio:.3} < pin {pinned_min} \
             (3:1 には元々届いていない既知乖離 — これは『さらに下がった』検知)"
        );
        // 参考: 3:1 自体は満たさないことをここでも明示する(誤って基準を
        // 満たしたように読めないよう、意図して assert しない側を書いておく)。
        assert!(
            ratio < BORDER_MIN,
            "border_default on {surface_name} が {BORDER_MIN}:1 を超えた({ratio:.3}) — \
             既知乖離のコメント・表を更新すること(改善が起きたなら喜ばしいが、\
             この pin テストの意図が変わる)"
        );
    }
}
