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
use motolii_ui::blitz_shell::{admit_dropped_paths, create_project_file, reseat_project, ProjectSeat};

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
