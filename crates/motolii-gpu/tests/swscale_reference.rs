//! 外部グラウンドトゥルースとの突き合わせ(レビュー指摘#3)。
//!
//! 自前のGPU変換とCPU参照実装は同じ式なので、式自体の誤りは内部比較では
//! 検出できない。**非均一パターン**(グラデーション+斜めエッジ)をffmpegの
//! swscale(in_color_matrix=bt709明示)で変換した結果と比較し、係数・レンジ・
//! クロマ位置の系統誤差に網をかける。
//!
//! swscaleとはクロマ補間アルゴリズムが同一ではないため、完全一致ではなく
//! 「平均誤差が小さく、最大誤差もエッジの補間差の範囲内」を合格条件とする。
//!
//! GPUアダプタ or ffmpegが無い環境ではskip(CIでは両方必ず実行)。

use std::io::{Read, Write};
use std::process::{Command, Stdio};

use motolii_core::{ColorSpace, CpuFrame, FrameDesc, PixelFormat, RationalTime};
use motolii_gpu::{download_rgba, GpuCtx, YuvToRgba};

const W: u32 = 64;
const H: u32 = 64;

fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 非均一テストパターン: Y=水平グラデ+斜めエッジ、U=垂直グラデ、V=斜めグラデ
fn make_pattern() -> CpuFrame {
    let desc = FrameDesc::yuv(W, H, PixelFormat::Yuv420p, ColorSpace::Rec709Limited);
    let (w, h) = (W as usize, H as usize);
    let (cw, ch) = (w / 2, h / 2);
    let mut data = vec![0u8; desc.data_size()];
    // Y面: limitedレンジ内の水平グラデ + 斜めの明暗エッジ
    for y in 0..h {
        for x in 0..w {
            let grad = 16 + (x * 219 / (w - 1)) as u8;
            let v = if x + y < w { grad } else { 235 - grad + 16 };
            data[y * w + x] = v.clamp(16, 235);
        }
    }
    // U面: 垂直グラデ、V面: 斜めグラデ(limitedクロマレンジ内)
    let (u_off, v_off) = (w * h, w * h + cw * ch);
    for y in 0..ch {
        for x in 0..cw {
            data[u_off + y * cw + x] = (16 + y * 224 / (ch - 1)) as u8;
            data[v_off + y * cw + x] = (16 + (x + y) * 224 / (cw + ch - 2)) as u8;
        }
    }
    CpuFrame::new(desc, RationalTime::ZERO, data)
}

/// swscaleで同じフレームをRGBAへ変換する(matrix/range明示)。
fn swscale_convert(frame: &CpuFrame) -> Vec<u8> {
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-f", "rawvideo", "-pix_fmt", "yuv420p"])
        .args(["-s", &format!("{W}x{H}")])
        .args(["-i", "-"])
        .args([
            "-vf",
            // in_h_chr_pos=0(左cosited)/in_v_chr_pos=128(垂直中間)を明示し、
            // クロマsiting式の正しさまで比較対象に含める(第3回レビュー#4)
            "scale=in_color_matrix=bt709:in_range=tv:out_range=pc:in_h_chr_pos=0:in_v_chr_pos=128",
        ])
        .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ffmpeg");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&frame.data)
        .expect("write yuv");
    let mut out = Vec::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut out)
        .expect("read rgba");
    let status = child.wait().expect("ffmpeg wait");
    assert!(status.success(), "swscale conversion failed");
    assert_eq!(out.len(), (W * H * 4) as usize);
    out
}

#[test]
fn gpu_matches_swscale_on_nonuniform_pattern() {
    if !ffmpeg_available() {
        eprintln!("SKIP: ffmpeg not found");
        return;
    }
    let Ok(gpu) = GpuCtx::new_headless() else {
        eprintln!("SKIP: no GPU adapter");
        return;
    };

    let frame = make_pattern();
    let reference = swscale_convert(&frame);

    let mut conv = YuvToRgba::new(&gpu);
    let ours = download_rgba(&gpu, &conv.convert(&gpu, &frame).unwrap()).unwrap();

    // アルファはswscale出力も255のはずだが、比較はRGBのみで行う
    let mut sum = 0u64;
    let mut max = 0i32;
    let mut n = 0u64;
    for (i, (&a, &b)) in ours.iter().zip(&reference).enumerate() {
        if i % 4 == 3 {
            continue;
        }
        let d = (a as i32 - b as i32).abs();
        sum += d as u64;
        max = max.max(d);
        n += 1;
    }
    let mean = sum as f64 / n as f64;
    eprintln!("vs swscale: mean abs diff = {mean:.3}, max = {max}");

    // 平均誤差: 係数・レンジの系統誤差があればここが大きく崩れる
    assert!(mean <= 2.5, "mean diff {mean:.3} > 2.5 (systematic error?)");
    // 最大誤差: siting明示済みのため締めた上限(実測7、補間アルゴリズム差の余裕込み)
    assert!(
        max <= 24,
        "max diff {max} > 24 (chroma siting/upsample bug?)"
    );
}
