//! NDC で示された落下点を canonical 座標へ戻す。
//!
//! **`app/`(旧egui アプリ)から引き剥がしたもの。** 中身は `CompCamera` の逆射影だけで
//! アプリの状態を一切見ないのに、`app/browser.rs` に置かれていたせいで
//! `product_runtime/` の14ファイルが旧アプリ側へ依存していた。
//! 旧アプリを畳むための最初の一歩として、依存の向きだけを正した(実装は不変)。

use motolii_core::{CanonicalPoint, CompCamera};

/// NDC(-1..1)の点を canonical 座標へ戻す。逆射影して往復が一致しなければ `None`。
///
/// 往復の照合を残しているのは、カメラが退化しているとき(高さ0、aspect 0 など)に
/// 数値だけは出てしまうため。`1e-9` は元実装のまま。
pub(crate) fn canonical_drop_from_ndc(camera: CompCamera, ndc: [f64; 2]) -> Option<[f64; 2]> {
    if !ndc[0].is_finite() || !ndc[1].is_finite() {
        return None;
    }
    let qx =
        ndc[0] * camera.aspect_num() as f64 / camera.aspect_den() as f64 * camera.height() / 2.0;
    let qy = ndc[1] * camera.height() / 2.0;
    let cos_r = camera.roll_radians().cos();
    let sin_r = camera.roll_radians().sin();
    let center = camera.center();
    let point = CanonicalPoint {
        x: center.x + cos_r * qx - sin_r * qy,
        y: center.y + sin_r * qx + cos_r * qy,
    };
    let projected = camera.world_to_ndc(point).ok()?;
    if !point.x.is_finite()
        || !point.y.is_finite()
        || (projected.0 - ndc[0]).abs() > 1e-9
        || (projected.1 - ndc[1]).abs() > 1e-9
    {
        return None;
    }
    Some([point.x, point.y])
}
