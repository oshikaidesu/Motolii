#![allow(deprecated)]

//! D8: 単一writer + Arcスナップショット所有権(F-2)の並行契約。
//!
//! 型レベル禁止(`&mut Document`はmotolii-doc外deny)は`mut_document_deny`が担保。
//! ここでは「編集中に読み手が古いスナップショットで完走する」を機械判定する。

use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use motolii_doc::{render_with_snapshot, Bpm, Document, DocumentWriter, WriterMessage};

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn arc_document_is_send_sync_for_reader_threads() {
    assert_send_sync::<Arc<Document>>();
}

#[test]
fn render_thread_finishes_on_stale_snapshot_while_writer_edits() {
    let mut writer = DocumentWriter::new(Document::new_v1());
    let initial_bpm = writer.snapshot().bpm.num();
    let snap = writer.snapshot();
    assert_eq!(snap.version, 1);
    assert_eq!(render_with_snapshot(&snap), 1);

    let barrier = Arc::new(Barrier::new(2));
    let snap_for_render = Arc::clone(&snap);
    let barrier_render = Arc::clone(&barrier);

    let render = thread::spawn(move || {
        barrier_render.wait();
        // 擬似レンダ: スナップショットを長めに読む間、writerが並行編集する
        let mut checksum = 0u64;
        for _ in 0..200_000 {
            checksum = checksum
                .wrapping_add(snap_for_render.version as u64)
                .wrapping_add(snap_for_render.bpm.num() as u64)
                .wrapping_add(snap_for_render.min_reader_version as u64);
            // 読み手が持つのは不変Arc — writerのeditは別クローンを触る
            std::hint::black_box(&snap_for_render.composition);
        }
        (
            checksum,
            snap_for_render.version,
            snap_for_render.bpm.num(),
            Arc::strong_count(&snap_for_render),
        )
    });

    barrier.wait();
    for i in 0..2_000 {
        writer.edit(|doc| {
            doc.version = 2;
            doc.bpm = Bpm::try_new(120 + (i % 40), 1).unwrap();
        });
        if i % 200 == 0 {
            thread::yield_now();
        }
    }

    let (checksum, seen_version, seen_bpm, _) = render.join().unwrap();
    assert!(checksum > 0);
    // 古いスナップショットの観測値は編集前のまま
    assert_eq!(seen_version, 1);
    assert_eq!(seen_bpm, initial_bpm);
    assert_eq!(snap.version, 1);
    assert_eq!(snap.bpm.num(), initial_bpm);

    // writer側は最新
    let after = writer.snapshot();
    assert_eq!(after.version, 2);
    assert_ne!(after.bpm.num(), initial_bpm);
    assert!(writer.revision >= 2_000);
}

#[test]
fn background_thread_delivers_message_only_writer_applies() {
    let (tx, rx) = mpsc::channel::<WriterMessage>();
    let worker = thread::spawn(move || {
        // バックグラウンド成果 — Documentを直接触らずメッセージだけ返す
        thread::sleep(Duration::from_millis(5));
        tx.send(WriterMessage::SetBpm(Bpm::try_new(96, 1).unwrap()))
            .unwrap();
    });

    let mut writer = DocumentWriter::new(Document::new_v1());
    let before = writer.snapshot();
    assert_eq!(before.bpm.num(), Bpm::DEFAULT.num());

    let msg = rx.recv().expect("background message");
    writer.apply(msg);
    worker.join().unwrap();

    let after = writer.snapshot();
    assert_eq!(after.bpm.num(), 96);
    assert_eq!(before.bpm.num(), Bpm::DEFAULT.num());
    assert_eq!(writer.revision, 1);
}

#[test]
fn multiple_reader_threads_share_one_immutable_snapshot() {
    let mut writer = DocumentWriter::new(Document::new_v1());
    writer.edit(|doc| {
        doc.bpm = Bpm::try_new(132, 1).unwrap();
    });
    let snap = writer.snapshot();
    let barrier = Arc::new(Barrier::new(5));

    let mut handles = Vec::new();
    for _ in 0..4 {
        let snap = Arc::clone(&snap);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            assert_eq!(snap.bpm.num(), 132);
            assert_eq!(render_with_snapshot(&snap), 1);
            snap.version
        }));
    }

    barrier.wait();
    // 読み手動作中にwriterがさらに編集しても、配布済みsnapは不変
    writer.edit(|doc| {
        doc.version = 9;
        doc.bpm = Bpm::try_new(200, 1).unwrap();
    });

    for h in handles {
        assert_eq!(h.join().unwrap(), 1);
    }
    assert_eq!(snap.version, 1);
    assert_eq!(snap.bpm.num(), 132);
    assert_eq!(writer.snapshot().version, 9);
}
