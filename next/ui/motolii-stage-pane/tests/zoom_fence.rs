//! `src/zoom.rs`(発注 2026-08-22、B24 ズーム束+B31 選択の残り)の ORACLE。
//! **未実行**(裁定189: この波の検収線は `cargo check --tests` まで、
//! `cargo test` は次波)— 数値は手計算で導出済みだが、実行して確認したもの
//! ではない旨をここに明記する。
//!
//! `src/` 直下の `#[cfg(test)]` ではなく独立 integration test にしてあるのは
//! 発注書 EXACT TARGET の「新規ファイルは `src/zoom.rs` + `tests/zoom_fence.rs`
//! の2本」指定に従うため(`render_frame_fence.rs` と同じ「新ファイルは
//! `tests/` 側」の置き場所)。

use glam::Affine2;
use iced::{Point, Rectangle};

use motolii_core::CompSpec;
use motolii_engine::ObservationCamera;
use motolii_stage_pane::zoom::{
    actual_size_zoom, deselect_all_layers, fit_to_gizmo_target, fit_to_world_bbox,
    invert_layer_selection, named_zoom_level, select_all_layers, step_layer_selection, zoom_step,
    LayerStepDirection, NamedZoomLevel, ZoomStepDirection, ZOOM_LADDER,
};
use motolii_stage_pane::GizmoTarget;
use motolii_store::{Composition, Document, Intent, LayerId, LayerMeta, LayerSource, LayerTiming};

const COMP: CompSpec = CompSpec {
    width: 640,
    height: 360,
};

fn bounds_matching_comp_aspect() -> Rectangle {
    // 640x360 と同じ 16:9 — letterbox 無し、rect が bounds いっぱい。
    Rectangle::new(Point::ORIGIN, iced::Size::new(640.0, 360.0))
}

fn bounds_double_comp() -> Rectangle {
    // 同アスペクトで2倍のウィンドウ — actual_size_zoom は 0.5 になるはず
    // (letterbox が既に comp を2倍へ引き伸ばしている分を打ち消す)。
    Rectangle::new(Point::ORIGIN, iced::Size::new(1280.0, 720.0))
}

// ---------------------------------------------------------------------------
// ZOOM_LADDER / zoom_step
// ---------------------------------------------------------------------------

#[test]
fn zoom_ladder_is_sorted_ascending_and_centers_on_one() {
    let mut sorted = ZOOM_LADDER.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(sorted, ZOOM_LADDER, "ラダーは昇順のはず");
    assert!(ZOOM_LADDER.contains(&1.0), "1.0(無変換点)を含むはず");
}

#[test]
fn zoom_step_in_jumps_to_the_next_rung_above_current() {
    let current = ObservationCamera {
        pan: [10.0, 20.0],
        zoom: 1.0,
    };
    let next = zoom_step(current, ZoomStepDirection::In);
    assert_eq!(next.zoom, 2.0);
    assert_eq!(next.pan, [10.0, 20.0], "段ステップは pan を変えない(ビュー中心アンカー)");
}

#[test]
fn zoom_step_out_jumps_to_the_next_rung_below_current() {
    let current = ObservationCamera {
        pan: [0.0, 0.0],
        zoom: 1.0,
    };
    let next = zoom_step(current, ZoomStepDirection::Out);
    assert_eq!(next.zoom, 0.5);
}

#[test]
fn zoom_step_snaps_an_off_ladder_value_toward_the_requested_direction() {
    let current = ObservationCamera {
        pan: [0.0, 0.0],
        zoom: 3.0,
    };
    assert_eq!(zoom_step(current, ZoomStepDirection::In).zoom, 4.0);
    assert_eq!(zoom_step(current, ZoomStepDirection::Out).zoom, 2.0);
}

#[test]
fn zoom_step_clamps_at_the_ladder_ends() {
    let top = ObservationCamera {
        pan: [0.0, 0.0],
        zoom: 16.0,
    };
    assert_eq!(zoom_step(top, ZoomStepDirection::In).zoom, 16.0, "最上段はそのまま");

    let bottom = ObservationCamera {
        pan: [0.0, 0.0],
        zoom: 0.125,
    };
    assert_eq!(zoom_step(bottom, ZoomStepDirection::Out).zoom, 0.125, "最下段はそのまま");
}

// ---------------------------------------------------------------------------
// actual_size_zoom / named_zoom_level
// ---------------------------------------------------------------------------

#[test]
fn actual_size_zoom_is_one_when_bounds_matches_comp_pixel_for_pixel() {
    let zoom = actual_size_zoom(bounds_matching_comp_aspect(), COMP).expect("letterbox が組める");
    assert!((zoom - 1.0).abs() < 1e-4);
}

#[test]
fn actual_size_zoom_is_half_when_the_window_is_double_the_comp_size() {
    // letterbox が 640→1280 に2倍へ引き伸ばしている分を、観測ズーム0.5で
    // 打ち消して初めて「1 comp px = 1 screen px」に戻る。
    let zoom = actual_size_zoom(bounds_double_comp(), COMP).expect("letterbox が組める");
    assert!((zoom - 0.5).abs() < 1e-4);
}

#[test]
fn named_zoom_level_fit_is_always_the_default_observation_camera() {
    let fit = named_zoom_level(bounds_double_comp(), COMP, NamedZoomLevel::Fit)
        .expect("Fit は退化 bounds/comp でも None を返さない");
    assert_eq!(fit, ObservationCamera::default());
}

#[test]
fn named_zoom_level_actual_size_half_double_scale_off_the_same_baseline() {
    let bounds = bounds_double_comp();
    let actual = named_zoom_level(bounds, COMP, NamedZoomLevel::ActualSize).unwrap();
    let half = named_zoom_level(bounds, COMP, NamedZoomLevel::Half).unwrap();
    let double = named_zoom_level(bounds, COMP, NamedZoomLevel::Double).unwrap();

    assert!((actual.zoom - 0.5).abs() < 1e-4);
    assert!((half.zoom - 0.25).abs() < 1e-4);
    assert!((double.zoom - 1.0).abs() < 1e-4);
    assert_eq!(actual.pan, [0.0, 0.0]);
    assert_eq!(half.pan, [0.0, 0.0]);
    assert_eq!(double.pan, [0.0, 0.0]);
}

// ---------------------------------------------------------------------------
// fit_to_world_bbox / fit_to_gizmo_target
// ---------------------------------------------------------------------------

#[test]
fn fit_to_world_bbox_centers_pan_on_the_bbox_center() {
    // bbox 中心 = comp 中心(320,180)なので pan は [0,0] のはず。
    let camera = fit_to_world_bbox(COMP, [270.0, 155.0], [370.0, 205.0]).expect("comp は非退化");
    assert_eq!(camera.pan, [0.0, 0.0]);
    // zoom_x = 640/100 = 6.4, zoom_y = 360/50 = 7.2 → min=6.4 * margin(0.9) = 5.76。
    assert!((camera.zoom - 5.76).abs() < 1e-3, "zoom={}", camera.zoom);
}

#[test]
fn fit_to_world_bbox_offsets_pan_when_the_bbox_is_not_centered() {
    // bbox 中心 = (100,100)、comp 中心=(320,180) → pan = (100-320, 100-180) = (-220,-80)。
    let camera = fit_to_world_bbox(COMP, [50.0, 50.0], [150.0, 150.0]).unwrap();
    assert_eq!(camera.pan, [-220.0, -80.0]);
}

#[test]
fn fit_to_world_bbox_clamps_a_degenerate_point_bbox_to_the_zoom_ceiling() {
    let camera = fit_to_world_bbox(COMP, [320.0, 180.0], [320.0, 180.0]).unwrap();
    assert_eq!(camera.zoom, 20.0, "点 bbox は無限大 zoom → 上限クランプ");
}

#[test]
fn fit_to_world_bbox_is_none_for_a_degenerate_comp() {
    let degenerate = CompSpec {
        width: 0,
        height: 360,
    };
    assert!(fit_to_world_bbox(degenerate, [0.0, 0.0], [10.0, 10.0]).is_none());
}

/// [`GizmoTarget`] 経由の bbox 抽出 — anchor=content中央・rotation=0 の単純ケース
/// (`gizmo.rs::tests::target()` と同じ形の fixture、100x50 の層を comp 中央へ)。
#[test]
fn fit_to_gizmo_target_matches_the_manual_bbox_of_an_unrotated_layer() {
    let target = GizmoTarget {
        layer: LayerId(1),
        size: [100.0, 50.0],
        anchor: [50.0, 25.0],
        position: [320.0, 180.0],
        scale: [1.0, 1.0],
        rotation_degrees: 0.0,
        skew_degrees: 0.0,
        skew_axis_degrees: 0.0,
        world_from_parent: Affine2::IDENTITY,
    };
    let camera = fit_to_gizmo_target(COMP, &target).expect("comp は非退化");
    let manual = fit_to_world_bbox(COMP, [270.0, 155.0], [370.0, 205.0]).unwrap();
    assert_eq!(camera, manual, "bbox 抽出が gizmo.rs 側の既知 bbox(270,155)-(370,205)と一致するはず");
}

// ---------------------------------------------------------------------------
// 選択の残り(B31)
// ---------------------------------------------------------------------------

fn doc_with_three_layers() -> (Document, [LayerId; 3]) {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: motolii_core::Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: Composition::default_background(),
    }))
    .unwrap();
    let layers = [LayerId(1), LayerId(2), LayerId(3)];
    for (index, &layer) in layers.iter().enumerate() {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Solid {
                        rgba: [255, 0, 0, 255],
                        width: 10,
                        height: 10,
                    },
                    order: index as i16,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .unwrap();
    }
    (doc, layers)
}

#[test]
fn select_all_layers_returns_every_present_layer_in_stacking_order() {
    let (doc, layers) = doc_with_three_layers();
    assert_eq!(select_all_layers(&doc.view()), layers.to_vec());
}

#[test]
fn deselect_all_layers_is_always_empty() {
    assert_eq!(deselect_all_layers(), Vec::<LayerId>::new());
}

#[test]
fn invert_layer_selection_returns_the_complement_within_present_layers() {
    let (doc, layers) = doc_with_three_layers();
    let view = doc.view();
    let inverted = invert_layer_selection(&view, &[layers[1]]);
    assert_eq!(inverted, vec![layers[0], layers[2]]);
}

#[test]
fn invert_layer_selection_ignores_ids_that_no_longer_exist() {
    let (doc, layers) = doc_with_three_layers();
    let view = doc.view();
    let inverted = invert_layer_selection(&view, &[LayerId(999)]);
    assert_eq!(inverted, layers.to_vec(), "存在しない id は補集合の計算に影響しない");
}

#[test]
fn step_layer_selection_moves_to_the_adjacent_layer_in_stacking_order() {
    let (doc, layers) = doc_with_three_layers();
    let view = doc.view();
    assert_eq!(
        step_layer_selection(&view, Some(layers[1]), LayerStepDirection::Next),
        Some(layers[2])
    );
    assert_eq!(
        step_layer_selection(&view, Some(layers[1]), LayerStepDirection::Previous),
        Some(layers[0])
    );
}

#[test]
fn step_layer_selection_clamps_at_the_stacking_order_boundaries() {
    let (doc, layers) = doc_with_three_layers();
    let view = doc.view();
    assert_eq!(
        step_layer_selection(&view, Some(layers[2]), LayerStepDirection::Next),
        Some(layers[2]),
        "末尾で Next はクランプ"
    );
    assert_eq!(
        step_layer_selection(&view, Some(layers[0]), LayerStepDirection::Previous),
        Some(layers[0]),
        "先頭で Previous はクランプ"
    );
}

#[test]
fn step_layer_selection_falls_back_to_the_first_or_last_layer_when_unanchored() {
    let (doc, layers) = doc_with_three_layers();
    let view = doc.view();
    assert_eq!(
        step_layer_selection(&view, None, LayerStepDirection::Next),
        Some(layers[0])
    );
    assert_eq!(
        step_layer_selection(&view, None, LayerStepDirection::Previous),
        Some(layers[2])
    );
    // 存在しない id も「アンカー無し」と同じ扱い。
    assert_eq!(
        step_layer_selection(&view, Some(LayerId(999)), LayerStepDirection::Next),
        Some(layers[0])
    );
}

#[test]
fn step_layer_selection_is_none_when_there_are_no_layers() {
    let doc = Document::new();
    let view = doc.view();
    assert_eq!(step_layer_selection(&view, None, LayerStepDirection::Next), None);
}
