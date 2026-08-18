//! Stage 島の live 評価済みフレーム — M-3 outcome(playhead 時刻の合成フレームが
//! iced shell の Stage 島に映る)の pixel oracle。
//!
//! `stage_island_pixels.rs`(M-2)は `install_probe_frame` で CPU から直接
//! 差した既知絵を審判する。ここは一段手前 — **live Document を座らせて**、
//! `StageIsland{ document: Some(..), playhead, .. }` が
//! `motolii_ui::stage_frame_seat::StageFrameSeat` を実際に回し、export と同じ
//! 評価(`build_document_frame_graph` + `render_graph_cached`)を経た絵が
//! iced の widget を通って出てくることを画素で見る。
//!
//! fixture は実素材ではなく `RECT_LAYER_SOURCE`(`motolii_doc` 組み込みの
//! 単色 rect layer)で、`crates/motolii-ui/src/stage_frame_seat.rs` のテスト
//! (`two_shot_project`)と同じ組み方。評価は別 thread の非同期なので、
//! 「絵が出るまで redraw を刻んで待つ」形の retry loop で審判する
//! (合否は**毎回 snapshot を画素分類した結果そのもの** — 共有 static の
//! カウンタは使わない。`FRAME_SEAT_EVIDENCE` は process 全体で1つで、この
//! file の複数 `#[test]` が並列に走ると取り合いになるため)。
//!
//! pixel 証拠は `docs/reviews/evidence/iced-m3-stage-frame-seat/` へ書く。

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    Clip, ClipSource, DocParam, Document, ItemEnvelope, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_shell_iced::stage_island::{self, StageIsland};

/// レンダリングを軽くするための小さい解像度。**`Document::new_current` の
/// 既定 camera aspect(16:9)と一致させる** — `motolii_core::camera` は
/// フレームの寸法が camera aspect と食い違うと評価そのものを蹴る
/// (`frame WxH does not match camera aspect N/D`。`Composition` に aspect の
/// setter は無く、`resolution` だけを独立に差し替えるとこれに当たる)。
/// widget もこれと同じ縦横比で構える — M-2 の pixel oracle と同じ「正対なら
/// 画枠いっぱいに写る」前提を使う。
const DOC_WIDTH: u32 = 96;
const DOC_HEIGHT: u32 = 54;

/// 単色の rect layer 1本だけを持つ Document。`RECT_LAYER_SOURCE` は
/// `crates/motolii-ui/src/stage_frame_seat.rs` の `green_project` /
/// `two_shot_project` フィクスチャと同じ組み込み plugin — 実素材を読まずに
/// 既知色の合成結果を作れる。
fn solid_color_document(color: [f64; 4], duration_seconds: i64) -> Document {
    let mut document = Document::new_current();
    document
        .composition
        .set_resolution(Some((DOC_WIDTH, DOC_HEIGHT)))
        .expect("resolution");
    let track = document.track_ids.allocate("V1").expect("track");
    let layer = document.layers.allocate("solid").expect("layer");
    let duration = RationalTime::from_seconds(duration_seconds);
    let item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(layer),
        start: RationalTime::ZERO,
        duration,
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
    });
    document.tracks.push(Track {
        id: track,
        items: vec![item],
    });
    document.composition.duration = duration;
    document.validate().expect("solid color document");
    document
}

/// 時刻ごとに別の色になる2ショットの Document。playhead 追従の審判に使う
/// (`stage_frame_seat.rs` の `two_shot_project` と同じ組み方)。
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
        .join("../../docs/reviews/evidence/iced-m3-stage-frame-seat")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dominant {
    Red,
    Green,
    Blue,
    Other,
}

/// `stage_island_pixels.rs` の支配チャンネル分類と同じ考え方(赤/緑/青のみ)。
fn classify(r: u8, g: u8, b: u8) -> Dominant {
    let (r, g, b) = (i32::from(r), i32::from(g), i32::from(b));
    let hi = 96;
    let lo = 80;
    if r > hi && g < lo && b < lo {
        Dominant::Red
    } else if g > hi && r < lo && b < lo {
        Dominant::Green
    } else if b > hi && r < lo && g < lo {
        Dominant::Blue
    } else {
        Dominant::Other
    }
}

/// 画枠の内側(縁から 15% 逃がす)を格子で数え、支配色の多数決を返す。
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

/// 与えた Document + playhead で widget を**1つだけ**組み、非同期評価が届くまで
/// **同じ** `Simulator`(= 同じ thread_local `Embed` / `StageFrameSeat`)に
/// 何度も snapshot を撮らせて待つ。`Simulator` を作り直すと `Pipeline::new` が
/// 新しい token を配って `Embed` ごと建て直り、席も worker thread も
/// 最初からになってしまう(=決して収束しない)ので、ループの**外側**で1回だけ
/// 組む。
///
/// **`expected` に届くまで待つ** — 評価が終わる前の Rerun シーン(背景/グリッド)
/// も `Other` として最初から高い share で多数決が決まってしまうので、
/// 「share が高い」だけで止めると1周目の絵(=まだ評価済みフレームが乗っていない
/// 絵)で止まってしまう。合否は呼び手が `expected` との一致で判定する
/// (共有 static のカウンタには頼らない)。
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
            // 16:9(`DOC_WIDTH`/`DOC_HEIGHT` と同じ縦横比)。
            iced::Size::new(160.0, 90.0),
            iced::Element::from(
                iced::widget::shader(island)
                    .width(iced::Fill)
                    .height(iced::Fill),
            ),
        );

    let mut last: Option<(Dominant, f64, PathBuf)> = None;
    for attempt in 0..300 {
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
        if attempt % 20 == 0 {
            // `--nocapture` 診断: 席の実測(process 全体で1つの static だが、
            // ここは合否に使わず printf debugging 専用)。
            println!(
                "{label}: attempt={attempt} winner={winner:?} share={share:.2} \
                 frame_seat_evidence={:?}",
                stage_island::frame_seat_evidence()
            );
        }
        if winner == expected && share > 0.7 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // 失敗時の診断: Stage 島の報告(初期化・texture import・評価の失敗)は
    // `Message::StageReported` になって出てくる。合否には使わない
    // (プロセス単位の mailbox は他 test と取り合いうる)が、原因の当たりを
    // 付けるのに読む価値はある。mailbox は `RedrawRequested` が来るまで
    // widget 木の外へ出てこないので、明示的に1回流す(`stage_reports.rs`
    // と同じ二段構え)。
    let _ = ui.simulate([common::redraw()]);
    let reported: Vec<String> = ui
        .into_messages()
        .filter_map(|message| match message {
            motolii_shell_iced::Message::StageReported(lines) => Some(lines),
            _ => None,
        })
        .flatten()
        .collect();
    if !reported.is_empty() {
        println!("{label}: StageReported 診断: {reported:?}");
    }

    last.expect("少なくとも1回は試す")
}

/// outcome 1: 評価済みフレームが Stage 島へ渡る。
///
/// `document: None` の probe 経路(`stage_island_pixels.rs`)と違い、ここは
/// **live Document を座らせただけ**で `StageFrameSeat` が export と同じ評価を
/// 走らせ、その結果が widget を通って出てくることを見る。赤一色の fixture を
/// 置いたら、画枠の内側は赤が支配的になるはず。
#[test]
fn an_evaluated_document_frame_lands_on_the_stage_island() {
    let Some(()) = common::gpu_or_skip() else {
        return;
    };
    stage_island::install_rerun_device_floor();

    let document = Arc::new(solid_color_document([1.0, 0.0, 0.0, 1.0], 2));
    let (winner, share, evidence) =
        wait_for_dominant(document, 0.0, Dominant::Red, "live-frame-lands-red");

    assert_eq!(
        winner,
        Dominant::Red,
        "画枠の内側が Red でなく {winner:?}(占有 {share:.2})。\
         live Document の評価済みフレームが Stage 島へ渡っていない。証拠: {}",
        evidence.display()
    );
    assert!(
        share > 0.7,
        "Red の占有が {share:.2} しかない。証拠: {}",
        evidence.display()
    );
}

/// outcome 2: playhead を動かすと Stage 島に出る絵が変わる。
///
/// 2ショットの fixture(赤→青)で、playhead 0.0 と 1.5 のそれぞれで別の色が
/// 支配的になることを見る。egui shell の `the_frame_follows_the_playhead`
/// (`stage_frame_seat.rs`)の iced 版。
#[test]
fn moving_the_playhead_changes_the_frame_on_the_stage_island() {
    let Some(()) = common::gpu_or_skip() else {
        return;
    };
    stage_island::install_rerun_device_floor();

    let document = Arc::new(two_shot_document());

    let (first_winner, first_share, first_evidence) = wait_for_dominant(
        Arc::clone(&document),
        0.0,
        Dominant::Red,
        "live-frame-follows-t0-red",
    );
    assert_eq!(
        first_winner,
        Dominant::Red,
        "t=0 は Red のはずが {first_winner:?}(占有 {first_share:.2})。証拠: {}",
        first_evidence.display()
    );

    let (second_winner, second_share, second_evidence) = wait_for_dominant(
        Arc::clone(&document),
        1.5,
        Dominant::Blue,
        "live-frame-follows-t1-blue",
    );
    assert_eq!(
        second_winner,
        Dominant::Blue,
        "t=1.5 は Blue のはずが {second_winner:?}(占有 {second_share:.2})。証拠: {}",
        second_evidence.display()
    );

    assert_ne!(
        first_winner, second_winner,
        "playhead を動かしても Stage 島の絵が変わっていない(古いフレームのまま)"
    );
}
