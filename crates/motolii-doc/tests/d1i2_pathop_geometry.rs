//! D1i-2: PathOp意味論表(docs/specs/M2-document-model.md)の幾何ゴールデン。
//! 本ファイルのアサーションは意味論ゴールデン(GR-PV-5) — 数値の更新は「新variant」でのみ許可し、
//! 既存アサーションの書き換えは禁止(AGENTS.md「テストを『直して』通さない」)。

use motolii_doc::pathgeom::{
    apply, Contour, Path, Point, ResolvedPathOp, ResolvedTransform, Vertex,
};
use motolii_doc::{CompositeOrder, LineJoin, PathOpError, PointType, TrimMode};

fn p(x: f64, y: f64) -> Point {
    Point { x, y }
}

fn approx(a: Point, b: Point) {
    assert!(
        (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6,
        "expected {:?}, got {:?}",
        b,
        a
    );
}

// --- 退化規約: 空パス/頂点1以下は恒等 ---

#[test]
fn empty_path_is_identity_for_every_op() {
    let path = Path::default();
    for op in [
        ResolvedPathOp::PuckerBloat { amount: 0.5 },
        ResolvedPathOp::ZigZag {
            amount: 0.1,
            ridges: 3.0,
            point_type: PointType::Corner,
        },
        ResolvedPathOp::RoundCorners { radius: 0.1 },
        ResolvedPathOp::Twist {
            angle: 1.0,
            center: Point::ZERO,
        },
    ] {
        assert_eq!(apply(&path, &op, 0.0).unwrap(), path);
    }
}

#[test]
fn single_vertex_contour_is_identity() {
    let path = Path {
        contours: vec![Contour::open([p(0.3, -0.2)])],
    };
    let op = ResolvedPathOp::PuckerBloat { amount: 1.0 };
    assert_eq!(apply(&path, &op, 0.0).unwrap(), path);
    let op = ResolvedPathOp::ZigZag {
        amount: 0.2,
        ridges: 2.0,
        point_type: PointType::Corner,
    };
    assert_eq!(apply(&path, &op, 0.0).unwrap(), path);
}

// --- pucker_bloat ---

#[test]
fn pucker_bloat_amount_zero_is_identity() {
    let path = Path {
        contours: vec![Contour::closed([
            p(-1.0, -1.0),
            p(1.0, -1.0),
            p(1.0, 1.0),
            p(-1.0, 1.0),
        ])],
    };
    let out = apply(&path, &ResolvedPathOp::PuckerBloat { amount: 0.0 }, 0.0).unwrap();
    assert_eq!(out, path);
}

#[test]
fn pucker_bloat_plus_one_collapses_to_centroid() {
    let path = Path {
        contours: vec![Contour::closed([
            p(-1.0, -1.0),
            p(1.0, -1.0),
            p(1.0, 1.0),
            p(-1.0, 1.0),
        ])],
    };
    let out = apply(&path, &ResolvedPathOp::PuckerBloat { amount: 1.0 }, 0.0).unwrap();
    for v in &out.contours[0].vertices {
        approx(v.point, p(0.0, 0.0));
    }
}

#[test]
fn pucker_bloat_minus_one_doubles_distance_from_centroid() {
    let path = Path {
        contours: vec![Contour::closed([
            p(-1.0, -1.0),
            p(1.0, -1.0),
            p(1.0, 1.0),
            p(-1.0, 1.0),
        ])],
    };
    let out = apply(&path, &ResolvedPathOp::PuckerBloat { amount: -1.0 }, 0.0).unwrap();
    let expected = [p(-2.0, -2.0), p(2.0, -2.0), p(2.0, 2.0), p(-2.0, 2.0)];
    for (v, e) in out.contours[0].vertices.iter().zip(expected) {
        approx(v.point, e);
    }
}

#[test]
fn pucker_bloat_tangent_moves_opposite_to_vertex() {
    // 接線は頂点と逆向きに補間(意味論表)。頂点が重心へamount*d動くとき、絶対ハンドル位置は
    // +amount*dだけ動く(頂点の変位と符号が反転する)ことを固定する。
    use motolii_doc::pathgeom::Vertex;
    let centroid = p(0.0, 0.0);
    let vertex = p(1.0, 0.0);
    let contour = Contour {
        vertices: vec![
            Vertex {
                point: vertex,
                in_tangent: p(0.0, 0.0),
                out_tangent: p(0.0, 0.2),
            },
            Vertex {
                point: p(-1.0, 0.0),
                in_tangent: p(0.0, -0.2),
                out_tangent: p(0.0, 0.0),
            },
        ],
        closed: false,
    };
    let path = Path {
        contours: vec![contour],
    };
    let amount = 0.5;
    let out = apply(&path, &ResolvedPathOp::PuckerBloat { amount }, 0.0).unwrap();
    let v0 = &out.contours[0].vertices[0];
    let d = vertex.x - centroid.x; // 重心はcentroid_of([(1,0),(-1,0)]) = (0,0) なので d=1.0
    approx(v0.point, p(1.0 - amount * d, 0.0));
    // out_tangent = 元(0,0.2) + handle_shift(2*amount*d, 0) — x成分にのみ逆向きの変位が乗る。
    approx(v0.out_tangent, p(2.0 * amount * d, 0.2));
}

// --- zig_zag ---

#[test]
fn zig_zag_single_ridge_corner_on_open_segment() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(2.0, 0.0)])],
    };
    let op = ResolvedPathOp::ZigZag {
        amount: 0.5,
        ridges: 1.0,
        point_type: PointType::Corner,
    };
    let out = apply(&path, &op, 0.0).unwrap();
    let pts: Vec<Point> = out.contours[0].vertices.iter().map(|v| v.point).collect();
    assert_eq!(pts.len(), 3);
    approx(pts[0], p(0.0, 0.0));
    approx(pts[1], p(1.0, 0.5));
    approx(pts[2], p(2.0, 0.0));
    // corner: ゼロタンジェント(直線)
    for v in &out.contours[0].vertices {
        approx(v.in_tangent, Point::ZERO);
        approx(v.out_tangent, Point::ZERO);
    }
}

#[test]
fn zig_zag_amount_zero_is_identity() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(2.0, 0.0)])],
    };
    let op = ResolvedPathOp::ZigZag {
        amount: 0.0,
        ridges: 5.0,
        point_type: PointType::Smooth,
    };
    let out = apply(&path, &op, 0.0).unwrap();
    assert_eq!(out, path);
}

#[test]
fn zig_zag_negative_amount_flips_first_peak_direction() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(2.0, 0.0)])],
    };
    let op = ResolvedPathOp::ZigZag {
        amount: -0.5,
        ridges: 1.0,
        point_type: PointType::Corner,
    };
    let out = apply(&path, &op, 0.0).unwrap();
    approx(out.contours[0].vertices[1].point, p(1.0, -0.5));
}

#[test]
fn zig_zag_bezier_input_differs_from_chord_only() {
    let chord = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(2.0, 0.0)])],
    };
    let curved = Path {
        contours: vec![Contour {
            vertices: vec![
                Vertex {
                    point: p(0.0, 0.0),
                    in_tangent: Point::ZERO,
                    out_tangent: p(1.0, 0.8),
                },
                Vertex {
                    point: p(2.0, 0.0),
                    in_tangent: p(-1.0, 0.8),
                    out_tangent: Point::ZERO,
                },
            ],
            closed: false,
        }],
    };
    let op = ResolvedPathOp::ZigZag {
        amount: 0.5,
        ridges: 1.0,
        point_type: PointType::Corner,
    };
    let chord_out = apply(&chord, &op, 0.0).unwrap();
    let curved_out = apply(&curved, &op, 0.0).unwrap();
    assert_ne!(
        chord_out.contours[0].vertices[1].point, curved_out.contours[0].vertices[1].point,
        "bezier arc midpoint must differ from chord midpoint"
    );
    approx(chord_out.contours[0].vertices[1].point, p(1.0, 0.5));
    assert!(
        curved_out.contours[0].vertices[1].point.y > 0.5,
        "curved ridge peak should bulge further outward than chord"
    );
}

// --- round_corners ---

#[test]
fn round_corners_right_angle_fillet_matches_analytic_tangent_points() {
    // prev=(0,0), corner=(1,0), next=(1,1): 90°コーナーをradius=0.2でfillet。
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0)])],
    };
    let out = apply(&path, &ResolvedPathOp::RoundCorners { radius: 0.2 }, 0.0).unwrap();
    let pts: Vec<Point> = out.contours[0].vertices.iter().map(|v| v.point).collect();
    assert_eq!(pts.len(), 4, "start + 2 arc endpoints(90°=1segment) + end");
    approx(pts[0], p(0.0, 0.0));
    approx(pts[1], p(0.8, 0.0)); // 接線点1(cur - radius/tan(45°) を prev方向へ)
    approx(pts[2], p(1.0, 0.2)); // 接線点2(next方向へ)
    approx(pts[3], p(1.0, 1.0));
}

#[test]
fn round_corners_radius_zero_is_identity() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0)])],
    };
    let out = apply(&path, &ResolvedPathOp::RoundCorners { radius: 0.0 }, 0.0).unwrap();
    assert_eq!(out, path);
}

#[test]
fn round_corners_skips_open_path_endpoints() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0)])],
    };
    let out = apply(&path, &ResolvedPathOp::RoundCorners { radius: 0.2 }, 0.0).unwrap();
    let pts: Vec<Point> = out.contours[0].vertices.iter().map(|v| v.point).collect();
    approx(*pts.first().unwrap(), p(0.0, 0.0));
    approx(*pts.last().unwrap(), p(1.0, 1.0));
}

// --- offset ---

#[test]
fn offset_open_path_is_typed_unsupported() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(1.0, 0.0)])],
    };
    let err = apply(
        &path,
        &ResolvedPathOp::Offset {
            distance: 0.1,
            line_join: LineJoin::Miter,
            miter_limit: 4.0,
        },
        0.0,
    )
    .unwrap_err();
    assert_eq!(err, PathOpError::OpenPathOffsetUnsupported);
}

#[test]
fn offset_miter_square_expands_by_distance_with_sharp_corners() {
    let path = Path {
        contours: vec![Contour::closed([
            p(0.0, 0.0),
            p(1.0, 0.0),
            p(1.0, 1.0),
            p(0.0, 1.0),
        ])],
    };
    let out = apply(
        &path,
        &ResolvedPathOp::Offset {
            distance: 0.1,
            line_join: LineJoin::Miter,
            miter_limit: 4.0,
        },
        0.0,
    )
    .unwrap();
    let pts: Vec<Point> = out.contours[0].vertices.iter().map(|v| v.point).collect();
    assert_eq!(pts.len(), 4);
    let expected = [p(-0.1, -0.1), p(1.1, -0.1), p(1.1, 1.1), p(-0.1, 1.1)];
    for (got, want) in pts.iter().zip(expected) {
        approx(*got, want);
    }
}

#[test]
fn offset_bevel_square_adds_chamfer_points_at_corners() {
    let path = Path {
        contours: vec![Contour::closed([
            p(0.0, 0.0),
            p(1.0, 0.0),
            p(1.0, 1.0),
            p(0.0, 1.0),
        ])],
    };
    let out = apply(
        &path,
        &ResolvedPathOp::Offset {
            distance: 0.1,
            line_join: LineJoin::Bevel,
            miter_limit: 4.0,
        },
        0.0,
    )
    .unwrap();
    // Bevelは各頂点で2点(prev_b, cur_a)を残す = 4角×2 = 8頂点。
    assert_eq!(out.contours[0].vertices.len(), 8);
}

#[test]
fn offset_negative_distance_shrinks_inward() {
    let path = Path {
        contours: vec![Contour::closed([
            p(0.0, 0.0),
            p(1.0, 0.0),
            p(1.0, 1.0),
            p(0.0, 1.0),
        ])],
    };
    let out = apply(
        &path,
        &ResolvedPathOp::Offset {
            distance: -0.1,
            line_join: LineJoin::Miter,
            miter_limit: 4.0,
        },
        0.0,
    )
    .unwrap();
    let pts: Vec<Point> = out.contours[0].vertices.iter().map(|v| v.point).collect();
    let expected = [p(0.1, 0.1), p(0.9, 0.1), p(0.9, 0.9), p(0.1, 0.9)];
    for (got, want) in pts.iter().zip(expected) {
        approx(*got, want);
    }
}

#[test]
fn offset_bezier_input_differs_from_chord_only() {
    let corner = Path {
        contours: vec![Contour::closed([p(0.0, 0.0), p(2.0, 0.0), p(1.0, -1.0)])],
    };
    // 上辺(0,0)-(2,0)だけベジエで外側へ膨らませる。
    let curved = Path {
        contours: vec![Contour {
            vertices: vec![
                Vertex {
                    point: p(0.0, 0.0),
                    in_tangent: Point::ZERO,
                    out_tangent: p(1.0, 0.6),
                },
                Vertex {
                    point: p(2.0, 0.0),
                    in_tangent: p(-1.0, 0.6),
                    out_tangent: Point::ZERO,
                },
                Vertex::corner(p(1.0, -1.0)),
            ],
            closed: true,
        }],
    };
    let op = ResolvedPathOp::Offset {
        distance: 0.1,
        line_join: LineJoin::Miter,
        miter_limit: 4.0,
    };
    let corner_out = apply(&corner, &op, 0.0).unwrap();
    let curved_out = apply(&curved, &op, 0.0).unwrap();
    assert_ne!(corner_out, curved_out);
}

// --- trim ---

#[test]
fn trim_parallel_extracts_length_window_on_straight_segment() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(4.0, 0.0)])],
    };
    let out = apply(
        &path,
        &ResolvedPathOp::Trim {
            start: 0.25,
            end: 0.75,
            offset: 0.0,
            mode: TrimMode::Parallel,
        },
        0.0,
    )
    .unwrap();
    assert_eq!(out.contours.len(), 1);
    let pts: Vec<Point> = out.contours[0].vertices.iter().map(|v| v.point).collect();
    approx(pts[0], p(1.0, 0.0));
    approx(*pts.last().unwrap(), p(3.0, 0.0));
}

#[test]
fn trim_start_equals_end_yields_empty_path() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(4.0, 0.0)])],
    };
    let out = apply(
        &path,
        &ResolvedPathOp::Trim {
            start: 0.5,
            end: 0.5,
            offset: 0.0,
            mode: TrimMode::Parallel,
        },
        0.0,
    )
    .unwrap();
    assert!(out.contours.is_empty());
}

#[test]
fn trim_sequential_treats_two_contours_as_one_connected_length() {
    // 各長さ2のセグメントを2本(合計長4)。sequentialでstart=0,end=0.75は
    // 1本目全部+2本目の半分(長さ3分)を連結長として切り出す。
    let path = Path {
        contours: vec![
            Contour::open([p(0.0, 0.0), p(2.0, 0.0)]),
            Contour::open([p(10.0, 0.0), p(10.0, 2.0)]),
        ],
    };
    let out = apply(
        &path,
        &ResolvedPathOp::Trim {
            start: 0.0,
            end: 0.75,
            offset: 0.0,
            mode: TrimMode::Sequential,
        },
        0.0,
    )
    .unwrap();
    // 輪郭境界をまたぐため2本の輪郭に分かれる。
    assert_eq!(out.contours.len(), 2);
    let first: Vec<Point> = out.contours[0].vertices.iter().map(|v| v.point).collect();
    let second: Vec<Point> = out.contours[1].vertices.iter().map(|v| v.point).collect();
    approx(first[0], p(0.0, 0.0));
    approx(*first.last().unwrap(), p(2.0, 0.0));
    approx(second[0], p(10.0, 0.0));
    approx(*second.last().unwrap(), p(10.0, 1.0));
}

#[test]
fn trim_offset_wraps_window_across_zero_boundary() {
    let path = Path {
        contours: vec![Contour::closed([
            p(0.0, 0.0),
            p(4.0, 0.0),
            p(4.0, 4.0),
            p(0.0, 4.0),
        ])],
    };
    // 周長16。start=0.9,end=0.1相当をoffset=0で直接: end<start→+1して0.9..1.1、coverage=0.2。
    let out = apply(
        &path,
        &ResolvedPathOp::Trim {
            start: 0.9,
            end: 0.1,
            offset: 0.0,
            mode: TrimMode::Parallel,
        },
        0.0,
    )
    .unwrap();
    // ラップするため2本に分かれる(0.9..1.0 と 0.0..0.1)。
    assert_eq!(out.contours.len(), 2);
}

// --- twist ---

#[test]
fn twist_self_normalizes_by_max_radius_in_contour() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(0.5, 0.0), p(1.0, 0.0)])],
    };
    let angle = std::f64::consts::FRAC_PI_2;
    let out = apply(
        &path,
        &ResolvedPathOp::Twist {
            angle,
            center: Point::ZERO,
        },
        0.0,
    )
    .unwrap();
    let pts: Vec<Point> = out.contours[0].vertices.iter().map(|v| v.point).collect();
    approx(pts[0], p(0.0, 0.0)); // r=0: 不変
    approx(pts[2], p(1.0, 0.0)); // r=max: 角度0(不変)
    let half = std::f64::consts::FRAC_PI_4;
    approx(pts[1], p(0.5 * half.cos(), 0.5 * half.sin())); // r=0.5: 45°回転
}

#[test]
fn twist_all_points_at_center_is_identity() {
    let path = Path {
        contours: vec![Contour::open([p(1.0, 1.0), p(1.0, 1.0)])],
    };
    let out = apply(
        &path,
        &ResolvedPathOp::Twist {
            angle: 1.0,
            center: p(1.0, 1.0),
        },
        0.0,
    )
    .unwrap();
    assert_eq!(out, path);
}

// --- wiggle ---

#[test]
fn wiggle_is_deterministic_given_same_seed_and_time() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(1.0, 0.0), p(0.5, 1.0)])],
    };
    let op = ResolvedPathOp::Wiggle {
        amp: 0.05,
        freq: 2.0,
        seed: 12345,
    };
    let a = apply(&path, &op, 0.37).unwrap();
    let b = apply(&path, &op, 0.37).unwrap();
    assert_eq!(a, b);
}

#[test]
fn wiggle_different_seeds_produce_different_output() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(1.0, 0.0), p(0.5, 1.0)])],
    };
    let a = apply(
        &path,
        &ResolvedPathOp::Wiggle {
            amp: 0.05,
            freq: 2.0,
            seed: 1,
        },
        0.37,
    )
    .unwrap();
    let b = apply(
        &path,
        &ResolvedPathOp::Wiggle {
            amp: 0.05,
            freq: 2.0,
            seed: 2,
        },
        0.37,
    )
    .unwrap();
    assert_ne!(a, b);
}

#[test]
fn wiggle_zero_amplitude_is_identity() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(1.0, 0.0)])],
    };
    let out = apply(
        &path,
        &ResolvedPathOp::Wiggle {
            amp: 0.0,
            freq: 2.0,
            seed: 7,
        },
        0.5,
    )
    .unwrap();
    assert_eq!(out, path);
}

// --- repeater ---

#[test]
fn repeater_translates_each_copy_by_incremental_position() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0), p(0.0, 1.0)])],
    };
    let op = ResolvedPathOp::Repeater {
        copies: 3.0,
        offset: 0.0,
        transform: ResolvedTransform {
            position: p(1.0, 0.0),
            ..ResolvedTransform::IDENTITY
        },
        composite: CompositeOrder::Above,
        start_opacity: 1.0,
        end_opacity: 1.0,
    };
    let out = apply(&path, &op, 0.0).unwrap();
    assert_eq!(out.contours.len(), 3);
    for (k, c) in out.contours.iter().enumerate() {
        approx(c.vertices[0].point, p(k as f64, 0.0));
        approx(c.vertices[1].point, p(k as f64, 1.0));
    }
}

#[test]
fn repeater_below_composite_reverses_copy_order() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0)])],
    };
    let op = ResolvedPathOp::Repeater {
        copies: 3.0,
        offset: 0.0,
        transform: ResolvedTransform {
            position: p(1.0, 0.0),
            ..ResolvedTransform::IDENTITY
        },
        composite: CompositeOrder::Below,
        start_opacity: 1.0,
        end_opacity: 1.0,
    };
    let out = apply(&path, &op, 0.0).unwrap();
    let first: Vec<Point> = out.contours[0].vertices.iter().map(|v| v.point).collect();
    approx(first[0], p(2.0, 0.0)); // k=2が最初(Below)
    let last: Vec<Point> = out.contours[2].vertices.iter().map(|v| v.point).collect();
    approx(last[0], p(0.0, 0.0)); // k=0が最後
}

#[test]
fn repeater_rotation_composes_across_integer_copies() {
    let path = Path {
        contours: vec![Contour::open([p(1.0, 0.0)])],
    };
    let op = ResolvedPathOp::Repeater {
        copies: 2.0,
        offset: 0.0,
        transform: ResolvedTransform {
            rotation: std::f64::consts::FRAC_PI_2,
            ..ResolvedTransform::IDENTITY
        },
        composite: CompositeOrder::Above,
        start_opacity: 1.0,
        end_opacity: 1.0,
    };
    let out = apply(&path, &op, 0.0).unwrap();
    approx(out.contours[0].vertices[0].point, p(1.0, 0.0)); // k=0: 恒等
    approx(out.contours[1].vertices[0].point, p(0.0, 1.0)); // k=1: 90°回転
}

#[test]
fn repeater_zero_copies_yields_empty_path() {
    let path = Path {
        contours: vec![Contour::open([p(0.0, 0.0)])],
    };
    let op = ResolvedPathOp::Repeater {
        copies: 0.0,
        offset: 0.0,
        transform: ResolvedTransform::IDENTITY,
        composite: CompositeOrder::Above,
        start_opacity: 1.0,
        end_opacity: 1.0,
    };
    let out = apply(&path, &op, 0.0).unwrap();
    assert!(out.contours.is_empty());
}

#[test]
fn repeater_fractional_offset_linearly_blends_matrix_components() {
    // v1簡略近似(モジュールdoc注記): 実数冪はLie群指数写像ではなく整数冪間の行列線形補間。
    // ここではその近似の具体値を意味論ゴールデンとして固定する(将来の近似変更は新variant相当の扱い)。
    let path = Path {
        contours: vec![Contour::open([p(1.0, 0.0)])],
    };
    let op = ResolvedPathOp::Repeater {
        copies: 1.0,
        offset: 0.5,
        transform: ResolvedTransform {
            rotation: std::f64::consts::FRAC_PI_2,
            ..ResolvedTransform::IDENTITY
        },
        composite: CompositeOrder::Above,
        start_opacity: 1.0,
        end_opacity: 1.0,
    };
    let out = apply(&path, &op, 0.0).unwrap();
    approx(out.contours[0].vertices[0].point, p(0.5, 0.5));
}

#[test]
fn repeater_negative_offset_applies_inverse_transform() {
    // offset=-1 で M^(-1) を適用(Lottie Repeaterのスタックシフト)。
    let path = Path {
        contours: vec![Contour::open([p(1.0, 0.0)])],
    };
    let op = ResolvedPathOp::Repeater {
        copies: 1.0,
        offset: -1.0,
        transform: ResolvedTransform {
            rotation: std::f64::consts::FRAC_PI_2,
            ..ResolvedTransform::IDENTITY
        },
        composite: CompositeOrder::Above,
        start_opacity: 1.0,
        end_opacity: 1.0,
    };
    let out = apply(&path, &op, 0.0).unwrap();
    approx(out.contours[0].vertices[0].point, p(0.0, -1.0));
}
