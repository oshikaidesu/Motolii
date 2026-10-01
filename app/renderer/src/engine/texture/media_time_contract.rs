use motolii_edit::{Document, Intent};
use super::*;
use crate::doc::store::{
    Composition, Fps, LayerId, LayerMeta, LayerSource, LayerTiming,
    RationalTime,
};
use crate::render::engine::Engine;

fn ffmpeg_available() -> bool {
    crate::render::media::test_encoders_available(&["libx264"])
}

/// 24fps で「1 秒黒、1 秒白」の動画を作る。
fn black_then_white_24fps(dir: &std::path::Path) -> std::path::PathBuf {
    let clip = dir.join("black_then_white_24.mp4");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args([
            "-v", "error", "-y",
            "-f", "lavfi", "-i", "color=c=black:s=64x64:r=24:d=1",
            "-f", "lavfi", "-i", "color=c=white:s=64x64:r=24:d=1",
            "-filter_complex", "[0][1]concat=n=2:v=1:a=0",
            "-pix_fmt", "yuv420p", "-c:v", "libx264",
        ])
        .arg(&clip)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "video fixture failed");
    clip
}

fn center_pixel(engine: &mut Engine, doc: &Document, frame: i64) -> [u8; 3] {
    let at = RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap();
    let rgba = engine.render_frame(&doc.view(), at).unwrap();
    assert!(
        engine.layer_failures().is_empty(),
        "frame {frame}: {:?}",
        engine.layer_failures()
    );
    let i = ((32 * 64) + 32) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

#[test]
fn a_24fps_clip_in_a_30fps_comp_keeps_its_own_speed() {
    if !ffmpeg_available() {
        eprintln!("skip: ffmpeg not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let clip = black_then_white_24fps(dir.path());

    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        // 層が出なかった時に黒と見分けるため、背景は赤。
        background: [1.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None },
            order: 0,
            timing: LayerTiming::place(0, Some(60), 60),
        },
    })
    .unwrap();

    let mut engine = Engine::new().unwrap();
    // 0.9 秒: 素材でも 0.9 秒なので黒。fps を取り違えると 27/24 = 1.125 秒で白になる。
    let at_0_9 = center_pixel(&mut engine, &doc, 27);
    // 1.1 秒: 白。
    let at_1_1 = center_pixel(&mut engine, &doc, 33);
    assert!(at_0_9[0] < 40 && at_0_9[1] < 40 && at_0_9[2] < 40, "0.9s should be black, got {at_0_9:?}");
    assert!(at_1_1[0] > 200 && at_1_1[1] > 200 && at_1_1[2] > 200, "1.1s should be white, got {at_1_1:?}");
}

/// 可変フレームレートの素材も第一線。24fps の黒 1 秒 + 60fps の白 1 秒を 1 本にし、0.9 秒が黒、1.1 秒が白。
#[test]
fn a_variable_frame_rate_clip_is_placed_by_time() {
    if !ffmpeg_available() {
        eprintln!("skip: ffmpeg not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let clip = dir.path().join("vfr.mp4");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args([
            "-v", "error", "-y",
            "-f", "lavfi", "-i", "color=c=black:s=64x64:r=24:d=1",
            "-f", "lavfi", "-i", "color=c=white:s=64x64:r=60:d=1",
            "-filter_complex", "[0][1]concat=n=2:v=1:a=0",
            "-fps_mode", "vfr", "-pix_fmt", "yuv420p", "-c:v", "libx264",
        ])
        .arg(&clip)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "video fixture failed");
    let info = crate::render::media::probe(&clip).expect("VFR must be admitted");
    assert!(info.nb_frames.is_some_and(|n| n == 84), "24 + 60 frames, got {:?}", info.nb_frames);

    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [1.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None },
            order: 0,
            timing: LayerTiming::place(0, Some(60), 60),
        },
    })
    .unwrap();
    let mut engine = Engine::new().unwrap();
    let at_0_9 = center_pixel(&mut engine, &doc, 27);
    let at_1_1 = center_pixel(&mut engine, &doc, 33);
    assert!(at_0_9.iter().all(|c| *c < 40), "0.9s should be black, got {at_0_9:?}");
    assert!(at_1_1.iter().all(|c| *c > 200), "1.1s should be white, got {at_1_1:?}");
}

/// 上の不透明な動画に丸ごと覆われた動画は復号も描画もしない。半透明なら両方描く。
#[test]
fn a_video_hidden_behind_an_opaque_video_is_not_drawn() {
    if !ffmpeg_available() {
        eprintln!("skip: ffmpeg not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let clip = |name: &str, color: &str| {
        let path = dir.path().join(name);
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", &format!("color=c={color}:s=64x64:r=30:d=1"), "-pix_fmt", "yuv420p", "-c:v", "libx264"])
            .arg(&path)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success());
        path
    };
    let below = clip("below.mp4", "black");
    let above = clip("above.mp4", "white");

    let build = |top_opacity: Option<f64>| {
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 30, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
        for (id, path, order) in [(1, &below, 0), (2, &above, 1)] {
            let layer = LayerId(id);
            doc.apply(Intent::AddLayer(layer)).unwrap();
            doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_str().unwrap().to_owned(), fingerprint: None }, order, timing: LayerTiming::place(0, Some(30), 30) } }).unwrap();
        }
        if let Some(opacity) = top_opacity {
            doc.apply(Intent::SetConstant { layer: LayerId(2), property: crate::doc::store::PropertyId::new(crate::doc::store::property::OPACITY).unwrap(), value: crate::doc::eval::Value::F64(opacity) }).unwrap();
        }
        doc
    };

    let mut engine = Engine::new().unwrap();
    let opaque = build(None);
    // 1 コマ目は寸を知るために両方開く。2 コマ目から隠れが効く。
    let _ = center_pixel(&mut engine, &opaque, 1);
    let px = center_pixel(&mut engine, &opaque, 2);
    assert!(px.iter().all(|c| *c > 200), "the white layer on top should show, got {px:?}");
    assert_eq!(engine.drawn_layers(), 1, "the covered layer must not be drawn");

    let translucent = build(Some(0.5));
    let _ = center_pixel(&mut engine, &translucent, 1);
    let _ = center_pixel(&mut engine, &translucent, 2);
    assert_eq!(engine.drawn_layers(), 2, "a translucent layer hides nothing");
}

/// 30 コマ目から始まる層は、再生中にその瞬間から描かれる。先読みが復号器を開いておくから。
#[test]
fn a_layer_that_starts_later_is_drawn_from_its_first_frame_while_playing() {
    if !ffmpeg_available() {
        eprintln!("skip: ffmpeg not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let clip = dir.path().join("late.mp4");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=1", "-pix_fmt", "yuv420p", "-c:v", "libx264"])
        .arg(&clip)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success());
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(30, Some(30), 90) } }).unwrap();

    let fps = Fps::try_new(30, 1).unwrap();
    let mut engine = Engine::new().unwrap();
    engine.set_realtime(true);
    // 再生中の型: 描いたら先を温める。層が始まる 15 コマ前から温まり始める。
    for frame in 0..30 {
        let t = RationalTime::try_from_frame(frame, fps).unwrap();
        let _ = engine.render_frame(&doc.view(), t).unwrap();
        engine.warm_upcoming(&doc.view(), t).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    let at_start = center_pixel(&mut engine, &doc, 30);
    assert_eq!(engine.drawn_layers(), 1, "the layer must be drawn on its first frame");
    assert!(at_start.iter().all(|c| *c > 200), "first frame should already be white, got {at_start:?}");
}

/// bt2020 HLG(iPhone の HDR)の素材も断らない。白は白、赤は赤のまま出る。
#[test]
fn hdr_bt2020_clips_are_admitted_and_shown() {
    if !ffmpeg_available() {
        eprintln!("skip: ffmpeg not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let clip = dir.path().join("hlg.mp4");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args([
            "-v", "error", "-y",
            "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=0.5",
            "-f", "lavfi", "-i", "color=c=red:s=64x64:r=30:d=0.5",
            "-filter_complex", "[0][1]concat=n=2:v=1:a=0",
            "-pix_fmt", "yuv420p", "-c:v", "libx264",
            "-colorspace", "bt2020nc", "-color_primaries", "bt2020", "-color_trc", "arib-std-b67", "-color_range", "tv",
        ])
        .arg(&clip)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success());
    crate::render::media::probe(&clip).expect("HDR must be admitted");
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 30, background: [0.0, 1.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, Some(30), 30) } }).unwrap();
    let mut engine = Engine::new().unwrap();
    let white = center_pixel(&mut engine, &doc, 7);
    let red = center_pixel(&mut engine, &doc, 22);
    assert!(white.iter().all(|c| *c >= 240), "white should stay white, got {white:?}");
    assert!(red[0] >= 200 && red[1] <= 60 && red[2] <= 60, "red should stay red, got {red:?}");
}

/// 音だけの層(mp3)は絵を持たず、失敗にもならない。動画の層と並べても再生は止まらない。
#[test]
fn an_audio_only_layer_is_silent_on_stage_and_not_a_failure() {
    if !crate::render::media::test_encoders_available(&["libx264", "libmp3lame"]) {
        eprintln!("skip: ffmpeg encoders missing");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let song = dir.path().join("song.mp3");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "sine=frequency=440:sample_rate=48000:duration=1", "-c:a", "libmp3lame"])
        .arg(&song).status().unwrap();
    assert!(status.success());
    let clip = dir.path().join("clip.mp4");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=1", "-pix_fmt", "yuv420p", "-c:v", "libx264"])
        .arg(&clip).status().unwrap();
    assert!(status.success());
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 30, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
    for (id, path) in [(1, &clip), (2, &song)] {
        let layer = LayerId(id);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_str().unwrap().to_owned(), fingerprint: None }, order: id as i16, timing: LayerTiming::place(0, Some(30), 30) } }).unwrap();
    }
    let mut engine = Engine::new().unwrap();
    let px = center_pixel(&mut engine, &doc, 5);
    assert!(px.iter().all(|c| *c > 200), "the video still shows, got {px:?}");
    assert_eq!(engine.drawn_layers(), 1);
}

/// 一度描いたコマは cache に入り、2 周目は復号器に聞かない。上限を超えると使われてから古い物が消える。
#[test]
fn frames_seen_once_come_from_the_cache_within_the_budget() {
    if !ffmpeg_available() {
        eprintln!("skip: ffmpeg not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let clip = dir.path().join("loop.mp4");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=1", "-pix_fmt", "yuv420p", "-c:v", "libx264"])
        .arg(&clip).status().unwrap();
    assert!(status.success());
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 30, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, Some(30), 30) } }).unwrap();

    let mut engine = Engine::new().unwrap();
    for frame in 0..10 {
        let _ = center_pixel(&mut engine, &doc, frame);
    }
    // 写しは次の frame の頭で入るので、直後は最後の 1 枚がまだ待ち。
    let (hits, entries, bytes) = engine.video_frame_cache_stats();
    assert_eq!((hits, entries, bytes), (0, 9, 9 * 64 * 64 * 4), "first pass fills the cache");
    for frame in 0..10 {
        let px = center_pixel(&mut engine, &doc, frame);
        assert!(px.iter().all(|c| *c > 200), "cached frame must look the same, got {px:?}");
    }
    let (hits, entries, _) = engine.video_frame_cache_stats();
    assert_eq!((hits, entries), (10, 10), "second pass hits every frame");

    let mut small = Engine::new().unwrap();
    small.set_video_frame_cache_budget(3 * 64 * 64 * 4);
    for frame in 0..10 {
        let _ = center_pixel(&mut small, &doc, frame);
    }
    let (_, entries, bytes) = small.video_frame_cache_stats();
    assert!(entries <= 3 && bytes <= 3 * 64 * 64 * 4, "budget holds: {entries} entries, {bytes} bytes");
}

/// tv range の bt709 素材。白は 255、黒は 0、赤は赤。range を取り違えると白が 235・黒が 16 になる。
#[test]
fn limited_range_bt709_colors_come_out_right() {
    if !ffmpeg_available() {
        eprintln!("skip: ffmpeg not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let clip = dir.path().join("wbr_709_tv.mp4");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args([
            "-v", "error", "-y",
            "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=0.5",
            "-f", "lavfi", "-i", "color=c=black:s=64x64:r=30:d=0.5",
            "-f", "lavfi", "-i", "color=c=red:s=64x64:r=30:d=0.5",
            "-filter_complex", "[0][1][2]concat=n=3:v=1:a=0",
            "-pix_fmt", "yuv420p", "-c:v", "libx264", "-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709", "-color_range", "tv",
        ])
        .arg(&clip)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "video fixture failed");

    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 45,
        background: [0.0, 1.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None },
            order: 0,
            timing: LayerTiming::place(0, Some(45), 45),
        },
    })
    .unwrap();

    let mut engine = Engine::new().unwrap();
    let white = center_pixel(&mut engine, &doc, 7);
    let black = center_pixel(&mut engine, &doc, 22);
    let red = center_pixel(&mut engine, &doc, 37);
    assert!(white.iter().all(|c| *c >= 250), "white should be 255, got {white:?}");
    assert!(black.iter().all(|c| *c <= 5), "black should be 0, got {black:?}");
    assert!(red[0] >= 240 && red[1] <= 20 && red[2] <= 20, "red should stay red, got {red:?}");
}

/// 1080p H.264、同じ測り方。
#[test]
#[ignore]
fn full_hd_playback_pace() {
    if !ffmpeg_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let clip = dir.path().join("k1.mp4");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=1920x1080:rate=30:duration=3", "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p"])
        .arg(&clip)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success());
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: 1920, height: 1080, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, Some(90), 90) } }).unwrap();
    let mut engine = Engine::new().unwrap();
    let fps = Fps::try_new(30, 1).unwrap();
    let _ = engine.render_frame(&doc.view(), RationalTime::try_from_frame(0, fps).unwrap()).unwrap();
    let started = std::time::Instant::now();
    for frame in 1..61 {
        let _ = engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
    }
    println!("PROBE room=video verdict=1080p-pace ms-per-frame={:.1}", started.elapsed().as_secs_f64() * 1000.0 / 60.0);
}

/// 4K H.264 を順に 60 コマ描いた時の 1 コマの時間。数字を見る物(`cargo test -- --ignored four_k`)。
#[test]
#[ignore]
fn four_k_playback_pace() {
    if !ffmpeg_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let clip = dir.path().join("k4.mp4");
    let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=3840x2160:rate=30:duration=3", "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p"])
        .arg(&clip)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success());
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: 3840, height: 2160, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, Some(90), 90) } }).unwrap();
    let mut engine = Engine::new().unwrap();
    let fps = Fps::try_new(30, 1).unwrap();
    let _ = engine.render_frame(&doc.view(), RationalTime::try_from_frame(0, fps).unwrap()).unwrap();
    let started = std::time::Instant::now();
    for frame in 1..61 {
        let _ = engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
    }
    let per_frame_ms = started.elapsed().as_secs_f64() * 1000.0 / 60.0;
    println!("PROBE room=video verdict=4k-pace ms-per-frame={per_frame_ms:.1} failures={:?}", engine.layer_failures());
}
