//! 一覧 projection(裁定162 切片 B1)— `StoreView::assets()` から Browser が描く
//! 行を組む純関数。**IO も評価もしない** — `StoreView` が既に読んだ台帳をそのまま
//! 並べ替えるだけの読み専用の投影(`timeline_pane::rows` と同じ形)。
//!
//! 移植元(意味の正本)は旧 `crates/motolii-shell-iced/src/browser.rs` の
//! `BrowserCard` 投影だが、この切片(B1)の範囲は一覧そのものまで — rail/filter
//! (B2)・視覚(B3、`browser-library.html` 構造)・サムネ(B4/B5)はまだこの
//! crate に無い。

use motolii_store::{AssetId, StoreView};

/// 一覧1行ぶんの投影。`Asset` から Browser が要る最小の面だけを切り出す
/// (`AssetListItem { id, name, kind, path }` — EXACT TARGET #1)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetListItem {
    pub id: AssetId,
    pub name: String,
    /// `Asset::asset_type` をそのまま運ぶ(opaque 文字列、例 `video/mp4`)。
    /// rail/filter(B2)がここから種別を導出する — この切片では種別判定をしない。
    pub kind: String,
    pub path: Option<String>,
}

/// 台帳の一覧を投影する。**順序は決定論(admit 順 = `AssetId` 昇順)** —
/// `StoreView::assets()` が既に `AssetTable`(`BTreeMap` 内部)の順で返すので、
/// この関数自身は並べ替えない(既にある決定論を壊さない、`view.rs` の
/// `assets()` doc 参照)。
///
/// `StoreView::assets()` が `Err` を返す(壊れた Document)場合は空を返す —
/// この投影は表示専用(`meta`/`markers` 読み系と同じ「表示は空へ丸める」流儀)。
/// 「読めない」ことそのものを扱いたい呼び手は `store.assets()` を直接呼ぶこと。
pub fn assets(store: &StoreView<'_>) -> Vec<AssetListItem> {
    store
        .assets()
        .unwrap_or_default()
        .into_iter()
        .map(|asset| AssetListItem {
            id: asset.id,
            name: asset.name,
            kind: asset.asset_type,
            path: asset.path_absolute,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{AssetDraft, Document, Intent};

    fn draft(content_hash: &str, name: &str) -> AssetDraft {
        AssetDraft {
            name: name.to_owned(),
            asset_type: "video/mp4".to_owned(),
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

    /// ORACLE (a): fixture 級の `StoreView`(台帳2件)→ projection 2件・
    /// 順序決定論(admit 順 = id 順)。
    #[test]
    fn projects_two_assets_in_admission_order() {
        let mut doc = Document::new();
        doc.apply(Intent::AdmitAsset {
            draft: draft("sha256:first", "first"),
        })
        .unwrap();
        doc.apply(Intent::AdmitAsset {
            draft: draft("sha256:second", "second"),
        })
        .unwrap();

        let items = assets(&doc.view());
        assert_eq!(items.len(), 2, "台帳2件が projection に2件現れない");
        assert_eq!(items[0].name, "first");
        assert_eq!(items[1].name, "second");
        assert!(
            items[0].id < items[1].id,
            "admit 順(id 昇順)を保っていない"
        );
    }

    /// 何も admit していない Document は空の projection(`markers`/`masks` と
    /// 同じ「無い=空」)。
    #[test]
    fn empty_table_projects_to_empty_list() {
        let doc = Document::new();
        assert_eq!(assets(&doc.view()), Vec::new());
    }
}
