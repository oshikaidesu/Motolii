//! 既存 `crate::media_library` の投影を、格子描画が使える形へ**写すだけ**の薄い層。
//!
//! フォルダ参照・一覧・種別判定・path解決の意味は `media_library.rs` が既に持っている。
//! ここでは走査も拡張子判定も**再実装しない**。やっているのは2つだけ:
//!   1. `LibraryProjection` から image kind の項目を取り、`resolve()` で実pathへ落とす
//!   2. atlas上限(下記)で表示枚数を切る
//!
//! `media_library::MAX_ITEMS` は 256 だが、`vello_hybrid` のatlasは
//! **CSS表示サイズではなく元解像度**で消費する
//! (`docs/reviews/2026-08-15-blitz-ui-runtime-probe.md:224`)。
//! 既定atlasは 4096x4096 x 最大8面 ≒ 134M px(同:257)で、
//! ただし面積計算では収まらない。実測(本capsule, 元寸1480x1400):
//!   - 20枚 → atlasに20枚(全部載る)
//!   - 45枚 → atlasに**30枚**。残り15枚は載らず、cardは空のまま描かれる
//! shelf packing で 4096/1480 = 2列、4096/1400 = 2段 → 1面4枚 x 8面 = 32枚が上限。
//! 上の観測から「面積ではなく枚数が効く」と読んでいた。
//!
//! ## 訂正(`thumbnail.rs` 導入時の実測)
//! `<img>` を縮小実体(248x168 = 元寸の1/37の画素)へ差し替えても、
//! `motolii-blitz-dump browser`(面 900x520)の `atlas images` は **30 のまま**だった。
//! 枚数を振ると 12→12 / 24→24 / 30→30 / 36→**30** / 45→**30** と 30 で飽和する。
//! 30 は 6列 x 5行 = **その面に出るcardの枚数**そのもの
//! (行の上端 = `TOP` + 行 x (`CELL_H` + `PAD`) なので、520px の面には5行目まで掛かる)。
//! つまり dump が見せる 30 は atlas の天井ではなく **viewport culling** の数で、
//! 「45枚 → 30枚」も同じ数字だった。shelf packing の 32 が近かったのは偶然の可能性が高い。
//! 元寸時代に「載らない15枚」と見えていたものは、そもそも面の外で描かれない分。
//!
//! **サムネイル化後にこの上限へ当たるかは、まだ測れていない。**
//! 45枚全部を縮小実体で出しても 1.9 MPx で天井には遠いはずだが、
//! それを確かめるには45枚全部が面に出る大きさ(合体シェルの 896x1216 など)で
//! 測り直すしかない。測っていないので `DEFAULT_MAX_ITEMS` は消さずに残す。
//! virtualization も未着手。

use std::path::Path;

use crate::media_library::{LibraryFilter, MediaLibrary, ResolvedLibraryFile};

/// C6 POSITIVE ORACLE(元寸PNG45枚)が通る枚数。実測の根拠は上のコメント。
pub const DEFAULT_MAX_ITEMS: usize = 45;

/// `media_library` の image kind ラベル。`media_kind()` の戻り値の写し。
const IMAGE_KIND: &str = "image";

/// 表示1項目。`media_library` の解決結果をそのまま使う(独自の項目型を作らない)。
pub type BrowserItem = ResolvedLibraryFile;

/// フォルダ配下の画像を、既存 media library 経由で最大 `max_items` 件返す。
pub fn image_items(dir: &Path, max_items: usize) -> Vec<BrowserItem> {
    let library = MediaLibrary::with_root(dir.to_path_buf());
    library
        .filtered_items(&LibraryFilter::Kind(IMAGE_KIND.to_owned()))
        .into_iter()
        .filter_map(|item| library.resolve(&item.id))
        .take(max_items)
        .collect()
}
