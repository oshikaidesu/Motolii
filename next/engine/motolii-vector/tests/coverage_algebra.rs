//! mask 消化 第2切片(MK2、R9 の切片割り)のオラクル — coverage 被覆合成代数。
//!
//! **先に落ちる試験として書いた**(実装より前)。`motolii_vector::coverage` は
//! まだ存在しない。見るのは全部 byte 列であって、内部の tiny-skia 実装には触れない
//! (`motolii_vector::render` の出力を模した `Coverage` を直接組み立てて代数だけを試す
//! — ラスタライズ自体は MK1 のオラクル `mask_rasterize.rs` が既に縛っている)。
//!
//! 代数の性質(裁定に相当する規約、AE `mask-mode` 同型):
//! - `add`      = 飽和加算(`u8::saturating_add`) — 論理和寄り。単位元は `0`
//! - `subtract` = 飽和差(`u8::saturating_sub`) — 手前の覆いから引く。**非可換**
//! - `intersect`= `min` — 論理積寄り。単位元は `255`。**可換**
//! - `lighten`  = `max`(union 系、add と同じ単位元 `0`)
//! - `darken`   = `min`(intersect と同じ式 — 二値化された coverage では
//!   AE の Intersect と Darken は同じ絵になる。裁定は `coverage.rs` の doc 参照)
//! - `difference` = 対称差(`u8::abs_diff`)

use motolii_vector::coverage::{add, difference, intersect, subtract, Coverage};

/// Coverage を手で組み立てる。値は読みやすさのため u8 の列のまま並べる。
fn cov(width: u32, height: u32, bytes: impl Into<Vec<u8>>) -> Coverage {
    Coverage {
        width,
        height,
        bytes: bytes.into(),
    }
}

// ---------------------------------------------------------------------------
// (a) 単位元
// ---------------------------------------------------------------------------

/// `add` の単位元は空(全画素 `0`)。`空 add b == b`。
#[test]
fn add_with_empty_is_identity() {
    let b = cov(2, 2, [10, 200, 0, 255]);
    let empty = Coverage::empty(2, 2);
    let result = add(&empty, &b).expect("同じ寸法なので落ちないはず");
    assert_eq!(result.bytes, b.bytes, "空を add しても b と一致しない");
}

/// `intersect` の単位元は全通過(全画素 `255`)。`全通過 intersect b == b`。
#[test]
fn intersect_with_full_is_identity() {
    let b = cov(2, 2, [10, 200, 0, 255]);
    let full = Coverage::full(2, 2);
    let result = intersect(&full, &b).expect("同じ寸法なので落ちないはず");
    assert_eq!(
        result.bytes, b.bytes,
        "全通過を intersect しても b と一致しない"
    );
}

// ---------------------------------------------------------------------------
// (b) 可換/非可換
// ---------------------------------------------------------------------------

/// `intersect` (= min) は可換。
#[test]
fn intersect_is_commutative() {
    let a = cov(2, 2, [10, 200, 50, 255]);
    let b = cov(2, 2, [80, 5, 50, 0]);
    let ab = intersect(&a, &b).expect("同じ寸法");
    let ba = intersect(&b, &a).expect("同じ寸法");
    assert_eq!(ab.bytes, ba.bytes, "min は可換のはずが結果が違う");
}

/// `subtract` (= 飽和差) は非可換 — 手前の覆いから引く操作なので順序が意味を持つ。
#[test]
fn subtract_is_not_commutative() {
    let a = cov(2, 2, [200, 10, 255, 0]);
    let b = cov(2, 2, [50, 100, 0, 0]);
    let ab = subtract(&a, &b).expect("同じ寸法");
    let ba = subtract(&b, &a).expect("同じ寸法");
    assert_ne!(
        ab.bytes, ba.bytes,
        "a-b と b-a が一致した。この入力では非可換であるはず"
    );
    // 手計算: a-b = [200-50, 10-100(飽和0), 255-0, 0-0] = [150, 0, 255, 0]
    assert_eq!(ab.bytes, vec![150, 0, 255, 0]);
    // 手計算: b-a = [50-200(飽和0), 100-10, 0-255(飽和0), 0-0] = [0, 90, 0, 0]
    assert_eq!(ba.bytes, vec![0, 90, 0, 0]);
}

/// `add` は飽和する — `200 + 100` は `255` に飽和して単純な数値和にならない。
#[test]
fn add_saturates_instead_of_wrapping() {
    let a = cov(1, 1, [200]);
    let b = cov(1, 1, [100]);
    let result = add(&a, &b).expect("同じ寸法");
    assert_eq!(
        result.bytes,
        vec![255],
        "u8 を超えたら飽和するはず(u8オーバーフローの折り返しはNG)"
    );
}

/// `difference` は対称差 — `|a-b|` で順序を問わず同じ値になる(可換だが intersect とは別式)。
#[test]
fn difference_is_symmetric_and_matches_hand_math() {
    let a = cov(2, 2, [200, 10, 255, 0]);
    let b = cov(2, 2, [50, 100, 0, 0]);
    let ab = difference(&a, &b).expect("同じ寸法");
    let ba = difference(&b, &a).expect("同じ寸法");
    assert_eq!(ab.bytes, ba.bytes, "対称差なので a-b も b-a も同じはず");
    assert_eq!(ab.bytes, vec![150, 90, 255, 0]);
}

// ---------------------------------------------------------------------------
// (c) byte 決定論
// ---------------------------------------------------------------------------

/// 同じ入力から2回計算しても byte 一致する(CPU の純関数であることを縛る)。
#[test]
fn same_inputs_produce_byte_identical_results_twice() {
    let a = cov(
        4,
        4,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    );
    let b = cov(
        4,
        4,
        [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
    );
    let first = add(&a, &b).expect("同じ寸法");
    let second = add(&a, &b).expect("同じ寸法");
    assert_eq!(first.bytes, second.bytes, "同じ入力なのに byte が違う");
}

// ---------------------------------------------------------------------------
// (d) 寸法不一致は黙って通さない(裁定37 と同じ形)
// ---------------------------------------------------------------------------

/// 寸法の違う Coverage 同士を合成しようとしたら `Err`。黙って小さい方に合わせたり
/// panic したりしない — 「マスクの canvas がずれている」というバグを隠さない。
#[test]
fn mismatched_dimensions_are_rejected_not_silently_coerced() {
    let a = Coverage::empty(2, 2);
    let b = Coverage::empty(3, 3);
    assert!(
        add(&a, &b).is_err(),
        "寸法が違う Coverage の合成が黙って通ってしまった"
    );
}
