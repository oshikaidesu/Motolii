use super::*;

fn gradient(units: GradientUnits) -> Gradient {
    Gradient {
        kind: GradientType::Linear,
        start: Point { x: 0.0, y: 0.5 },
        end: Point { x: 1.0, y: 0.5 },
        stops: Vec::new(),
        stop_ids: Vec::new(),
        next_stop_id: 0,
        blend: GradientBlend::default(),
        units,
    }
}

#[test]
fn a_document_without_units_keeps_user_space_and_writes_nothing_new() {
    let json = serde_json::to_value(gradient(GradientUnits::UserSpaceOnUse)).unwrap();
    assert!(json.get("units").is_none());
    let read: Gradient = serde_json::from_value(json).unwrap();
    assert_eq!(read.units, GradientUnits::UserSpaceOnUse);
}

#[test]
fn an_object_bounding_box_gradient_spans_the_object() {
    let g = gradient(GradientUnits::ObjectBoundingBox).in_user_space([10.0, 0.0, 50.0, 20.0]);
    assert_eq!((g.start, g.end), (Point { x: 10.0, y: 10.0 }, Point { x: 50.0, y: 10.0 }));
    assert!((g.parameter(Point { x: 20.0, y: 5.0 }) - 0.25).abs() < 1e-12);
}

#[test]
fn an_angle_is_kept_as_seen_and_the_line_spans_the_box_corner_to_corner() {
    // 45° in the unit square, on a 2:1 box: still 45° on screen, long enough to reach both corners.
    let diagonal = Gradient { start: Point { x: 0.0, y: 0.0 }, end: Point { x: 1.0, y: 1.0 }, ..gradient(GradientUnits::ObjectBoundingBox) };
    let g = diagonal.in_user_space([0.0, 0.0, 40.0, 20.0]);
    let d = g.end.sub(g.start);
    assert!((d.y.atan2(d.x).to_degrees() - 45.0).abs() < 1e-9);
    assert!(g.parameter(Point { x: 0.0, y: 0.0 }) < 1e-9 && (g.parameter(Point { x: 40.0, y: 20.0 }) - 1.0).abs() < 1e-9);
}

#[test]
fn a_radial_gradient_stays_round_on_a_wide_box() {
    let radial = Gradient { kind: GradientType::Radial, start: Point { x: 0.5, y: 0.5 }, end: Point { x: 1.0, y: 0.5 }, ..gradient(GradientUnits::ObjectBoundingBox) };
    let g = radial.in_user_space([0.0, 0.0, 40.0, 20.0]);
    let (right, down) = (g.parameter(Point { x: 30.0, y: 10.0 }), g.parameter(Point { x: 20.0, y: 20.0 }));
    assert!((right - down).abs() < 1e-9, "equal distances, equal t: {right} vs {down}");
}

#[test]
fn object_bounds_follow_the_curve_not_its_control_points() {
    let bulge = Contour {
        closed: false,
        vertices: vec![
            Vertex { point: Point { x: 0.0, y: 0.0 }, in_tangent: Point { x: 0.0, y: 0.0 }, out_tangent: Point { x: 0.0, y: 10.0 } },
            Vertex { point: Point { x: 10.0, y: 0.0 }, in_tangent: Point { x: 0.0, y: 10.0 }, out_tangent: Point { x: 0.0, y: 0.0 } },
        ],
    };
    let b = geometry_bounds([&bulge]).unwrap();
    assert!((b[3] - 7.5).abs() < 1e-9, "the curve peaks at 3/4 of its control height: {b:?}");
}
