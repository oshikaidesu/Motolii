//! `Composition.camera` の投影数学(裁定113/115)。
//!
//! **世界は1つ・カメラは1つ・全員 z=0 既定**(裁定113)。カメラレイヤ・レイヤ別カメラ・
//! shot switch は持たない。preview は透視カメラの 2.5D で「インパクト重視」(裁定115) —
//! 層の z を動かした瞬間に視差が出る。**向きは固定軸のまま**: カメラは常に world +Z を
//! 向く(旧正本が Spatial 変種の難所に挙げた「向きの表現・handedness・特異点」はまだ
//! 開けない)。動くのは `center`(パン。実体はカメラ位置の平行移動 = truck — 視線の向きは
//! 変えないので、動かすだけで depth に応じた視差が自然に出る)/ `zoom`(実効画角。
//! カメラそのものは動かさない、レンズのズームと同じ扱い)/ `roll`(view の前方軸まわりの
//! 回転)だけ。
//!
//! **AE と同じ符号**: layer の z が大きいほどカメラから遠ざかる(奥へ退く)。カメラは
//! z=0 平面より手前(-Z 側)に置き、+Z を向く。
//!
//! ここが「投影の正本」— `LayerPlacement::from_transform` が変換順序の正本であるのと
//! 同じ形(1箇所だけで組む)。`motolii-compositor` は `camera_projection` が返す値を
//! そのまま `re_renderer::view_builder::Projection::Perspective` と
//! `macaw::IsoTransform` へ渡すだけで、ここでの計算をやり直さない。

use crate::CompSpec;

/// この comp 時刻でのカメラの姿。track が無ければ既定(裁定20 と同じ扱い)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedCamera {
    /// comp 中心からのパン量(ピクセル、world 単位)。**絶対位置ではなくオフセット** —
    /// 他の property と同じく「キーを打っていない = 0(パン無し)」にするため(裁定20)。
    pub center: [f32; 2],
    /// 1.0 が既定(パンなし・zoomなしの元の画角)。値が大きいほど拡大して見える。
    pub zoom: f32,
    /// 度・時計回り(`property::ROTATION` と同じ符号の約束)。view の前方軸まわりの回転。
    pub roll_degrees: f32,
}

impl Default for ResolvedCamera {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            zoom: 1.0,
            roll_degrees: 0.0,
        }
    }
}

/// zoom=1 での垂直画角(度)。
///
/// **自由なデザイン定数**であることを明記する — 裁定113/115 はカメラの動く量を
/// center/zoom/roll としか決めておらず、「zoom=1 のときの実際の画角」はどこにも
/// 書かれていない(平面正射影には画角という概念自体が無いため、決定の対象外だった)。
/// 55° は「近づく/離れるで奥行きに気付く程度の遠近感が出るが、極端な広角(fisheye 感)には
/// ならない」中庸値(35mm 判のやや広角寄りの標準レンズに近い)。preview が
/// 「インパクト重視」(裁定115)なので、遠近感がほぼ出ない極端な望遠は避けた。
pub const CAMERA_BASE_VERTICAL_FOV_DEGREES: f32 = 55.0;

/// GPU へ渡す形(view の位置・向き、投影のパラメータ)。
#[derive(Clone, Copy, Debug)]
pub struct CameraProjection {
    /// world 空間でのカメラの位置。
    pub eye: glam::Vec3,
    /// world → view の回転。`macaw::IsoTransform` の `rotation` へそのまま渡せる。
    pub rotation: glam::Quat,
    pub vertical_fov_radians: f32,
    pub aspect_ratio: f32,
    pub near_plane_distance: f32,
}

/// カメラ位置から z=0 平面までの距離。**zoom によらず固定**。
///
/// zoom はレンズの画角を変えるだけでカメラの物理的な位置は動かさない(実カメラの
/// ズームレンズと同じ扱い)。ドリー(距離そのものを変える)は裁定115 の変数に無いので
/// 実装しない。
fn base_distance(comp: CompSpec) -> f32 {
    let half_fov = (CAMERA_BASE_VERTICAL_FOV_DEGREES * 0.5).to_radians();
    (comp.height as f32 * 0.5) / half_fov.tan()
}

/// world の z 平面(`LayerPlacement::z`)からカメラまでの距離。**AE と同じ符号** —
/// z が大きいほど遠い。z が `-base_distance(comp)` に近づく/超えるとカメラの
/// near plane を割り込む(**既知の限界**、裁定に自由変数が無いための帰結。近すぎる/
/// 背後に回った層は再クリップされるかカメラの裏に出る。未対応)。
pub fn distance_from_camera(comp: CompSpec, z: f32) -> f32 {
    base_distance(comp) + z
}

/// `comp`/`camera` から view + projection を組む。**投影の正本はここ1箇所**。
pub fn camera_projection(comp: CompSpec, camera: ResolvedCamera) -> CameraProjection {
    let distance = base_distance(comp);
    // zoom<=0 は画角が発散する(定義できない)ので下限で止める。
    let zoom = camera.zoom.max(1e-3);
    let half_base_fov = (CAMERA_BASE_VERTICAL_FOV_DEGREES * 0.5).to_radians();
    let vertical_fov_radians = 2.0 * (half_base_fov.tan() / zoom).atan();

    let eye = glam::vec3(
        comp.width as f32 * 0.5 + camera.center[0],
        comp.height as f32 * 0.5 + camera.center[1],
        -distance,
    );

    // 向きの固定軸: 常に world +Z を向く。180°(X軸まわり)は world の
    // (x右, y下, z=カメラから見て奥)を re_renderer の view 規約
    // (x右, y上, camera は -Z を向く)へ揃えるための基準姿勢で、利用者からは
    // 動かせない(裁定115「向きの表現はまだ開けない」)。
    let base = glam::Quat::from_axis_angle(glam::Vec3::X, std::f32::consts::PI);
    // roll は基準姿勢の後、view の前方軸まわりに掛ける追加回転。時計回り正
    // (`property::ROTATION` と同じ約束)にするため角度を反転する
    // (view の Z 軸は画面手前を向くので、+Z まわりの数学的正回転は画面上は反時計回り)。
    let roll = glam::Quat::from_axis_angle(glam::Vec3::Z, -camera.roll_degrees.to_radians());
    let rotation = roll * base;

    CameraProjection {
        eye,
        rotation,
        vertical_fov_radians,
        aspect_ratio: comp.width as f32 / comp.height as f32,
        // near は distance よりだいぶ手前に置く。z を大きく手前(カメラに近い側)へ
        // 動かした層が near を割り込むとクリップされる(前段の doc 参照、既知の限界)。
        near_plane_distance: (distance * 1e-3).max(1e-3),
    }
}

impl CameraProjection {
    /// world → view。`macaw::IsoTransform::transform_point3` と同じ規約
    /// (`rotation * p + translation`)で組み立てられる形は呼び手(`motolii-compositor`)
    /// 側に譲る — ここは `Mat4` で完結させ、macaw への依存を core に持ち込まない。
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_quat(self.rotation) * glam::Mat4::from_translation(-self.eye)
    }

    pub fn projection_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_infinite_reverse_rh(
            self.vertical_fov_radians,
            self.aspect_ratio,
            self.near_plane_distance,
        )
    }
}

/// 固定した world z の平面に限れば、このカメラの写像は**アフィン**である
/// (`w = -view_z` が x,y に依存しない定数になるため、透視除算後もアフィンのまま)。
///
/// pinned layer(裁定113)は**このアフィンの逆行列**を層の transform に掛けてから
/// カメラへ渡すことで、カメラ(center/zoom/roll)がどう動いても画面上不動になる
/// (2パス目を増やさずに済む — 実装は `motolii-compositor` 側)。
///
/// z=0 で呼んだ結果が「全層 z=0・カメラ既定のとき正射影と一致する」試験の土台でもある:
/// 既定カメラでの `camera_screen_from_world_at_z(comp, ResolvedCamera::default(), 0.0)` は
/// 恒等写像(x,y をそのままピクセルへ)になるはずで、`tests` がそれを機械精度で縛る。
pub fn camera_screen_from_world_at_z(
    comp: CompSpec,
    camera: ResolvedCamera,
    z: f32,
) -> glam::Affine2 {
    let projection = camera_projection(comp, camera);
    let clip_from_world = projection.projection_matrix() * projection.view_matrix();

    let to_pixel = |world_xy: glam::Vec2| -> glam::Vec2 {
        let clip = clip_from_world * glam::Vec4::new(world_xy.x, world_xy.y, z, 1.0);
        let ndc = glam::vec2(clip.x / clip.w, clip.y / clip.w);
        // NDC: x は -1(左)..+1(右)。y は +1(上)..-1(下)(`perspective_infinite_reverse_rh`
        // は標準 RH: view の +Y は画面の上)。pixel は原点左上・y 下向き(comp 座標、裁定14)。
        glam::vec2(
            (ndc.x + 1.0) * 0.5 * comp.width as f32,
            (1.0 - ndc.y) * 0.5 * comp.height as f32,
        )
    };

    let origin = to_pixel(glam::Vec2::ZERO);
    let x_axis = to_pixel(glam::Vec2::X) - origin;
    let y_axis = to_pixel(glam::Vec2::Y) - origin;
    glam::Affine2::from_cols(x_axis, y_axis, origin)
}

/// [`camera_screen_from_world_at_z`] の z=0 版。pinned layer の「カメラを打ち消す」用途は
/// 常にこの平面を基準にする(裁定113 の既定平面)。
pub fn camera_screen_from_world_z0(comp: CompSpec, camera: ResolvedCamera) -> glam::Affine2 {
    camera_screen_from_world_at_z(comp, camera, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMP: CompSpec = CompSpec {
        width: 640,
        height: 360,
    };

    fn approx(a: glam::Vec2, b: glam::Vec2, eps: f32) {
        assert!(
            (a.x - b.x).abs() < eps && (a.y - b.y).abs() < eps,
            "{a:?} != {b:?} (eps={eps})"
        );
    }

    /// **縛る試験1**: 全層 z=0・カメラ既定のとき、z=0 平面の写像は恒等
    /// (comp のピクセル座標をそのまま返す) — 旧正射影(裁定14)と一致する土台。
    #[test]
    fn default_camera_matches_identity_pixel_mapping_at_z0() {
        let affine = camera_screen_from_world_z0(COMP, ResolvedCamera::default());
        for corner in [
            glam::vec2(0.0, 0.0),
            glam::vec2(COMP.width as f32, 0.0),
            glam::vec2(0.0, COMP.height as f32),
            glam::vec2(COMP.width as f32, COMP.height as f32),
            glam::vec2(320.0, 180.0),
        ] {
            approx(affine.transform_point2(corner), corner, 1e-2);
        }
    }

    /// **縛る試験3前半**: zoom で z=0 の板が comp 中心を軸に等方に拡大する。
    #[test]
    fn zoom_scales_the_z0_plane_isotropically_around_comp_center() {
        let camera = ResolvedCamera {
            zoom: 2.0,
            ..Default::default()
        };
        let affine = camera_screen_from_world_z0(COMP, camera);
        let center = glam::vec2(COMP.width as f32 * 0.5, COMP.height as f32 * 0.5);

        approx(affine.transform_point2(center), center, 1e-2);

        let probe = center + glam::vec2(50.0, 30.0);
        let mapped = affine.transform_point2(probe);
        let expected = center + (probe - center) * 2.0;
        approx(mapped, expected, 1e-1);
    }

    /// **縛る試験3後半**: roll で z=0 の板が comp 中心まわりに時計回りへ回る。
    /// `property::ROTATION` と同じ「度・時計回り」の約束に揃えてある。
    #[test]
    fn roll_rotates_the_z0_plane_clockwise_around_comp_center() {
        let camera = ResolvedCamera {
            roll_degrees: 90.0,
            ..Default::default()
        };
        let affine = camera_screen_from_world_z0(COMP, camera);
        let center = glam::vec2(COMP.width as f32 * 0.5, COMP.height as f32 * 0.5);

        // 中心から「右」(+X)にある点は、時計回り90°で「下」(+Y)へ来るはず
        // (12時の位置が3時へ来るのと同じ向き)。
        let right = center + glam::vec2(40.0, 0.0);
        let mapped = affine.transform_point2(right);
        let expected_down = center + glam::vec2(0.0, 40.0);
        approx(mapped, expected_down, 1e-1);
    }

    /// パン(center)は camera 位置の平行移動(truck)。**カメラを右へ動かすと絵は
    /// 左へ寄る**(自分が右へ動けば景色は相対的に左へ流れる、実カメラと同じ)。
    #[test]
    fn panning_the_camera_shifts_the_z0_plane_the_opposite_way() {
        let camera = ResolvedCamera {
            center: [50.0, 0.0],
            ..Default::default()
        };
        let affine = camera_screen_from_world_z0(COMP, camera);
        let comp_center = glam::vec2(COMP.width as f32 * 0.5, COMP.height as f32 * 0.5);
        let mapped = affine.transform_point2(comp_center);
        assert!(
            mapped.x < comp_center.x,
            "center を +X へ動かしたら絵は -X 側へ寄るはず: {mapped:?}"
        );
    }

    /// **縛る試験2の土台**: z が違う2枚の板は、同じ camera pan でも画面上の移動量が違う
    /// (視差)。カメラに近い(z が小さい/カメラ寄りの)板の方が大きく動く。
    #[test]
    fn nearer_planes_move_more_than_farther_planes_under_the_same_pan() {
        let before = ResolvedCamera::default();
        let after = ResolvedCamera {
            center: [30.0, 0.0],
            ..Default::default()
        };

        let near_z = -200.0; // カメラに近い(手前)
        let far_z = 400.0; // カメラから遠い(奥)

        let probe = glam::vec2(COMP.width as f32 * 0.5, COMP.height as f32 * 0.5);

        let near_before = camera_screen_from_world_at_z(COMP, before, near_z).transform_point2(probe);
        let near_after = camera_screen_from_world_at_z(COMP, after, near_z).transform_point2(probe);
        let far_before = camera_screen_from_world_at_z(COMP, before, far_z).transform_point2(probe);
        let far_after = camera_screen_from_world_at_z(COMP, after, far_z).transform_point2(probe);

        let near_delta = (near_after - near_before).length();
        let far_delta = (far_after - far_before).length();

        assert!(
            near_delta > far_delta,
            "近い板({near_delta})が遠い板({far_delta})より大きく動くはず"
        );
    }

    /// z=0 の距離は zoom によらず一定(zoom はレンズの画角だけを変える設計)。
    #[test]
    fn distance_from_camera_is_stable_across_zoom() {
        let d = distance_from_camera(COMP, 0.0);
        assert!(d > 0.0);
        // z が大きいほど遠い(AE と同じ符号)。
        assert!(distance_from_camera(COMP, 100.0) > d);
        assert!(distance_from_camera(COMP, -100.0) < d);
    }
}
