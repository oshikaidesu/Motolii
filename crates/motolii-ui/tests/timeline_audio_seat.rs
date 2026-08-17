//! 再生 audio 座席の配線: `--project` の座席(`ProjectSeat`)が project root
//! (document path の親)を Timeline エディタへ渡すこと。
//!
//! root は soundtrack の asset path 解決(`resolve_asset_path`)に使われ、
//! これが無いと相対 path の soundtrack を持つ project が「鳴らせない」へ落ちる。
//! clock の同期そのもののテストは `timeline_editor::audio_seat` の unit 側にある
//! (headless transport fixture で判定。実 device は soft gate)。

use std::path::PathBuf;

use motolii_doc::Document;
use motolii_ui::blitz_shell::ProjectSeat;
use motolii_ui::timeline_editor::lab_fixture;

/// 一時projectを1つ作る(`blitz_shell_editor_seat.rs` と同じ経路)。
fn create_project(tag: &str, document: &Document) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-audio-seat-{tag}-{nanos}"));
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

/// seat が document path の親を editor の project root として座らせる。
#[test]
fn the_seat_hands_the_project_root_to_the_editor() {
    let (document, _names) = lab_fixture();
    let path = create_project("root", &document);
    let seat = ProjectSeat::open(&path).expect("open temp project");
    assert_eq!(
        seat.editor().project_root(),
        path.parent(),
        "root は document path の親(CLI export と同じ規約)"
    );
}
