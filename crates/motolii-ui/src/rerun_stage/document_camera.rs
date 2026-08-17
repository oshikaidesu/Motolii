//! Stage の既定カメラ — **composition 面へ正対する document camera**。
//!
//! 「編集時の既定表示も document camera(未定義なら正対)とし、現在の斜め固定視点は
//! view camera の初期値バグとして扱う」(2026-08-18 利用者裁定,
//! `docs/reviews/2026-08-18-rerun-as-composition-foundation.md`)。
//!
//! ここに在るのは**値の導出だけ**で、Rerun へ置くのは `adapter.rs`。
//!
//! なぜ probe から呼ばないか: 導出の実物は
//! `spikes/rerun-e0-composition-probe`(E0 (b))で先に測ったものだが、spike は
//! 使い捨ての足場である。製品が spike へ依存すると spike を畳めなくなるので、
//! **式は製品側に持つ**(probe 側はそのまま残す。fork rev を上げたときの
//! 常設 oracle として別の役目がある)。
//!
//! ## 世界の中での composition 面
//!
//! `SpatialStage::copy_gpu_image` は `cell_size = 1/height` で画像を置き、
//! `Transform3D::from_translation([-0.5*aspect, -0.5, -0.01])` で中央へ寄せる
//! (fork の `spatial_stage.rs`)。つまり合成フレームは**必ず**
//!
//! - 高さ 1.0(y ∈ [-0.5, 0.5])
//! - 幅 comp_aspect(x ∈ [-comp_aspect/2, comp_aspect/2])
//! - z = [`COMPOSITION_PLANE_Z`]、法線は +Z
//!
//! に立つ。Document の正準座標(高さ1.0固定・原点中央・Y-up)と同じ寸法なので、
//! 面の大きさを測り直す必要が無い — 縦横比だけ分かればカメラが決まる。

use re_view_spatial::StageCamera;

/// document camera の垂直画角(60°)。
///
/// 正対なので画角そのものは絵に影響しない(距離が画角に追随して決まるため)。
/// 極端に狭いと数値誤差が大きく、極端に広いと周辺のレイヤーが強く歪むので、
/// 実写カメラの標準レンズあたりに置いてある。E0 (b) で測った値と同じ。
pub(super) const DOCUMENT_CAMERA_FOV_Y: f32 = std::f32::consts::FRAC_PI_3;

/// 合成フレーム面の z。`copy_gpu_image` が自分の path へ書く値である。
pub(super) const COMPOSITION_PLANE_Z: f32 = -0.01;

/// composition 面の半分の高さ。世界の中での高さは常に 1.0。
pub(super) const COMPOSITION_HALF_HEIGHT: f32 = 0.5;

/// composition 面へ正対し、面が画枠に収まる document camera。
///
/// `comp_aspect` は composition の幅/高さ、`viewport_aspect` は Stage 面の幅/高さ。
/// どちらも「まだ分からない」ときは、もう片方と同じ形を渡す
/// ([`Self`] を呼ぶ側の [`super::EmbeddedSpatialStage`] がそうしている)
/// — その仮定では縦合わせと横合わせが一致するので、既知の側だけで距離が決まる。
pub(super) fn document_camera(comp_aspect: f32, viewport_aspect: f32) -> StageCamera {
    let distance = document_camera_distance(comp_aspect, viewport_aspect, DOCUMENT_CAMERA_FOV_Y);
    StageCamera::new(
        // 面の正面(+Z 側)に、面の中心の真正面から立つ。
        [0.0, 0.0, COMPOSITION_PLANE_Z + distance],
        [0.0, 0.0, COMPOSITION_PLANE_Z],
        // 画面の上 = Document の上。scene の view coordinates
        // (`adapter::STAGE_SCENE_VIEW_COORDINATES`)と同じ +Y。
        [0.0, 1.0, 0.0],
    )
    .with_fov_y_radians(DOCUMENT_CAMERA_FOV_Y)
}

/// composition 面が画枠に収まる、面からの距離。**絵合わせの定数ではなく式である。**
///
/// 距離 `d` の面に映るのは、透視投影の定義から
///
/// ```text
/// 可視半高 = d * tan(fov_y / 2)
/// 可視半幅 = 可視半高 * viewport_aspect
/// ```
///
/// これを comp の半高 0.5 / 半幅 `0.5 * comp_aspect` 以上にする `d` を、縦横それぞれ
/// 解いて**遠いほうを採る**。片方だけで決めると、画枠より横長の comp は左右が、
/// 縦長の comp は上下が切れる。
///
/// 等号で採るので comp の縁は画枠の縁にちょうど載る(余白を持たせない)。
/// 余白を入れると Rerun の world grid と背景がその帯に覗くため。
pub(super) fn document_camera_distance(comp_aspect: f32, viewport_aspect: f32, fov_y: f32) -> f32 {
    let comp_aspect = sane_aspect(comp_aspect);
    let viewport_aspect = sane_aspect(viewport_aspect);
    let half_fov_tan = (fov_y * 0.5).tan();
    let fit_height = COMPOSITION_HALF_HEIGHT / half_fov_tan;
    let fit_width = COMPOSITION_HALF_HEIGHT * comp_aspect / (half_fov_tan * viewport_aspect);
    fit_height.max(fit_width)
}

/// 縦横比として意味を成さない値(0 / 負 / NaN / ∞)は正方へ落とす。
///
/// ここへ来るのは面の大きさが未確定なフレームや、幅0の pane。カメラを NaN に
/// してしまうと Stage が丸ごと黒くなるので、**絵が出る値**へ倒す。
fn sane_aspect(aspect: f32) -> f32 {
    if aspect.is_finite() && aspect > 0.0 {
        aspect
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 距離と画角から、comp 平面の位置で画枠に収まる半幅・半高を出す。
    /// **透視投影の定義そのまま**で、[`document_camera_distance`] の当て方は通らない。
    fn half_visible_extent(distance: f32, fov_y: f32, viewport_aspect: f32) -> (f32, f32) {
        let half_height = distance * (fov_y * 0.5).tan();
        (half_height * viewport_aspect, half_height)
    }

    fn camera_distance(camera: &StageCamera) -> f32 {
        camera.position[2] - camera.look_target[2]
    }

    /// comp と画枠の形をひととおり振って、**どれも収まり、どちらか一辺が接する**こと。
    ///
    /// 収まるだけなら遠ざければ済むので、「接する」まで見て初めて
    /// 「画枠に合わせた」と言える。定数を1つ直書きした実装はここで落ちる
    /// — 縦長画枠と横長画枠が同じ距離になってしまうため。
    #[test]
    fn the_composition_touches_the_frame_and_never_spills_out() {
        // (comp_aspect, viewport_aspect)
        let cases = [
            (16.0 / 9.0, 16.0 / 9.0), // 画枠と同じ形
            (16.0 / 9.0, 4.0 / 3.0),  // 画枠のほうが縦長 → 横合わせが効く
            (4.0 / 3.0, 21.0 / 9.0),  // 画枠のほうが横長 → 縦合わせが効く
            (1.0, 1.0),               // 正方
            (2.35, 1.0),              // シネスコを正方の pane へ
            (9.0 / 16.0, 16.0 / 9.0), // 縦動画を横長 pane へ
        ];
        for (comp_aspect, viewport_aspect) in cases {
            let camera = document_camera(comp_aspect, viewport_aspect);
            let fov_y = camera.fov_y_radians.expect("画角が置かれていない");
            let (half_visible_w, half_visible_h) =
                half_visible_extent(camera_distance(&camera), fov_y, viewport_aspect);

            let half_comp_w = COMPOSITION_HALF_HEIGHT * comp_aspect;
            let slack_w = half_visible_w - half_comp_w;
            let slack_h = half_visible_h - COMPOSITION_HALF_HEIGHT;

            const EPSILON: f32 = 1.0e-5;
            assert!(
                slack_w > -EPSILON,
                "comp {comp_aspect} / 画枠 {viewport_aspect}: 左右がはみ出している(余り {slack_w})"
            );
            assert!(
                slack_h > -EPSILON,
                "comp {comp_aspect} / 画枠 {viewport_aspect}: 上下がはみ出している(余り {slack_h})"
            );
            assert!(
                slack_w.abs() < EPSILON || slack_h.abs() < EPSILON,
                "comp {comp_aspect} / 画枠 {viewport_aspect}: どの辺も画枠に接していない\
                 (余り 横 {slack_w} / 縦 {slack_h}) — 画枠に合わせていない"
            );
        }
    }

    /// 画角90°・正方の画枠に正方の comp なら、距離は解析的に 0.5。
    /// `tan(45°) = 1` なので `0.5 / 1 = 0.5` である。
    ///
    /// 上の照合が「実装の式を実装の式で検算」に見えないよう、閉じた形も1点置く。
    #[test]
    fn a_right_angle_lens_frames_the_unit_square_from_half_a_unit_away() {
        let distance = document_camera_distance(1.0, 1.0, std::f32::consts::FRAC_PI_2);
        assert!(
            (distance - 0.5).abs() < 1.0e-5,
            "画角90°での距離が 0.5 でない: {distance}"
        );
    }

    /// 距離は画角に追随する。狭い画角ほど下がる。
    /// 画角を無視した定数はここで落ちる。
    #[test]
    fn a_narrower_lens_stands_further_back() {
        let wide = document_camera_distance(1.0, 1.0, std::f32::consts::FRAC_PI_2);
        let narrow = document_camera_distance(1.0, 1.0, std::f32::consts::FRAC_PI_2 * 0.5);
        assert!(
            narrow > wide * 2.0,
            "画角を半分にしても距離が伸びていない: 広 {wide} / 狭 {narrow}"
        );
    }

    /// カメラは面の中心の真正面に立ち、面のほうを向く。
    #[test]
    fn the_camera_stands_in_front_of_the_plane_centre() {
        let camera = document_camera(16.0 / 9.0, 4.0 / 3.0);
        assert_eq!(camera.look_target, [0.0, 0.0, COMPOSITION_PLANE_Z]);
        assert_eq!(camera.position[0], 0.0);
        assert_eq!(camera.position[1], 0.0);
        assert!(camera.position[2] > COMPOSITION_PLANE_Z);
        assert_eq!(camera.up, [0.0, 1.0, 0.0]);
    }

    /// 縦横比が壊れていても、カメラは有限のまま(Stage を黒くしない)。
    #[test]
    fn a_broken_aspect_still_yields_a_camera_that_can_draw() {
        for broken in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            let camera = document_camera(broken, broken);
            assert!(
                camera.position.iter().all(|value| value.is_finite()),
                "縦横比 {broken} でカメラ位置が壊れた: {camera:?}"
            );
            // 正方へ倒れるので、正方の comp をちょうど収める距離になる。
            let square = document_camera_distance(1.0, 1.0, DOCUMENT_CAMERA_FOV_Y);
            assert!(
                (camera_distance(&camera) - square).abs() < 1.0e-5,
                "縦横比 {broken} が正方へ倒れていない"
            );
        }
    }
}
