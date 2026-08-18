//! Stage 島の pixel oracle — M-2 outcome 1(絵が出る)と 2(camera seat・正対既定)。
//!
//! E0 probe(`spikes/rerun-e0-composition-probe`)と同じ4象限の既知絵を
//! 評価済みフレームの席へ差し、**iced の shader widget を通った後の絵**
//! (運転席 snapshot)で象限の色を照合する。正対(document camera)でなければ
//! 象限は期待の場所に写らないので、これが camera seat 結線の審判でもある。
//!
//! pixel 証拠は `docs/reviews/evidence/iced-m2-stage-island/` へ書く
//! (`Snapshot::matches_image` は無ければ作る仕様なので、毎回消してから書く)。
//!
//! 色の照合は E0 と同じ**支配チャンネル分類**(赤/緑/青/黄)。ガンマや
//! ブレンドで値が動いても支配チャンネルは動かない。

mod common;

use std::path::PathBuf;

use motolii_shell_iced::stage_island::{self, StageIsland};

/// 4象限の既知絵(左上=赤 / 右上=緑 / 左下=青 / 右下=黄)。E0 と同じ配色。
fn quadrant_frame(width: u32, height: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let left = x < width / 2;
            let top = y < height / 2;
            let color: [u8; 4] = match (top, left) {
                (true, true) => [255, 0, 0, 255],
                (true, false) => [0, 255, 0, 255],
                (false, true) => [0, 0, 255, 255],
                (false, false) => [255, 255, 0, 255],
            };
            rgba.extend_from_slice(&color);
        }
    }
    rgba
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dominant {
    Red,
    Green,
    Blue,
    Yellow,
    Other,
}

/// E0 の oracle と同じ考え方の支配チャンネル分類。
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
    } else if r > hi && g > hi && b < lo {
        Dominant::Yellow
    } else {
        Dominant::Other
    }
}

fn evidence_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/reviews/evidence/iced-m2-stage-island")
}

/// 象限の中身を格子で数え、支配色の多数決を返す。
fn majority(image: &image::RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> (Dominant, f64) {
    use std::collections::HashMap;
    let mut counts: HashMap<Dominant, u32> = HashMap::new();
    let mut total = 0u32;
    let steps = 12u32;
    for iy in 0..steps {
        for ix in 0..steps {
            let x = x0 + (x1 - x0) * (ix * 2 + 1) / (steps * 2);
            let y = y0 + (y1 - y0) * (iy * 2 + 1) / (steps * 2);
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

/// 4象限の既知絵が、iced の widget を通って**正対で**画枠に写る。
///
/// - 絵が出ること自体が outcome 1(黒いままなら全象限 Other で落ちる)
/// - 象限が期待の場所に出ることが outcome 2(Rerun の既定 eye(斜め見下ろし)の
///   ままなら、象限の対応も面積も崩れて落ちる)
#[test]
fn the_known_frame_maps_head_on_through_the_iced_widget() {
    let Some(()) = common::gpu_or_skip() else {
        return;
    };
    stage_island::install_rerun_device_floor();

    // 絵 320x240(aspect 4:3)を、同じ形の widget(320x240)へ。
    // comp と面が同じ形なので、正対 camera は絵を画枠ちょうどに写す。
    stage_island::install_probe_frame(320, 240, quadrant_frame(320, 240));

    let island = StageIsland {
        composition_aspect: Some(320.0 / 240.0),
        grab_probe: None,
    };
    let mut ui: iced_test::Simulator<'_, motolii_shell_iced::Message> = iced_test::Simulator::with_size(
        iced_test::core::Settings::default(),
        iced::Size::new(320.0, 240.0),
        iced::Element::from(
            iced::widget::shader(island)
                .width(iced::Fill)
                .height(iced::Fill),
        ),
    );

    // 1周目で stage が建ち(重い)、2周目で落ち着いた絵を撮る。
    let _ = ui
        .snapshot(&iced::Theme::Dark)
        .expect("headless snapshot が撮れる");
    let snapshot = ui
        .snapshot(&iced::Theme::Dark)
        .expect("headless snapshot が撮れる");

    // 証拠を残す(matches_image は無ければ書く仕様なので、消してから書く)。
    let dir = evidence_dir();
    std::fs::create_dir_all(&dir).expect("evidence dir");
    let path = dir.join("head-on-quadrants.png");
    let written = dir.join("head-on-quadrants-wgpu.png");
    let _ = std::fs::remove_file(&written);
    assert!(
        snapshot.matches_image(&path).expect("PNG が書ける"),
        "証拠 PNG が書けない"
    );

    let image = image::open(&written)
        .expect("書いた証拠 PNG を読み戻せる")
        .to_rgba8();
    let (width, height) = image.dimensions();

    // 象限の内側(縁から 12% 逃がす)を多数決で分類する。
    let inset_x = width * 12 / 100;
    let inset_y = height * 12 / 100;
    let cases = [
        ("左上", inset_x, inset_y, width / 2 - inset_x, height / 2 - inset_y, Dominant::Red),
        ("右上", width / 2 + inset_x, inset_y, width - inset_x, height / 2 - inset_y, Dominant::Green),
        ("左下", inset_x, height / 2 + inset_y, width / 2 - inset_x, height - inset_y, Dominant::Blue),
        ("右下", width / 2 + inset_x, height / 2 + inset_y, width - inset_x, height - inset_y, Dominant::Yellow),
    ];
    for (name, x0, y0, x1, y1, expected) in cases {
        let (winner, share) = majority(&image, x0, y0, x1, y1);
        assert_eq!(
            winner, expected,
            "{name}の象限が {expected:?} でなく {winner:?}(占有 {share:.2})。\
             正対 camera で写っていない(または絵が出ていない)。証拠: {}",
            written.display()
        );
        assert!(
            share > 0.7,
            "{name}の象限の {expected:?} 占有が {share:.2} しかない。証拠: {}",
            written.display()
        );
    }
}

/// widget を通したドラッグが stage まで届く(素通し / orbit 中は流れる)。
///
/// 検証層エラーが1つでも出れば `Message::StageReported` に載って出てくるので、
/// 「流れた・そして GPU に蹴られていない」を同時に言える。orbit 中に Rerun が
/// 線分(orbit 目印)を引こうとしたときに落ちない、が seam 2 の側面支援になる
/// (document camera が sticky の間は目印が出ない可能性があり、その場合この
/// 検証層の主張は空振りだが、tally の主張は空振りしない)。
#[test]
fn a_drag_through_the_widget_reaches_the_stage_without_validation_errors() {
    let Some(()) = common::gpu_or_skip() else {
        return;
    };
    stage_island::install_rerun_device_floor();

    let island = StageIsland {
        composition_aspect: Some(1.0),
        grab_probe: None,
    };
    let mut ui: iced_test::Simulator<'_, motolii_shell_iced::Message> = iced_test::Simulator::with_size(
        iced_test::core::Settings::default(),
        iced::Size::new(200.0, 200.0),
        iced::Element::from(
            iced::widget::shader(island)
                .width(iced::Fill)
                .height(iced::Fill),
        ),
    );

    let (forwarded_before, _) = stage_island::input_tally();
    let mut events = vec![iced::Event::Mouse(iced::mouse::Event::CursorMoved {
        position: iced::Point::new(100.0, 100.0),
    })];
    events.push(iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
        iced::mouse::Button::Left,
    )));
    for step in 1..=12 {
        events.push(iced::Event::Mouse(iced::mouse::Event::CursorMoved {
            position: iced::Point::new(100.0 + step as f32 * 4.0, 100.0 - step as f32 * 2.0),
        }));
    }
    events.push(iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
        iced::mouse::Button::Left,
    )));
    let _ = ui.simulate(events);

    // prepare(= stage へ流す)を回してから、報告の吐き出しをもう1周。
    let _ = ui.snapshot(&iced::Theme::Dark);
    let _ = ui.simulate([common::redraw()]);

    let (forwarded_after, _) = stage_island::input_tally();
    assert!(
        forwarded_after - forwarded_before >= 14,
        "ドラッグ一式(move+press+12 move+release)が橋を渡っていない: {} 手",
        forwarded_after - forwarded_before
    );

    let reported: Vec<String> = ui
        .into_messages()
        .filter_map(|message| match message {
            motolii_shell_iced::Message::StageReported(lines) => Some(lines),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(
        reported.is_empty(),
        "ドラッグで stage が失敗を報告した(GPU に蹴られている疑い): {reported:?}"
    );
}

/// ダミー掴み領域の中で始めたジェスチャは **Rerun へ1手も流れない**(掴み中)。
///
/// 調停3状態の状態遷移そのものは `stage_arbiter` の単体テストが審判済み。
/// ここは「widget の実配線でも swallow が起きる」ことを tally で見る。
#[test]
fn a_gesture_inside_the_dummy_grab_region_is_swallowed() {
    let Some(()) = common::gpu_or_skip() else {
        return;
    };
    stage_island::install_rerun_device_floor();

    let island = StageIsland {
        composition_aspect: Some(1.0),
        grab_probe: Some(motolii_shell_iced::stage_arbiter::GrabRegion {
            x: 40.0,
            y: 40.0,
            width: 40.0,
            height: 40.0,
        }),
    };
    let mut ui: iced_test::Simulator<'_, motolii_shell_iced::Message> = iced_test::Simulator::with_size(
        iced_test::core::Settings::default(),
        iced::Size::new(200.0, 200.0),
        iced::Element::from(
            iced::widget::shader(island)
                .width(iced::Fill)
                .height(iced::Fill),
        ),
    );

    let (_, swallowed_before) = stage_island::input_tally();
    let events = vec![
        iced::Event::Mouse(iced::mouse::Event::CursorMoved {
            position: iced::Point::new(60.0, 60.0),
        }),
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::CursorMoved {
            position: iced::Point::new(120.0, 120.0),
        }),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
            iced::mouse::Button::Left,
        )),
    ];
    let _ = ui.simulate(events);
    let (_, swallowed_after) = stage_island::input_tally();

    // press + 領域外への move + release の3手が丸ごと飲まれる
    // (先頭の hover move だけは素通しで流れてよい)。
    assert!(
        swallowed_after - swallowed_before >= 3,
        "掴み領域で始めたジェスチャが Rerun へ漏れている: swallow {} 手",
        swallowed_after - swallowed_before
    );
}
