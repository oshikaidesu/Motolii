//! 素材台帳(裁定162: Browser の一覧の正本 = Document 所有の台帳、旧 asset.rs 移植)。
//!
//! ここで固定するのは3つ:
//! - `AdmitAsset`/`RemoveAsset` も他の Intent と同じく `edit` timeline 上の普通の
//!   刻み(1 intent = 1 undo)
//! - 同一 `content_hash` の二重 admission は台帳を増やさず既存 id を使い回す
//! - 保存は履歴を畳んでも台帳が残る(`markers`/`slots` と同じく `flattened()` の
//!   手列挙から漏れていないことの確認、裁定108(b) と同型)

use motolii_store::{Asset, AssetDraft, Document, Intent};

fn draft(content_hash: &str) -> AssetDraft {
    AssetDraft {
        name: "clip".to_owned(),
        asset_type: "video/mp4".to_owned(),
        content_hash: content_hash.to_owned(),
        path_absolute: Some("/project/media/clip.mp4".to_owned()),
        path_project_relative: Some("media/clip.mp4".to_owned()),
        file_name: Some("clip.mp4".to_owned()),
        size_bytes: Some(1024),
        head_hash: None,
        tail_hash: None,
        duration: None,
    }
}

/// まだ何も admit していない Document は空の台帳(`markers`/`masks` と同じ
/// 「無い=空」の扱い)。
#[test]
fn no_assets_is_an_empty_list_not_an_error() {
    let doc = Document::new();
    assert_eq!(doc.view().assets().unwrap(), Vec::new());
}

/// AdmitAsset → assets() に現れる。RemoveAsset → 消える。undo で両方とも戻る
/// (ORACLE (a))。
#[test]
fn admit_appears_and_remove_disappears_and_undo_reverts_each() {
    let mut doc = Document::new();

    doc.apply(Intent::AdmitAsset {
        draft: draft("sha256:aaa"),
    })
    .unwrap();
    let assets = doc.view().assets().unwrap();
    assert_eq!(assets.len(), 1, "admit 後に台帳へ現れない");
    let id = assets[0].id;
    assert_eq!(
        doc.view().asset(id).unwrap().map(|a| a.name),
        Some("clip".to_owned())
    );

    doc.apply(Intent::RemoveAsset { asset: id }).unwrap();
    assert!(
        doc.view().assets().unwrap().is_empty(),
        "remove 後も台帳に残っている"
    );

    // remove の undo: 台帳へ戻る。
    assert!(doc.undo());
    assert_eq!(
        doc.view().assets().unwrap().len(),
        1,
        "RemoveAsset の undo で台帳に戻らない(1 intent = 1 undo)"
    );
    assert_eq!(doc.view().asset(id).unwrap().map(|a| a.id), Some(id));

    // admit の undo: 台帳から消える。
    assert!(doc.undo());
    assert!(
        doc.view().assets().unwrap().is_empty(),
        "AdmitAsset の undo で台帳から消えない(1 intent = 1 undo)"
    );

    // redo で両方戻る。
    assert!(doc.redo());
    assert_eq!(doc.view().assets().unwrap().len(), 1);
    assert!(doc.redo());
    assert!(doc.view().assets().unwrap().is_empty());
}

/// 同一 `content_hash` の二重 Admit → 台帳は1件のまま、既存 id を使い回す
/// (ORACLE (b))。
#[test]
fn duplicate_content_hash_admission_keeps_a_single_entry_with_the_existing_id() {
    let mut doc = Document::new();

    doc.apply(Intent::AdmitAsset {
        draft: draft("sha256:same"),
    })
    .unwrap();
    let first_id = doc.view().assets().unwrap()[0].id;

    doc.apply(Intent::AdmitAsset {
        draft: draft("sha256:same"),
    })
    .unwrap();

    let assets = doc.view().assets().unwrap();
    assert_eq!(
        assets.len(),
        1,
        "同一 content_hash の二重 admission が台帳を2件に増やした"
    );
    assert_eq!(
        assets[0].id, first_id,
        "二重 admission で id が変わった(既存 id を使い回すはず)"
    );
}

/// 内容が違う draft は別エントリになる(重複統合が誤爆して別素材を握り潰さないこと)。
#[test]
fn distinct_content_hashes_get_distinct_entries() {
    let mut doc = Document::new();
    doc.apply(Intent::AdmitAsset {
        draft: draft("sha256:one"),
    })
    .unwrap();
    doc.apply(Intent::AdmitAsset {
        draft: draft("sha256:two"),
    })
    .unwrap();

    let mut assets = doc.view().assets().unwrap();
    assets.sort_by(|a, b| a.content_hash.cmp(&b.content_hash));
    assert_eq!(assets.len(), 2);
    assert_eq!(
        assets
            .iter()
            .map(|a| a.content_hash.as_str())
            .collect::<Vec<_>>(),
        vec!["sha256:one", "sha256:two"]
    );
}

/// RemoveAsset が台帳に無い id を指すとエラーになる(黙って no-op にしない —
/// `AssetTable::remove` の `NotFound` をそのまま `StoreError` へ運ぶ)。
#[test]
fn removing_an_unknown_asset_id_is_an_error() {
    let mut doc = Document::new();
    doc.apply(Intent::AdmitAsset {
        draft: draft("sha256:aaa"),
    })
    .unwrap();
    let bogus = motolii_store::AssetId::from_raw(999);
    assert!(doc.apply(Intent::RemoveAsset { asset: bogus }).is_err());
}

/// 保存は履歴を畳む(裁定56)。台帳は `meta`/`composition` の中に無いぶん、
/// ここで固定しておかないと `flattened()` の手列挙から黙って落ちる
/// (`markers` の同型試験、裁定108(b) と同型)。ORACLE (c)。
#[test]
fn assets_survive_a_save_and_load_round_trip() {
    let mut doc = Document::new();
    doc.apply(Intent::AdmitAsset {
        draft: draft("sha256:roundtrip"),
    })
    .unwrap();
    let id = doc.view().assets().unwrap()[0].id;

    let dir = std::env::temp_dir().join(format!("motolii-asset-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("assets.rrd");
    doc.save(&file).unwrap();
    let loaded = Document::load(&file).unwrap();

    let assets: Vec<Asset> = loaded.view().assets().unwrap();
    assert_eq!(assets.len(), 1, "保存で畳む時に台帳が落ちている");
    assert_eq!(assets[0].id, id);
    assert_eq!(assets[0].content_hash, "sha256:roundtrip");
    assert_eq!(
        assets[0].path_project_relative.as_deref(),
        Some("media/clip.mp4")
    );

    // 保存後に admit すると、読み込んだ台帳の `next` が引き継がれ、
    // 退役済み id(このケースでは無いが)を再利用しないことを確認する。
    let mut loaded = loaded;
    loaded
        .apply(Intent::AdmitAsset {
            draft: draft("sha256:after-load"),
        })
        .unwrap();
    let new_id = loaded
        .view()
        .assets()
        .unwrap()
        .into_iter()
        .find(|a| a.content_hash == "sha256:after-load")
        .unwrap()
        .id;
    assert_ne!(new_id, id, "読み込み後の採番が既存 id と衝突した");
}
