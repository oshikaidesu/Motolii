//! 静止画書き出しの契約 — PNG が実在し、寸法が comp と一致する。
//!
//! `Engine::render_frame` を直接呼ぶだけなので ffmpeg サイドカーは要らない
//! (`ffmpeg_or_skip` を通さない — 動画書き出しと違う経路であることの証拠でもある)。

use motolii_core::Fps;
use motolii_engine::Engine;
use motolii_export::export_still;
use motolii_store::{Composition, Document, Intent, LayerId, LayerMeta, LayerSource};
use motolii_store::LayerTiming;
use motolii_testkit::tmp_dir;

const W: u32 = 48;
const H: u32 = 32;
const FRAMES: i64 = 10;

fn document() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: FRAMES,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [255, 0, 0, 255],
                width: 16,
                height: 16,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
    doc
}

#[test]
fn still_export_writes_a_png_matching_comp_dimensions() {
    let dir = tmp_dir("export-still");
    let out = dir.join("frame.png");
    let doc = document();
    let mut engine = Engine::new().unwrap();

    let report = export_still(&mut engine, &doc.view(), 3, &out).unwrap();

    assert_eq!(report.frames_written, 1);
    assert_eq!(report.out_path, out);
    assert!(out.exists(), "PNG が無い");

    let decoded = image::open(&out)
        .unwrap_or_else(|err| panic!("書き出した PNG を読めない: {err}"))
        .to_rgba8();
    assert_eq!(
        (decoded.width(), decoded.height()),
        (W, H),
        "PNG の寸法が comp と一致しない"
    );
}

#[test]
fn still_export_at_last_valid_frame_succeeds() {
    let dir = tmp_dir("export-still-last-frame");
    let out = dir.join("last.png");
    let doc = document();
    let mut engine = Engine::new().unwrap();

    // 半開 [0, duration_frames) の最後の有効フレーム。
    let report = export_still(&mut engine, &doc.view(), FRAMES - 1, &out).unwrap();
    assert_eq!(report.frames_written, 1);
    assert!(out.exists());
}

/// **M15 / D1 と同じ精神 — preview = export**。静止画も
/// `Engine::render_frame` の出力そのものであることを、PNG を読み直して
/// 突き合わせて確かめる。PNG は無劣化(可逆)なので動画書き出しの
/// `exported_frames_are_the_preview_frames` と違い誤差許容は要らない —
/// ただし alpha は doc に書いた既知限界どおり premultiplied のまま出るので、
/// RGB 側だけを厳密一致で見る(alpha=255 の不透明背景なので実質は全チャンネル
/// 一致するが、判定の意図を明確にするため RGB のみ書く)。
#[test]
fn still_export_pixels_match_render_frame_exactly() {
    let dir = tmp_dir("export-still-exact");
    let out = dir.join("exact.png");
    let doc = document();
    let mut engine = Engine::new().unwrap();

    export_still(&mut engine, &doc.view(), 5, &out).unwrap();

    let preview = engine
        .render_frame(&doc.view(), motolii_store::RationalTime::try_new(5, 30).unwrap())
        .unwrap();
    let decoded = image::open(&out).unwrap().to_rgba8();
    let raw = decoded.into_raw();

    assert_eq!(raw.len(), preview.len(), "バイト長が一致しない");
    for i in (0..raw.len()).step_by(4) {
        assert_eq!(
            &raw[i..i + 3],
            &preview[i..i + 3],
            "pixel {} の RGB が render_frame と一致しない",
            i / 4
        );
    }
}
