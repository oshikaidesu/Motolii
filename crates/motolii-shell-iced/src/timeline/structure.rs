//! Timeline の **構造操作の絵**: 行の畳み開閉(▶ / ◇◆)・Property 行・
//! rename のインライン編集・L(ロック)ボタン。
//!
//! [`super::canvas`] の行ループから呼ばれる薄い描画関数群と、
//! [`super::semantics`] の hit-test 幾何を使う純粋な補助だけを置く
//! (2026-08-19 構造操作レーン。柵: `canvas.rs` 本体はここへ委ねる呼び出しの
//! 追加だけに留め、実体の描画・判定コードは全部この新ファイルへ)。
//!
//! ## 移植元の対応表(egui 版 `timeline_editor/mod.rs`)
//!
//! | ここ | 移植元 | 意味 |
//! |---|---|---|
//! | [`draw_fold_arrow`] | `rail_glyph` の ▾/▸ 呼び出し | 子の開閉矢印 |
//! | [`draw_params_toggle`] | `rail_glyph` の ◆/◇ 呼び出し | param 行の開閉 |
//! | [`draw_lock_button`] | `rail_button` の `("L", ItemFlag::Lock)` | ロック |
//! | [`draw_rename_box`] | `TextEdit::singleline` 分岐 | 行名のその場編集 |
//!
//! **`RowKind::Property(param)` 分岐(param の子行の絵)は [`super::keys`] へ
//! ある。** 元々ここにも `draw_property_row` / `param_color` / `param_label`
//! の重複実装があったが、`canvas.rs` の draw ループが `RowKind::Property` を
//! 先に判定して無条件 `continue` するため、ここの実装は1度も呼ばれない
//! 到達不能コードだった(2026-08-19 UIトンマナ統一 campaign で実測・削除。
//! `param_color` は `keys.rs` 側の実装と同じ `ParamRef::Position` に別々の
//! 色を返す食い違いも持っていた — 経緯は `timeline/palette.rs` のモジュール
//! doc)。

use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{alignment, Color, Pixels, Point, Size};

use motolii_doc::LayerId;

use crate::theme::Tokens;

/// 子の開閉矢印。**egui 版の ▾/▸ をテキストで描く方針をそのまま写す**
/// (フォントに無い記号を使わない、が egui 側の理由 — この canvas も同じ
/// フォント制約を持つので同じ選択をする)。
#[allow(clippy::too_many_arguments)]
pub fn draw_fold_arrow(frame: &mut Frame, x0: f32, x1: f32, y_center: f32, open: bool, ink: Color) {
    frame.fill_text(Text {
        content: if open {
            "\u{25be}".to_owned()
        } else {
            "\u{25b8}".to_owned()
        },
        position: Point::new((x0 + x1) * 0.5, y_center),
        color: ink,
        size: Pixels(11.0),
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center.into(),
        ..Text::default()
    });
}

/// param 行の開閉(◇/◆)。egui 版と同じ記号(黒ダイヤ=開・白抜き=閉)。
pub fn draw_params_toggle(
    frame: &mut Frame,
    x0: f32,
    x1: f32,
    y_center: f32,
    open: bool,
    ink: Color,
) {
    frame.fill_text(Text {
        content: if open {
            "\u{25c6}".to_owned()
        } else {
            "\u{25c7}".to_owned()
        },
        position: Point::new((x0 + x1) * 0.5, y_center),
        color: ink,
        size: Pixels(11.0),
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center.into(),
        ..Text::default()
    });
}

/// L ボタン。M / S(`draw_flag_button` 相当、`canvas.rs` に既存)と同じ枠付き
/// 角丸のスタイルを、L 専用の3値(own lock / 継承だけの lock / 平常)で描く。
/// **継承ぶんは薄い塗りだけ**(押しても外れない旨は effective ロックの薄塗りで
/// 見せる — egui 版 `LOCK_INHERITED` の移植)。
#[allow(clippy::too_many_arguments)]
pub fn draw_lock_button(
    frame: &mut Frame,
    x_range: (f32, f32),
    y0: f32,
    h: f32,
    own_lock: bool,
    inherited_only: bool,
    hovered: bool,
    tokens: &Tokens,
) {
    let (x0, x1) = x_range;
    let w = (x1 - x0).max(1.0);
    let accent = tokens.action_active;
    let (border, background, text_color) = if own_lock {
        (accent, Color { a: 0.18, ..accent }, accent)
    } else if hovered {
        (
            tokens.border_strong,
            tokens.surface_panel,
            tokens.text_primary,
        )
    } else {
        (tokens.border_default, tokens.surface_app, tokens.text_muted)
    };
    let rect = Path::rounded_rectangle(Point::new(x0, y0), Size::new(w, h), 2.0.into());
    if inherited_only && !own_lock {
        // 親から受けているだけ: 押しても外れない旨を薄い塗りで先に言う。
        frame.fill(&rect, Color { a: 0.22, ..accent });
    }
    frame.fill(&rect, background);
    frame.stroke(&rect, Stroke::default().with_color(border).with_width(1.0));
    frame.fill_text(Text {
        content: "L".to_owned(),
        position: Point::new(x0 + w * 0.5, y0 + h * 0.5),
        color: text_color,
        size: Pixels(9.0),
        font: iced::Font::MONOSPACE,
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center.into(),
        ..Text::default()
    });
}

/// rename 中の行の、その場編集の見た目。**本物のテキスト入力は持たない**
/// (canvas 手描きの制約 — 文字はキーボード event から `TimelinePane::renaming`
/// の buffer へ積み、ここは buffer をそのまま描くだけ)。カーソルは末尾固定の
/// 縦棒(この canvas に caret 位置の概念が無い — 全選択 → 上書きと同じ手触り)。
pub fn draw_rename_box(
    frame: &mut Frame,
    name_rect: iced::Rectangle,
    buffer: &str,
    tokens: &Tokens,
) {
    let bg = Path::rectangle(
        Point::new(name_rect.x, name_rect.y),
        Size::new(name_rect.width, name_rect.height),
    );
    frame.fill(&bg, tokens.surface_app);
    frame.stroke(
        &bg,
        Stroke::default()
            .with_color(tokens.action_active)
            .with_width(1.0),
    );
    frame.fill_text(Text {
        content: buffer.to_owned(),
        position: Point::new(name_rect.x + 4.0, name_rect.y + name_rect.height * 0.5),
        color: tokens.text_primary,
        size: Pixels(11.0),
        align_y: alignment::Vertical::Center.into(),
        ..Text::default()
    });
    // caret: 文字の右にもう1本立てるだけ(正確な文字幅を測っていない —
    // canvas に text measurement の口が無いので、末尾の目印としてだけ機能する)。
    let caret_x = name_rect.x + 4.0 + buffer.len() as f32 * 6.2;
    if caret_x < name_rect.x + name_rect.width - 2.0 {
        frame.fill_rectangle(
            Point::new(caret_x, name_rect.y + 3.0),
            Size::new(1.0, name_rect.height - 6.0),
            tokens.text_primary,
        );
    }
}

/// 名前をその枠の幅に収める。**この canvas に egui 版 `with_clip_rect` の
/// ような口が無い**ので、描く前に文字数で切る(2026-08-19 発見: 長い名前が
/// ◇/◆ や M/S/L に重なって描かれていた — `row_name_right_edge` と対で使う)。
/// 平均文字幅は比例フォント size 11 の近似値(実測ではなく概算 — ピクセル
/// 単位の一致ではなく「重ならない」ことが目的)。
pub fn truncate_name(name: &str, max_width: f32) -> String {
    const AVG_CHAR_W: f32 = 6.4;
    if max_width <= 0.0 {
        return String::new();
    }
    let max_chars = (max_width / AVG_CHAR_W).floor() as usize;
    let len = name.chars().count();
    if len <= max_chars {
        return name.to_owned();
    }
    if max_chars <= 1 {
        return "…".to_owned();
    }
    let mut truncated: String = name.chars().take(max_chars - 1).collect();
    truncated.push('…');
    truncated
}

/// 二重クリックの間合い(ms)。rename を始める入口の1つ(rail の名前を
/// ダブルクリック)。OS 標準の 500ms 前後という一般的な値を採る — 基準
/// (egui)にはダブルクリック起点そのものが無いので写す先が無い、この
/// レーンの新規判断である。
pub const DOUBLE_CLICK_MS: u128 = 450;

/// 直前のクリックと同じ layer・間合い内かどうか。canvas ローカル状態
/// (`TimelineCanvasState`)が呼ぶ純関数。
pub fn is_double_click(
    last: Option<(LayerId, std::time::Instant)>,
    layer: LayerId,
    now: std::time::Instant,
) -> bool {
    match last {
        Some((last_layer, at)) => {
            last_layer == layer && now.saturating_duration_since(at).as_millis() <= DOUBLE_CLICK_MS
        }
        None => false,
    }
}
