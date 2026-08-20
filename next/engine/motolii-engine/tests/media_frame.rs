//! **実素材が板として合成に出る**ことの通し試験。
//!
//! ffmpeg で既知の色の動画を作り、Document に置き、engine が出した画素を見る。
//! ここが通ると「素材 → decode → GPU → 合成 → 画素」が1本で繋がったことになる。

use std::process::Command;

use motolii_compositor::CompSpec;
use motolii_engine::Engine;
use motolii_store::{Composition, Document, Intent, LayerId, LayerMeta, LayerSource, RationalTime};
use motolii_store::LayerTiming;
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};

const W: u32 = 64;
const H: u32 = 64;

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}

/// 前半が赤・後半が青の2秒の動画。**時間で色が変わる**ので、
/// comp 時刻が素材のフレームへ正しく写っているかまで見える。
fn make_video(dir: &std::path::Path) -> String {
    let path = dir.join("two-tone.mp4");
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=128x128:d=1:r=30",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=128x128:d=1:r=30",
            "-filter_complex",
            "[0:v][1:v]concat=n=2:v=1[out]",
            "-map",
            "[out]",
            "-pix_fmt",
            "yuv420p",
            "-c:v",
            "libx264",
            "-crf",
            "18",
        ])
        .arg(&path)
        .output()
        .expect("ffmpeg を起動できない");
    assert!(
        status.status.success(),
        "fixture 生成に失敗: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    path.to_string_lossy().into_owned()
}

#[test]
fn real_media_becomes_a_layer() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("engine-media-frame");
    let path = make_video(&dir);

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: motolii_store::Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Media {
                path: path.clone(),
                fingerprint: None,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let comp = CompSpec {
        width: W,
        height: H,
    };
    let mut engine = Engine::new().expect("engine");

    // 0.5秒 = 赤の区間
    let early = engine
        .render_frame(&doc.view(), RationalTime::try_new(1, 2).unwrap())
        .unwrap();
    let early_px = pixel(&early, 32, 32);

    // 1.5秒 = 青の区間
    let late = engine
        .render_frame(&doc.view(), RationalTime::try_new(3, 2).unwrap())
        .unwrap();
    let late_px = pixel(&late, 32, 32);

    println!("実素材: 0.5秒={early_px:?} 1.5秒={late_px:?}");

    assert!(
        early_px[0] > early_px[2],
        "前半は赤が勝つはず(YUV→RGB が上流経路で通っているか): {early_px:?}"
    );
    assert!(
        late_px[2] > late_px[0],
        "後半は青が勝つはず(comp 時刻が素材のフレームへ写っているか): {late_px:?}"
    );
    assert_ne!(early, late, "時刻で絵が変わらないなら素材を読めていない");
}

/// 素材の実寸が layer の既定の大きさになること(track も declared も無い時)。
#[test]
fn media_natural_size_fills_the_layer() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("engine-media-size");
    let path = make_video(&dir);

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: motolii_store::Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Media {
                path,
                fingerprint: None,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    // comp は 64x64、素材は 128x128。素材の実寸で置かれるので comp を覆い切る。
    let frame = engine
        .render_frame(&doc.view(), RationalTime::try_new(0, 30).unwrap())
        .unwrap();

    for (x, y) in [(1, 1), (62, 1), (1, 62), (62, 62), (32, 32)] {
        let px = pixel(&frame, x, y);
        assert!(
            px[0] > 60,
            "({x},{y}) が素材で埋まっていない = 実寸が効いていない: {px:?}"
        );
    }
}
