//! 再生の pixel oracle(2026-08-19 iced 再生機構移植レーン)— outcome 2 の検収項目
//! 「(b) 停止時に Stage が正しい絵になる」。
//!
//! `stage_island_live_frame.rs`(M-3)は「与えた playhead で Stage が正しい絵を
//! 出す」ことを既に審判している(`moving_the_playhead_changes_the_frame_on_
//! the_stage_island`)。ここが足すのは1点だけ: **その playhead が、実際に
//! `TimelineEditor::toggle_playing` / `advance_playback` を「時計を進める」形で
//! 走らせた結果として着地した値である**こと。Stage 側の評価経路
//! (`seat.editor().playhead_seconds()` → `StageFrameSeat`)はこのレーンで
//! 1行も変えていないので、ここで確かめるのは実質「再生が正しい playhead を
//! 書く」ことと「その値を Stage が正しく拾う」ことの合流である。
//!
//! `advance_playback` の `dt` は決定的な値を直接渡す(`motolii-ui` 側の
//! ユニットテストと同じ流儀 — 実 sleep はしない)。GPU が無い環境は skip する
//! (既存の pixel oracle と同じ soft gate)。
//!
//! 証拠は `docs/reviews/evidence/iced-timeline-port/playback/` へ書く。

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    Clip, ClipSource, DocParam, Document, DocumentWriter, ItemEnvelope, Track, TrackItem,
    RECT_LAYER_SOURCE,
};
use motolii_shell_iced::stage_island::{self, StageIsland};
use motolii_ui::timeline_editor::TimelineEditor;

/// `Document::new_current` の既定 camera aspect(16:9)と一致させる
/// (`stage_island_live_frame.rs` と同じ理由 — 食い違うと評価そのものが蹴られる)。
const DOC_WIDTH: u32 = 96;
const DOC_HEIGHT: u32 = 54;

/// 時刻ごとに別の色になる2ショットの Document(赤 0-1s → 青 1-2s)。
/// `stage_island_live_frame.rs::two_shot_document` と同じ組み方
/// (この file は自己完結させる — 既存 test file と並走レーンで取り合わない)。
fn two_shot_document() -> Document {
    let mut document = Document::new_current();
    document
        .composition
        .set_resolution(Some((DOC_WIDTH, DOC_HEIGHT)))
        .expect("resolution");
    let track = document.track_ids.allocate("V1").expect("track");
    let one_second = RationalTime::from_seconds(1);
    let mut items = Vec::new();
    for (index, (start, color)) in [
        (RationalTime::ZERO, [1.0, 0.0, 0.0, 1.0]),
        (one_second, [0.0, 0.0, 1.0, 1.0]),
    ]
    .into_iter()
    .enumerate()
    {
        let layer = document
            .layers
            .allocate(&format!("shot-{index}"))
            .expect("layer");
        items.push(TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start,
            duration: one_second,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([
                    ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                    ("size".into(), DocParam::const_vec2([4.0, 4.0])),
                    ("color".into(), DocParam::const_color(color)),
                ]),
                extra: Default::default(),
            },
        }));
    }
    document.tracks.push(Track { id: track, items });
    document.composition.duration = RationalTime::from_seconds(2);
    document.validate().expect("two shot document");
    document
}

fn evidence_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/reviews/evidence/iced-timeline-port/playback")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dominant {
    Red,
    Blue,
    Other,
}

/// `stage_island_pixels.rs` / `stage_island_live_frame.rs` と同じ支配チャンネル分類。
fn classify(r: u8, g: u8, b: u8) -> Dominant {
    let (r, g, b) = (i32::from(r), i32::from(g), i32::from(b));
    let hi = 96;
    let lo = 80;
    if r > hi && g < lo && b < lo {
        Dominant::Red
    } else if b > hi && r < lo && g < lo {
        Dominant::Blue
    } else {
        Dominant::Other
    }
}

fn majority_dominant(image: &image::RgbaImage) -> (Dominant, f64) {
    use std::collections::HashMap;
    let (width, height) = image.dimensions();
    let inset_x = width * 15 / 100;
    let inset_y = height * 15 / 100;
    let mut counts: HashMap<Dominant, u32> = HashMap::new();
    let mut total = 0u32;
    let steps = 16u32;
    for iy in 0..steps {
        for ix in 0..steps {
            let x = inset_x + (width - 2 * inset_x) * (ix * 2 + 1) / (steps * 2);
            let y = inset_y + (height - 2 * inset_y) * (iy * 2 + 1) / (steps * 2);
            let pixel = image.get_pixel(x, y);
            *counts
                .entry(classify(pixel[0], pixel[1], pixel[2]))
                .or_default() += 1;
            total += 1;
        }
    }
    let (winner, count) = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .expect("1点は数えている");
    (winner, f64::from(count) / f64::from(total))
}

/// 与えた Document + playhead で widget を組み、非同期評価が届くまで待つ。
/// `stage_island_live_frame.rs::wait_for_dominant` と同じ組み方(evidence の
/// 置き場だけこのレーン専用に変えてある)。
fn wait_for_dominant(
    document: Arc<Document>,
    playhead: f32,
    expected: Dominant,
    label: &str,
) -> (Dominant, f64, PathBuf) {
    let aspect = DOC_WIDTH as f32 / DOC_HEIGHT as f32;
    let dir = evidence_dir();
    std::fs::create_dir_all(&dir).expect("evidence dir");
    let path = dir.join(format!("{label}.png"));
    let written = dir.join(format!("{label}-wgpu.png"));

    let island = StageIsland {
        composition_aspect: Some(aspect),
        document: Some(document),
        playhead,
        grab_probe: None,
    };
    let mut ui: iced_test::Simulator<'_, motolii_shell_iced::Message> =
        iced_test::Simulator::with_size(
            iced_test::core::Settings::default(),
            iced::Size::new(160.0, 90.0),
            iced::Element::from(
                iced::widget::shader(island)
                    .width(iced::Fill)
                    .height(iced::Fill),
            ),
        );

    let mut last: Option<(Dominant, f64, PathBuf)> = None;
    for _ in 0..300 {
        let snapshot = ui
            .snapshot(&iced::Theme::Dark)
            .expect("headless snapshot が撮れる");
        let _ = std::fs::remove_file(&written);
        assert!(
            snapshot.matches_image(&path).expect("PNG が書ける"),
            "証拠 PNG が書けない"
        );
        let image = image::open(&written)
            .expect("書いた証拠 PNG を読み戻せる")
            .to_rgba8();
        let (winner, share) = majority_dominant(&image);
        last = Some((winner, share, written.clone()));
        if winner == expected && share > 0.7 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    last.expect("少なくとも1回は試す")
}

/// outcome 2 (b): `Space` で再生を始め、`dt` を「時計を進める」形で1.5s ぶん
/// 進めてから `Space` でもう一度止めると、その playhead(2ショット目=青の
/// 範囲)で Stage が正しい絵を出す。
#[test]
fn playback_stops_on_the_correct_frame_after_ticking_into_the_second_shot() {
    let Some(()) = common::gpu_or_skip() else {
        return;
    };
    stage_island::install_rerun_device_floor();

    let document = two_shot_document();
    assert!(
        document.soundtrack.is_none(),
        "soundtrack 無し前提(壁時計の経路を確かめたい)"
    );
    let catalog = Arc::new(
        motolii_plugin::reference::reference_catalog().expect("reference catalog"),
    );
    let writer = DocumentWriter::new(document, catalog).expect("writer");
    let mut editor = TimelineEditor::new(writer);

    // 「時計を進める」= dt を直接渡す(motolii-ui 側のユニットテストと同じ、
    // 決定的な進め方。実 sleep はしない)。0 → 1.5s。
    editor.toggle_playing();
    assert!(editor.is_playing());
    for _ in 0..30 {
        editor.advance_playback(0.05);
        assert!(
            editor.is_playing(),
            "1.5s は composition 全長(2s)の途中のはずが、途中で再生が止まった: {}",
            editor.playhead_seconds()
        );
    }
    // Space でもう一度止める。
    editor.toggle_playing();
    assert!(!editor.is_playing(), "2度目の toggle_playing で止まっていない");
    assert!(
        (editor.playhead_seconds() - 1.5).abs() < 0.1,
        "1.5s 付近で止まったはずが {}",
        editor.playhead_seconds()
    );

    let stopped_document = Arc::clone(editor.document());
    let stopped_playhead = editor.playhead_seconds();

    let (winner, share, evidence) = wait_for_dominant(
        stopped_document,
        stopped_playhead,
        Dominant::Blue,
        "playback-stops-on-second-shot",
    );
    assert_eq!(
        winner,
        Dominant::Blue,
        "再生を1.5sぶん進めて止めたのに、Stage が2ショット目(青)を映していない \
         (占有 {share:.2})。証拠: {}",
        evidence.display()
    );
    assert!(
        share > 0.7,
        "青の占有が {share:.2} しかない。証拠: {}",
        evidence.display()
    );
}
