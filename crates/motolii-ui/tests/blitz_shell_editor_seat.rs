//! shell 統合: `--project` の座席(`ProjectSeat`)で開いた実プロジェクトを、
//! Timeline エディタの操作 API が**唯一の writer** 経由で編集できることを確かめる。
//!
//! 一時 project を開き、lab のテストと同じ入力シミュレーション(掴む → 動かす →
//! 離す)で move を1回適用する。writer の document が変化して undo 台帳が伸び、
//! undo で元へ戻ること、そして seat が配る snapshot が編集後の Document を
//! 指し直すことを見る。ここに第二の writer も Document clone 編集も出てこない。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use motolii_doc::{find_item_location, Document, LayerId, TrackItem};
use motolii_ui::blitz_shell::ProjectSeat;
use motolii_ui::timeline_editor::lab_fixture;

/// 一時projectを1つ作る。経路は `examples/create_timeline_lab_project.rs` と同じ
/// (`ProjectSession::acquire` → `save_document`)。lock は返して `ProjectSeat::open`
/// が取り直す。
fn create_project(tag: &str, document: &Document) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-editor-seat-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("temp project dir");
    let path = dir.join("project.json");
    let mut session =
        motolii_doc::ProjectSession::acquire(&path, &motolii_doc::ResourceLimits::production())
            .expect("acquire temp project");
    session
        .save_document(document, &motolii_doc::SaveOptions::default())
        .expect("save temp project");
    drop(session);
    path
}

fn layer_named(names: &HashMap<LayerId, String>, want: &str) -> LayerId {
    *names
        .iter()
        .find(|(_, name)| name.as_str() == want)
        .map(|(layer, _)| layer)
        .expect("fixture layer")
}

fn clip_start_seconds(document: &Document, layer: LayerId) -> f64 {
    let (_, _, item) = find_item_location(document, layer).expect("layer lives in the document");
    match item {
        TrackItem::Clip(clip) => clip.start.as_seconds_f64(),
        other => panic!("expected a clip, found {other:?}"),
    }
}

/// エディタの move が seat の writer を通り、undo で戻る。
#[test]
fn seated_editor_moves_a_clip_through_the_single_writer_and_undoes_it() {
    let (document, names) = lab_fixture();
    let path = create_project("move", &document);
    let layer = layer_named(&names, "Background");

    let mut seat = ProjectSeat::open(&path).expect("open temp project");
    let before_snapshot = seat.snapshot();
    let before_start = clip_start_seconds(&before_snapshot, layer);
    assert_eq!(before_start, 0.0, "fixture の Background は 0s 始まり");

    let editor = seat.editor_mut();
    let undo_before = editor.undo_len();
    let revision_before = editor.revision();

    // 入力シミュレーション: 1.0s の位置で掴み、3.0s へ動かして離す。
    // lab のテスト(`moving_a_clip_carries_its_position_keys`)と同じ経路で、
    // マウスの代わりに操作 API を呼ぶだけ。
    editor.begin_clip_move(layer, 1.0);
    editor.drag_to(3.0); // +2.0s
    editor.release();

    let after = editor.document();
    let after_start = clip_start_seconds(after, layer);
    assert!(
        (after_start - 2.0).abs() < 1e-3,
        "move が writer を通って document を変えた: {after_start}"
    );
    assert_eq!(
        editor.undo_len(),
        undo_before + 1,
        "1ドラッグ = 1 Undo 単位で台帳が伸びる"
    );
    assert!(
        editor.revision() > revision_before,
        "編集で writer の revision が進む"
    );

    // seat が配る snapshot も編集後の Document を指し直す(Stage が映すのはこれ)。
    let seated = seat.snapshot();
    assert!(
        (clip_start_seconds(&seated, layer) - 2.0).abs() < 1e-3,
        "seat の snapshot が編集後を映す"
    );

    let editor = seat.editor_mut();
    editor.undo().expect("undo");
    let restored = clip_start_seconds(editor.document(), layer);
    assert!(
        (restored - before_start).abs() < 1e-3,
        "undo で掴む前へ戻る: {restored}"
    );
}

/// エディタが持つ snapshot は writer の出した Arc そのもの(clone 編集ではない)。
#[test]
fn seated_editor_snapshot_is_the_writer_snapshot_itself() {
    let (document, _names) = lab_fixture();
    let path = create_project("snapshot", &document);
    let mut seat = ProjectSeat::open(&path).expect("open temp project");

    let served = seat.snapshot();
    let editor = seat.editor_mut();
    assert!(
        Arc::ptr_eq(&served, editor.document()),
        "seat は editor の cached snapshot をそのまま配る"
    );
}
