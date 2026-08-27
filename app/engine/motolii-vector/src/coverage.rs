//! `Raster` の**後処理** — 複数マスクの被覆(coverage)バッファ同士を画素ごとに
//! 重ねるブール代数(裁定160 発注γ MK2、`docs/reviews/2026-08-21-backend-gap-seam-survey.md`
//! の切片割り MK1→**MK2**→MK3)。
//!
//! crate doc の「出口は1本([`crate::render`]だけ)」は**図形の記述 → premultiplied
//! RGBA8 という経路**についての宣言であって、ここが競合するわけではない。この module
//! が扱うのは既にラスタライズが終わった後の raster 同士(coverage バイト列)で、
//! 図形記述を経由しない第二のラスタライズ経路を作るわけではない —
//! だからここだけは新しい公開名前空間(`motolii_vector::coverage::*`)として独立させ、
//! crate 根の `pub use` 一覧(`render` が読む語彙)には混ぜない。
//!
//! # AE 対応表(mask-mode、`next/reference/normal-map.tsv` の該当 id)
//!
//! | mode | tsv id | 画素毎の式(byte, 0..=255) | 意味 |
//! |---|---|---|---|
//! | [`add`] | 748/749 | `a.saturating_add(b)` | 論理和寄り。単位元は `0`(空) |
//! | [`subtract`] | 794/795 | `a.saturating_sub(b)` | 手前の覆いから引く。**非可換** |
//! | [`intersect`] | 759/760 | `a.min(b)` | 論理積寄り。単位元は `255`(全通過)。可換 |
//! | [`lighten`] | 763 | `a.max(b)` | add と同じ union 系(単位元 `0`) |
//! | [`darken`] | 752/753 | `a.min(b)` | **intersect と同じ式**(下記) |
//! | [`difference`] | 754/755 | `a.abs_diff(b)` | 対称差(XOR の連続版) |
//!
//! `intersect` と `darken` が同じ式なのは実装の手抜きではない — この crate の
//! coverage は二値化されたブール的な被覆率(裁定160 MK1 の白 fill トリック由来)であって、
//! フェザーの階調そのものは持たない。二値の世界では「両方に居る画素だけ残す」
//! (intersect の意味)と「暗い方(=覆いが薄い方)を採る」(darken の意味)は
//! 同じ画素集合に落ちる。AE の UI がこの2つを別モードとして持つのは操作者への
//! 説明のためであって、coverage 1chの式としては同型になる。
//!
//! # 先頭マスクの単位元(「topmost mask」の AE 挙動と同型)
//!
//! マスクが1枚も無ければ全通過(すべての画素が [`Coverage::full`])——layer は
//! 覆いを一切受けない。マスクが1枚以上あるとき、**列の先頭は「見えない手前の覆い」
//! との合成として畳まれる**が、この「見えない手前の覆い」は固定値ではなく
//! **先頭マスク自身の mode の単位元**を使う: [`add`]/[`lighten`] は `0`(空)、
//! [`subtract`]/[`intersect`]/[`darken`]/[`difference`] は `255`(全)。
//!
//! これが意図的な選択である理由: 常に `255`(全通過)から始めて先頭マスクの
//! mode を素直に適用すると、`add` は `255.saturating_add(m)` が即座に `255` へ
//! 飽和し、**先頭マスクの形が結果に一切効かない退化系**になる。実際の AE は
//! 「先頭(最上段)マスクの mode によって見た目が変わる」という既知の挙動を持つ
//! (Add/Lighten を先頭に置くとマスク内側だけが見える、Subtract/Difference を
//! 先頭に置くとマスク内側に**穴が開く**、Intersect/Darken を先頭に置くと
//! Add と同じ絵になる)。単位元をモードごとに選ぶ実装はこの実機挙動と一致する
//! (`next/engine/motolii-engine/src/mask.rs` の `fold_masks` が実際に畳む場所)。
//! マスクが0枚のときの「全通過」だけは、上記のどのモードにも属さない別枠の既定。

use crate::Raster;

/// 単一チャンネルの被覆(coverage)バッファ — 画素ごとに1バイト(`0..=255`)。
///
/// [`Raster`](4ch premultiplied RGBA)とは別の型 — mask のラスタライズは
/// [`crate::render`] を白 fill で呼んで alpha だけを読む取り決め
/// (`next/engine/motolii-engine/src/mask.rs` の doc 参照)なので、
/// [`Coverage::from_raster_alpha`] がその1chだけを抜き出す変換を持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    pub width: u32,
    pub height: u32,
    /// 行優先・隙間なし(`width * height` バイト)。
    pub bytes: Vec<u8>,
}

impl Coverage {
    /// 全画素 `255` — 「無 mask = 全通過」の既定、および intersect 系の単位元。
    pub fn full(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes: vec![255u8; (width as usize) * (height as usize)],
        }
    }

    /// 全画素 `0` — add/lighten 系の単位元。
    pub fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes: vec![0u8; (width as usize) * (height as usize)],
        }
    }

    /// [`Raster`] の alpha チャンネル(4バイトごとの4つ目)だけを抜き出す。
    ///
    /// mask のラスタライズは白の不透明 fill を固定して使う取り決め
    /// (`premultiplied` なので `R=G=B=alpha`)なので、alpha を読めば coverage
    /// そのものが読める(`next/engine/motolii-engine/src/mask.rs` の
    /// `rasterize_mask_coverage` doc と同じ前提)。
    pub fn from_raster_alpha(raster: &Raster) -> Self {
        let bytes = raster
            .premultiplied_rgba8
            .chunks_exact(4)
            .map(|px| px[3])
            .collect();
        Self {
            width: raster.width,
            height: raster.height,
            bytes,
        }
    }
}

/// 寸法が違う Coverage を渡された。**黙って小さい方に合わせたり panic したりしない**
/// (裁定37と同じ形 — mask の canvas がずれているのは呼び手のバグで、隠すと
/// 「マスクが効いていないように見える」という利用者から発見しづらい欠陥になる)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("coverage の寸法が合わない: {aw}x{ah} と {bw}x{bh}")]
pub struct CoverageSizeMismatch {
    pub aw: u32,
    pub ah: u32,
    pub bw: u32,
    pub bh: u32,
}

fn zip_pixels(
    a: &Coverage,
    b: &Coverage,
    f: impl Fn(u8, u8) -> u8,
) -> Result<Coverage, CoverageSizeMismatch> {
    if a.width != b.width || a.height != b.height {
        return Err(CoverageSizeMismatch {
            aw: a.width,
            ah: a.height,
            bw: b.width,
            bh: b.height,
        });
    }
    let bytes = a
        .bytes
        .iter()
        .zip(b.bytes.iter())
        .map(|(&x, &y)| f(x, y))
        .collect();
    Ok(Coverage {
        width: a.width,
        height: a.height,
        bytes,
    })
}

/// AE `Add`(tsv id 748/749)。飽和加算 — 論理和寄り。単位元は `0`。
pub fn add(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::saturating_add)
}

/// AE `Subtract`(tsv id 794/795)。飽和差 — `a` から `b` を引く。**非可換**。
pub fn subtract(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::saturating_sub)
}

/// AE `Intersect`(tsv id 759/760)。`min` — 論理積寄り。単位元は `255`。可換。
pub fn intersect(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::min)
}

/// AE `Lighten`(tsv id 763)。`max` — add と同じ union 系(単位元 `0`)。
pub fn lighten(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::max)
}

/// AE `Darken`(tsv id 752/753)。`min` — module doc の「intersect と同じ式」参照。
pub fn darken(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::min)
}

/// AE `Difference`(tsv id 754/755)。対称差(`|a-b|`)。可換だが intersect とは別式。
pub fn difference(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::abs_diff)
}
