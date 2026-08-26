//! 人がファイルを入れる2つの入口(OS ドロップ / New・Open)を、**dialog を開かずに**
//! 判定する。
//!
//! dialog(`rfd`)は窓と人の手を要求するのでここには出てこない。テストが呼ぶのは
//! dialog の**後ろ**にある関数だけで、製品も同じ関数を同じ順で呼ぶ:
//!
//! - ドロップ … `admit_dropped_paths` → `TimelineEditor::import_dropped_media`
//!   → probe → `AssetDraft::from_probed_source` → `prepare_admit_asset`
//!   → `prepare_place_asset_clip`(CLI `document_edit.rs` と同じ列)
//! - New    … `create_project_file`(CLI `new_document` と同じ意味)
//! - Open   … `reseat_project`(旧座席の lock を返す順もここで見る)

use std::path::{Path, PathBuf};

use motolii_doc::{Document, TrackItem};
use motolii_testkit::ffmpeg_or_skip;
use motolii_ui::blitz_shell::{
    admit_dropped_paths, create_project_file, reseat_project, ProjectSeat,
};

/// 使い捨ての作業 dir。project も、落とすファイルもここへ置く。
fn work_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-file-entry-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// repo に入っている実 media(starter kit)。ffprobe が要る。
fn starter_clip() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/mocks-ui/starter-media/media/starter-clip.mp4")
        .canonicalize()
        .expect("starter clip lives in the repo")
}

/// repo に入っている実 audio(starter kit)。曲を貼る側の入口で使う。
fn starter_tone() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/mocks-ui/starter-media/media/starter-tone.wav")
        .canonicalize()
        .expect("starter tone lives in the repo")
}

/// 落とすための静止画1枚を作る(repo に置かず、その場で作って捨てる)。
fn make_still_image(path: &Path) {
    let status = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-y"])
        .args(["-f", "lavfi", "-i", "color=c=red:s=64x48:d=0.04"])
        .args(["-frames:v", "1"])
        .arg(path)
        .status()
        .expect("spawn ffmpeg");
    assert!(
        status.success(),
        "ffmpeg failed to write {}",
        path.display()
    );
}

fn clips(document: &Document) -> Vec<&motolii_doc::Clip> {
    document
        .tracks
        .iter()
        .flat_map(|track| track.items.iter())
        .filter_map(|item| match item {
            TrackItem::Clip(clip) => Some(clip),
            _ => None,
        })
        .collect()
}

/// New: 空コンポ + V1 トラック 1 本。CLI の `new_document` と同じ中身で、
/// そのまま開ける(= `--project` 起動と同じ状態になる)。
#[test]
fn a_new_project_opens_as_an_empty_composition_with_one_track() {
    let dir = work_dir("new");
    let path = dir.join("untitled.json");
    create_project_file(&path).expect("create a new project");

    let seat = ProjectSeat::open(&path).expect("the new project opens");
    let document = seat.snapshot();
    assert_eq!(document.tracks.len(), 1, "受け皿のトラックが1本ある");
    assert!(document.tracks[0].items.is_empty(), "中身は空");
    assert_eq!(document.assets.len(), 0, "素材もまだ無い");
    assert_eq!(seat.path(), path, "座席は開いた project を憶えている");
}

/// New は既にあるファイルを踏まない(CLI と同じ)。
#[test]
fn a_new_project_refuses_to_overwrite_an_existing_file() {
    let dir = work_dir("new-exists");
    let path = dir.join("untitled.json");
    create_project_file(&path).expect("first create");
    assert!(
        create_project_file(&path).is_err(),
        "既にある project を黙って踏み潰さない"
    );
}

/// ドロップ: probe → import → **playhead の位置**へ clip。
#[test]
fn dropping_media_imports_it_and_places_a_clip_at_the_playhead() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = work_dir("drop");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create a new project");
    let mut seat = ProjectSeat::open(&path).expect("open");

    // playhead を 0 でない所へ置く。**そこが着地点**であることを見たいので。
    seat.editor_mut().set_playhead_seconds(2.0);
    let status = admit_dropped_paths(Some(&mut seat), &[starter_clip()]);

    let document = seat.snapshot();
    assert_eq!(document.assets.len(), 1, "素材が1つ入った");
    let clips = clips(&document);
    assert_eq!(clips.len(), 1, "clip が1つ置かれた");
    assert!(
        (clips[0].start.as_seconds_f64() - 2.0).abs() < 1e-3,
        "clip は playhead の位置に立つ: {}",
        clips[0].start.as_seconds_f64()
    );
    assert_eq!(seat.editor().undo_len(), 1, "1本のドロップ = 1 Undo 単位");
    assert!(
        status.contains("starter-clip"),
        "何が入ったかを言う: {status}"
    );
}

/// **画像のドロップも clip になる**(2026-08-18 レーンA)。
/// [初回タッチ観察](../../../docs/reviews/2026-08-18-user-first-touch-observations.md)(2)
/// では、ここで理由つきの skip になっていた。
#[test]
fn dropping_a_still_image_places_a_clip_like_any_other_media() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = work_dir("drop-image");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create a new project");
    let still = dir.join("hero.png");
    make_still_image(&still);

    let mut seat = ProjectSeat::open(&path).expect("open");
    let outcome = seat.editor_mut().import_dropped_media(&[still.clone()]);

    assert!(
        outcome.skipped.is_empty(),
        "画像はもう飛ばされない: {:?}",
        outcome.skipped
    );
    assert_eq!(outcome.placed.len(), 1, "画像も clip として着地する");

    let document = seat.snapshot();
    assert_eq!(document.assets.len(), 1, "素材が1つ入った");
    let asset = document.assets.iter().next().expect("the admitted asset");
    assert_eq!(asset.asset_type, "image/png");
    let clips = clips(&document);
    assert_eq!(clips.len(), 1, "clip が1つ置かれた");
    assert!(
        document.soundtrack.is_none(),
        "画像は曲にならない(音声だけの分岐)"
    );
    assert_eq!(seat.editor().undo_len(), 1, "1本のドロップ = 1 Undo 単位");
}

// ---------------------------------------------------------------------------
// 曲を貼る(音声ドロップ)
// ---------------------------------------------------------------------------

/// **まだ曲が無い project へ音声を落としたら、それが曲になる。**
/// clip は増えない(帯が即出るのが人の期待 — CapCut / Ableton と同じ既定)。
#[test]
fn dropping_audio_into_a_project_without_a_soundtrack_makes_it_the_soundtrack() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = work_dir("soundtrack");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create a new project");
    let mut seat = ProjectSeat::open(&path).expect("open");

    let status = admit_dropped_paths(Some(&mut seat), &[starter_tone()]);

    let document = seat.snapshot();
    let soundtrack = document.soundtrack.expect("音声は曲として貼られる");
    assert_eq!(document.assets.len(), 1, "素材が1つ入った");
    let asset = document
        .assets
        .get(soundtrack.asset)
        .expect("曲は取り込んだ素材そのものを指す");
    assert!(
        asset.asset_type.starts_with("audio/"),
        "曲になったのは音声素材: {}",
        asset.asset_type
    );
    assert_eq!(
        soundtrack.start_offset.as_seconds_f64(),
        0.0,
        "頭から鳴る(offset の UI はまだ無い)"
    );
    assert_eq!(soundtrack.master_gain(), 1.0, "音量はそのまま");
    assert!(clips(&document).is_empty(), "clip は増えない");
    // **貼った瞬間に帯が出る。** 帯も再生も同じ cached snapshot を読むので、
    // 高さが 0 でない = 新しい曲が拾えている(取り直しの合図は revision)
    assert!(
        seat.editor().waveform_band_height() > 0.0,
        "波形帯がその場で出る: {}",
        seat.editor().waveform_band_height()
    );
    assert_eq!(seat.editor().undo_len(), 1, "1本のドロップ = 1 Undo 単位");
    assert!(
        status.contains("soundtrack") && status.contains("starter-tone"),
        "どちらになったかを一言で言う: {status}"
    );
}

/// **既に曲がある所へ落ちた2本目の音声は clip になる。** 曲は黙って差し替わらない。
#[test]
fn dropping_a_second_audio_file_places_a_clip_and_keeps_the_soundtrack() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = work_dir("soundtrack-second");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create a new project");
    let mut seat = ProjectSeat::open(&path).expect("open");

    admit_dropped_paths(Some(&mut seat), &[starter_tone()]);
    let first = seat.snapshot().soundtrack.expect("1本目が曲になっている");

    let status = admit_dropped_paths(Some(&mut seat), &[starter_tone()]);

    let document = seat.snapshot();
    assert_eq!(
        document.soundtrack.expect("曲は残る").asset,
        first.asset,
        "2本目は曲を差し替えない"
    );
    assert_eq!(clips(&document).len(), 1, "2本目は clip として置かれる");
    assert_eq!(seat.editor().undo_len(), 2, "ドロップ2回 = Undo 2段");
    assert!(status.contains("placed"), "clip 側の文言に戻る: {status}");
}

/// **動画は曲が無くても従来どおり clip。** 音声だけの分岐である。
#[test]
fn dropping_video_still_places_a_clip_even_without_a_soundtrack() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = work_dir("soundtrack-video");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create a new project");
    let mut seat = ProjectSeat::open(&path).expect("open");

    admit_dropped_paths(Some(&mut seat), &[starter_clip()]);

    let document = seat.snapshot();
    assert!(document.soundtrack.is_none(), "動画は曲にならない");
    assert_eq!(clips(&document).len(), 1, "clip として置かれる");
}

/// **Undo 1回で曲ごと戻る。** 取り込みと曲付けは同じ1 gesture なので、
/// 素材だけ台帳に残る中途半端を作らない。
#[test]
fn undoing_a_soundtrack_drop_takes_the_asset_back_out_too() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = work_dir("soundtrack-undo");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create a new project");
    let mut seat = ProjectSeat::open(&path).expect("open");

    admit_dropped_paths(Some(&mut seat), &[starter_tone()]);
    assert!(seat.snapshot().soundtrack.is_some(), "貼れている前提");

    seat.editor_mut().undo().expect("Undo は1回で戻る");

    let document = seat.snapshot();
    assert!(document.soundtrack.is_none(), "曲が外れる");
    assert_eq!(document.assets.len(), 0, "素材も一緒に戻る");
    assert_eq!(seat.editor().undo_len(), 0, "台帳も空に戻る");
}

/// probe できないファイルは**理由つきで飛ばす**。Document は触らない。
#[test]
fn an_unprobeable_file_is_skipped_with_a_reason() {
    let dir = work_dir("skip");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create a new project");
    let note = dir.join("notes.txt");
    std::fs::write(&note, b"not media").expect("write note");

    let mut seat = ProjectSeat::open(&path).expect("open");
    let outcome = seat.editor_mut().import_dropped_media(&[note.clone()]);

    assert!(outcome.placed.is_empty(), "何も置かれない");
    assert_eq!(outcome.skipped.len(), 1, "飛ばした1本を数える");
    assert_eq!(outcome.skipped[0].0, note);
    assert!(
        !outcome.skipped[0].1.is_empty(),
        "飛ばした理由が空でない: {:?}",
        outcome.skipped[0]
    );
    let document = seat.snapshot();
    assert_eq!(document.assets.len(), 0, "Document は動かない");
    assert_eq!(seat.editor().undo_len(), 0, "undo 台帳にも積まない");
    assert!(
        seat.editor().status().contains("notes.txt"),
        "status に飛ばしたファイルが出る: {}",
        seat.editor().status()
    );
}

/// 良いものと悪いものを一度に落としたら、**良いほうは通る**。
#[test]
fn a_mixed_drop_places_what_it_can_and_skips_the_rest() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = work_dir("mixed");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create a new project");
    let note = dir.join("notes.txt");
    std::fs::write(&note, b"not media").expect("write note");

    let mut seat = ProjectSeat::open(&path).expect("open");
    let outcome = seat
        .editor_mut()
        .import_dropped_media(&[note.clone(), starter_clip()]);

    assert_eq!(outcome.placed.len(), 1, "動画のほうは置かれる");
    assert_eq!(outcome.skipped.len(), 1, "テキストのほうは飛ぶ");
    assert_eq!(clips(&seat.snapshot()).len(), 1);
}

/// project が開いていない所へ落としたら、**先に project を作る/開く**と案内する。
#[test]
fn dropping_without_a_project_asks_for_one() {
    let status = admit_dropped_paths(None, &[starter_clip()]);
    assert!(
        status.contains("Cmd+N") && status.contains("Cmd+O"),
        "作る/開く口を名指しで案内する: {status}"
    );
}

/// Open(座席の差し替え): **別の project なら旧座席を生かしたまま開き**、
/// 通ってから旧 session を落とす。落ちた旧 session の lock は返る。
#[test]
fn reseating_swaps_the_project_and_releases_the_old_lock() {
    let dir = work_dir("reseat");
    let first = dir.join("first.json");
    let second = dir.join("second.json");
    create_project_file(&first).expect("first");
    create_project_file(&second).expect("second");

    let mut seat = Some(ProjectSeat::open(&first).expect("open first"));
    reseat_project(&mut seat, &second).expect("reseat to the second project");
    assert_eq!(
        seat.as_ref().expect("seated").path(),
        second,
        "座席は second を指す"
    );

    // 旧 session の lock が返っている(= 誰でも first を開き直せる)。
    ProjectSeat::open(&first).expect("the first project is no longer locked");
}

/// 同じ project を開き直せる(**先に旧 session を落として lock を返す**)。
#[test]
fn reseating_the_same_project_reopens_it() {
    let dir = work_dir("reseat-same");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create");

    let mut seat = Some(ProjectSeat::open(&path).expect("open"));
    reseat_project(&mut seat, &path).expect("同じ project を開き直せる");
    assert_eq!(seat.as_ref().expect("seated").path(), path);
}

/// 開けない project を選んでも**今の席は失われない**(理由だけ返る)。
#[test]
fn reseating_to_an_unopenable_project_keeps_the_current_seat() {
    let dir = work_dir("reseat-bad");
    let path = dir.join("project.json");
    create_project_file(&path).expect("create");
    let missing = dir.join("nope").join("project.json");

    let mut seat = Some(ProjectSeat::open(&path).expect("open"));
    let error = reseat_project(&mut seat, &missing).expect_err("開けない");
    assert!(!error.is_empty(), "理由を言う");
    assert_eq!(
        seat.as_ref().expect("seated").path(),
        path,
        "今の席はそのまま"
    );
}
