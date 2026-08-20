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
//! Stage は `motolii_engine::Engine` が実際に GPU 合成した RGBA をそのまま貼り、
//! Timeline は `timeline_pane` と**同じ投影関数**(`rows`/`frame_to_x`)を使って
//! 同じ位置関係を再現するが、**ここで実際に塗るのはこのモジュール自身**
//! (iced の `canvas::Frame` ではなく `image::RgbaImage` へ矩形・線を直接塗る —
//! iced の canvas 描画パスは wgpu レンダラを要るため headless では使えない)。
//! header/transport/status 帯は色面だけの帯として再現する。
//!
//! **正直な限界**: 文字(層名・timecode・status 文言)は描かない。フォント
//! ラスタライズには新しい依存(ab_glyph 等)が要り、発注書の「新依存禁止」に
//! 触れるため見送った。この器具は「トンマナ(色・位置・密度)の照合」が目的で、
//! 文字の可読性検分はこの器具の対象外(実窓のスクリーンショットで別途見る)。

use image::{Rgba, RgbaImage};

use crate::{timeline_pane, Shell};

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

/// 矩形1枚(x, y, w, h)。`fill_rect`/`blit_letterboxed` の引数をまとめて
/// clippy の `too_many_arguments` を素直に避ける(意味も無い8引数を並べない)。
#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
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

/// `shell.view()` の並び(header/stage/timeline/transport/status、`spacing_m` の
/// 間隔・`spacing_l` の全体 padding)を、Tokens の実値でそのまま再現する。
pub fn render(shell: &Shell) -> RgbaImage {
    let dims = shell.tokens().dims;
    let colors = shell.tokens().colors;
    let rows = shell.timeline_rows();
    let markers = shell.markers();
    let session = shell.session();
    let composition = shell.composition();
    let duration_frames = composition.as_ref().map(|c| c.duration_frames).unwrap_or(0);
    let fps = composition.as_ref().map(|c| c.fps);

    let padding = dims.spacing_l;
    let gap = dims.spacing_m;
    let content_width = CANVAS_WIDTH as f32 - padding * 2.0;

    let header_h = dims.panel_header_height;
    let stage_aspect = composition
        .as_ref()
        .map(|c| c.height as f32 / c.width.max(1) as f32)
        .unwrap_or(9.0 / 16.0);
    let stage_h = (content_width * stage_aspect).clamp(dims.row_height * 4.0, 700.0);
    // ルーラー帯 = row_height(timeline_pane::TimelinePane::ruler_height と同じ流用)。
    let timeline_h = dims.row_height + dims.row_height * rows.len() as f32;
    let transport_h = dims.transport_band;
    let status_h = dims.row_height;

    let total_h =
        padding * 2.0 + header_h + stage_h + timeline_h + transport_h + status_h + gap * 4.0;

    let mut canvas = RgbaImage::from_pixel(
        CANVAS_WIDTH,
        total_h.round().max(1.0) as u32,
        to_rgba(colors.surface_app, 1.0),
    );

    let mut y = padding;

    // header — panel header 帯。中身のボタン矩形3枚は色面だけ(文字は描かない)。
    fill_rect(
        &mut canvas,
        padding,
        y,
        content_width,
        header_h,
        to_rgba(colors.surface_panel, 1.0),
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

    // stage — neutral letterbox(D8)+ 実際に GPU 合成された RGBA。
    fill_rect(
        &mut canvas,
        padding,
        y,
        content_width,
        stage_h,
        to_rgba(colors.surface_app, 1.0),
    );
    if let Some((w, h, pixels)) = shell.frame_rgba() {
        blit_letterboxed(
            &mut canvas,
            pixels,
            w,
            h,
            Rect {
                x: padding,
                y,
                w: content_width,
                h: stage_h,
            },
        );
    }
    y += stage_h + gap;

    // timeline — timeline_pane::draw と同じ位置関係(frame_to_x を共有)。
    let timeline_top = y;
    fill_rect(
        &mut canvas,
        padding,
        y,
        content_width,
        timeline_h,
        to_rgba(colors.surface_panel, 1.0),
    );
    let ruler_h = dims.row_height;
    fill_rect(
        &mut canvas,
        padding,
        y,
        content_width,
        ruler_h,
        to_rgba(colors.surface_raised, 1.0),
    );

    for marker in &markers {
        let Some(fps) = fps else { continue };
        let Ok(frame_no) = marker.time.try_to_frame_floor(fps) else {
            continue;
        };
        let x = padding + timeline_pane::frame_to_x(frame_no, content_width, duration_frames);
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
        let row_top = timeline_top + ruler_h + dims.row_height * index as f32;
        if row.selected {
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
            padding + timeline_pane::frame_to_x(row.start, content_width, duration_frames);
        let end_x = (padding
            + timeline_pane::frame_to_x(row.start + row.duration, content_width, duration_frames))
        .max(start_x + 1.0);
        let bar_color = if row.hidden {
            colors.text_muted
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
    }

    let playhead_x =
        padding + timeline_pane::frame_to_x(session.playhead, content_width, duration_frames);
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
    let status_color = if shell.status().is_some() {
        colors.status_warning
    } else {
        colors.text_muted
    };
    fill_rect(
        &mut canvas,
        padding,
        y,
        content_width,
        status_h,
        to_rgba(colors.surface_panel, 1.0),
    );
    fill_rect(
        &mut canvas,
        padding,
        y,
        dims.spacing_xs,
        status_h,
        to_rgba(status_color, 1.0),
    );

    canvas
}

/// 1フレーム描いて PNG を書き、終了する口。`main.rs` の `--fixture --screenshot`
/// から呼ばれる。
pub fn write_png(shell: &Shell, path: &std::path::Path) -> Result<(), String> {
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
