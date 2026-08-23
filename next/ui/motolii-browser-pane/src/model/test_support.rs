//! SP-6(裁定220 レーン)分割: 元 `model.rs` の単一 `mod tests` が持っていた
//! fixture ヘルパー(`draft`/`draft_typed`/`admit_all`/`mixed_ledger`)を、
//! `projection`/`tabs` 両方の試験から共有するために切り出したもの
//! (どちらも同じ fixture を使っていた — 中身は移送のみ、ロジック変更なし)。

use super::projection::assets;
use super::AssetListItem;
use motolii_store::{AssetDraft, Document, Intent};

pub(crate) fn draft(content_hash: &str, name: &str) -> AssetDraft {
    draft_typed(content_hash, name, "video/mp4")
}

/// [`draft`] の種別指定版(B2: rail/filter の種別テストが要る)。
pub(crate) fn draft_typed(content_hash: &str, name: &str, asset_type: &str) -> AssetDraft {
    AssetDraft {
        name: name.to_owned(),
        asset_type: asset_type.to_owned(),
        content_hash: content_hash.to_owned(),
        path_absolute: Some(format!("/project/media/{name}.mp4")),
        path_project_relative: None,
        file_name: Some(format!("{name}.mp4")),
        size_bytes: Some(1024),
        head_hash: None,
        tail_hash: None,
        duration: None,
    }
}

pub(crate) fn admit_all(doc: &mut Document, drafts: Vec<AssetDraft>) {
    for draft in drafts {
        doc.apply(Intent::AdmitAsset { draft }).unwrap();
    }
}

pub(crate) fn mixed_ledger() -> Vec<AssetListItem> {
    let mut doc = Document::new();
    admit_all(
        &mut doc,
        vec![
            draft_typed("sha256:v1", "intro-clip", "video/mp4"),
            draft_typed("sha256:i1", "logo-mark", "image/png"),
            draft_typed("sha256:a1", "room-tone", "audio/wav"),
            draft_typed("sha256:v2", "cutaway", "video/mov"),
        ],
    );
    assets(&doc.view())
}
