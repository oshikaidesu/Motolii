//! YUV420p→RGBAのCPU参照実装(シェーダと同一式)。

use motolii_core::{CpuFrame, PixelFormat};
use motolii_gpu::ColorParams;

/// テクセル中心=+0.5の連続座標でのバイリニアサンプル(ClampToEdge)。
fn sample_bilinear(plane: &[u8], w: usize, h: usize, pos_x: f32, pos_y: f32) -> f32 {
    let tx = pos_x - 0.5;
    let ty = pos_y - 0.5;
    let x0 = tx.floor();
    let y0 = ty.floor();
    let fx = tx - x0;
    let fy = ty - y0;
    let xi = |x: f32| (x.max(0.0) as usize).min(w - 1);
    let yi = |y: f32| (y.max(0.0) as usize).min(h - 1);
    let p = |x: usize, y: usize| plane[y * w + x] as f32;
    let (x0u, x1u) = (xi(x0), xi(x0 + 1.0));
    let (y0u, y1u) = (yi(y0), yi(y0 + 1.0));
    let top = p(x0u, y0u) * (1.0 - fx) + p(x1u, y0u) * fx;
    let bot = p(x0u, y1u) * (1.0 - fx) + p(x1u, y1u) * fx;
    top * (1.0 - fy) + bot * fy
}

/// CPU参照実装: シェーダと同一式・同一係数でYUV420pをRGBAへ変換する。
/// ゴールデンテストの理論値として使う(GPUと数値一致すべき)。
pub fn yuv_to_rgba_reference(frame: &CpuFrame) -> Vec<u8> {
    assert_eq!(frame.desc.format, PixelFormat::Yuv420p);
    let p = ColorParams::for_color_space(frame.desc.color_space).expect("yuv color space");
    let (w, h) = (frame.desc.width as usize, frame.desc.height as usize);
    let (cw, ch) = (w / 2, h / 2);
    let y_size = w * h;
    let c_size = cw * ch;
    let y_plane = &frame.data[..y_size];
    let u_plane = &frame.data[y_size..y_size + c_size];
    let v_plane = &frame.data[y_size + c_size..y_size + 2 * c_size];

    let mut out = vec![0u8; w * h * 4];
    for row in 0..h {
        for col in 0..w {
            let yv = y_plane[row * w + col] as f32;
            // シェーダと同一のsiting位置(水平=左cosited、垂直=中間)でバイリニア
            let pos_x = col as f32 * 0.5 + 0.5;
            let pos_y = row as f32 * 0.5 + 0.25;
            let uv = sample_bilinear(u_plane, cw, ch, pos_x, pos_y);
            let vv = sample_bilinear(v_plane, cw, ch, pos_x, pos_y);

            let yl = (yv - p.y_off) * p.y_scale;
            let cb = (uv - 128.0) * p.c_scale;
            let cr = (vv - 128.0) * p.c_scale;
            let r = yl + p.crv * cr;
            let g = yl + p.cgu * cb + p.cgv * cr;
            let b = yl + p.cbu * cb;

            let o = (row * w + col) * 4;
            out[o] = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
            out[o + 3] = 255;
        }
    }
    out
}
