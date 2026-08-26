//! ATTRS section(blend/speed/matte)。
//!
//! **持つ**: Blend 巡回ボタンの対応表と次値([`SUPPORTED_BLEND_MODES`]/
//! [`next_blend_mode`])・Speed 欄(SP1 第一波)の %⇄`motolii_store::Speed`
//! 写像([`percent_to_speed_ratio`]/[`speed_percent`])・ATTRS section の view
//! ([`attrs_section`]/`speed_row`)。
//!
//! **持たない**: Name/Hidden/ラベル色は ident 帯([`crate`] 直下の
//! `ident_band`)や [`crate::chrome`] の仕事 ── 発注書が「attrs.rs は
//! blend/speed」と名指ししたとおり、ここでは扱わない。
//! `LayerTiming`/`Intent::SetTiming` の組み立て・duration 再計算も
//! `motolii-shell` root の仕事(crate doc 参照)で、ここには置かない。
//! **Matte の意味と書き口は [`crate::matte`] の仕事**(2026-08-22 発注
//! 「レイヤーを指す」文法) ── `attrs_section` は `crate::matte::matte_row` を
//! 呼ぶだけで、`LayerAttrs.matte` そのものの巡回・書き込みロジックは持たない
//! (`attrs.rs` 自身が持つのは blend/speed のまま、という発注書の名指しを
//! 崩さない)。

use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{button, column, row as row_widget, text, text_input};
use iced::{Element, Length};

use motolii_settings_pane::chrome::section_header;
use crate::projection::AttrsProjection;
use crate::transform::format_number;
use crate::chrome::{bordered_row, flat_button_style, name_input_style, value_cell_padding};
use crate::Message;

// ---------------------------------------------------------------------------
// Speed 欄(ATTRS、SP1 第一波)— %⇄`motolii_store::Speed` の写像だけをここに置く。
// ---------------------------------------------------------------------------
//
// **`LayerTiming`/`Intent::SetTiming` の組み立て・duration 再計算はここでは
// 行わない**: duration 再計算の純関数(`retimed_duration`、supervisor 決定4)は
// `motolii-timeline-pane::clip_gesture` に住む(第二波 Shift+端drag と共有する
// ため、δ 採択理由)が、この crate は `motolii-timeline-pane` へ依存できない
// (`Cargo.toml` は今回の発注書 ALLOWLIST に含まれない — root→pane の一方向
// 依存を保つ既存の判断を、新しい循環を作らずに守った結果)。**両方に依存できる
// `motolii-shell` root がその組み立てを担う**(`Shell::apply_speed` —
// `commit_inspector_field` が `Value`/`Intent::SetTrack` まで組むのと違う分担、
// RETURN の FINDING 参照)。ここが持つのは「% ⇄ (num, den)」の純粋な往復だけ。

/// 表示 % → `motolii_store::Speed` の `(num, den)`。**p は正の有限値のみ受理**
/// (0・負・NaN・∞は `None` — supervisor 決定3「0 は拒否」)。**分母は 1000 固定**
/// (表示の小数1桁をそのまま整数化できる最小の桁 — `Speed::try_new` の不変式
/// 「分母は正」を機械的に満たす、値を約分はしない)。
pub fn percent_to_speed_ratio(percent: f64) -> Option<(i64, i64)> {
    if !percent.is_finite() || percent <= 0.0 {
        return None;
    }
    let tenths = (percent * 10.0).round();
    if tenths <= 0.0 {
        // 丸めで0以下になる極小値(例: 0.04%)も同じ理由で拒む。
        return None;
    }
    Some((tenths as i64, 1000))
}

/// `Speed` の `(num, den)` → 表示 %(逆算、[`percent_to_speed_ratio`] の逆写像)。
/// `format_number(_, 1)` と組み合わせて小数1桁で表示する(view 側)。`den == 0`
/// は `Speed::try_new` の不変式により本来起こらないが、安全側で 100.0 を返す。
pub fn speed_percent(num: i64, den: i64) -> f64 {
    if den == 0 {
        return 100.0;
    }
    num as f64 / den as f64 * 100.0
}

/// Blend 巡回ボタンが回る mode の一覧。**engine 側の変換表
/// (`next/engine/motolii-engine/src/lib.rs::translate_blend_mode`)と同期を保つ義務が
/// ある**(発注書「決定済み事項」— 対応 mode の一覧を engine 側と同じ場所には置か
/// ない、という決定に沿って Inspector 側にハードコードする)。**BL3(2026-08-22)**で
/// 分離可能11種(Multiply〜Exclusion)を追加——並びは `motolii_store::BlendMode` の
/// 宣言順(AE のメニュー順同型、Normal 直後に Add)のまま。非分離4種
/// (Hue/Saturation/Color/Luminosity、BL4)はまだここに無い(engine 側 `translate_blend_mode`
/// が対応するまで、対応 mode だけを巡る発注書の決定どおり)。dropdown 化する時に
/// この二重化は解消する。
pub const SUPPORTED_BLEND_MODES: &[motolii_store::BlendMode] = &[
    motolii_store::BlendMode::Normal,
    motolii_store::BlendMode::Add,
    motolii_store::BlendMode::Multiply,
    motolii_store::BlendMode::Screen,
    motolii_store::BlendMode::Overlay,
    motolii_store::BlendMode::Darken,
    motolii_store::BlendMode::Lighten,
    motolii_store::BlendMode::ColorDodge,
    motolii_store::BlendMode::ColorBurn,
    motolii_store::BlendMode::HardLight,
    motolii_store::BlendMode::SoftLight,
    motolii_store::BlendMode::Difference,
    motolii_store::BlendMode::Exclusion,
];

/// Blend 巡回ボタンの次の値。**現在値が [`SUPPORTED_BLEND_MODES`] に無い場合**
/// (将来の下位互換 — engine がまだ対応していない mode が Document に既に入って
/// いた時)は `Err` にしない — 現在値をそのまま表示し続け、次クリックで一覧の
/// 先頭へ進む(発注書「決定済み事項」)。
pub fn next_blend_mode(current: motolii_store::BlendMode) -> motolii_store::BlendMode {
    let modes = SUPPORTED_BLEND_MODES;
    match modes.iter().position(|mode| *mode == current) {
        Some(i) => modes[(i + 1) % modes.len()],
        None => modes[0],
    }
}

// `value_input_style` は裁定160 切片5(pane split survey §2.4/§6)で
// `chrome::value_input_style` へ移設した(純粋な再配置・挙動ゼロ変更)。

/// **ATTRS**: mock 断片には対応が無い行(Blend)だけ残す — Name は ident 帯へ、
/// Hidden は M glyph へ移した(重複 chrome を残さない、supervisor 訂正 2026-08-20)。
/// blend は**クリックで巡回するボタン**(BL2、supervisor 決定済み — pick_list は
/// next/ 全体に前例が無いので導入しない)。巡回先は [`SUPPORTED_BLEND_MODES`]
/// (現状 Normal→Add→Normal の2値、engine が対応する分だけ)。意匠は新規発明せず
/// `motolii_settings_pane::chrome::button_style`(`checkerboard_row` と同じ「押すたび
/// 即トグル」の形、他の意味色ロールは足さない)を流用する。
pub(crate) fn attrs_section(
    attrs: &AttrsProjection,
    speed_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let blend_content = row_widget![
        text("Blend")
            .size(dims.theme().text.body)
            .color(colors.text_primary)
            .width(Length::Fill),
        button(text(attrs.blend_mode.clone()).size(dims.theme().text.body))
            .on_press(Message::CycleBlendMode)
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.theme().space.xs)
    .align_y(iced::alignment::Vertical::Center);

    // `.prow` 系の行として同じ hairline を使う(mock 断片には Blend 行自体は
    // 無いが、`.prow` の row grammar をそのまま延長する — 発注書 NON-GOALS に
    // ある「新しい視覚言語の発明」ではなく、既存 grammar の適用)。
    let blend_row = bordered_row(blend_content.into(), dims);

    column![
        section_header("ATTRS", dims, colors),
        blend_row,
        speed_row(attrs, speed_draft, dims, colors),
        crate::matte::matte_row(attrs.matte, &attrs.matte_candidates, dims, colors),
    ]
    .into()
}

/// Speed 行(SP1 第一波、supervisor 決定1-7)。**click→type**(drag-to-scrub は
/// 第一波に含めない、NON-GOALS)。**裁定214/217(2026-08-23)への対応は未完**:
/// Speed は時間軸に乗るべき property(A03)で drag も要る(A02)と判定済みだが、
/// 書き口が `motolii-shell::Shell::apply_speed`(`next/shell/motolii-shell/
/// src/lib.rs`)にあり、この発注は shell 側を明示的に touch 禁止されている
/// ため、ここでは `motolii_store::PropertyId::speed()`(器のみ、`store/attrs.rs`
/// 参照)を足しただけで留めた(RETURN 参照)——`text_input` は常に存在し、Name 欄
/// ([`ident_band`])と同じ「フォーカスするだけで打鍵できる」形。Enter
/// (`Message::SpeedSubmit`)で確定、下書きが無い間は投影の現在値を表示する。
/// Reset ボタンは 100% でも常に出す(押せるが変わらない = 無反応ゼロより一貫を
/// 優先、決定7)。
fn speed_row(
    attrs: &AttrsProjection,
    speed_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = speed_draft
        .map(|text| text.to_owned())
        .unwrap_or_else(|| format_number(attrs.speed_percent, 1));

    // 裁定170 M01: fork の text_input が借用寿命を返り値に縛るため
    // owned move(値不変)。
    // 線化 D2(裁定179): 常設の text_input なので `value_input_style`(常時箱)
    // ではなく name 欄と同じ [`name_input_style`](平常=素・hover=面+枠・
    // focus=箱+focus 縁)へ合流 — 2箇所で別の意匠を発明しない。
    let value_field = text_input("", displayed)
        .on_input(Message::SpeedInput)
        .on_submit(Message::SpeedSubmit)
        .size(dims.theme().text.body)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    let content = row_widget![
        text("Speed")
            .size(dims.theme().text.body)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
        text("%").size(dims.theme().text.caption).color(colors.text_muted),
        button(text("Reset").size(dims.theme().text.caption))
            .on_press(Message::ResetSpeed)
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.theme().space.xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}
