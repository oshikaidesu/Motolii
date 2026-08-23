use super::*;
use glam::Vec2;
use iced::{mouse, Point, Rectangle};
use motolii_core::{camera_screen_from_world_z0, CompSpec, LayerPlacement, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_store::{
    property, Composition, Document, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrsPatch,
    LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};
use motolii_tokens_rs::Dimensions;

const COMP: CompSpec = CompSpec {
    width: 640,
    height: 360,
};

fn bounds() -> Rectangle {
    Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0))
}

/// 100x50 の層を comp 中央へ、anchor は内容中央。bbox は (270,155)-(370,205)。
fn target() -> GizmoTarget {
    GizmoTarget {
        layer: LayerId(1),
        size: [100.0, 50.0],
        anchor: [50.0, 25.0],
        position: [320.0, 180.0],
        scale: [1.0, 1.0],
        rotation_degrees: 0.0,
        skew_degrees: 0.0,
        skew_axis_degrees: 0.0,
        world_from_parent: Affine2::IDENTITY,
    }
}

fn dims() -> Dimensions {
    Dimensions::default()
}

fn layout_with(observation: Option<ObservationCamera>, target: &GizmoTarget) -> GizmoLayout {
    gizmo_layout(
        bounds(),
        COMP,
        ResolvedCamera::default(),
        observation,
        target,
        dims(),
    )
    .expect("layout が組めるはず")
}

fn approx_point(actual: Point, expected: (f32, f32)) {
    assert!(
        (actual.x - expected.0).abs() < 1e-2 && (actual.y - expected.1).abs() < 1e-2,
        "{actual:?} != {expected:?}"
    );
}

/// 値の照合(f32 の逆行列を経由するので機械精度ではなく 1e-3 の帯で見る —
/// screen px/scale/度のどの単位でも視認できない差)。
fn approx_value(actual: GizmoValue, expected: GizmoValue) {
    let vec2_close =
        |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3;
    let ok = match (actual, expected) {
        (GizmoValue::Position(a), GizmoValue::Position(b))
        | (GizmoValue::Scale(a), GizmoValue::Scale(b)) => vec2_close(a, b),
        (GizmoValue::Rotation(a), GizmoValue::Rotation(b)) => (a - b).abs() < 1e-3,
        (
            GizmoValue::Anchor {
                anchor: a1,
                position: p1,
            },
            GizmoValue::Anchor {
                anchor: a2,
                position: p2,
            },
        ) => vec2_close(a1, a2) && vec2_close(p1, p2),
        _ => false,
    };
    assert!(ok, "{actual:?} != {expected:?}");
}

// -----------------------------------------------------------------
// layout(カメラ変換下の座標 — 発注 ORACLE)
// -----------------------------------------------------------------

/// 既定カメラ・恒等 letterbox なら bbox は層の comp 矩形そのもの。
#[test]
fn layout_puts_the_bbox_at_the_layers_comp_rect() {
    let layout = layout_with(None, &target());
    approx_point(layout.corners[0], (270.0, 155.0));
    approx_point(layout.corners[1], (370.0, 155.0));
    approx_point(layout.corners[2], (370.0, 205.0));
    approx_point(layout.corners[3], (270.0, 205.0));
    approx_point(layout.anchor, (320.0, 180.0));
    // 辺中点ハンドル(Top)は上辺の中央。
    let top_index = SCALE_HANDLES.iter().position(|h| *h == ScaleHandle::Top).unwrap();
    approx_point(layout.scale_handles[top_index], (320.0, 155.0));
}

/// **観測カメラ下の本命**: zoom 2(comp 中心アンカー)で bbox の角が
/// `camera_screen_from_world_z0` の写像どおりに動く(座標変換は既存関数の
/// 合成 — 独自の投影をしていないことの実測)。
#[test]
fn layout_follows_the_observation_camera() {
    let observation = ObservationCamera {
        pan: [10.0, -20.0],
        zoom: 2.0,
    };
    let layout = layout_with(Some(observation), &target());

    // 期待値は既存の正本写像で独立に計算する。
    let affine = camera_screen_from_world_z0(
        COMP,
        ResolvedCamera {
            center: observation.pan,
            zoom: observation.zoom,
            roll_degrees: 0.0,
        },
    );
    let expected = affine.transform_point2(Vec2::new(270.0, 155.0));
    approx_point(layout.corners[0], (expected.x, expected.y));
    let expected_anchor = affine.transform_point2(Vec2::new(320.0, 180.0));
    approx_point(layout.anchor, (expected_anchor.x, expected_anchor.y));
}

/// 回転ハンドルは上辺中点の外側 `gizmo_rotate_offset` px。
#[test]
fn layout_places_the_rotate_handle_outside_the_top_edge() {
    let layout = layout_with(None, &target());
    let offset = dims().gizmo_rotate_offset;
    approx_point(layout.top_center, (320.0, 155.0));
    approx_point(
        layout.rotate_handle.expect("回転ハンドルが出るはず"),
        (320.0, 155.0 - offset),
    );
}

/// 垂直反転(scale.y < 0)しても回転ハンドルは「上辺の外側」へついてくる
/// (screen では下向きに出る)。
#[test]
fn layout_rotate_handle_follows_a_flipped_layer() {
    let mut flipped = target();
    flipped.scale = [1.0, -1.0];
    let layout = layout_with(None, &flipped);
    let top = layout.top_center;
    let rotate = layout.rotate_handle.expect("回転ハンドルが出るはず");
    // 反転で上辺中点は中心より下(y=205)に写る — ハンドルはさらに外(下)。
    approx_point(top, (320.0, 205.0));
    assert!(
        rotate.y > top.y,
        "反転時は外側=下へ出るはず: rotate={rotate:?} top={top:?}"
    );
}

// -----------------------------------------------------------------
// hit-test(命中と優先順位 — 発注 ORACLE)
// -----------------------------------------------------------------

#[test]
fn hit_test_finds_each_handle_kind() {
    let layout = layout_with(None, &target());
    // 角。
    assert_eq!(
        gizmo_hit_test(&layout, Point::new(270.0, 155.0)),
        Some(GizmoHandle::Scale(ScaleHandle::TopLeft))
    );
    // 辺中点。
    assert_eq!(
        gizmo_hit_test(&layout, Point::new(370.0, 180.0)),
        Some(GizmoHandle::Scale(ScaleHandle::Right))
    );
    // 回転ハンドル。
    let rotate = layout.rotate_handle.unwrap();
    assert_eq!(gizmo_hit_test(&layout, rotate), Some(GizmoHandle::Rotate));
    // anchor ⊕(第2切片から drag 対象)。
    assert_eq!(
        gizmo_hit_test(&layout, layout.anchor),
        Some(GizmoHandle::Anchor)
    );
    // 内部 = body(anchor の hit_radius の外)。
    assert_eq!(
        gizmo_hit_test(&layout, Point::new(300.0, 180.0)),
        Some(GizmoHandle::Body)
    );
    // 全部の外 = None(下の層へ素通し)。
    assert_eq!(gizmo_hit_test(&layout, Point::new(100.0, 60.0)), None);
}

/// ハンドルは body に優先する(ハンドル位置は bbox 縁 = body でもあるが、
/// 掴めるのは小さい方)。
#[test]
fn hit_test_prefers_handles_over_the_body() {
    let layout = layout_with(None, &target());
    // 角のすぐ内側(半径内): body ではなく角。
    assert_eq!(
        gizmo_hit_test(&layout, Point::new(272.0, 157.0)),
        Some(GizmoHandle::Scale(ScaleHandle::TopLeft))
    );
}

/// レイヤーが小さく潰れてハンドル同士が重なったら、角が辺に勝つ
/// (2軸動かせる方を残す)。
#[test]
fn hit_test_prefers_corners_over_edges_when_handles_overlap() {
    let mut tiny = target();
    // 6x6 の極小層: 角と辺中点の距離は 3px < hit_radius(8)。
    tiny.size = [6.0, 6.0];
    tiny.anchor = [3.0, 3.0];
    let layout = layout_with(None, &tiny);
    let top_left_index = SCALE_HANDLES
        .iter()
        .position(|h| *h == ScaleHandle::TopLeft)
        .unwrap();
    let at_corner = layout.scale_handles[top_left_index];
    let hit = gizmo_hit_test(&layout, at_corner);
    assert_eq!(hit, Some(GizmoHandle::Scale(ScaleHandle::TopLeft)));
}

/// 回転ハンドルは重なった scale ハンドルより優先(外側の小さい物が勝つ)。
#[test]
fn hit_test_prefers_the_rotate_handle_over_scale_handles() {
    let layout = layout_with(None, &target());
    let rotate = layout.rotate_handle.unwrap();
    // 回転ハンドルと Top ハンドルの中間(両方の半径内になりうる点)でも
    // 回転ハンドル半径内なら Rotate。
    let probe = Point::new(rotate.x, rotate.y + layout.hit_radius * 0.9);
    assert_eq!(gizmo_hit_test(&layout, probe), Some(GizmoHandle::Rotate));
}

// -----------------------------------------------------------------
// move(カメラ/親変換下の値 — 発注 ORACLE)
// -----------------------------------------------------------------

#[test]
fn body_drag_moves_position_by_the_screen_delta_under_default_cameras() {
    let target = target();
    let layout = layout_with(None, &target);
    let mut drag = GizmoDragState::begin(target, &layout, GizmoHandle::Body, Point::new(300.0, 170.0))
        .expect("drag が始まるはず");
    assert!(!drag.moved());
    let value = drag.update(Point::new(310.0, 175.0), false);
    approx_value(value, GizmoValue::Position([330.0, 185.0]));
    assert!(drag.moved());
    assert_eq!(drag.last_value(), Some(value));
}

/// 観測カメラ zoom 2 の下では screen 10px = world 5px(絵の上で掴んだ場所に
/// カーソルがついてくる)。
#[test]
fn body_drag_compensates_for_observation_zoom() {
    let observation = ObservationCamera {
        pan: [0.0, 0.0],
        zoom: 2.0,
    };
    let target = target();
    let layout = layout_with(Some(observation), &target);
    let mut drag = GizmoDragState::begin(target, &layout, GizmoHandle::Body, Point::new(320.0, 180.0))
        .expect("drag が始まるはず");
    let value = drag.update(Point::new(330.0, 185.0), false);
    approx_value(value, GizmoValue::Position([325.0, 182.5]));
}

/// 親が2倍に拡大している時、world 10px の変位は親空間では 5px —
/// position は親空間の値なので 5 だけ動く。
#[test]
fn body_drag_solves_in_parent_space() {
    let mut parented = target();
    parented.world_from_parent = Affine2::from_scale(Vec2::new(2.0, 2.0));
    // 親の2倍で screen 上の絵は position*2 に写るが、drag の解は親空間で立つ。
    let layout = layout_with(None, &parented);
    let start = layout.anchor; // anchor の screen 位置から掴む
    let mut drag = GizmoDragState::begin(parented, &layout, GizmoHandle::Body, start)
        .expect("drag が始まるはず");
    let value = drag.update(Point::new(start.x + 10.0, start.y + 5.0), false);
    approx_value(value, GizmoValue::Position([325.0, 182.5]));
}

// -----------------------------------------------------------------
// scale(角=2軸・辺=1軸・Shift=比率固定 — map 680)
// -----------------------------------------------------------------

#[test]
fn corner_drag_scales_both_axes_about_the_anchor() {
    let target = target();
    let layout = layout_with(None, &target);
    let corner = Point::new(370.0, 205.0); // BottomRight
    let mut drag =
        GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::BottomRight), corner)
            .expect("drag が始まるはず");
    // 1.5倍の位置へ: anchor(320,180) + (75, 37.5)。
    let value = drag.update(Point::new(395.0, 217.5), false);
    approx_value(value, GizmoValue::Scale([1.5, 1.5]));
    // 非等方: x だけ 1.5 倍の位置。
    let value = drag.update(Point::new(395.0, 205.0), false);
    approx_value(value, GizmoValue::Scale([1.5, 1.0]));
}

/// Shift = 比率固定: 変化の大きい軸の率を両軸へ(map 680)。
#[test]
fn corner_drag_with_shift_keeps_the_aspect_ratio() {
    let target = target();
    let layout = layout_with(None, &target);
    let corner = Point::new(370.0, 205.0);
    let mut drag =
        GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::BottomRight), corner)
            .expect("drag が始まるはず");
    let value = drag.update(Point::new(395.0, 205.0), true);
    approx_value(value, GizmoValue::Scale([1.5, 1.5]));
}

/// 辺ハンドルは1軸だけ。y の分母(handle-anchor)が 0 でも x が解ける。
#[test]
fn edge_drag_scales_only_its_axis() {
    let target = target();
    let layout = layout_with(None, &target);
    let right = Point::new(370.0, 180.0);
    let mut drag =
        GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::Right), right)
            .expect("drag が始まるはず");
    // カーソルを右+下へ: y 成分は無視され x だけ 1.5。
    let value = drag.update(Point::new(395.0, 190.0), false);
    approx_value(value, GizmoValue::Scale([1.5, 1.0]));
    // Shift: 解けた1軸の率を両軸へ。
    let value = drag.update(Point::new(395.0, 190.0), true);
    approx_value(value, GizmoValue::Scale([1.5, 1.5]));
}

/// ハンドルを anchor の反対側へ引き抜くと負 scale(反転、AE と同じ)。
#[test]
fn dragging_through_the_anchor_flips_the_scale() {
    let target = target();
    let layout = layout_with(None, &target);
    let right = Point::new(370.0, 180.0);
    let mut drag =
        GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::Right), right)
            .expect("drag が始まるはず");
    let value = drag.update(Point::new(270.0, 180.0), false);
    approx_value(value, GizmoValue::Scale([-1.0, 1.0]));
}

/// 回転している層でも、掴んだハンドルがカーソルへ来る scale が解ける
/// (解は (R·K)⁻¹ 経由 — screen 位置は回転込みで一致する)。
#[test]
fn corner_drag_solves_under_layer_rotation() {
    let mut rotated = target();
    rotated.rotation_degrees = 90.0;
    let layout = layout_with(None, &rotated);
    let index = SCALE_HANDLES
        .iter()
        .position(|h| *h == ScaleHandle::BottomRight)
        .unwrap();
    let corner = layout.scale_handles[index];
    let mut drag =
        GizmoDragState::begin(rotated, &layout, GizmoHandle::Scale(ScaleHandle::BottomRight), corner)
            .expect("drag が始まるはず");
    // 90°回転で local (50,25)(anchor→BR)は親空間 (-25,50) に写る。2倍の位置へ。
    let value = drag.update(Point::new(320.0 - 50.0, 180.0 + 100.0), false);
    approx_value(value, GizmoValue::Scale([2.0, 2.0]));
}

// -----------------------------------------------------------------
// rotate(anchor 中心・Shift=15°スナップ — map 679)
// -----------------------------------------------------------------

/// 上(12時)から右(3時)へ回すと +90°(y-down で時計回り = store の符号)。
#[test]
fn rotate_drag_measures_the_clockwise_angle_about_the_anchor() {
    let target = target();
    let layout = layout_with(None, &target);
    let rotate = layout.rotate_handle.unwrap();
    let mut drag = GizmoDragState::begin(target, &layout, GizmoHandle::Rotate, rotate)
        .expect("drag が始まるはず");
    let value = drag.update(Point::new(320.0 + 41.0, 180.0), false);
    let GizmoValue::Rotation(degrees) = value else {
        panic!("Rotation が出るはず: {value:?}");
    };
    assert!((degrees - 90.0).abs() < 1e-3, "degrees={degrees}");
}

/// Shift = 最寄りの 15° 刻みへスナップ(map 679)。40° 相当 → 45°。
#[test]
fn rotate_drag_with_shift_snaps_to_15_degree_steps() {
    let target = target();
    let pivot = Vec2::new(320.0, 180.0);
    let v0 = Vec2::new(0.0, -41.0); // 真上
    let angle = (-90.0f32 + 40.0).to_radians();
    let v1 = Vec2::new(angle.cos(), angle.sin()) * 41.0;
    let degrees = rotation_value(&target, pivot + v0, pivot + v1, true);
    assert!((degrees - 45.0).abs() < 1e-3, "degrees={degrees}");
}

/// 観測カメラ(ズーム/パン)の下でも回転の値は変わらない — 角度は親空間で
/// 測る(発注 ORACLE「カメラ変換下の座標」)。
#[test]
fn rotate_drag_is_independent_of_the_observation_camera() {
    let observation = ObservationCamera {
        pan: [37.0, -12.0],
        zoom: 1.7,
    };
    let target = target();
    let plain = layout_with(None, &target);
    let observed = layout_with(Some(observation), &target);

    // 同じ親空間の2点を、それぞれの screen 系へ写してから drag に食わせる。
    let parent_start = Vec2::new(320.0, 139.0);
    let parent_end = Vec2::new(361.0, 180.0);
    let run = |layout: &GizmoLayout| -> f64 {
        let to_screen = |p: Vec2| {
            let s = layout.screen_from_parent.transform_point2(p);
            Point::new(s.x, s.y)
        };
        let mut drag =
            GizmoDragState::begin(target, layout, GizmoHandle::Rotate, to_screen(parent_start))
                .expect("drag が始まるはず");
        let GizmoValue::Rotation(degrees) = drag.update(to_screen(parent_end), false) else {
            panic!("Rotation が出るはず");
        };
        degrees
    };
    let a = run(&plain);
    let b = run(&observed);
    assert!((a - b).abs() < 1e-3, "カメラで回転値が変わっている: {a} vs {b}");
}

// -----------------------------------------------------------------
// drag 状態機械(Start/Move/Commit/Cancel の材料)
// -----------------------------------------------------------------

/// 空クリック(動かさず release)は Commit の材料を持たない — canvas 側は
/// Cancel を出す(契約: Start は必ず Commit/Cancel で閉じる)。
#[test]
fn a_click_without_movement_has_no_value_to_commit() {
    let target = target();
    let layout = layout_with(None, &target);
    let drag = GizmoDragState::begin(target, &layout, GizmoHandle::Body, Point::new(300.0, 180.0))
        .expect("drag が始まるはず");
    assert!(!drag.moved());
    assert_eq!(drag.last_value(), None);
}

/// Shift の途中切り替え(refresh)は動いた後だけ効く。
#[test]
fn refresh_reapplies_modifiers_only_after_movement() {
    let target = target();
    let layout = layout_with(None, &target);
    let corner = Point::new(370.0, 205.0);
    let mut drag =
        GizmoDragState::begin(target, &layout, GizmoHandle::Scale(ScaleHandle::BottomRight), corner)
            .expect("drag が始まるはず");
    assert_eq!(drag.refresh(true), None, "動く前の refresh は no-op のはず");
    let _ = drag.update(Point::new(395.0, 205.0), false);
    let refreshed = drag.refresh(true).expect("Shift on で比率固定へ再計算されるはず");
    approx_value(refreshed, GizmoValue::Scale([1.5, 1.5]));
}

/// handle → property の対応(shell 結線の宛先)。
#[test]
fn handles_map_to_their_properties() {
    assert_eq!(GizmoHandle::Body.property(), GizmoProperty::Position);
    assert_eq!(
        GizmoHandle::Scale(ScaleHandle::Top).property(),
        GizmoProperty::Scale
    );
    assert_eq!(GizmoHandle::Rotate.property(), GizmoProperty::Rotation);
    assert_eq!(GizmoHandle::Anchor.property(), GizmoProperty::Anchor);
    assert_eq!(GizmoProperty::Position.property_name(), property::POSITION);
    assert_eq!(GizmoProperty::Scale.property_name(), property::SCALE);
    assert_eq!(GizmoProperty::Rotation.property_name(), property::ROTATION);
    assert_eq!(GizmoProperty::Anchor.property_name(), property::ANCHOR);
    assert_eq!(
        GizmoValue::Anchor {
            anchor: [0.0, 0.0],
            position: [0.0, 0.0]
        }
        .property(),
        GizmoProperty::Anchor
    );
}

// -----------------------------------------------------------------
// anchor drag(第2切片 — AE pan-behind 型。未実行、波末一括)
// -----------------------------------------------------------------

/// **本命**: ⊕ が cursor の真下へ来て、position が補償される
/// (恒等変形なら anchor の移動量=position の移動量)。
#[test]
fn anchor_drag_lands_the_anchor_under_the_cursor_and_compensates_position() {
    let target = target();
    let (anchor, position) = anchor_value(&target, Vec2::new(330.0, 190.0));
    // M0⁻¹(330,190) = (330,190) - (270,155) = (60,35)。
    assert!((anchor[0] - 60.0).abs() < 1e-3 && (anchor[1] - 35.0).abs() < 1e-3);
    // RKS = 恒等なので補償は anchor の差分そのまま: (320,180)+(10,10)。
    assert!((position[0] - 330.0).abs() < 1e-3 && (position[1] - 190.0).abs() < 1e-3);
}

/// **不変量**: 回転+非等方 scale の下でも、anchor drag の前後で任意の
/// 局所点の親空間の像が動かない(見た目不動 — AE pan-behind の本体)。
#[test]
fn anchor_drag_keeps_the_image_stationary_under_rotation_and_scale() {
    let mut twisted = target();
    twisted.rotation_degrees = 30.0;
    twisted.scale = [2.0, 0.5];
    let cursor_parent = Vec2::new(300.0, 200.0);
    let (anchor, position) = anchor_value(&twisted, cursor_parent);

    let before = LayerPlacement::from_transform(
        twisted.anchor,
        twisted.position,
        twisted.scale,
        twisted.rotation_degrees,
        twisted.skew_degrees,
        twisted.skew_axis_degrees,
    );
    let after = LayerPlacement::from_transform(
        [anchor[0] as f32, anchor[1] as f32],
        [position[0] as f32, position[1] as f32],
        twisted.scale,
        twisted.rotation_degrees,
        twisted.skew_degrees,
        twisted.skew_axis_degrees,
    );
    for probe in [Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0), Vec2::new(37.0, 11.0)] {
        let b = before.transform_point2(probe);
        let a = after.transform_point2(probe);
        assert!(
            (a.x - b.x).abs() < 1e-2 && (a.y - b.y).abs() < 1e-2,
            "絵が動いた: probe={probe:?} before={b:?} after={a:?}"
        );
    }
    // ⊕ 自身は cursor の真下(M1(a1) = cursor_parent)。
    let landed = after.transform_point2(Vec2::new(anchor[0] as f32, anchor[1] as f32));
    assert!(
        (landed.x - cursor_parent.x).abs() < 1e-2
            && (landed.y - cursor_parent.y).abs() < 1e-2,
        "⊕ が cursor に吸い付いていない: {landed:?}"
    );
}

/// 観測カメラ zoom 2 の下でも drag 経路(screen → 親空間)ごと正しく解ける
/// (screen 10px = 親空間 5px)。
#[test]
fn anchor_drag_solves_under_the_observation_camera() {
    let observation = ObservationCamera {
        pan: [0.0, 0.0],
        zoom: 2.0,
    };
    let target = target();
    let layout = layout_with(Some(observation), &target);
    let start = layout.anchor;
    let mut drag = GizmoDragState::begin(target, &layout, GizmoHandle::Anchor, start)
        .expect("drag が始まるはず");
    let value = drag.update(Point::new(start.x + 10.0, start.y + 5.0), false);
    approx_value(
        value,
        GizmoValue::Anchor {
            anchor: [55.0, 27.5],
            position: [325.0, 182.5],
        },
    );
}

/// scale 0 で写像が潰れている層は解かない — 開始時の値を返すだけ
/// (`begin` の「掴めない物は掴めないまま」と同じ判断)。
#[test]
fn anchor_drag_with_a_degenerate_scale_keeps_the_start_values() {
    let mut flat = target();
    flat.scale = [0.0, 1.0];
    let (anchor, position) = anchor_value(&flat, Vec2::new(400.0, 300.0));
    assert_eq!(anchor, [50.0, 25.0]);
    assert_eq!(position, [320.0, 180.0]);
}

// -----------------------------------------------------------------
// 退化行列の再現試験(2026-08-22 発注: anchor_value で実測された
// panic クラスが gizmo.rs 内に残していた同じ経路を閉じる)。
// -----------------------------------------------------------------

/// **再現試験**: 層自身の scale を 0 にすると `screen_from_local` が退化する。
/// 修正前は body 判定が生 `.inverse()` を呼んでいて、退化行列(det=0)へ
/// 呼んだ瞬間 glam の `glam_assert!` で panic していた(`is_finite` チェックへ
/// 辿り着けない)。`checked_inverse` 経由になった今は、どのハンドルにも
/// 当たらない遠方の点で呼んでも panic せず `None` を返す。
#[test]
fn hit_test_does_not_panic_when_the_targets_own_scale_is_degenerate() {
    let mut flat = target();
    flat.scale = [0.0, 1.0];
    let layout = layout_with(None, &flat);
    // bbox は scale.x=0 で x=anchor.x(320) の線に潰れる。原点はどの
    // ハンドル/anchor からも十分離れている(hit_radius 内に入らない)。
    assert_eq!(gizmo_hit_test(&layout, Point::new(0.0, 0.0)), None);
}

/// **再現試験**: 親鎖(`world_from_parent`)が退化(scale 0)していると
/// `screen_from_parent` も退化する。修正前は `begin` が生 `.inverse()` を
/// 呼んでいて同じ経路で panic していた。`checked_inverse` 経由になった今は
/// 「掴めない物は掴めないまま」どおり `None` を返し、drag を始めない。
#[test]
fn begin_returns_none_when_the_parent_chain_is_degenerate() {
    let mut degenerate_parent = target();
    degenerate_parent.world_from_parent = Affine2::from_scale(Vec2::new(0.0, 1.0));
    let layout = layout_with(None, &degenerate_parent);
    let drag = GizmoDragState::begin(
        degenerate_parent,
        &layout,
        GizmoHandle::Body,
        Point::new(320.0, 180.0),
    );
    assert!(drag.is_none(), "退化した親鎖では drag を始めないはず");
}

/// 優先順位: 極小レイヤーで anchor が scale ハンドルに飲まれても角が勝つ
/// (2軸動かせる方を残す — 既存の角>辺と同じ原則)。逆に body とは
/// anchor が勝つ(小さい物が大きい物に勝つ)。
#[test]
fn hit_test_ranks_the_anchor_between_scale_handles_and_the_body() {
    // 通常寸: anchor(中央)は body の海の中 — anchor が勝つ。
    let layout = layout_with(None, &target());
    assert_eq!(
        gizmo_hit_test(&layout, Point::new(layout.anchor.x + 3.0, layout.anchor.y)),
        Some(GizmoHandle::Anchor)
    );

    // 極小 6x6: 角ハンドルと anchor が半径内で重なる — 角が勝つ。
    let mut tiny = target();
    tiny.size = [6.0, 6.0];
    tiny.anchor = [3.0, 3.0];
    let layout = layout_with(None, &tiny);
    let top_left_index = SCALE_HANDLES
        .iter()
        .position(|h| *h == ScaleHandle::TopLeft)
        .unwrap();
    assert_eq!(
        gizmo_hit_test(&layout, layout.scale_handles[top_left_index]),
        Some(GizmoHandle::Scale(ScaleHandle::TopLeft))
    );
}

// -----------------------------------------------------------------
// カーソル形状(Q0 の合図)
// -----------------------------------------------------------------

#[test]
fn resize_interaction_picks_the_axis_by_direction() {
    use mouse::Interaction;
    assert_eq!(resize_interaction(Vec2::new(1.0, 0.0)), Interaction::ResizingHorizontally);
    assert_eq!(resize_interaction(Vec2::new(-1.0, 0.0)), Interaction::ResizingHorizontally);
    assert_eq!(resize_interaction(Vec2::new(0.0, 1.0)), Interaction::ResizingVertically);
    // ↘(右下)= NW–SE 軸。
    assert_eq!(
        resize_interaction(Vec2::new(1.0, 1.0)),
        Interaction::ResizingDiagonallyDown
    );
    // ↗(右上)= NE–SW 軸。
    assert_eq!(
        resize_interaction(Vec2::new(1.0, -1.0)),
        Interaction::ResizingDiagonallyUp
    );
}

// -----------------------------------------------------------------
// gizmo_target(StoreView からの読み口)
// -----------------------------------------------------------------

fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

fn doc_with_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: motolii_core::Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: Composition::default_background(),
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [255, 0, 0, 255],
                    width: 100,
                    height: 50,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::POSITION).unwrap(),
        track: still(Value::Vec2([320.0, 180.0])),
    })
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::ANCHOR).unwrap(),
        track: still(Value::Vec2([50.0, 25.0])),
    })
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::ROTATION).unwrap(),
        track: still(Value::F64(30.0)),
    })
    .unwrap();
    (doc, layer)
}

/// 実 Document から局所値が読める(Inspector と同じ値の経路)。
#[test]
fn gizmo_target_reads_the_layers_transform_values() {
    let (doc, layer) = doc_with_layer();
    let view = doc.view();
    let target = gizmo_target(&view, layer, 0, [100.0, 50.0]).expect("target が組めるはず");
    assert_eq!(target.position, [320.0, 180.0]);
    assert_eq!(target.anchor, [50.0, 25.0]);
    assert_eq!(target.rotation_degrees, 30.0);
    assert_eq!(target.scale, [1.0, 1.0], "track が無い property は既定値");
    assert_eq!(target.world_from_parent, Affine2::IDENTITY, "親なし = 恒等");
}

/// 親を持つ層は親の local 変換が world_from_parent に入る(裁定173 H1 —
/// 親が動くと子のギズモも動く)。
#[test]
fn gizmo_target_composes_the_parent_chain() {
    let (mut doc, layer) = doc_with_layer();
    let parent = LayerId(2);
    doc.apply_all([
        Intent::AddLayer(parent),
        Intent::SetMeta {
            layer: parent,
            meta: LayerMeta {
                source: LayerSource::Null,
                order: 1,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer: parent,
        property: PropertyId::new(property::POSITION).unwrap(),
        track: still(Value::Vec2([40.0, 10.0])),
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            parent: Some(Some(parent)),
            ..Default::default()
        },
    })
    .unwrap();

    let view = doc.view();
    let target = gizmo_target(&view, layer, 0, [100.0, 50.0]).expect("target が組めるはず");
    let moved = target
        .world_from_parent
        .transform_point2(Vec2::new(0.0, 0.0));
    assert!(
        (moved.x - 40.0).abs() < 1e-4 && (moved.y - 10.0).abs() < 1e-4,
        "親の平行移動が合成されていない: {moved:?}"
    );
}

/// hidden な層はギズモを出さない(Q0: 触れない物を描かない)。
#[test]
fn gizmo_target_is_none_for_a_hidden_layer() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            ..Default::default()
        },
    })
    .unwrap();
    let view = doc.view();
    assert!(gizmo_target(&view, layer, 0, [100.0, 50.0]).is_none());
}
