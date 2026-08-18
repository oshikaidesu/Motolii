//! 静止画も置ける — red 先行(2026-08-18 レーンA)。
//!
//! [利用者の初回タッチ観察](../../../docs/reviews/2026-08-18-user-first-touch-observations.md)(2):
//! 画像は入口(admission の拡張子表)で拒まれていた。扉を開けても、その先の
//! `prepare_place_asset_clip` が `video/*` と `audio/*` しか知らないままでは
//! 「取り込めるが置けない」という半端が残る。
//!
//! 契約(このテストが定義する):
//! - `image/*` は **VideoComponent の列**で置ける(静止画は絵であって音ではない)
//! - 尺は既存の「長さ不明」の意味そのまま = start から composition の残り全部。
//!   静止画に尺は無いので、これは新しい意味ではなく既に決まっている意味の適用
//! - 置いた clip は音を持たない(`audio` は空)

use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_doc::{
    AssetDraft, AssetId, Clip, ClipSource, Command, Document, DocumentWriter, SourceFingerprintV1,
    Track, TrackItem,
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

/// 静止画の asset を1つ取り込む。probe は尺を測れないので `duration` は `None`。
fn admit_still(writer: &mut DocumentWriter, asset_type: &str, file_name: &str) -> AssetId {
    let fp = SourceFingerprintV1::from_reader(Cursor::new(b"pretend image bytes")).unwrap();
    let draft = AssetDraft::from_probed_source(
        asset_type,
        &fp,
        &Path::new("/media/proj/stills").join(file_name),
        Some(Path::new("/media/proj")),
    );
    let admit = writer.prepare_admit_asset(draft).unwrap();
    let id = admit.asset().id;
    let gesture = writer.begin_gesture();
    writer
        .apply_prepared_asset_admission(gesture, admit)
        .unwrap();
    id
}

fn placed_clip(command: &Command) -> &Clip {
    match command {
        Command::AddTrackItem {
            item: TrackItem::Clip(clip),
            ..
        } => clip,
        other => panic!("expected AddTrackItem(Clip), got {other:?}"),
    }
}

#[test]
fn a_still_image_places_as_a_video_component() {
    let mut writer = writer_with_track();
    let id = admit_still(&mut writer, "image/png", "hero.png");

    let command = writer.prepare_place_asset_clip(id, seconds(0)).unwrap();
    let clip = placed_clip(&command);
    match &clip.source {
        ClipSource::Asset {
            asset,
            video,
            audio,
        } => {
            assert_eq!(*asset, id);
            let video = video.as_ref().expect("静止画は絵の列で置かれる");
            assert_eq!(video.stream.ordinal, 0);
            assert!(audio.is_empty(), "静止画は音を持たない");
        }
        other => panic!("expected an asset-backed clip, got {other:?}"),
    }
}

#[test]
fn a_still_image_fills_what_remains_of_the_composition() {
    let mut writer = writer_with_track();
    let id = admit_still(&mut writer, "image/jpeg", "hero.jpg");
    // 前提の明示: 既定コンポは 10s。
    assert_eq!(writer.snapshot().composition.duration, seconds(10));

    let command = writer.prepare_place_asset_clip(id, seconds(0)).unwrap();
    assert_eq!(
        placed_clip(&command).duration,
        seconds(10),
        "静止画に尺は無い = 長さ不明の既存の意味(comp の残り全部)"
    );

    let command = writer.prepare_place_asset_clip(id, seconds(7)).unwrap();
    assert_eq!(
        placed_clip(&command).duration,
        seconds(3),
        "途中に置けば残りの分だけ"
    );
}

#[test]
fn every_admitted_image_type_can_be_placed() {
    for asset_type in ["image/png", "image/jpeg", "image/webp"] {
        let mut writer = writer_with_track();
        let id = admit_still(&mut writer, asset_type, "hero.bin");
        writer
            .prepare_place_asset_clip(id, seconds(0))
            .unwrap_or_else(|error| panic!("{asset_type} should be placeable: {error}"));
    }
}
