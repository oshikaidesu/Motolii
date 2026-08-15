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
//! **面積ではなく枚数が効く。** 元寸を直接 `<img>` に出す限りこの天井は動かない。
//! 載らない分でpanicはしなかった(60秒/176フレームで無事)が、
//! 見た目は欠ける。縮小実体の事前生成と virtualization が要るが、
//! それは本capsuleの範囲外なので枚数だけ切っておく。

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
