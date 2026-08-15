use super::*;

#[test]
fn stage_navigation_maps_wheel_and_pinch_to_egui_events() {
    let [
        Event::PointerMoved(position),
        Event::MouseWheel {
            unit,
            delta,
            modifiers,
            ..
        },
    ] = stage_navigation_events(3.0, -4.0, 0.0, stage_modifiers(5), 20.0, 30.0)
        .expect("wheel event")
    else {
        panic!("wheel must include pointer position and MouseWheel");
    };
    assert_eq!(position, Pos2::new(20.0, 30.0));
    assert_eq!(unit, MouseWheelUnit::Point);
    assert_eq!(delta, Vec2::new(3.0, -4.0));
    assert!(modifiers.shift && modifiers.alt);

    let [Event::PointerMoved(_), Event::Zoom(zoom)] =
        stage_navigation_events(0.0, 0.0, 0.2, Modifiers::NONE, 20.0, 30.0).expect("pinch event")
    else {
        panic!("pinch must include pointer position and Zoom");
    };
    assert!((zoom - 0.2_f32.exp()).abs() < f32::EPSILON);
    assert!(stage_navigation_events(f64::NAN, 0.0, 0.0, Modifiers::NONE, 0.0, 0.0).is_none());
    let modifiers = stage_modifiers(1 | 2 | 4 | 8);
    assert!(modifiers.shift && modifiers.ctrl && modifiers.alt);
    assert!(modifiers.mac_cmd && modifiers.command);
    assert_eq!(
        egui_pointer_button(StagePointerButton::Primary),
        PointerButton::Primary
    );
    assert_eq!(
        egui_pointer_button(StagePointerButton::Secondary),
        PointerButton::Secondary
    );
    assert_eq!(
        egui_pointer_button(StagePointerButton::Middle),
        PointerButton::Middle
    );
}

#[test]
fn pucker_preview_is_tessellated_as_curved_fill_and_stroke() {
    let path = pathgeom::apply(
        &rectangle_path(0.0, 0.0),
        &preview_path_operation("pucker-bloat").expect("fixture operation"),
        0.0,
    )
    .expect("fixture operation evaluates");
    let (fill, stroke) =
        tessellate_path(&path, Transform::default()).expect("Bezier path tessellates");

    assert!(
        fill.vertices.len() > 4,
        "curve uses adaptive vertices, not a rectangle fan"
    );
    assert!(!fill.indices.is_empty());
    assert!(
        stroke.vertices.len() > 8,
        "stroke follows the same curved path"
    );
}

#[test]
fn every_visible_path_operation_has_a_preview_evaluation() {
    for id in [
        "pucker-bloat",
        "zig-zag",
        "offset",
        "round-corners",
        "trim",
        "twist",
        "wiggle",
        "repeater",
    ] {
        let path = pathgeom::apply(
            &rectangle_path(0.0, 0.0),
            &preview_path_operation(id).expect("known path operation"),
            0.0,
        )
        .expect("fixture operation evaluates");
        let (_, stroke) =
            tessellate_path(&path, Transform::default()).expect("evaluated path tessellates");
        assert!(
            !stroke.indices.is_empty(),
            "{id} has a visible Stage outline"
        );
    }
}

#[test]
fn performance_gizmo_transform_moves_and_rotates_fixture_vertices() {
    let mesh = MeshData {
        vertices: vec![[1.0, 0.0, 0.0]],
        indices: vec![],
    }
    .transformed(Transform::from_scale_rotation_translation(
        DVec3::ONE,
        DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2),
        DVec3::new(0.5, -0.25, 0.75),
    ));

    let [x, y, z] = mesh.vertices[0];
    assert!((x - 0.5).abs() < 0.000_1);
    assert!((y - 0.75).abs() < 0.000_1);
    assert!((z - 0.75).abs() < 0.000_1);
}

#[test]
fn canonical_corners_map_to_fixture_mesh_space() {
    // seed unit rect: center(0,0) size(1,1) / 正方 viewport では旧写像と一致
    let corners = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
    let verts = mesh_vertices_from_canonical_corners(corners, 1, 1);
    // (nx,ny)=(cx+0.5, 0.5-cy) → (n-0.5) = (cx, -cy)
    assert_eq!(verts[0], [-0.5, 0.5, 0.0]);
    assert_eq!(verts[1], [0.5, 0.5, 0.0]);
    assert_eq!(verts[2], [0.5, -0.5, 0.0]);
    assert_eq!(verts[3], [-0.5, -0.5, 0.0]);
}

#[test]
fn canonical_to_normalized_uses_viewport_aspect() {
    // w=2h → h/w=0.5。cx=0.5 → nx=0.5+0.25=0.75
    let corners = [[0.5, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]];
    let verts = mesh_vertices_from_canonical_corners(corners, 2, 1);
    let nx = verts[0][0] as f64 + 0.5;
    assert!((nx - 0.75).abs() < 1e-9);
}

#[test]
fn aspect_fit_ndc_rect_for_16x9_viewports() {
    // 同一比率 → 全面
    assert_eq!(
        aspect_fit_ndc_rect(1920.0, 1080.0, 1920.0, 1080.0),
        [-1.0, -1.0, 2.0, 2.0]
    );
    // 正方: 横フル、縦 1080/1920 * 2
    assert_eq!(
        aspect_fit_ndc_rect(1920.0, 1080.0, 1920.0, 1920.0),
        [-1.0, -0.5625, 2.0, 1.125]
    );
    // ウルトラワイド: 縦フル、横 1920/3840 * 2
    assert_eq!(
        aspect_fit_ndc_rect(1920.0, 1080.0, 3840.0, 1080.0),
        [-0.5, -1.0, 1.0, 2.0]
    );
    // 縦長 9:16 viewport: scale=1080/1920
    let tall = aspect_fit_ndc_rect(1920.0, 1080.0, 1080.0, 1920.0);
    assert_eq!(tall[0], -1.0);
    assert_eq!(tall[2], 2.0);
    assert!((tall[3] - 0.632_812_5).abs() < 1e-6);
    assert!((tall[1] + 0.316_406_25).abs() < 1e-6);
}

#[test]
fn host_path_stroke_vertices_follow_projected_corners() {
    let before =
        path_stroke_from_canonical_corners([[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]])
            .expect("baseline path");
    let after =
        path_stroke_from_canonical_corners([[-0.4, -0.5], [0.6, -0.5], [0.6, 0.5], [-0.4, 0.5]])
            .expect("translated path");
    assert_ne!(
        before.vertices, after.vertices,
        "Stage path mesh must move when Document corners move"
    );
}

#[test]
fn host_path_fill_and_stroke_tessellate_from_corners() {
    let (fill, stroke) =
        path_meshes_from_canonical_corners([[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]])
            .expect("rect path tessellates");
    assert!(
        !fill.indices.is_empty(),
        "Stage layer fill must reach Mesh3D"
    );
    assert!(
        !stroke.indices.is_empty(),
        "Stage layer stroke must reach Mesh3D"
    );
}

#[test]
fn rerun_layer_entity_paths_remap_to_document_layer_identity() {
    assert_eq!(
        host_layer_id_from_entity_path("motolii/document/layers/42/fill"),
        Some("42")
    );
    assert_eq!(
        host_layer_id_from_entity_path("motolii/document/layers/42/path"),
        Some("42")
    );
    assert_eq!(
        host_layer_id_from_entity_path("/motolii/document/layers/42/path"),
        Some("42")
    );
    assert_eq!(
        host_layer_id_from_entity_path("motolii/document/frame"),
        None
    );
    assert_eq!(
        host_layer_id_from_entity_path("motolii/document/layers/42/other"),
        None
    );
}

#[test]
fn evaluated_frame_hides_opaque_fill_so_image_is_visible() {
    let corners = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
    let (fill, _) = path_meshes_from_canonical_corners(corners).expect("fill");
    let hidden = hidden_layer_mesh();
    assert!(
        host_layer_fill_is_visible(corners, false, false),
        "place must keep fill until evaluated Image"
    );
    assert!(
        !host_layer_fill_is_visible(corners, true, false),
        "evaluated frame must hide the opaque fill in front of the Image"
    );
    assert!(
        host_layer_fill_is_visible(corners, true, true),
        "gizmo preview must keep fill while the evaluated Image is stale"
    );
    assert_ne!(
        fill.vertices, hidden.vertices,
        "evaluated frame must not keep the opaque fill mesh in front of the Image"
    );
    assert_eq!(DOCUMENT_FRAME_ENTITY, "motolii/document/frame");
}

#[test]
fn stage_transform_edit_maps_translate_rotate_z_and_scale() {
    assert_eq!(
        stage_transform_edit(
            GizmoResult::Translation {
                delta: DVec3::ZERO.into(),
                total: DVec3::new(0.1, -0.2, 0.9).into(),
            },
            DQuat::IDENTITY,
        ),
        Some(AppStageTransformEdit::TranslateWorld([0.1, -0.2]))
    );
    assert_eq!(
        stage_transform_edit(
            GizmoResult::Rotation {
                axis: DVec3::Z.into(),
                delta: 0.0,
                total: 0.25,
                is_view_axis: false,
            },
            DQuat::IDENTITY,
        ),
        Some(AppStageTransformEdit::RotateZ(-0.25))
    );
    assert_eq!(
        stage_transform_edit(
            GizmoResult::Scale {
                total: DVec3::new(1.5, 0.5, 3.0).into(),
            },
            DQuat::IDENTITY,
        ),
        Some(AppStageTransformEdit::Scale([1.5, 0.5]))
    );
}

#[test]
fn stage_transform_edit_maps_local_translation_to_world() {
    let rotated = DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2);
    let Some(AppStageTransformEdit::TranslateWorld(delta)) = stage_transform_edit(
        GizmoResult::Translation {
            delta: DVec3::ZERO.into(),
            total: DVec3::new(0.1, 0.0, 0.0).into(),
        },
        rotated,
    ) else {
        panic!("local X translation must map to world XY");
    };
    assert!(delta[0].abs() < 1e-12);
    assert!((delta[1] - 0.1).abs() < 1e-12);
}

#[test]
fn stage_transform_edit_rejects_unsupported_rotation() {
    assert_eq!(
        stage_transform_edit(
            GizmoResult::Rotation {
                axis: DVec3::X.into(),
                delta: 0.1,
                total: 0.4,
                is_view_axis: false,
            },
            DQuat::IDENTITY,
        ),
        None
    );
    assert_eq!(
        stage_transform_edit(
            GizmoResult::Rotation {
                axis: DVec3::Y.into(),
                delta: 0.1,
                total: 0.4,
                is_view_axis: false,
            },
            DQuat::IDENTITY,
        ),
        None
    );
    assert_eq!(
        stage_transform_edit(
            GizmoResult::Arcball {
                delta: DQuat::IDENTITY.into(),
                total: DQuat::IDENTITY.into(),
            },
            DQuat::IDENTITY,
        ),
        None
    );
}

#[test]
fn stage_transform_edit_filters_noop() {
    let translate = stage_transform_edit(
        GizmoResult::Translation {
            delta: DVec3::ZERO.into(),
            total: DVec3::ZERO.into(),
        },
        DQuat::IDENTITY,
    )
    .expect("zero translation still maps");
    assert!(stage_transform_edit_is_noop(translate));

    let rotate = stage_transform_edit(
        GizmoResult::Rotation {
            axis: DVec3::Z.into(),
            delta: 0.0,
            total: 0.0,
            is_view_axis: false,
        },
        DQuat::IDENTITY,
    )
    .expect("zero rotation still maps");
    assert!(stage_transform_edit_is_noop(rotate));

    let scale = stage_transform_edit(
        GizmoResult::Scale {
            total: DVec3::ONE.into(),
        },
        DQuat::IDENTITY,
    )
    .expect("identity scale still maps");
    assert!(stage_transform_edit_is_noop(scale));
}
