//! D1m: project-scoped sidecar path isolation.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_doc::{
    generation_path_for_document, journal_path_for_document,
    legacy_shared_motolii_dir_for_document, load_catalog, motolii_dir_for_document,
    project_lock_path_for_document, project_sidecar_dir_for_document, restore_attempted_path,
    Document,
};

mod common;

use common::session::{open_recovered, save_journal};
use motolii_doc::SaveProjectOptions;

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1m-paths-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn same_directory_projects_have_disjoint_sidecar_paths() {
    let dir = unique_dir("disjoint");
    let path_a = dir.join("a.json");
    let path_b = dir.join("b.json");

    assert_ne!(
        motolii_dir_for_document(&path_a),
        motolii_dir_for_document(&path_b)
    );
    assert_ne!(
        project_lock_path_for_document(&path_a),
        project_lock_path_for_document(&path_b)
    );
    assert_ne!(
        journal_path_for_document(&path_a),
        journal_path_for_document(&path_b)
    );
    assert_eq!(
        motolii_dir_for_document(&path_a),
        project_sidecar_dir_for_document(&path_a)
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn same_directory_isolation_preserves_peer_bytes() {
    let dir = unique_dir("isolate");
    let path_a = dir.join("alpha.json");
    let path_b = dir.join("beta.json");
    let mut doc_a = Document::new_current();
    doc_a.bpm = motolii_doc::Bpm::try_new(111, 1).unwrap();
    let mut doc_b = Document::new_current();
    doc_b.bpm = motolii_doc::Bpm::try_new(222, 1).unwrap();

    save_journal(&path_a, &doc_a, &SaveProjectOptions::default());
    save_journal(&path_b, &doc_b, &SaveProjectOptions::default());

    let catalog_b_before = fs::read(journal_path_for_document(&path_b)).unwrap();
    let (_, opened_a) = open_recovered(&path_a);
    assert_eq!(opened_a.document.bpm.num(), 111);

    let catalog_b_after = fs::read(journal_path_for_document(&path_b)).unwrap();
    assert_eq!(catalog_b_before, catalog_b_after);
    let opened_b = open_recovered(&path_b).1;
    assert_eq!(opened_b.document.bpm.num(), 222);
    assert_ne!(
        load_catalog(&path_a).unwrap().unwrap().project_id,
        load_catalog(&path_b).unwrap().unwrap().project_id
    );
    assert_ne!(
        generation_path_for_document(&path_a, uuid::Uuid::nil()),
        generation_path_for_document(&path_b, uuid::Uuid::nil())
    );
    assert_ne!(
        restore_attempted_path(&path_a),
        restore_attempted_path(&path_b)
    );
    assert_ne!(
        legacy_shared_motolii_dir_for_document(&path_a),
        motolii_dir_for_document(&path_a)
    );
    let _ = fs::remove_dir_all(dir);
}
