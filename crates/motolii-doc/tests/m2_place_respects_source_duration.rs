//! place は素材の長さを尊重する — red 先行。
//!
//! [実走観察(2026-08-18)](../../../docs/reviews/2026-08-18-first-real-run-observations.md)(1):
//! 現行の `prepare_place_asset_clip` は尺を **start→composition end** で切る
//! (CU-101 の設計)。source が 4s でも 10s の clip が立ち、終端はフリーズフレームに
//! なる。普通のエディタの期待は「clip は source の長さで立つ」。
//!
//! 契約(このテストが定義する):
//! - `AssetDraft` は `duration: Option<RationalTime>` を運ぶ(probe が知っている値。
//!   `from_probed_source` の署名は変えず、既定 `None` の public field)
//! - admission で Document の `Asset` にも同じ optional duration が残る
//!   (無いなら書き出さない = 旧 reader は未知キー往復、locators と同じ互換方針)
//! - `prepare_place_asset_clip` の尺は `min(source duration, comp end - start)`。
//!   duration 不明の asset は従来どおり comp end まで

use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_doc::{
    AssetDraft, AssetId, Command, Document, DocumentWriter, SourceFingerprintV1, Track, TrackItem,
};

fn seconds(value: i64) -> RationalTime {
    RationalTime::try_new(value, 1).unwrap()
}

/// 空コンポ + V1 トラック(CLI `new` と同じ形)の writer。
fn writer_with_track() -> DocumentWriter {
    let mut document = Document::new_current();
    let track = document.track_ids.allocate("V1").unwrap();
    document.tracks.push(Track {
        id: track,
        items: vec![],
    });
    let catalog = Arc::new(motolii_plugin::reference::reference_catalog().unwrap());
    DocumentWriter::new(document, catalog).unwrap()
}

/// duration 付き(=probe 済み)の video asset を1つ取り込む。
fn admit_video(writer: &mut DocumentWriter, duration: Option<RationalTime>) -> AssetId {
    let fp = SourceFingerprintV1::from_reader(Cursor::new(b"pretend media bytes")).unwrap();
    let mut draft = AssetDraft::from_probed_source(
        "video/mp4",
        &fp,
        Path::new("/media/proj/clips/intro.mp4"),
        Some(Path::new("/media/proj")),
    );
    draft.duration = duration;
    let admit = writer.prepare_admit_asset(draft).unwrap();
    let id = admit.asset().id;
    let gesture = writer.begin_gesture();
    writer.apply_prepared_asset_admission(gesture, admit).unwrap();
    id
}

fn placed_clip_duration(command: &Command) -> RationalTime {
    match command {
        Command::AddTrackItem {
            item: TrackItem::Clip(clip),
            ..
        } => clip.duration,
        other => panic!("expected AddTrackItem(Clip), got {other:?}"),
    }
}

#[test]
fn a_probed_duration_survives_admission_into_the_document_asset() {
    let mut writer = writer_with_track();
    let id = admit_video(&mut writer, Some(seconds(4)));
    let snapshot = writer.snapshot();
    let asset = snapshot.assets.get(id).unwrap();
    assert_eq!(
        asset.duration,
        Some(seconds(4)),
        "probe が測った source の長さは Document の Asset に残る"
    );
}

#[test]
fn place_cuts_the_clip_at_the_source_end_not_the_composition_end() {
    let mut writer = writer_with_track();
    let id = admit_video(&mut writer, Some(seconds(4)));
    // 前提の明示: 既定コンポは 10s。source(4s)より長い。
    assert_eq!(writer.snapshot().composition.duration, seconds(10));

    let command = writer.prepare_place_asset_clip(id, seconds(0)).unwrap();
    assert_eq!(
        placed_clip_duration(&command),
        seconds(4),
        "source 4s の clip は 4s で立つ(comp end までのフリーズフレーム尾を作らない)"
    );
}

#[test]
fn place_near_the_composition_end_still_clamps_to_what_remains() {
    let mut writer = writer_with_track();
    let id = admit_video(&mut writer, Some(seconds(4)));

    let command = writer.prepare_place_asset_clip(id, seconds(8)).unwrap();
    assert_eq!(
        placed_clip_duration(&command),
        seconds(2),
        "comp の残り(10-8=2s)が source(4s)より短ければ残りで切る"
    );
}

#[test]
fn an_asset_without_a_known_duration_keeps_the_old_comp_end_behavior() {
    let mut writer = writer_with_track();
    let id = admit_video(&mut writer, None);

    let command = writer.prepare_place_asset_clip(id, seconds(0)).unwrap();
    assert_eq!(
        placed_clip_duration(&command),
        seconds(10),
        "長さ不明(生成系・stream)は従来どおり comp end まで"
    );
}
