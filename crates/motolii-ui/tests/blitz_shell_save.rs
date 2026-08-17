//! shell 統合: 保存(Cmd+S の後ろ)と信用の可視化。
//!
//! dialog(`rfd`)はここに出てこない。テストが呼ぶのは dialog の**後ろ**にある
//! 関数だけで、製品も同じ関数を同じ順で呼ぶ(`blitz_shell_file_entry.rs` と同じ形):
//!
//! - 保存      … `ProjectSeat::save`(既存の `ProjectSession::save_document` 経路。
//!               CLI `document_edit.rs` の `with_writer` と同じ書き戻し)
//! - 未保存表示 … `ProjectSeat::is_dirty`(status 帯が毎フレーム見る判定)
//! - 未保存 guard … `decide_unsaved`(Cmd+O / Cmd+N / 窓を閉じる、の前に挟む判断。
//!               dialog は選択を返すだけで、判断はこの関数が持つ)
//! - Undo/Redo ボタン … `TimelineEditor::undo_gesture` / `redo_gesture`
//!               (Cmd+Z / Shift+Cmd+Z と同じ入口。1クリック = 1 gesture 単位)

use std::collections::HashMap;
use std::path::PathBuf;

use motolii_doc::{find_item_location, Document, LayerId, TrackItem};
use motolii_ui::blitz_shell::{decide_unsaved, ProjectSeat, UnsavedChoice, UnsavedDecision};
use motolii_ui::timeline_editor::lab_fixture;

/// 一時projectを1つ作る。経路は `blitz_shell_editor_seat.rs` と同じ
/// (`ProjectSession::acquire` → `save_document`。lock は返して `ProjectSeat::open`
/// が取り直す)。
fn create_project(tag: &str, document: &Document) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-shell-save-{tag}-{nanos}"));
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

/// clip を1本 move する(1ドラッグ = 1 gesture = 1 undo 単位)。
fn move_clip(seat: &mut ProjectSeat, layer: LayerId, grab: f32, to: f32) {
    let editor = seat.editor_mut();
    editor.begin_clip_move(layer, grab);
    editor.drag_to(to);
    editor.release();
}

/// 保存: writer snapshot が project ファイルへ書き戻り、開き直しても編集が残る。
#[test]
fn saving_writes_the_edit_back_and_it_survives_a_reopen() {
    let (document, names) = lab_fixture();
    let path = create_project("roundtrip", &document);
    let layer = layer_named(&names, "Background");

    let mut seat = ProjectSeat::open(&path).expect("open temp project");
    assert_eq!(
        clip_start_seconds(&seat.snapshot(), layer),
        0.0,
        "fixture の Background は 0s 始まり"
    );

    // 1.0s で掴んで 3.0s へ(+2.0s)。編集は writer の中だけ。
    move_clip(&mut seat, layer, 1.0, 3.0);
    seat.save().expect("save writes the snapshot back");

    // 座席を捨てて開き直す — 保存済みなら編集は残る。
    drop(seat);
    let reopened = ProjectSeat::open(&path).expect("reopen the saved project");
    let restored = clip_start_seconds(&reopened.snapshot(), layer);
    assert!(
        (restored - 2.0).abs() < 1e-3,
        "保存した編集が開き直しでも残る: {restored}"
    );
}

/// dirty 判定の遷移: 開く=clean、編集=dirty、保存=clean、
/// そして **undo で保存時の内容へ戻ったら clean に戻る**(内容比較なので)。
#[test]
fn dirty_follows_edits_saves_and_undo_by_content() {
    let (document, names) = lab_fixture();
    let path = create_project("dirty", &document);
    let layer = layer_named(&names, "Background");

    let mut seat = ProjectSeat::open(&path).expect("open temp project");
    assert!(!seat.is_dirty(), "開いた直後は clean");

    move_clip(&mut seat, layer, 1.0, 3.0);
    assert!(seat.is_dirty(), "編集で dirty");

    seat.save().expect("save");
    assert!(!seat.is_dirty(), "保存で clean");

    move_clip(&mut seat, layer, 3.0, 5.0);
    assert!(seat.is_dirty(), "保存後の編集でまた dirty");

    seat.editor_mut().undo().expect("undo");
    assert!(
        !seat.is_dirty(),
        "undo で保存時の内容へ戻ったら clean(revision は進んでいても)"
    );

    seat.editor_mut().redo().expect("redo");
    assert!(seat.is_dirty(), "redo でまた保存内容から離れる = dirty");
}

/// undo 台帳の深さが保存時と同じでも、**内容が違えば dirty**。
/// (保存点より下まで undo → 別の編集、で undo_len だけ見る判定は騙される)
#[test]
fn dirty_is_not_fooled_by_matching_undo_depth() {
    let (document, names) = lab_fixture();
    let path = create_project("depth", &document);
    let layer = layer_named(&names, "Background");

    let mut seat = ProjectSeat::open(&path).expect("open temp project");

    // 編集 A(0→2s)で undo_len=1、そこで保存。
    move_clip(&mut seat, layer, 1.0, 3.0);
    let depth_at_save = seat.editor().undo_len();
    seat.save().expect("save");
    assert!(!seat.is_dirty(), "保存で clean");

    // 保存点より下へ undo(内容は開いた直後の状態 = 保存内容と違う)。
    seat.editor_mut().undo().expect("undo below the save point");
    assert!(seat.is_dirty(), "保存点より下へ戻ったら dirty");

    // 別の編集 B(0→5s)。undo_len は保存時と同じ深さへ戻るが、内容は別物。
    move_clip(&mut seat, layer, 1.0, 6.0);
    assert_eq!(
        seat.editor().undo_len(),
        depth_at_save,
        "前提: 台帳の深さは保存時と同じに見える"
    );
    assert!(
        seat.is_dirty(),
        "深さが同じでも内容が保存と違えば dirty(内容比較)"
    );
}

/// 未保存 guard の判断: dirty でなければ**訊かずに**続行、dirty なら3択が判断を決める。
#[test]
fn the_unsaved_guard_decides_from_dirty_and_the_choice() {
    // clean なら選択関数は呼ばれもしない。
    assert_eq!(
        decide_unsaved(false, || panic!("clean なのに確認を出した")),
        UnsavedDecision::Proceed,
        "clean は確認なしで続行"
    );
    assert_eq!(
        decide_unsaved(true, || UnsavedChoice::Save),
        UnsavedDecision::SaveThenProceed,
        "保存して続行"
    );
    assert_eq!(
        decide_unsaved(true, || UnsavedChoice::Discard),
        UnsavedDecision::Proceed,
        "破棄して続行"
    );
    assert_eq!(
        decide_unsaved(true, || UnsavedChoice::Cancel),
        UnsavedDecision::Stay,
        "キャンセル = いまの座席に留まる"
    );
}

/// Undo/Redo ボタンの操作列: ボタンが呼ぶ入口(`undo_gesture` / `redo_gesture`)は
/// Cmd+Z と同経路・同意味(1クリック = 1 gesture 単位)で、効かない時の
/// disabled 判定は台帳の深さ(`undo_len` / `redo_len`)が根拠になる。
#[test]
fn undo_redo_buttons_step_one_gesture_at_a_time() {
    let (document, names) = lab_fixture();
    let path = create_project("buttons", &document);
    let layer = layer_named(&names, "Background");

    let mut seat = ProjectSeat::open(&path).expect("open temp project");
    assert_eq!(
        seat.editor().undo_len(),
        0,
        "開いた直後: Undo は disabled 相当"
    );
    assert_eq!(
        seat.editor().redo_len(),
        0,
        "開いた直後: Redo は disabled 相当"
    );

    // 1ドラッグ = 1 gesture。ボタン1クリックで丸ごと戻る。
    move_clip(&mut seat, layer, 1.0, 3.0);
    assert_eq!(seat.editor().undo_len(), 1, "編集で Undo が enabled になる");

    seat.editor_mut().undo_gesture();
    assert!(
        (clip_start_seconds(&seat.snapshot(), layer)).abs() < 1e-3,
        "Undo ボタン1回でドラッグ丸ごと戻る"
    );
    assert_eq!(
        seat.editor().undo_len(),
        0,
        "戻したら Undo は disabled 相当"
    );
    assert_eq!(seat.editor().redo_len(), 1, "Redo が enabled になる");

    seat.editor_mut().redo_gesture();
    assert!(
        (clip_start_seconds(&seat.snapshot(), layer) - 2.0).abs() < 1e-3,
        "Redo ボタン1回でドラッグ丸ごと再適用"
    );
    assert_eq!(
        seat.editor().redo_len(),
        0,
        "使い切ったら Redo は disabled 相当"
    );

    // 効かない時に押されても落ちない(disabled 表示の裏で安全)。
    seat.editor_mut().redo_gesture();
    assert!(
        (clip_start_seconds(&seat.snapshot(), layer) - 2.0).abs() < 1e-3,
        "空の Redo は何もしない"
    );
}
