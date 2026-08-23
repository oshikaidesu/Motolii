//! 素材の投影(SP-6 分割: 元 `model.rs` から移送 — `AssetListItem` とその
//! `status`(A05: 素材の欠落バッジの元になる `AssetStatus` をそのまま運ぶ)、
//! `StoreView::assets()` からの一覧化・rail scope+検索での絞り込み
//! (`visible`)、素材置換(`asset_to_layer_source`/`can_replace_source`)。

use super::rail::RailScope;
use motolii_store::{Asset, AssetId, AssetStatus, LayerId, LayerSource, RationalTime, StoreView};

/// 一覧1行ぶんの投影。`Asset` から Browser が要る最小の面だけを切り出す
/// (`AssetListItem { id, name, kind, path, fingerprint, duration }` — EXACT
/// TARGET #1)。**`fingerprint` は B2 で追加**(`Asset::content_hash` をそのまま
/// 運ぶ) — 検索欄が「表示名/fingerprint マッチ」(OUTCOME)を満たすための面。
/// **`duration` は B3 で追加**(`Asset::duration` をそのまま運ぶ) — カード grid
/// の「尺」表示(B3 OUTCOME「種別アイコン+名前+尺のカード骨格」)のための面。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetListItem {
    pub id: AssetId,
    pub name: String,
    /// `Asset::asset_type` をそのまま運ぶ(opaque 文字列、例 `video/mp4`)。
    /// rail/filter(B2、[`category_of`])がここから種別を導出する。
    pub kind: String,
    pub path: Option<String>,
    /// `Asset::content_hash` の写し(検索欄のマッチ対象、[`visible`] 参照)。
    pub fingerprint: String,
    /// `Asset::duration` の写し。probe が尺を読めなかった素材は `None`
    /// (`Asset::duration` の doc と同じ「分かる時だけ入る」)— カード grid は
    /// [`format_duration`] で「—」へ丸める。
    pub duration: Option<RationalTime>,
    /// `Asset::status` をそのまま運ぶ(A05: `Asset` が既に持っている値を
    /// 転記するだけ — この投影は IO をしない純関数のままなので、ここで
    /// `Asset::resolve_status` を呼び直すことは絶対にしない)。読み込み直後の
    /// 大半は `motolii_store::AssetStatus::Unchecked` のまま — 「在る」とは
    /// 一度も言っていない既定値。
    pub status: AssetStatus,
}

/// rail/filter が読む粗い種別。`AssetListItem::kind` の prefix(`/` 区切りの
/// 前半)だけを見る — 意味起草タスク#14(正確な種別判定)の空席を埋めない
/// 暫定判定(`Shell::guess_asset_type` が作る疑似 MIME 文字列を前提にする、
/// この crate 冒頭 doc と同じ立場)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Video,
    Image,
    Audio,
    /// mime prefix が `video`/`image`/`audio` のいずれでもない
    /// (`application/*` 等)。`RailScope::AllMedia` には含まれるが、種別
    /// scope(Video/Images/Audio)には現れない。
    Other,
}

/// `kind` 文字列(`video/mp4` 等)→ [`Category`]。純関数、IO なし。
pub fn category_of(kind: &str) -> Category {
    match kind.split('/').next().unwrap_or("") {
        "video" => Category::Video,
        "image" => Category::Image,
        "audio" => Category::Audio,
        _ => Category::Other,
    }
}

impl Category {
    /// カード grid の caption(B3)が読む表示文言。`RailScope::label` と語彙は
    /// 揃えるが単数形(1件のカードの種別を言う文なので「Video」等そのまま —
    /// rail 側は scope の複数件を言う文脈なので同じ語で兼用できる)。
    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Image => "Image",
            Self::Audio => "Audio",
            Self::Other => "Other",
        }
    }

    /// カード grid の thumb に載せる種別グリフ(B3)。rail の種別行アイコン
    /// (`browser-semantics.html` `▣ Video`/`▧ Images`/`♪ Audio`)と同じ語彙を
    /// 再利用する — thumb とrail が別の記号語彙を持つと意味が2つに割れる。
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Video => "▣",
            Self::Image => "▧",
            Self::Audio => "♪",
            Self::Other => "▪",
        }
    }
}

/// 尺の表示整形(mm:ss)。**IO なし・純関数**。`None`(probe が尺を読めなかった
/// 素材、`Asset::duration` の doc 参照)は Inspector の「値が無い」慣用と同じ
/// 1文字「—」へ丸める。負値・NaN は現実の `Asset::duration` からは出ない想定
/// だが `max(0.0)` で防御的に 0 へ丸める(M2 系「入力起因で panic しない」流儀)。
pub fn format_duration(duration: Option<RationalTime>) -> String {
    let Some(duration) = duration else {
        return "—".to_owned();
    };
    let total_seconds = duration.as_seconds_f64().max(0.0).round() as u64;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes}:{seconds:02}")
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
    assets_with_status(store, &|_| None)
}

/// [`assets`] に「解決済みの在り処」を重ねる版。
///
/// `Asset::status` は `#[serde(skip)]`(今そこに在るかは**環境の事実**であって
/// 作品の内容ではない — `motolii_store::AssetStatus` doc)なので、store から
/// 読み直した `Asset` は必ず `Unchecked` に戻る。よって**呼び手が別に持っている
/// 解決結果**を渡してもらい、ここで重ねる。`None` を返した素材は store の値
/// (= `Unchecked`)のまま。
///
/// **この関数は IO をしない。** `resolve_status` は `canonicalize`(syscall)を
/// 呼ぶので、投影のたびに走らせてはいけない — 解決の頻度は呼び手(shell)の責任。
pub fn assets_with_status(
    store: &StoreView<'_>,
    resolved: &dyn Fn(AssetId) -> Option<AssetStatus>,
) -> Vec<AssetListItem> {
    store
        .assets()
        .unwrap_or_default()
        .into_iter()
        .map(|asset| {
            let overlay = resolved(asset.id);
            let mut item = asset_to_item(asset);
            if let Some(status) = overlay {
                item.status = status;
            }
            item
        })
        .collect()
}

/// `Asset` 1件 → `AssetListItem` 1件の写し(裁定218 検収条件のための
/// 名前付き分離 — `assets()` の `.map` クロージャそのものと同じ中身だが、
/// `StoreView`/`Document` を組み立てずに単独でテストできるよう外へ出した)。
/// **IO なし・値を落とさない転記のみ**([`AssetListItem::status`] doc 参照)。
fn asset_to_item(asset: Asset) -> AssetListItem {
    AssetListItem {
        id: asset.id,
        name: asset.name,
        kind: asset.asset_type,
        path: asset.path_absolute,
        fingerprint: asset.content_hash,
        duration: asset.duration,
        status: asset.status,
    }
}

/// rail scope + 検索文字列で [`assets`] の一覧をさらに絞る純関数(B2
/// EXACT TARGET)。**IO なし** — 呼び手が既に持つ投影をもう一段絞るだけ。
///
/// - scope: [`RailScope::matches`] で種別フィルタ(mock の rail 行/filter shelf
///   チップ、どちらから触っても同じ絞り込みになる — 2つの入口が同じ状態を書く
///   Ableton可視性原理どおり)。
/// - query: 表示名/fingerprint/path の部分一致・大小無視・前後空白無視
///   (OUTCOME「文字列は fingerprint/表示名マッチ」+ B08 第4切片で path を追加
///   — `AssetListItem::path` は B1 から既に投影に乗っていた実在属性
///   [`Asset::path_absolute`] の写しだが、これまで検索対象になっていなかった。
///   store に新しい属性を要求せず「検索の対象拡張」を満たせる箇所だった)。
///   空文字列は「絞らない」。
/// - 順序は [`assets`] と同じ決定論(admit 順)を保つ — この関数は並べ替えない
///   (並べ替えは [`sorted`] が別関数として担う — scope/query→sort の合成順)。
pub fn visible(items: &[AssetListItem], scope: RailScope, query: &str) -> Vec<AssetListItem> {
    let query = query.trim().to_lowercase();
    items
        .iter()
        .filter(|item| scope.matches(category_of(&item.kind)))
        .filter(|item| {
            query.is_empty()
                || item.name.to_lowercase().contains(&query)
                || item.fingerprint.to_lowercase().contains(&query)
                || item
                    .path
                    .as_deref()
                    .is_some_and(|path| path.to_lowercase().contains(&query))
        })
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// B08 map 616/617 消化: 素材置換(`Intent::SetSource` の UI 面 — store 側は
// 裁定112c で実装済み、UI だけが未着手だった行)。618(ドラッグ版)は
// Stage/Timeline 側の drop target が要る別 write-set のため対象外
// (crate 冒頭 doc 参照)。
// ---------------------------------------------------------------------------

/// `Asset` → `Intent::SetSource { layer, source }` へ渡す `LayerSource` を
/// 組む純関数。**IO なし** — この crate は `Intent` を発行しない
/// (supervisor が `AssetId` → `Asset` を引いた後にこれを呼び、`Some` なら
/// dispatch する、crate 冒頭 doc「shell 結線」参照)。
///
/// `path_absolute` を優先、無ければ `path_project_relative` へ落ちる —
/// 両方無い(非ファイル素材)は `None`(置換できる実体が無い)。`fingerprint`
/// は `Asset::content_hash`(admit 時から常に有る)をそのまま運ぶ —
/// `motolii-shell` の raw-file-drop 経路(実ハッシュを持たないので
/// `fingerprint: None`)とは事情が違う、ここは実ハッシュを持つので使う。
pub fn asset_to_layer_source(asset: &Asset) -> Option<LayerSource> {
    let path = asset
        .path_absolute
        .clone()
        .or_else(|| asset.path_project_relative.clone())?;
    Some(LayerSource::Media {
        path,
        fingerprint: Some(asset.content_hash.clone()),
    })
}

/// 置換先 layer のゲーティング(map 616/617「選択素材を置換」)。呼び手
/// (supervisor)は `Session::selected_layers` を `len() == 1` の時だけ
/// `Some` へ畳んだ物を渡す — 0件・2件以上の選択(曖昧な置換先)は `None`
/// のまま渡すこと(この codebase の既存慣習「曖昧な物には何もしない」)。
/// [`asset_to_layer_source`] が `None` を返す素材(パスが無い)も同様に拒む。
pub fn can_replace_source(single_selected_layer: Option<LayerId>, asset: &Asset) -> Option<LayerId> {
    let layer = single_selected_layer?;
    asset_to_layer_source(asset)?;
    Some(layer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::test_support::{draft, mixed_ledger};
    use motolii_store::{Document, Intent};

    /// **ORACLE**(裁定218 検収条件そのもの): `AssetStatus::Missing` を持つ
    /// `Asset` から作った `AssetListItem` は `status` を落とさず `Missing` の
    /// まま運ぶ — A05 の穴(Browser 投影が `Asset::status` を一度も見ていない
    /// 問題)を閉じたことの検収。
    #[test]
    fn asset_to_item_projects_missing_status_without_dropping_it() {
        let asset = Asset {
            id: AssetId::from_raw(0),
            name: "gone".to_owned(),
            asset_type: "video/mp4".to_owned(),
            content_hash: "sha256:gone".to_owned(),
            path_absolute: Some("/mnt/gone.mp4".to_owned()),
            path_project_relative: None,
            file_name: Some("gone.mp4".to_owned()),
            size_bytes: None,
            status: AssetStatus::Missing,
            head_hash: None,
            tail_hash: None,
            duration: None,
        };

        let item = asset_to_item(asset);

        assert_eq!(
            item.status,
            AssetStatus::Missing,
            "投影が Asset::status を落とした(A05 の穴が再発)"
        );
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
        assert!(items[0].id < items[1].id, "admit 順(id 昇順)を保っていない");
    }

    /// 何も admit していない Document は空の projection(`markers`/`masks` と
    /// 同じ「無い=空」)。
    #[test]
    fn empty_table_projects_to_empty_list() {
        let doc = Document::new();
        assert_eq!(assets(&doc.view()), Vec::new());
    }

    // -----------------------------------------------------------------
    // B2: category_of — `kind` の prefix だけを見る純関数。
    // -----------------------------------------------------------------

    #[test]
    fn category_of_reads_the_mime_style_prefix() {
        assert_eq!(category_of("video/mp4"), Category::Video);
        assert_eq!(category_of("image/svg+xml"), Category::Image);
        assert_eq!(category_of("audio/wav"), Category::Audio);
        assert_eq!(category_of("application/octet-stream"), Category::Other);
        assert_eq!(
            category_of(""),
            Category::Other,
            "空文字列は Other へ丸まるはず"
        );
    }

    // -----------------------------------------------------------------
    // B2 EXACT TARGET: `visible`(rail scope + 検索文字列で絞る純関数)。
    // -----------------------------------------------------------------

    /// **ORACLE**: 未実装時点(`visible` が無い/常に全件を返す等)ではこの
    /// 群は red — `RailScope::Video` は video 種別だけを残す。
    #[test]
    fn rail_scope_video_narrows_to_video_kind_only() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::Video, "");
        assert_eq!(narrowed.len(), 2, "video 2件のはず: {narrowed:?}");
        assert!(narrowed
            .iter()
            .all(|item| category_of(&item.kind) == Category::Video));
    }

    #[test]
    fn rail_scope_images_narrows_to_image_kind_only() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::Images, "");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "logo-mark");
    }

    #[test]
    fn rail_scope_audio_narrows_to_audio_kind_only() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::Audio, "");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "room-tone");
    }

    /// `AllMedia` は種別を問わず全件(`Category::Other` の取りこぼしも無い —
    /// mixed_ledger は Other を含まないが、境界の意味は `category_of` 試験が
    /// 別途保証する)。順序は admit 順のまま(この関数は並べ替えない)。
    #[test]
    fn rail_scope_all_media_keeps_every_item_in_admission_order() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::AllMedia, "");
        assert_eq!(narrowed, items, "AllMedia は絞らない・並べ替えないはず");
    }

    #[test]
    fn query_matches_display_name_case_insensitively() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::AllMedia, "INTRO");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "intro-clip");
    }

    /// OUTCOME「文字列は fingerprint/表示名マッチ」— fingerprint
    /// (`content_hash` の写し)の部分一致でも絞れる。
    #[test]
    fn query_matches_fingerprint_substring() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::AllMedia, "sha256:a1");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "room-tone");
    }

    #[test]
    fn empty_query_after_trimming_does_not_narrow() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::AllMedia, "   ");
        assert_eq!(narrowed.len(), items.len());
    }

    /// scope と query は同時に効く(AND) — mock の `state.source`/`state.tag`/
    /// `state.query` が同時に絞り込みへ効くのと同じ形。
    #[test]
    fn scope_and_query_combine_with_and_semantics() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::Video, "cutaway");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "cutaway");

        // video scope だが image の名前で検索 → 0件。
        assert!(visible(&items, RailScope::Video, "logo").is_empty());
    }

    // -----------------------------------------------------------------
    // B3 EXACT TARGET: format_duration(カード grid の「尺」表示、純関数)。
    // -----------------------------------------------------------------

    #[test]
    fn format_duration_renders_none_as_an_em_dash() {
        assert_eq!(format_duration(None), "—");
    }

    #[test]
    fn format_duration_renders_whole_seconds_as_mmss() {
        let five_seconds = RationalTime::try_new(5, 1).unwrap();
        assert_eq!(format_duration(Some(five_seconds)), "0:05");
    }

    #[test]
    fn format_duration_carries_minutes_over() {
        let two_minutes_five = RationalTime::try_new(125, 1).unwrap();
        assert_eq!(format_duration(Some(two_minutes_five)), "2:05");
    }

    #[test]
    fn format_duration_rounds_to_the_nearest_second() {
        // 4.6秒 → 丸めて5秒(切り捨てだと利用者に「短く見える」誤差になる)。
        let almost_five = RationalTime::try_new(23, 5).unwrap();
        assert_eq!(format_duration(Some(almost_five)), "0:05");
    }

    // -----------------------------------------------------------------
    // B08 第4切片(素材の整理): 検索の対象拡張(path)+ SortKey/sorted +
    // ViewMode。**未実行**(発注書の検収線は `cargo check --tests` まで)。
    // -----------------------------------------------------------------

    /// OUTCOME「検索の対象拡張」— path(`AssetListItem::path`、
    /// `Asset::path_absolute` の写し)の部分一致でも絞れる。名前とは無関係な
    /// 語で、path にだけ現れる語で検索する(名前一致との偶然の重なりを排除)。
    #[test]
    fn query_matches_path_substring_even_when_the_name_does_not() {
        let items = vec![AssetListItem {
            id: motolii_store::AssetId::from_raw(0),
            name: "field-recording".to_owned(),
            kind: "audio/wav".to_owned(),
            path: Some("/mnt/archive/legacy-drive/tape-07.wav".to_owned()),
            fingerprint: "sha256:tape07".to_owned(),
            duration: None,
            status: AssetStatus::Unchecked,
        }];
        let narrowed = visible(&items, RailScope::AllMedia, "tape-07");
        assert_eq!(narrowed.len(), 1, "path 部分一致で絞れない: {narrowed:?}");

        assert!(
            visible(&items, RailScope::AllMedia, "legacy-drive").len() == 1,
            "path の途中セグメントでも絞れるはず"
        );
    }

    /// path が無い(`None`)素材は path 一致では絞られない(panic もしない —
    /// `Option::is_some_and` の素通し)。名前一致では引き続き絞れる。
    #[test]
    fn query_ignores_missing_path_without_panicking() {
        let items = vec![AssetListItem {
            id: motolii_store::AssetId::from_raw(0),
            name: "generated-noise".to_owned(),
            kind: "audio/wav".to_owned(),
            path: None,
            fingerprint: "sha256:noise".to_owned(),
            duration: None,
            status: AssetStatus::Unchecked,
        }];
        assert!(visible(&items, RailScope::AllMedia, "mnt").is_empty());
        assert_eq!(visible(&items, RailScope::AllMedia, "generated").len(), 1);
    }

    // -----------------------------------------------------------------
    // B08 map 616/617: asset_to_layer_source / can_replace_source(純関数)。
    // -----------------------------------------------------------------

    fn asset_with_paths(path_absolute: Option<&str>, path_project_relative: Option<&str>) -> Asset {
        Asset {
            id: AssetId::from_raw(0),
            name: "clip".to_owned(),
            asset_type: "video/mp4".to_owned(),
            content_hash: "sha256:abc".to_owned(),
            path_absolute: path_absolute.map(str::to_owned),
            path_project_relative: path_project_relative.map(str::to_owned),
            // 2026-08-23 A-3: 「今そこに在るか」は環境の事実で保存されない
            // (`#[serde(skip)]`)。試験の固定値は「未確認」が正しい既定。
            status: motolii_store::AssetStatus::Unchecked,
            file_name: None,
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
            duration: None,
        }
    }

    /// **ORACLE**: `path_absolute` がある時はそちらを優先する。
    #[test]
    fn asset_to_layer_source_prefers_the_absolute_path() {
        let asset = asset_with_paths(Some("/abs/clip.mp4"), Some("media/clip.mp4"));
        let source = asset_to_layer_source(&asset).expect("path があるので Some のはず");
        assert_eq!(
            source,
            LayerSource::Media {
                path: "/abs/clip.mp4".to_owned(),
                fingerprint: Some("sha256:abc".to_owned()),
            }
        );
    }

    /// `path_absolute` が無ければ `path_project_relative` へ落ちる。
    #[test]
    fn asset_to_layer_source_falls_back_to_the_project_relative_path() {
        let asset = asset_with_paths(None, Some("media/clip.mp4"));
        let source = asset_to_layer_source(&asset).expect("relative path があるので Some のはず");
        assert_eq!(
            source,
            LayerSource::Media {
                path: "media/clip.mp4".to_owned(),
                fingerprint: Some("sha256:abc".to_owned()),
            }
        );
    }

    /// 両方無い(非ファイル素材)は `None` — 置換できる実体が無い。
    #[test]
    fn asset_to_layer_source_is_none_without_any_path() {
        let asset = asset_with_paths(None, None);
        assert_eq!(asset_to_layer_source(&asset), None);
    }

    /// 選択0件は拒む(曖昧な置換先を作らない)。
    #[test]
    fn can_replace_source_refuses_when_no_layer_is_selected() {
        let asset = asset_with_paths(Some("/abs/clip.mp4"), None);
        assert_eq!(can_replace_source(None, &asset), None);
    }

    /// **ORACLE**: 単一選択+有効な素材 → その layer を返す。
    #[test]
    fn can_replace_source_returns_the_single_selected_layer() {
        let asset = asset_with_paths(Some("/abs/clip.mp4"), None);
        let layer = LayerId(7);
        assert_eq!(can_replace_source(Some(layer), &asset), Some(layer));
    }

    /// 単一選択があってもパスの無い素材は拒む。
    #[test]
    fn can_replace_source_refuses_an_asset_without_a_usable_path() {
        let asset = asset_with_paths(None, None);
        let layer = LayerId(7);
        assert_eq!(can_replace_source(Some(layer), &asset), None);
    }
}
