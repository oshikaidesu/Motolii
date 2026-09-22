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
    let g = gradient(GradientUnits::ObjectBoundingBox);
    let bounds = [10.0, 0.0, 50.0, 20.0];
    assert!((g.parameter_in(Point { x: 20.0, y: 5.0 }, bounds) - 0.25).abs() < 1e-12);
    assert!((g.parameter_in(Point { x: 50.0, y: 5.0 }, bounds) - 1.0).abs() < 1e-12);
    let user = gradient(GradientUnits::UserSpaceOnUse);
    assert_eq!(user.parameter_in(Point { x: 0.25, y: 5.0 }, bounds), user.parameter(Point { x: 0.25, y: 5.0 }));
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
