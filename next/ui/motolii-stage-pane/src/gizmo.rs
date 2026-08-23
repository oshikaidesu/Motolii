//! Stage ギズモ第1弾(裁定124: スクラッチ・意味の手本=AE・見た目=先例の慣習形) —
//! 選択レイヤーの bbox+ハンドル8点+回転ハンドル+anchor 表示と、
//! drag = move(本体)/scale(角・辺)/rotate(回転ハンドル)。
//!
//! ## 意味(全部実装済みの経路を呼ぶだけ — ここは「絵と手」)
//!
//! - 値の意味は Inspector 値編集と同じ property(`position`/`scale`/`rotation`)。
//!   このモジュールは **Document を一切書かない** — [`GizmoDrag`] message を publish
//!   するだけで、shell 側(結線は supervisor)が Inspector drag と同じ経路
//!   (`Document::set_transient` → 確定時に `Intent::SetTrack`、キー持ち track へは
//!   AE 作法の playhead upsert)へ写す。**1 drag = 1 commit**(1 gesture = 1 undo)。
//! - **scale/rotate の不動点は anchor**(AE と同じ: comp panel のハンドル drag は
//!   Scale/Rotation property だけを書き、position は動かさない — anchor が数学的な
//!   不動点になる)。move は position だけを書く。
//! - Shift = 比率固定(scale、map 680「Modify Scale constrained to aspect ratio」)/
//!   15° スナップ(rotate、map 679「Modify Rotation in 15° increments」)。
//!   Esc = キャンセル(transient を捨てて drag 前の値へ戻る)。
//!
//! ## 見た目(AE/Figma/Canva 系の慣習形 — 独自形を発明しない)
//!
//! - bbox hairline(accent =「選択の器」)+角4・辺4 の正方形ハンドル8点
//!   (Figma/AE の selection handles と同形)。
//! - 回転ハンドル = 上辺中点から外側へ stem で繋いだ小円(Canva/PowerPoint/
//!   Google Slides の慣習形 — Figma の「角の外側の不可視ゾーン」は Q0
//!   (触れそうで触れない/触れるのに見えない)に反するので採らない)。
//! - anchor = ⊕(円+十字、AE のアンカーポイントの慣習形)。第1切片は表示のみ
//!   だったが、**第2切片(B22 波、2026-08-22)で drag 対象**: anchor drag =
//!   anchor 変更+position 補償(AE の pan-behind と同じ「見た目不動」 —
//!   [`anchor_value`] doc の導出参照)。2 property を [`GizmoValue::Anchor`] が
//!   対で運ぶ(片方だけ書くと絵が跳ぶため、1 message に両方乗せる)。
//!
//! ## 座標系(このモジュール内は **bounds ローカル** で閉じる)
//!
//! `canvas::Program` の `bounds` は window 絶対座標で届くが、`draw` の `Frame` は
//! bounds 原点へ translate された**ローカル座標**、`cursor.position_in(bounds)` も
//! ローカル座標を返す(iced_widget-0.14 `canvas.rs::draw` の `with_translation` 実測)。
//! そのためここでは letterbox 矩形を **bounds 原点=0 に正規化してから** 組む
//! ([`letterbox_screen_from_comp`])— 入力(hit)と出力(描画)が同じローカル系に
//! 揃う。写像の合成は既存部品だけ(新しい投影数学は無い):
//!
//! ```text
//! screen_from_local = letterbox ∘ camera_screen_from_world_z0(comp, 観測 or レンダリングカメラ)
//!                     ∘ world_from_parent ∘ LayerPlacement::from_transform(局所値)
//! ```
//!
//! ## テスト方針(モジュール冒頭 lib.rs と同じ)
//!
//! 計算は全て純関数([`gizmo_layout`]・[`gizmo_hit_test`]・[`move_value`]・
//! [`scale_value`]・[`rotation_value`]・[`GizmoDragState`])— `canvas::Program`
//! (`GizmoOverlay`)はこれらへ委譲するだけの薄い翻訳層。試験は純関数を直接呼ぶ。

use glam::Affine2;

/// scale 解の分母(`handle - anchor` の成分、レイヤーローカル px)がこれ未満なら
/// その軸は解けない(anchor がハンドルの線上にある)— 開始時の scale を保つ。
const SOLVE_EPS: f32 = 1e-3;

/// `Affine2::inverse()` への**唯一の生呼び出し口**(scale=0 の det=1 特例を除く —
/// [`scale_value`] の `SAFE-INVERSE` 注記参照)。退化行列(det=0、例: scale 0)へ
/// glam の `Mat2::inverse()` を呼ぶと、結果を返す前に `glam_assert!(det != 0.0)` で
/// 自己アサートして panic する(`debug-glam-assert`/`glam-assert` feature が
/// workspace のどこかで有効化され unify されているため) — 「呼んでから
/// `is_finite()` で後始末する」だと**その場で panic**して後始末に辿り着けない。
/// det を先に見て、退化(0 または非有限)なら inverse を呼ばずに `None` を返す。
///
/// この製品での「退化しうる `Affine2` の逆行列」の**唯一の正解の形**
/// (`anchor_value` で最初に着地した形をここへ集約した)。新しい流儀を発明せず、
/// 退化しうる `.inverse()` はここを経由すること — `tests/inverse_fence.rs` が
/// この規律を縛る。
pub(crate) fn checked_inverse(m: Affine2) -> Option<Affine2> {
    let det = m.matrix2.determinant();
    if !det.is_finite() || det == 0.0 {
        return None;
    }
    // SAFE-INVERSE: 直前で det を検査済み(この関数の外では生 inverse を呼ばない)。
    let inv = m.inverse();
    inv.is_finite().then_some(inv)
}

// ---------------------------------------------------------------------------
// pane ローカル Message(発注書「`GizmoDrag { target_property, phase }` 級」)。
// 既存の [`crate::Message`] へ variant を足すと shell 側の exhaustive match が
// 壊れる(shell はこのレーンの write-set 外)ため、**独立した message 型**にして
// ある — supervisor が root 側で `.map(...)` して畳む。
// ---------------------------------------------------------------------------

mod drag;
mod handles;
mod math;
mod overlay;
mod types;

pub use drag::GizmoDragState;
pub use handles::{gizmo_hit_test, gizmo_layout, GizmoHandle, GizmoLayout, ScaleHandle, SCALE_HANDLES};
pub(crate) use handles::letterbox_screen_from_comp;
pub use math::{anchor_value, move_value, rotation_value, scale_value};
pub use overlay::{resize_interaction, GizmoInteraction, GizmoOverlay};
pub use types::{gizmo_target, GizmoDrag, GizmoPhase, GizmoProperty, GizmoTarget, GizmoValue};

#[cfg(test)]
mod tests;
