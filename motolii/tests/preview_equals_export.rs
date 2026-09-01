//! 窓に出る絵と、書き出しに出る絵は同じ物でなければならない。
//! 憲法「評価経路を2本にしない」の見張り。

mod testkit;

/// 同じ時刻の絵は、そこへどう辿り着いても同じでなければならない。
///
/// デコーダが間に合わない時に前のコマを返していると、窓(毎フレーム回る)と
/// 書き出し(1コマ1回)で違う絵が出る。憲法「評価経路を2本にしない」の実物。
#[test]
fn the_same_time_gives_the_same_picture_however_you_got_there() {
    use motolii::doc::store::{
        Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource,
        LayerTiming, RationalTime,
    };
    use motolii::render::engine::Engine;

    if !testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = testkit::tmp_dir("preview-equals-export-video");
    let clip = dir.join("clip.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-v", "error", "-y", "-f", "lavfi", "-i",
            "testsrc=size=64x64:rate=30:duration=1", "-pix_fmt", "yuv420p",
        ])
        .arg(&clip)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "video fixture failed");

    let build = || {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 30,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File {
                    path: clip.to_str().unwrap().to_owned(),
                    fingerprint: None,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 30),
            },
        })
        .unwrap();
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch { name: Some("clip".into()), ..Default::default() },
        })
        .unwrap();
        doc
    };

    let at = RationalTime::try_new(10, 30).unwrap();

    // 層が描かれなかった時の絵(背景だけ)。これと一致したら「出ていない」。
    let empty_doc = {
        let mut d = Document::new();
        d.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 30,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        d
    };
    let black = Engine::new()
        .unwrap()
        .render_frame(&empty_doc.view(), at)
        .unwrap();

    // 書き出しの一手目 —— 冷えたまま、その時刻を1回だけ頼む。
    let export_doc = build();
    let mut export = Engine::new().unwrap();
    let from_export = export.render_frame(&export_doc.view(), at).unwrap();

    // 窓 —— 実時間で回り続けて、デコーダが温まってから同じ時刻を見る。
    // 温まるのに要るのは呼んだ回数ではなく**実時間**(FFmpeg の起動)。
    let preview_doc = build();
    let mut preview = Engine::new().unwrap();
    let mut from_preview = preview.render_frame(&preview_doc.view(), at).unwrap();
    for _ in 0..150 {
        if from_preview != black {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        from_preview = preview.render_frame(&preview_doc.view(), at).unwrap();
    }
    assert_ne!(from_preview, black, "窓が温まっても絵が出ない");

    assert!(
        export.layer_failures().is_empty(),
        "書き出しで絵が落ちている: {:?}",
        export.layer_failures()
    );
    let differing = from_export
        .iter()
        .zip(&from_preview)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        differing, 0,
        "同じ時刻なのに、辿り着き方で絵が違う({differing}/{} バイト、書き出しは背景のまま={})",
        from_export.len(),
        from_export == black
    );
}

/// 出た絵そのものを基準と突き合わせる。
///
/// 「同じ時刻なら同じ絵」は**辿り方によらない**ことしか見ていない。両方が
/// 同じだけ壊れたら気づけないので、**絵の中身**にも基準を1枚置く。
/// 器具は `testkit`(無ければ作って目視を促し、在れば許容差で比べる)。
#[test]
fn the_first_frame_of_the_fixture_still_looks_like_itself() {
    use motolii::render::engine::Engine;
    use motolii::doc::store::RationalTime;

    // fixture 自身が持っている再生位置。層が重なっている所を選ぶ ——
    // 0秒はほとんど黒で、基準にしても何も捕まえない。
    let fx = motolii::doc::fixture::build();
    let at = RationalTime::try_new(fx.playhead, 30).unwrap();
    let mut engine = Engine::new().unwrap();
    let pixels = engine.render_frame(&fx.doc.view(), at).unwrap();

    let comp = fx.doc.view().composition().unwrap().expect("comp");
    testkit::assert_rgba_matches_golden_file(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/fixture_playhead.png"),
        "fixture の再生位置の1コマ",
        testkit::RgbaImageDesc { width: comp.width, height: comp.height },
        &pixels,
        2,
    );
}
