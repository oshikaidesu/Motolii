//! `--fixture --screenshot <path>` の検分器具 — 1フレームを PNG へ書いて終了する口。
//!
//! **iced_test に snapshot 機能は無い**(`iced_test-0.14.0` の公開 API を実測 —
//! `emulator.rs`/`simulator.rs`/`instruction.rs` のどれにも画素を返す口が無い。
//! `Simulator` はレイアウトと hit-test だけの headless 器具であって、rasterize は
//! しない)。iced 本体のフル描画も、実ウィンドウ(wgpu surface)を開かずに
//! oneshot でレンダするには headless compositor の自前配線が要る — この器具の
//! スコープ(トンマナ検分の instrument 1本)には過剰。
//!
//! 発注書が明示的に許す代替(「無理なら stage+pane の合成PNG」)を採る:
//! Stage は `motolii_engine::Engine` が実際に GPU 合成した RGBA を貼るが、
//! **市松が有効なら `lib.rs::refresh_frame`/`build_stage_handle` と同じ入力
//! (`Shell::checkerboard_preview_rgba`、裁定141「背景を敷かない」合成)へ
//! `settings_pane::composite_checkerboard` を自分でも当てる** — この器具の
//! 仕事は「実際に画面へ出る絵」の検分なので、`frame_rgba()`(常に背景込みの
//! export 真値)をそのまま貼ると市松トグルの効果が画面から消えてしまう
//! (裁定141以降、市松は不透明背景でも見えるはずの状態)。`frame_rgba()`
//! 自体は書き換えない(export/screenshot 生値の不変は保つ)。
//! Timeline は `timeline_pane` と**同じ投影関数**(`rows`/`frame_to_x`/
//! `time_band_segment_frames`)を使って同じ位置関係(明暗リズム・hairline の
//! 位置含む、裁定148)を再現するが、**ここで実際に塗るのはこのモジュール自身**
//! (iced の `canvas::Frame` ではなく `image::RgbaImage` へ矩形・線を直接塗る —
//! iced の canvas 描画パスは wgpu レンダラを要るため headless では使えない)。
//! header/transport/status 帯は色面 + hairline 縁で再現する(裁定139 の
//! shell chrome への展開)。Settings パネルは `--settings-open` フラグが立って
//! いる間だけ、header の下へ Inspector と同じ hairline 積みで描く。
//!
//! **正直な限界**: 文字(層名・timecode・status 文言)は描かない。フォント
//! ラスタライズには新しい依存(ab_glyph 等)が要り、発注書の「新依存禁止」に
//! 触れるため見送った。この器具は「トンマナ(色・位置・密度)の照合」が目的で、
//! 文字の可読性検分はこの器具の対象外(実窓のスクリーンショットで別途見る)。

use image::{Rgba, RgbaImage};

use crate::inspector_pane::SelectionProjection;
use crate::tokens::{Colors, Dimensions};
use crate::{settings_pane, timeline_pane, Shell};

fn to_rgba(color: iced::Color, alpha: f32) -> Rgba<u8> {
    let to_u8 = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    Rgba([to_u8(color.r), to_u8(color.g), to_u8(color.b), to_u8(alpha)])
}

/// アルファ合成の src-over。canvas の初期値は既に不透明(surface 色)なので、
/// dst 側のアルファは常に1として扱う。
fn blend_pixel(canvas: &mut RgbaImage, x: i64, y: i64, color: Rgba<u8>) {
    if x < 0 || y < 0 || x as u32 >= canvas.width() || y as u32 >= canvas.height() {
        return;
    }
    let a = color[3] as f32 / 255.0;
    if a <= 0.0 {
        return;
    }
    let dst = canvas.get_pixel_mut(x as u32, y as u32);
    for channel in 0..3 {
        let src = color[channel] as f32;
        let base = dst[channel] as f32;
        dst[channel] = (src * a + base * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
    }
}

fn fill_rect(canvas: &mut RgbaImage, x: f32, y: f32, w: f32, h: f32, color: Rgba<u8>) {
    let x0 = x.floor() as i64;
    let y0 = y.floor() as i64;
    let x1 = (x + w).ceil() as i64;
    let y1 = (y + h).ceil() as i64;
    for py in y0..y1 {
        for px in x0..x1 {
            blend_pixel(canvas, px, py, color);
        }
    }
}

/// 縦線(マーカー・playhead・ルーラー目盛り)。`width_px` は罫線幅 token。
fn stroke_v(canvas: &mut RgbaImage, x: f32, y0: f32, y1: f32, width_px: f32, color: Rgba<u8>) {
    fill_rect(
        canvas,
        x - width_px / 2.0,
        y0,
        width_px.max(1.0),
        (y1 - y0).max(1.0),
        color,
    );
}

/// 横線(Timeline の行区切り hairline)。`stroke_v` の水平版 — `stroke_rect`
/// (4辺一律)とは違い、こちらは border-bottom 1本だけを直接引ける(canvas 描画
/// なので Inspector 側の「既知の限界」は無い)。
fn stroke_h(canvas: &mut RgbaImage, x0: f32, x1: f32, y: f32, width_px: f32, color: Rgba<u8>) {
    fill_rect(
        canvas,
        x0,
        y - width_px / 2.0,
        (x1 - x0).max(1.0),
        width_px.max(1.0),
        color,
    );
}

/// 任意の2点を結ぶ線分(`stroke_v`/`stroke_h`は軸並行専用 — 観測カメラの
/// フレーム枠 overlay はレンダリングカメラに roll が付くと軸並行にならない
/// ので、こちらを使う)。AA は掛けない — 経路上を `width_px` 角の正方形で
/// 敷き詰めるだけの素朴な実装(この instrument の「正直な限界」どおり、
/// トンマナ照合が目的で描画品質は対象外)。
fn stroke_line(canvas: &mut RgbaImage, x0: f32, y0: f32, x1: f32, y1: f32, width_px: f32, color: Rgba<u8>) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let length = (dx * dx + dy * dy).sqrt();
    let half = width_px.max(1.0) / 2.0;
    if length < half {
        fill_rect(canvas, x0 - half, y0 - half, width_px.max(1.0), width_px.max(1.0), color);
        return;
    }
    let steps = length.ceil().max(1.0) as u32;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = x0 + dx * t;
        let y = y0 + dy * t;
        fill_rect(canvas, x - half, y - half, width_px.max(1.0), width_px.max(1.0), color);
    }
}

/// 閉じた多角形(頂点を順に結び、最後に先頭へ戻る)を [`stroke_line`] で描く。
/// 観測カメラのフレーム枠(4隅)専用 — 頂点数を特定の4に固定しない一般形。
fn stroke_closed_polyline(canvas: &mut RgbaImage, points: &[(f32, f32)], width_px: f32, color: Rgba<u8>) {
    if points.len() < 2 {
        return;
    }
    for i in 0..points.len() {
        let (x0, y0) = points[i];
        let (x1, y1) = points[(i + 1) % points.len()];
        stroke_line(canvas, x0, y0, x1, y1, width_px, color);
    }
}

/// 矩形1枚(x, y, w, h)。`fill_rect`/`blit_letterboxed` の引数をまとめて
/// clippy の `too_many_arguments` を素直に避ける(意味も無い8引数を並べない)。
#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

/// 矩形の4辺だけを塗る(`iced_core::Border` の実装どおり — per-edge API が無く
/// 4辺一律にしかできない、`inspector_pane.rs::bordered_row` の doc 参照)。
/// Inspector の hairline はこの形で再現する — `stroke_v` は縦線1本(playhead/
/// marker)専用なので使い回さない。
fn stroke_rect(canvas: &mut RgbaImage, rect: Rect, width_px: f32, color: Rgba<u8>) {
    let w = width_px.max(1.0);
    fill_rect(canvas, rect.x, rect.y, rect.w, w, color); // top
    fill_rect(canvas, rect.x, rect.y + rect.h - w, rect.w, w, color); // bottom
    fill_rect(canvas, rect.x, rect.y, w, rect.h, color); // left
    fill_rect(canvas, rect.x + rect.w - w, rect.y, w, rect.h, color); // right
}

/// Stage の RGBA(comp 解像度)を、letterbox を保ったまま矩形へ最近傍でブリットする
/// (D8: letterbox は neutral — 呼び出し側が先に背景を塗ってから呼ぶ)。
fn blit_letterboxed(canvas: &mut RgbaImage, src: &[u8], src_w: u32, src_h: u32, dst: Rect) {
    if src_w == 0 || src_h == 0 || dst.w <= 0.0 || dst.h <= 0.0 {
        return;
    }
    let src_aspect = src_w as f32 / src_h as f32;
    let dst_aspect = dst.w / dst.h;
    let (fit_w, fit_h) = if src_aspect > dst_aspect {
        (dst.w, dst.w / src_aspect)
    } else {
        (dst.h * src_aspect, dst.h)
    };
    let origin_x = dst.x + (dst.w - fit_w) / 2.0;
    let origin_y = dst.y + (dst.h - fit_h) / 2.0;

    for py in 0..fit_h.round().max(1.0) as u32 {
        for px in 0..fit_w.round().max(1.0) as u32 {
            let sx = ((px as f32 / fit_w) * src_w as f32) as u32;
            let sy = ((py as f32 / fit_h) * src_h as f32) as u32;
            let sx = sx.min(src_w - 1);
            let sy = sy.min(src_h - 1);
            let i = ((sy * src_w + sx) * 4) as usize;
            if i + 3 >= src.len() {
                continue;
            }
            let color = Rgba([src[i], src[i + 1], src[i + 2], src[i + 3]]);
            blend_pixel(
                canvas,
                origin_x as i64 + px as i64,
                origin_y as i64 + py as i64,
                color,
            );
        }
    }
}

const CANVAS_WIDTH: u32 = 1600;

// ---------------------------------------------------------------------------
// Inspector 領域 — `inspector_pane.rs` と同じ tokens・同じ読み口
// (`Shell::inspector_selection`)から、寸法・区切り・面色だけを再現する。
// **文字は描かない**(このファイル冒頭の「正直な限界」と同じ理由 — 新依存
// 禁止)。ident 帯・column header 行・TRANSFORM/APPEARANCE/ATTRS の
// section spacer・property 行(hairline + 3値セル)・hint 行を、矩形と
// hairline 罫線だけで積む。
// ---------------------------------------------------------------------------

/// Inspector 領域の左端 x 座標。既存の Stage/Timeline 列(`CANVAS_WIDTH` 幅、
/// 手つかず)のすぐ右に `spacing_m`(pane 間の gap、`Shell::view` の
/// `row![inspector, stage_pane].spacing(dims.spacing_m)` と同じ token)を
/// 空けて置く。
pub fn inspector_region_x(dims: Dimensions) -> f32 {
    CANVAS_WIDTH as f32 + dims.spacing_m
}

/// Inspector 領域の上端 y 座標。`Shell::view` の実レイアウトでは
/// `row![inspector, stage_pane]` が同じ高さを分け合う(inspector は Stage と
/// 同じ行に同居する)ので、この instrument でも Stage と同じ y から始める
/// (header 帯の直下)。
pub fn inspector_region_top(dims: Dimensions) -> f32 {
    dims.spacing_l + dims.panel_header_height + dims.spacing_m
}

/// ident 帯の高さ。実 widget は `column![name_field, subtitle].spacing(0.0)`
/// を `padding([spacing_s, spacing_m])` で包む(`ident_band` 参照) — 文字を
/// 描かないこの instrument では、行高の近似(`body_text` + `caption_text` +
/// 上下 `spacing_s`)で矩形の高さだけ再現する。
fn inspector_ident_height(dims: Dimensions) -> f32 {
    dims.spacing_s * 2.0 + dims.body_text + dims.caption_text
}

/// hint 行の高さ。実 widget は `padding([spacing_xs, spacing_m])` (`hint_row`
/// 参照) — 同じ近似。
fn inspector_hint_height(dims: Dimensions) -> f32 {
    dims.spacing_xs * 2.0 + dims.caption_text
}

/// Inspector 領域全体の高さ。`selection` が `None`(選択なし)なら
/// `empty_state` 相当(header 帯だけ)、`Some` なら
/// `selected_body`(`inspector_pane.rs`)と同じ行の並びを積む —
/// ident 帯 → column header 行 → TRANSFORM(4行: Position/Scale/Rotation/
/// Anchor)→ APPEARANCE(1行: Opacity)→ ATTRS(Blend 1行)→ hint 行。
/// 各 section 見出し自体は背景/border を持たない(裁定137/139)ので、
/// 高さだけ足す spacer として数える。
pub fn inspector_content_height(dims: Dimensions, selection: Option<&SelectionProjection>) -> f32 {
    let header_h = dims.inspector_section_header_height;
    let Some(selection) = selection else {
        return header_h;
    };
    let transform_rows = selection
        .transform
        .iter()
        .filter(|row| row.label != "Opacity")
        .count() as f32;
    let opacity_rows = selection
        .transform
        .iter()
        .filter(|row| row.label == "Opacity")
        .count() as f32;
    let section_h = dims.inspector_section_header_height;
    let row_h = dims.inspector_row_height;

    header_h
        + inspector_ident_height(dims)
        + row_h // column header 行
        + section_h // "TRANSFORM"
        + transform_rows * row_h
        + section_h // "APPEARANCE"
        + opacity_rows * row_h
        + section_h // "ATTRS"
        + row_h // Blend 行
        + inspector_hint_height(dims)
}

/// `shell` の実際の選択(`Shell::inspector_selection` — `render` と同じ読み口)
/// から Inspector 領域の高さを引く。テストが `render` と同じ数値を二重に
/// 書かずに済むための口。
pub fn inspector_region_height(shell: &Shell) -> f32 {
    inspector_content_height(shell.dims(), shell.inspector_selection().as_ref())
}

/// property 行1本(`inspector_pane.rs::transform_row` と同じ形):
/// `border_hairline_weak` の枠 + 3値セル(常に `surface_app` 背景 —
/// absent/animated/editable のどれでも `value_cell`/`boxed_value`/
/// `blank_value_cell` は同じ箱色を使う、実装参照)。Key 列(`reserved_glyph`)は
/// Q0 により中身も枠も無いので描かない。
fn draw_property_row(canvas: &mut RgbaImage, dims: Dimensions, colors: Colors, x: f32, y: f32, width: f32) {
    let row_h = dims.inspector_row_height;
    stroke_rect(
        canvas,
        Rect { x, y, w: width, h: row_h },
        dims.border_width,
        to_rgba(colors.border_hairline_weak, colors.border_hairline_weak.a),
    );

    let cell_w = dims.inspector_value_width;
    let cell_h = (row_h - dims.spacing_s).max(1.0);
    let cell_y = y + (row_h - cell_h) / 2.0;
    let gap = dims.spacing_xs;
    let block_w = cell_w * 3.0 + gap * 2.0 + gap + dims.inspector_glyph_width;
    let mut cx = x + width - dims.spacing_m - block_w;
    for _ in 0..3 {
        fill_rect(canvas, cx, cell_y, cell_w, cell_h, to_rgba(colors.surface_app, 1.0));
        cx += cell_w + gap;
    }
}

/// Inspector 領域全体を描く。`(x, y)` は左上、返り値は描いた高さ
/// (`inspector_content_height` と同じ値 — 呼び手が2度計算しない)。
fn draw_inspector(
    canvas: &mut RgbaImage,
    dims: Dimensions,
    colors: Colors,
    x: f32,
    y: f32,
    selection: Option<&SelectionProjection>,
) -> f32 {
    let width = dims.inspector_panel_width;
    let total_h = inspector_content_height(dims, selection);

    // pane 全体の背景(`inspector_pane::view` の外側 container と同じ
    // `surface_panel`)+ 縁(`border_default`)。
    fill_rect(canvas, x, y, width, total_h, to_rgba(colors.surface_panel, 1.0));
    stroke_rect(
        canvas,
        Rect { x, y, w: width, h: total_h },
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );

    let mut cy = y;

    // header("Inspector" タイトル帯) — 背景は panel 地のまま、縁だけ持つ。
    let header_h = dims.inspector_section_header_height;
    stroke_rect(
        canvas,
        Rect { x, y: cy, w: width, h: header_h },
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );
    cy += header_h;

    let Some(selection) = selection else {
        // `empty_state` 相当 — これ以上描く行が無い。
        return total_h;
    };

    // ident 帯 — `surface_raised` 背景 + `border_default` 縁。
    let ident_h = inspector_ident_height(dims);
    fill_rect(canvas, x, cy, width, ident_h, to_rgba(colors.surface_raised, 1.0));
    stroke_rect(
        canvas,
        Rect { x, y: cy, w: width, h: ident_h },
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );
    cy += ident_h;

    // column header 行(`.cols`)— `border_default` の不透明 hairline。
    let col_h = dims.inspector_row_height;
    stroke_rect(
        canvas,
        Rect { x, y: cy, w: width, h: col_h },
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );
    cy += col_h;

    // TRANSFORM section spacer(背景/border 無し、高さだけ)+ 行本体。
    // 行の中身(値)は読まない — この instrument は箱の寸法だけ検分する。
    cy += dims.inspector_section_header_height;
    let transform_rows = selection.transform.iter().filter(|r| r.label != "Opacity").count();
    for _ in 0..transform_rows {
        draw_property_row(canvas, dims, colors, x, cy, width);
        cy += dims.inspector_row_height;
    }

    // APPEARANCE section spacer + Opacity 行。
    cy += dims.inspector_section_header_height;
    let opacity_rows = selection.transform.iter().filter(|r| r.label == "Opacity").count();
    for _ in 0..opacity_rows {
        draw_property_row(canvas, dims, colors, x, cy, width);
        cy += dims.inspector_row_height;
    }

    // ATTRS section spacer + Blend 行(hairline のみ、値セルは無い)。
    cy += dims.inspector_section_header_height;
    let blend_h = dims.inspector_row_height;
    stroke_rect(
        canvas,
        Rect { x, y: cy, w: width, h: blend_h },
        dims.border_width,
        to_rgba(colors.border_hairline_weak, colors.border_hairline_weak.a),
    );
    cy += blend_h;

    // hint 行 — `border_default` 縁のみ。
    let hint_h = inspector_hint_height(dims);
    stroke_rect(
        canvas,
        Rect { x, y: cy, w: width, h: hint_h },
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );

    total_h
}

// ---------------------------------------------------------------------------
// Settings 領域 — `settings_pane.rs` と同じ行の並び(裁定139 の線化展開)。
// `--settings-open` フラグ(`main.rs`)が立っている間だけ描く。
// ---------------------------------------------------------------------------

/// Settings の各行高の近似。実 widget は文字を含む自然高(`preset_row`)や
/// `inspector_row_height` 固定高(background/checkerboard/ui_scale)が混在する
/// — このインストゥルメントは文字を描かないので、`preset_row`(ボタン列)も
/// 同じ `inspector_row_height` で近似する(`inspector_hint_height` と同じ
/// 「正直な限界」の割り切り)。
fn settings_content_height(dims: Dimensions, has_composition: bool) -> f32 {
    let header_h = dims.inspector_section_header_height;
    if !has_composition {
        return header_h + dims.inspector_row_height; // 「comp が無い」文言1行ぶん。
    }
    let row_h = dims.inspector_row_height;
    let hint_h = inspector_hint_height(dims);
    // background・preset(近似)・hint・ui_scale の4行(市松は裁定163で Stage
    // 下縁状態帯へ引っ越し済み — `settings_pane::view` からも撤去済み)。
    header_h + row_h + row_h + hint_h + row_h
}

/// Settings 領域全体を描く。`(x, y)` は左上、返り値は描いた高さ
/// (`settings_content_height` と同じ値)。`inspector_pane.rs::bordered_row`/
/// `settings_pane.rs::hairline_bottom` と同じ「各行を hairline で区切る」を
/// 矩形の枠(4辺一律の既知の近似、`draw_property_row` と同じ trade-off)で
/// 再現する。
fn draw_settings(
    canvas: &mut RgbaImage,
    dims: Dimensions,
    colors: Colors,
    x: f32,
    y: f32,
    width: f32,
    has_composition: bool,
) -> f32 {
    let total_h = settings_content_height(dims, has_composition);

    fill_rect(canvas, x, y, width, total_h, to_rgba(colors.surface_panel, 1.0));
    stroke_rect(
        canvas,
        Rect { x, y, w: width, h: total_h },
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );

    let header_h = dims.inspector_section_header_height;
    let mut cy = y + header_h; // "SETTINGS" 見出しは背景/border 無し(裁定137/139)、高さだけ。

    if !has_composition {
        return total_h;
    }

    let row_h = dims.inspector_row_height;
    let hint_h = inspector_hint_height(dims);
    for row_h_i in [row_h, row_h, hint_h, row_h] {
        stroke_rect(
            canvas,
            Rect { x, y: cy, w: width, h: row_h_i },
            dims.border_width,
            to_rgba(colors.border_hairline_weak, colors.border_hairline_weak.a),
        );
        cy += row_h_i;
    }

    total_h
}

// ---------------------------------------------------------------------------
// Browser 領域(裁定162 切片 B3) — `motolii_browser_pane::view` と同じ構造
// (rail + カード grid)を矩形近似で再現する。`--browser-open` フラグ
// (`main.rs`)が立っている間だけ描く。**文字は描かない**(このファイル冒頭の
// 「正直な限界」と同じ理由)。
//
// **既知の陳腐化(2026-08-22 shell pane_grid 化レーン)**: 実 `Shell::view`
// では Browser は既に「header 直下の全幅の帯」ではなく「Inspector/Stage/
// Timeline の左に並ぶ全高カラム」(`pane_layout.rs` 冒頭 doc、
// `pane_layout::build_configuration`)。この instrument の下の
// `render()`/`draw_browser` 呼び出しは**まだ旧位置(帯)のまま**——
// `padding`(この関数内で「canvas 全体の外側余白」と「Stage/Timeline 列の
// 左端 x」の**両方**を兼ねる変数、`render()` 冒頭〜末尾で150行超に渡って
// 再利用)を Browser 開閉に応じて動かすのは、markers/ルーラー目盛/
// レーンバー等の下流計算すべてに波及する非自明な変更で、この器具の
// 「トンマナ照合」という狭いスコープに対して不釣り合いに大きい・かつ
// この pane_grid 化レーン自身の red→green オラクル(`pane_layout.rs` の
// 純関数試験、`tests/suite/q0_fence.rs` 等)の対象外(この instrument に
// 対する自動 fence は無い——`tests/suite/fixture.rs` の geometry 試験は
// Browser 閉状態しか検分していない)。**意図的に見送った**——直すなら
// 専用のレーンで `padding` を「outer margin」と「content column left x」に
// 分離してから着手すること(このコメントが唯一の記録)。
// ---------------------------------------------------------------------------

/// カード grid のカード1行ぶんの高さの近似(`motolii_browser_pane::card_view`
/// と同じ比: サムネ = `CARD_WIDTH_ROW_HEIGHT_RATIO × row_height` を
/// `THUMB_ASPECT_W`:`THUMB_ASPECT_H` で割った高さ + 名前/caption 2行ぶんの
/// `micro_text` + 内側 padding)。値そのものは複製しない(`motolii_browser_pane`
/// の `pub const` 3本を読むだけ)。
fn browser_card_row_height(dims: Dimensions) -> f32 {
    let card_width = dims.row_height * motolii_browser_pane::CARD_WIDTH_ROW_HEIGHT_RATIO;
    let thumb_h =
        card_width * motolii_browser_pane::THUMB_ASPECT_H / motolii_browser_pane::THUMB_ASPECT_W;
    thumb_h + dims.spacing_xs * 3.0 + dims.micro_text * 2.0
}

/// Browser 領域全体を描く。`area`(`Rect`、`draw_settings` と違い引数を
/// 1つの struct へ畳んで clippy `too_many_arguments` を避ける — `Rect` 自身の
/// doc「意味も無い8引数を並べない」と同じ理由)の `h` は
/// `motolii_browser_pane::PANEL_HEIGHT_ROW_HEIGHT_RATIO × row_height`
/// (呼び出し元と同じ値、`render` 参照)。**サムネ矩形は実データ由来** —
/// `item_count` 件ぶん(上限あり、パネルの高さを超えて描いても見えないので
/// 意味が無い)を rail の右側(カード grid 領域)へ実際に並べる。列数・幅比は
/// `motolii_browser_pane::{GRID_COLUMNS,CARD_WIDTH_ROW_HEIGHT_RATIO}` と揃える。
fn draw_browser(canvas: &mut RgbaImage, dims: Dimensions, colors: Colors, area: Rect, item_count: usize) {
    fill_rect(canvas, area.x, area.y, area.w, area.h, to_rgba(colors.surface_panel, 1.0));
    stroke_rect(canvas, area, dims.border_width, to_rgba(colors.border_default, 1.0));

    // rail(mock `.librarySidebar` 相当、`browser_pane::rail_view` の
    // `FillPortion(1)` を width 比で近似)。
    let rail_w = area.w / 5.0;
    fill_rect(canvas, area.x, area.y, rail_w, area.h, to_rgba(colors.surface_raised, 0.4));
    stroke_v(canvas, area.x + rail_w, area.y, area.y + area.h, dims.border_width, to_rgba(colors.border_default, 1.0));

    // カード grid(rail の右、`catalog_view` の `FillPortion(4)` 領域)。
    let catalog_x = area.x + rail_w + dims.spacing_xs;
    let catalog_w = (area.w - rail_w - dims.spacing_xs * 2.0).max(0.0);
    let card_row_h = browser_card_row_height(dims);
    let card_w = ((catalog_w - dims.spacing_s) / motolii_browser_pane::GRID_COLUMNS as f32).max(0.0);
    let mut cy = area.y + dims.spacing_m;
    let mut column = 0usize;
    for _ in 0..item_count {
        if cy + card_row_h > area.y + area.h {
            break; // パネルの高さを超えたら描かない(実 widget もこの下は scroll で隠れる)。
        }
        let cx = catalog_x + column as f32 * (card_w + dims.spacing_s);
        fill_rect(canvas, cx, cy, card_w, card_row_h, to_rgba(colors.surface_raised, 1.0));
        stroke_rect(canvas, Rect { x: cx, y: cy, w: card_w, h: card_row_h }, dims.border_width, to_rgba(colors.border_default, 1.0));
        column += 1;
        if column == motolii_browser_pane::GRID_COLUMNS {
            column = 0;
            cy += card_row_h + dims.spacing_s;
        }
    }
}

/// `shell.view()` の並び(header/[settings]/stage/timeline/transport/status、
/// `spacing_m` の間隔・`spacing_l` の全体 padding)を、Tokens の実値でそのまま
/// 再現する。
/// 裁定171 v2(M4)で `&mut Shell` になった — GPU 高速路の間 `frame_rgba()`
/// (下記)が `&mut self` になったため(`Shell::ensure_rgba_fresh` doc 参照)。
/// この instrument は元々 CLI 一発ツール専用(`main.rs` の `--screenshot`)なので
/// 可変参照でも支障はない。
pub fn render(shell: &mut Shell) -> RgbaImage {
    // `ui_scale` 適用済み(`Shell::dims` — 適用点1箇所)。この instrument も
    // 生の `tokens.dims` を直接読まない。
    let dims = shell.dims();
    let colors = shell.tokens().colors;
    let rows = shell.timeline_rows();
    // property 行(キー行、第2波 T3・裁定148/151) — `timeline_pane::TimelinePane`
    // と同じ投影(`Shell::timeline_property_rows`)。選択 layer の下に挿入する
    // ぶんだけ、その後ろの層行を押し下げる(`timeline_pane::layer_row_top`、
    // `timeline/canvas.rs::draw` と同じ計算)。
    let property_rows = shell.timeline_property_rows();
    let selected_row_index = timeline_pane::selected_row_index(&rows, shell.session());
    let markers = shell.markers();
    // 裁定171 v2(M4): `frame_rgba()` が `&mut self` になったので(`Shell::
    // ensure_rgba_fresh` doc 参照)、`&Session` を長く借りたままにできない —
    // 使う値(`playhead`)だけ先に所有値として抜き出す(下の `stage_source`
    // 計算より前)。
    let playhead = shell.session().playhead;
    let composition = shell.composition();
    let duration_frames = composition.as_ref().map(|c| c.duration_frames).unwrap_or(0);
    let fps = composition.as_ref().map(|c| c.fps);
    let inspector_selection = shell.inspector_selection();

    let padding = dims.spacing_l;
    let gap = dims.spacing_m;
    let content_width = CANVAS_WIDTH as f32 - padding * 2.0;

    let header_h = dims.panel_header_height;
    let stage_aspect = composition
        .as_ref()
        .map(|c| c.height as f32 / c.width.max(1) as f32)
        .unwrap_or(9.0 / 16.0);
    let stage_h = (content_width * stage_aspect).clamp(dims.row_height * 4.0, 700.0);
    // Stage 下縁状態帯(裁定163 S 空間スコア)。`status_h`(下記)と同じ
    // 「専用 token が無いので `row_height` を流用する」近似 — `lib.rs::
    // stage_pane`/`stage::state_band_view` の実 widget は明示 `.height()` を
    // 持たない自然高なので、この instrument 側も既存の帯(status)と同じ
    // 近似で位置だけ再現する(モジュール冒頭 doc「正直な限界」どおり)。
    let stage_band_h = dims.row_height;
    let stage_picture_h = (stage_h - stage_band_h).max(dims.row_height);
    // ルーラー帯 = row_height(timeline_pane::TimelinePane::ruler_height と同じ流用)。
    // + property 行(キー行)ぶん(`TimelinePane::content_height` と同じ式)。
    let timeline_h = dims.row_height
        + dims.row_height * rows.len() as f32
        + dims.timeline_param_row_height * property_rows.len() as f32;
    let transport_h = dims.transport_band;
    let status_h = dims.row_height;

    // Settings パネル(`--settings-open`)。開いている間だけ header の下へ差し込む
    // — 実レイアウト(`Shell::view` の `column![header, [settings], row!, ...]`)
    // と同じく、開いた分だけ gap が1本増える(header-settings 間も spacing_m)。
    let settings_open = shell.settings_panel_open();
    let settings_h = if settings_open {
        settings_content_height(dims, composition.is_some())
    } else {
        0.0
    };
    let settings_gap_count = if settings_open { 1 } else { 0 };

    // Browser パネル(`--browser-open`、裁定162 切片 B3)。Settings と同じ
    // 「開いている間だけ差し込む」分岐 — `Shell::view` では Settings の**下**
    // (header 直下ではなく)に積む(`lib.rs::Shell::view` の押し順どおり)。
    let browser_open = shell.browser_panel_open();
    let browser_h = if browser_open {
        dims.row_height * motolii_browser_pane::PANEL_HEIGHT_ROW_HEIGHT_RATIO
    } else {
        0.0
    };
    let browser_gap_count = if browser_open { 1 } else { 0 };

    let total_h = padding * 2.0
        + header_h
        + settings_h
        + browser_h
        + stage_h
        + timeline_h
        + transport_h
        + status_h
        + gap * (4.0 + settings_gap_count as f32 + browser_gap_count as f32);

    // Inspector 領域(Stage/Timeline 列の右) — 幅・高さの両方で canvas を広げる。
    // `Shell::view` の実レイアウトでは Inspector は Stage と同じ行に同居する
    // (`row![inspector, stage_pane]`)が、この instrument は元々 Stage/Timeline を
    // 1列の手組み合成で描いており、その列を左に残したまま右へ新しい列を足す
    // (発注書 EXACT TARGET 1「右側に」)。[`inspector_region_top`] は「Settings/
    // Browser 閉」を前提にした固定式(既存 fixture テストの契約)なので、開いて
    // いる分はここで自前に足す。
    let inspector_x = inspector_region_x(dims);
    let inspector_top = inspector_region_top(dims)
        + settings_h
        + gap * settings_gap_count as f32
        + browser_h
        + gap * browser_gap_count as f32;
    let inspector_h = inspector_content_height(dims, inspector_selection.as_ref());
    let canvas_width = (inspector_x + dims.inspector_panel_width).round().max(CANVAS_WIDTH as f32) as u32;
    let canvas_height = total_h.max(inspector_top + inspector_h + padding);

    let mut canvas = RgbaImage::from_pixel(
        canvas_width,
        canvas_height.round().max(1.0) as u32,
        to_rgba(colors.surface_app, 1.0),
    );

    let mut y = padding;

    // header — panel header 帯。中身のボタン矩形3枚は色面だけ(文字は描かない)。
    // `lib.rs::Shell::header` と同じ地(`surface_panel`)+ hairline 縁(裁定139 の
    // shell chrome への展開)。
    fill_rect(
        &mut canvas,
        padding,
        y,
        content_width,
        header_h,
        to_rgba(colors.surface_panel, 1.0),
    );
    stroke_rect(
        &mut canvas,
        Rect { x: padding, y, w: content_width, h: header_h },
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );
    let button_w = 72.0_f32.min((content_width - gap * 2.0) / 3.0);
    let mut bx = padding + dims.spacing_s;
    for _ in 0..3 {
        fill_rect(
            &mut canvas,
            bx,
            y + dims.spacing_xs,
            button_w,
            header_h - dims.spacing_xs * 2.0,
            to_rgba(colors.surface_raised, 1.0),
        );
        bx += button_w + dims.spacing_s;
    }
    y += header_h + gap;

    // settings — 開いている時だけ(裁定139 の Settings 線化)。
    if settings_open {
        draw_settings(&mut canvas, dims, colors, padding, y, content_width, composition.is_some());
        y += settings_h + gap;
    }

    // browser — 開いている時だけ(裁定162 切片 B3)。`Shell::view` の押し順
    // どおり Settings の下・main row の上に積む。
    if browser_open {
        let item_count = shell.assets().len();
        let area = Rect { x: padding, y, w: content_width, h: browser_h };
        draw_browser(&mut canvas, dims, colors, area, item_count);
        y += browser_h + gap;
    }

    // stage — neutral letterbox(D8)+ 実際に GPU 合成された RGBA。
    fill_rect(
        &mut canvas,
        padding,
        y,
        content_width,
        stage_h,
        to_rgba(colors.surface_app, 1.0),
    );
    // `frame_rgba()` は常に背景込みの export 真値(市松を絶対に乗せない、
    // `settings_pane` doc 参照)。この instrument は「実際に画面へ出る絵」の
    // 検分が目的なので、市松 ON の間は
    // `lib.rs::build_stage_handle`/`refresh_frame` と同じ入力(裁定141の
    // 「背景を敷かない」合成、`Shell::checkerboard_preview_rgba`)へ
    // `composite_checkerboard` を当てる — `frame_rgba()` 自体は一切書き換えない
    // (`tests/settings_drive.rs::checkerboard_toggle_never_touches_the_raw_export_rgba`
    // の不変を screenshot 側からも壊さない)。**観測カメラ(裁定157)が最優先** —
    // 有効な間は `Shell::observation_rgba()`(`Engine::render_frame_with_view_camera`
    // の結果)を貼る。`frame_rgba()` 自体は観測中でも一切書き換えない(`lib.rs::
    // refresh_frame` の不変と同じ)。
    // 裁定171 v2(M4): `frame_rgba()`/`observation_rgba()`/
    // `checkerboard_preview_rgba()` はどれも `&self.frame` を借りた
    // `&[u8]` を返すので、`&mut self` になった `frame_rgba()`(`Shell::
    // ensure_rgba_fresh` doc 参照)とチェーン(`.or_else(|| ...)`)すると
    // 借用が衝突する——各枝で即座に `to_vec()` して所有値へ変換してから
    // 分岐する(意味は無改造、借用の形だけ変えた)。
    let observation_active = shell.observation().is_some();
    let checkerboard_active = shell.checkerboard_enabled();
    let stage_source: Option<(u32, u32, Vec<u8>)> = if observation_active {
        match shell.observation_rgba() {
            Some((w, h, pixels)) => Some((w, h, pixels.to_vec())),
            None => shell.frame_rgba().map(|(w, h, pixels)| (w, h, pixels.to_vec())),
        }
    } else if checkerboard_active {
        match shell.checkerboard_preview_rgba() {
            Some((w, h, pixels)) => Some((w, h, pixels.to_vec())),
            None => shell.frame_rgba().map(|(w, h, pixels)| (w, h, pixels.to_vec())),
        }
    } else {
        shell.frame_rgba().map(|(w, h, pixels)| (w, h, pixels.to_vec()))
    };
    // 絵(ヒーロー、S5a)は帯の高さぶんだけ縮める — 実 widget の
    // `container(body).height(Length::Fill)` が状態帯と同じ column を分け合う
    // のと同じ配分(`lib.rs::stage_pane` 参照)。
    let stage_rect = Rect {
        x: padding,
        y,
        w: content_width,
        h: stage_picture_h,
    };
    if let Some((w, h, pixels)) = stage_source {
        if !observation_active && checkerboard_active {
            let mut composited = pixels;
            settings_pane::composite_checkerboard(w, h, &mut composited, colors);
            blit_letterboxed(&mut canvas, &composited, w, h, stage_rect);
        } else {
            blit_letterboxed(&mut canvas, &pixels, w, h, stage_rect);
        }
    }
    // フレーム枠 overlay(裁定157・S3)。観測中(`Some`)だけ、レンダリングカメラの
    // 写る範囲を hairline で重ねる — `lib.rs::stage_pane`(実 canvas 描画)と
    // **同じ計算**(`stage::StageOverlay::frame_corners_on_screen`)を呼ぶ。
    if let Some(overlay) = shell.stage_overlay() {
        let bounds = iced::Rectangle {
            x: stage_rect.x,
            y: stage_rect.y,
            width: stage_rect.w,
            height: stage_rect.h,
        };
        if let Some(corners) = overlay.frame_corners_on_screen(bounds) {
            let points: Vec<(f32, f32)> = corners.iter().map(|p| (p.x, p.y)).collect();
            stroke_closed_polyline(
                &mut canvas,
                &points,
                dims.border_width * 1.5,
                to_rgba(colors.action_active, 1.0),
            );
        }
    }
    // 状態帯そのもの(発注書 EXACT TARGET・ORACLE (d))。文字(倍率・市松の
    // on/off・視点状態)は描かない(モジュール冒頭 doc の正直な限界どおり)——
    // `lib.rs::status_band`/実 widget の `state_band_item` と同じ「border の
    // みで背景は塗らない」grammar(S4 text 段・ink 弱)を位置だけ再現する。
    // **S5b の実測点**: この帯は `action_active`(ヒーロー上の選択 ACCENT、
    // 直前の frame 枠 overlay や後述の playhead に使われる色)を一切使わない
    // — 弱い hairline(`border_hairline_weak`)だけなので、この PNG 上で帯の
    // 最大 ink はヒーロー領域の ACCENT より必ず弱い。
    stroke_rect(
        &mut canvas,
        Rect { x: padding, y: y + stage_picture_h, w: content_width, h: stage_band_h },
        dims.border_width,
        to_rgba(colors.border_hairline_weak, colors.border_hairline_weak.a),
    );
    y += stage_h + gap;

    // timeline — timeline_pane::draw と同じ位置関係(frame_to_x を共有)。
    // レーンバー(行ヘッダ列、裁定147): クリップ面は `rail_width` ぶん右へ
    // ずれる。`frame_to_x` 自体には触れず、呼び出し側(このブロック)で
    // `padding + rail_width` を足す — `timeline/canvas.rs::draw` と同じ約束。
    let timeline_top = y;
    let rail_width = dims.timeline_lane_bar_width;
    let clip_width = (content_width - rail_width).max(0.0);
    let clip_x0 = padding + rail_width;
    fill_rect(
        &mut canvas,
        padding,
        y,
        content_width,
        timeline_h,
        to_rgba(colors.surface_panel, 1.0),
    );
    let ruler_h = dims.row_height;
    // ルーラー帯の地は rail を除いたクリップ面だけ(rail の corner は
    // `surface_panel` の地のまま — `timeline/canvas.rs::draw` と同じ)。
    fill_rect(
        &mut canvas,
        clip_x0,
        y,
        clip_width,
        ruler_h,
        to_rgba(colors.surface_raised, 1.0),
    );
    // ルーラー帯とクリップ面の境界(`timeline/canvas.rs::draw` と同じ
    // hairline — 面色の塗り分け=`surface_raised` だけに頼らない)。**全幅**
    // (rail 込み)で引く — レーンバーの corner とクリップ面のルーラーは同じ
    // 横の区切りを共有する。
    stroke_rect(
        &mut canvas,
        Rect { x: padding, y, w: content_width, h: ruler_h },
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );

    // ルーラーの小目盛/大目盛(利用者裁定 2026-08-21 夜、`timeline_pane::
    // tick_steps` — `timeline/canvas.rs::draw_ruler_ticks` と同じ刻み・追随のみ)。
    // 文字は描かない(モジュール冒頭 doc の「正直な限界」どおり — フレーム番号
    // ラベルはフォントラスタライズが要るため見送り)が、線の長さ/強さの差
    // (小目盛=短く弱い hairline、大目盛=長く強い)だけは既存の描画手段
    // (`stroke_v`)で再現できるので、階層そのものはこの PNG でも視認できる。
    if duration_frames > 0 && clip_width > 0.0 {
        let (minor, major) =
            timeline_pane::tick_steps(fps, duration_frames, clip_width, dims.row_height);
        let last_frame = (duration_frames - 1).max(0);
        let mut frame_no: i64 = 0;
        while frame_no <= last_frame {
            let is_major = frame_no % major == 0;
            let x = clip_x0 + timeline_pane::frame_to_x(frame_no, clip_width, duration_frames);
            // `timeline/canvas.rs::draw_ruler_ticks` と同じ「ルーラー下端から
            // spacing 分だけ上へ」— 大目盛は spacing_m(長い)、小目盛は
            // spacing_s(短い)。
            let offset = if is_major { dims.spacing_m } else { dims.spacing_s };
            let top = (y + ruler_h - offset).max(y);
            let color = if is_major {
                to_rgba(colors.border_strong, 1.0)
            } else {
                to_rgba(colors.border_hairline_weak, colors.border_hairline_weak.a)
            };
            stroke_v(&mut canvas, x, top, y + ruler_h, dims.border_width, color);
            frame_no += minor;
        }
    }

    // 明暗のリズム(裁定148・§1.6): 行方向ゼブラ → 時間方向の順に「地」を
    // 重ねる。`timeline_pane.rs::TimelinePane::draw` と同じ token・同じ順序
    // (区切りの手段ではない — 区切りは下の行 hairline が担う)。
    let rows_top = timeline_top + ruler_h;
    let rows_bottom = timeline_top + timeline_h;
    for index in 0..rows.len() {
        if index % 2 == 0 {
            continue;
        }
        let row_top = rows_top
            + timeline_pane::layer_row_top(
                dims.row_height,
                dims.timeline_param_row_height,
                property_rows.len(),
                selected_row_index,
                index,
            );
        fill_rect(
            &mut canvas,
            padding,
            row_top,
            content_width,
            dims.row_height,
            to_rgba(colors.timeline_row_zebra, colors.timeline_row_zebra.a),
        );
    }
    if duration_frames > 0 && clip_width > 0.0 && rows_bottom > rows_top {
        let segment_frames =
            timeline_pane::time_band_segment_frames(fps, duration_frames, clip_width, dims.row_height);
        let mut segment_index: i64 = 0;
        let mut start_frame: i64 = 0;
        while start_frame < duration_frames {
            let end_frame = (start_frame + segment_frames).min(duration_frames);
            if segment_index % 2 == 1 {
                let x0 = clip_x0 + timeline_pane::frame_to_x(start_frame, clip_width, duration_frames);
                let x1 = (clip_x0
                    + timeline_pane::frame_to_x(end_frame, clip_width, duration_frames))
                .max(x0 + 1.0);
                fill_rect(
                    &mut canvas,
                    x0,
                    rows_top,
                    x1 - x0,
                    rows_bottom - rows_top,
                    to_rgba(colors.timeline_time_band, colors.timeline_time_band.a),
                );
            }
            start_frame = end_frame;
            segment_index += 1;
        }
    }

    // 時間方向の縦線 — 全目盛の投影(利用者裁定 2026-08-21 夜・σ EXACT
    // TARGET 3、`timeline/canvas.rs::draw_tick_lines` と同じ刻み・追随のみ)。
    // 帯(面・粗いリズム)とは別の周波数(線・全目盛の細かいリズム) — 帯の
    // 直後・bar 描画(下の層の行ループ)の前に置く(帯→縦線→bar の描画順)。
    // mock は f=0(rail 境界と重なる位置)を引かない — `frame_no = minor` から
    // 開始してその踏襲(`timeline/canvas.rs::draw_tick_lines` と同じ)。
    if duration_frames > 0 && clip_width > 0.0 && rows_bottom > rows_top {
        let (minor, major) =
            timeline_pane::tick_steps(fps, duration_frames, clip_width, dims.row_height);
        let last_frame = (duration_frames - 1).max(0);
        let mut frame_no = minor;
        while frame_no <= last_frame {
            let is_major = frame_no % major == 0;
            let x = clip_x0 + timeline_pane::frame_to_x(frame_no, clip_width, duration_frames);
            let color = if is_major {
                to_rgba(colors.timeline_grid_major, colors.timeline_grid_major.a)
            } else {
                to_rgba(colors.timeline_grid_minor, colors.timeline_grid_minor.a)
            };
            stroke_v(&mut canvas, x, rows_top, rows_bottom, dims.border_width, color);
            frame_no += minor;
        }
    }

    for marker in &markers {
        let Some(fps) = fps else { continue };
        let Ok(frame_no) = marker.time.try_to_frame_floor(fps) else {
            continue;
        };
        let x = clip_x0 + timeline_pane::frame_to_x(frame_no, clip_width, duration_frames);
        stroke_v(
            &mut canvas,
            x,
            timeline_top,
            timeline_top + ruler_h,
            dims.border_width * 2.0,
            to_rgba(colors.way_timeline, 1.0),
        );
    }

    for (index, row) in rows.iter().enumerate() {
        let row_top = timeline_top
            + ruler_h
            + timeline_pane::layer_row_top(
                dims.row_height,
                dims.timeline_param_row_height,
                property_rows.len(),
                selected_row_index,
                index,
            );
        if row.selected {
            // 全幅(rail 込み)— `timeline/canvas.rs::draw` と同じ理由
            // (選択は行そのものの状態、クリップ面だけの状態ではない)。
            fill_rect(
                &mut canvas,
                padding,
                row_top,
                content_width,
                dims.row_height,
                to_rgba(colors.state_selected, 1.0),
            );
        }
        let start_x =
            clip_x0 + timeline_pane::frame_to_x(row.start, clip_width, duration_frames);
        let end_x = (clip_x0
            + timeline_pane::frame_to_x(row.start + row.duration, clip_width, duration_frames))
        .max(start_x + 1.0);
        // `timeline/canvas.rs::draw` の bar_color と同じ優先順位(この instrument は
        // その再現なので追随する) — hidden が既定色より優先、label_color は
        // Some の時だけ既定(way_timeline)の代わりに使う。dragging はこの静止画
        // instrument には無い(row.dragging は常に false、`Shell::timeline_rows`
        // が組む rows がドラッグ中の overlay を持たない)。
        let bar_color = if row.hidden {
            colors.text_muted
        } else if let Some(color) = row
            .label_color
            .and_then(|index| colors.label_palette.get(index as usize))
        {
            *color
        } else {
            colors.way_timeline
        };
        fill_rect(
            &mut canvas,
            start_x,
            row_top + dims.spacing_xs,
            (end_x - start_x).max(1.0),
            (dims.row_height - dims.spacing_s).max(1.0),
            to_rgba(bar_color, 1.0),
        );

        // 行の区切り(裁定139: 面色の塗り分け=ゼブラの明暗だけに頼らず hairline
        // を足す — `TimelinePane::draw` の行末 hairline と同じ)。全幅(rail 込み)。
        stroke_h(
            &mut canvas,
            padding,
            padding + content_width,
            row_top + dims.row_height,
            dims.border_width,
            to_rgba(colors.border_hairline_weak, colors.border_hairline_weak.a),
        );

        // レーンバー(行ヘッダ列、裁定147): スウォッチ + M/S/L の枠
        // (`timeline/lane_bar.rs::draw` と同じ幾何 — この instrument は文字を
        // 描かないので枠と色面だけ再現する)。
        let swatch_size = dims.spacing_m;
        let swatch_y = row_top + (dims.row_height - swatch_size) / 2.0;
        fill_rect(
            &mut canvas,
            padding + dims.spacing_s,
            swatch_y,
            swatch_size,
            swatch_size,
            to_rgba(colors.way_timeline, 1.0),
        );

        let glyph_w = dims.inspector_glyph_width;
        let glyph_h = (dims.row_height - dims.spacing_xs).max(1.0);
        let glyph_y = row_top + (dims.row_height - glyph_h) / 2.0;
        let glyph_gap = dims.spacing_xs;
        let block_w = glyph_w * 3.0 + glyph_gap * 2.0;
        let mut glyph_x = padding + rail_width - dims.spacing_s - block_w;
        for active in [row.hidden, row.solo, row.locked] {
            let border_color = if active { colors.action_active } else { colors.border_default };
            stroke_rect(
                &mut canvas,
                Rect { x: glyph_x, y: glyph_y, w: glyph_w, h: glyph_h },
                dims.border_width,
                to_rgba(border_color, 1.0),
            );
            glyph_x += glyph_w + glyph_gap;
        }
    }

    // property 行(キー行、第2波 T3・裁定148/151) — `timeline/key_rows.rs::draw`
    // と同じ位置関係(選択 layer の行のすぐ下)。**文字は描かない**(この
    // instrument の「正直な限界」どおり、property 名のラベルは省く)。キーは
    // 菱形の代わりに正方形マーク(`dims.spacing_m` 角、canon の描画寸法8pxと
    // 同値)で近似する — この instrument はトンマナ(色・位置)の照合が目的で、
    // 形状の忠実さは対象外(モジュール doc 冒頭)。
    if let Some(selected) = selected_row_index {
        if !property_rows.is_empty() {
            let param_h = dims.timeline_param_row_height;
            let param_top = timeline_top + ruler_h + dims.row_height * (selected as f32 + 1.0);
            for (index, row) in property_rows.iter().enumerate() {
                let row_top = param_top + param_h * index as f32;
                if index % 2 == 1 {
                    fill_rect(
                        &mut canvas,
                        padding,
                        row_top,
                        content_width,
                        param_h,
                        to_rgba(colors.timeline_row_zebra, colors.timeline_row_zebra.a),
                    );
                }
                stroke_h(
                    &mut canvas,
                    padding,
                    padding + content_width,
                    row_top + param_h,
                    dims.border_width,
                    to_rgba(colors.border_hairline_weak, colors.border_hairline_weak.a),
                );
                let mark = dims.spacing_m;
                let half_mark = mark / 2.0;
                let mark_y = row_top + (param_h - mark) / 2.0;
                for key in &row.keys {
                    let cx = clip_x0
                        + timeline_pane::frame_to_x(key.frame, clip_width, duration_frames);
                    let mark_x = cx - half_mark;
                    let mark_color = if key.selected {
                        colors.action_active
                    } else {
                        colors.way_timeline
                    };
                    fill_rect(&mut canvas, mark_x, mark_y, mark, mark, to_rgba(mark_color, 1.0));
                }
            }
        }
    }

    // rail とクリップ面の境界(強い hairline、`timeline/lane_bar.rs::draw` と
    // 同じ)。行の縦積みぶん全体を通す。
    stroke_v(
        &mut canvas,
        clip_x0,
        timeline_top,
        rows_bottom,
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );

    let playhead_x = clip_x0 + timeline_pane::frame_to_x(playhead, clip_width, duration_frames);
    stroke_v(
        &mut canvas,
        playhead_x,
        timeline_top,
        timeline_top + timeline_h,
        dims.border_width * 1.5,
        to_rgba(colors.action_active, 1.0),
    );
    y += timeline_h + gap;

    // transport — 帯色のみ(timecode は文字なので描かない)。
    fill_rect(
        &mut canvas,
        padding,
        y,
        content_width,
        transport_h,
        to_rgba(colors.surface_panel, 1.0),
    );
    y += transport_h + gap;

    // status — 拒否・警告があれば警告色の細い縁取りだけ付ける(色トンマナの照合用)。
    // `lib.rs::status_band` と同じ hairline 縁(裁定139 の shell chrome への展開 —
    // 背景は塗らない、`inspector_pane.rs::hint_row` と同じ「border のみ」grammar)。
    let status_color = if shell.status().is_some() {
        colors.status_warning
    } else {
        colors.text_muted
    };
    stroke_rect(
        &mut canvas,
        Rect { x: padding, y, w: content_width, h: status_h },
        dims.border_width,
        to_rgba(colors.border_default, 1.0),
    );
    fill_rect(
        &mut canvas,
        padding,
        y,
        dims.spacing_xs,
        status_h,
        to_rgba(status_color, 1.0),
    );

    draw_inspector(
        &mut canvas,
        dims,
        colors,
        inspector_x,
        inspector_top,
        inspector_selection.as_ref(),
    );

    canvas
}

/// 1フレーム描いて PNG を書き、終了する口。`main.rs` の `--fixture --screenshot`
/// から呼ばれる。
pub fn write_png(shell: &mut Shell, path: &std::path::Path) -> Result<(), String> {
    let canvas = render(shell);
    let (width, height) = (canvas.width(), canvas.height());
    image::save_buffer(
        path,
        canvas.as_raw(),
        width,
        height,
        image::ColorType::Rgba8,
    )
    .map_err(|error| error.to_string())
}
