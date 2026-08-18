//! `<img>` が指す縮小実体を作る。**寸法をここで決めない。**
//!
//! 元画像をそのまま `<img>` に出すと `vello_hybrid` の atlas は
//! **CSS表示サイズではなく元解像度**で消費する(実測は `library_view.rs` 冒頭)。
//! `docs/mocks` の1960x1300級PNG 45枚では30枚でatlasが尽き、残り15枚は
//! cardが空のまま描かれていた。合体シェルの実測でも Browser面だけが 107 ms/frame
//! (他は Timeline 42 / Inspector 49 / Stage 2.9)。
//! つまり「45枚中30枚」は atlas の天井というより**縮小実体を作っていない症状**なので、
//! ここで作って `<img>` の行き先だけを差し替える。
//!
//! ## ここで決めないこと
//! - **card の見た目**: CSSも配置も `markup.rs` のまま。変えるのは `src` の行き先だけ
//! - **走査・種別判定・path解決**: `crate::media_library` が持つ(`library_view.rs`)
//! - **サムネイルの寸法**: `.th` の表示寸法(`theme.rs`)からの導出で、下記の式が全部

use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rustc_hash::FxHasher;

use super::library_view::BrowserItem;
use super::theme;

/// hidpi の余裕。`BrowserBlitzPanel` は CSS px の面を `scale`(= `pixels_per_point`)倍の
/// 物理px textureへ等倍で描く(`mod.rs:96-107`)。Retina は 2.0 なので、
/// 1 CSS px は最大2物理px になる。これ以上大きい実体を持ってもatlasを無駄に食うだけ。
const HIDPI_HEADROOM: f64 = 2.0;

/// 生成先の既定。`std::env::temp_dir()` 配下に置く。**リポジトリ内には置かない**
/// (生成物であって成果物ではないため)。
const CACHE_DIR_NAME: &str = "motolii-browser-thumbs";

/// 生成先を差し替える環境変数。temp_dir が使えない環境や、
/// 生成物を残して確かめたい時のための逃げ道。
const CACHE_DIR_ENV: &str = "MOTOLII_BROWSER_THUMB_DIR";

/// サムネイルの上限寸法(px)。**表示寸法からの導出**であって選んだ値ではない。
///
/// `.th`(card内の画像枠)の表示寸法は `theme::THUMB_W` x `theme::THUMB_H`
/// = 124 x 84 CSS px。これは `theme::CELL_W/CELL_H` から引いた値で、
/// 元は `spikes/blitz-probe/src/bin/browser_panel.rs` の格子寸法。
/// `ui/motolii-rn/src/productStyles.ts` 側の thumb(browserThumb height 52 /
/// thumbnailOnlyThumb 42 / browserListThumb 52x36)はどれもこれより小さいので、
/// この枠を賄えれば製品側の3種も賄える。
/// そこへ hidpi 分(`HIDPI_HEADROOM` = 2倍)の余裕を足して 248 x 168。
/// `.th` は `object-fit:contain` なので、縦横比を保ったままこの枠に収める。
fn max_size() -> (u32, u32) {
    (
        (theme::THUMB_W * HIDPI_HEADROOM).ceil() as u32,
        (theme::THUMB_H * HIDPI_HEADROOM).ceil() as u32,
    )
}

/// 項目ごとの縮小実体のpath。`items` と**同じ長さ・同じ並び**で返す。
/// `Err` は作れなかった項目で、その card は画像なしで描かれる(`markup.rs`)。
/// **元画像へフォールバックしない** — 戻すと重さの原因がそのまま残る。
///
/// 落ちた理由は**呼び手へ返す**。`eprintln!` で書くと、窓しか無い運転席では
/// 誰も読まないまま「画像が出ない」だけが残る(2026-08-18 外部診断 F-09)。
pub(crate) fn prepare(items: &[BrowserItem]) -> Vec<Result<PathBuf, String>> {
    items
        .iter()
        .map(|item| thumbnail_for(&item.path))
        .collect()
}

/// 1枚分。既にキャッシュにあれば再生成しない。
fn thumbnail_for(source: &Path) -> Result<PathBuf, String> {
    let (max_w, max_h) = max_size();
    let dir = cache_dir();
    let key = cache_key(source)?;
    // 寸法も名前に入れる。導出が変わった時に古い実体を掴まないため。
    let out = dir.join(format!("{key:016x}-{max_w}x{max_h}.png"));
    if out.is_file() {
        return Ok(out);
    }

    fs::create_dir_all(&dir).map_err(|error| format!("{} : {error}", dir.display()))?;
    let image = image::ImageReader::open(source)
        .map_err(|error| error.to_string())?
        .with_guessed_format()
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| error.to_string())?;
    // `resize` は縦横比を保ったまま枠へ収める。縮小率が大きいので
    // Triangle(縮小率に応じて広がるkernel)を使う — Nearest だと mock の細い線が消える。
    let thumb = image
        .resize(max_w, max_h, image::imageops::FilterType::Triangle)
        .to_rgba8();

    // 同じ実体を別プロセスが同時に書いても壊れないよう、一時fileへ書いてから rename する。
    let staging = dir.join(format!(".{key:016x}-{}.png", std::process::id()));
    thumb
        .save(&staging)
        .map_err(|error| format!("{} : {error}", staging.display()))?;
    fs::rename(&staging, &out).map_err(|error| format!("{} : {error}", out.display()))?;
    Ok(out)
}

/// キャッシュの置き場所。既定は `std::env::temp_dir()` 配下。
fn cache_dir() -> PathBuf {
    match std::env::var_os(CACHE_DIR_ENV) {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => std::env::temp_dir().join(CACHE_DIR_NAME),
    }
}

/// キャッシュキー。**元fileのpathとmtime**から作る。
/// 同名で中身が差し替わった時に古い実体を掴まないよう mtime を混ぜ、
/// 取り違えの余地を減らすため大きさも混ぜる。
fn cache_key(source: &Path) -> Result<u64, String> {
    let meta = fs::metadata(source).map_err(|error| error.to_string())?;
    let mtime = meta
        .modified()
        .map_err(|error| error.to_string())?
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    let mut hasher = FxHasher::default();
    source.hash(&mut hasher);
    mtime.as_nanos().hash(&mut hasher);
    meta.len().hash(&mut hasher);
    Ok(hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_size_is_derived_from_the_card_thumb_box() {
        // 表示寸法(124x84 CSS px)の hidpi 2倍。値を直接決めていないことの見張り。
        assert_eq!(
            max_size(),
            (
                (theme::THUMB_W * 2.0).ceil() as u32,
                (theme::THUMB_H * 2.0).ceil() as u32
            )
        );
    }

    #[test]
    fn cache_dir_defaults_under_temp_dir() {
        // **リポジトリ内には置かない**ことの見張り。
        // 環境変数で差し替えられている時は既定を見られないので何もしない。
        if std::env::var_os(CACHE_DIR_ENV).is_some() {
            return;
        }
        assert!(cache_dir().starts_with(std::env::temp_dir()));
    }
}
