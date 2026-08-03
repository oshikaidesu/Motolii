//! SDK-S0I: language-neutral Path2D semantic fixture consumer.
//!
//! This is deliberately test-only. The fixture vocabulary is not a public SDK,
//! Document schema, package manifest, or TypeScript surface.

use std::collections::HashMap;
use std::sync::OnceLock;

use motolii_doc::pathgeom::{apply, Contour, Path, Point, ResolvedPathOp, Vertex};
use motolii_doc::{LineJoin, PathOpError};
use serde::Deserialize;

const FIXTURE_JSON: &str =
    include_str!("../../../docs/reviews/evidence/sdk-s0-path2d/sdk-s0-path2d.fixture.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FixtureSpace {
    CanonicalLocal,
    World,
}

#[derive(Debug, Clone, PartialEq)]
enum FixtureInput {
    Path2D { path: Path, space: FixtureSpace },
    Missing,
    OtherType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FixtureDiagnostic {
    MissingInput,
    TypeMismatch,
    SpaceMismatch,
    NonFiniteDistance,
    OpenContourUnsupported,
    BudgetExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DiagnosticTarget {
    Source,
    Distance,
    SourceBudget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FixtureFailure {
    reason: FixtureDiagnostic,
    target: DiagnosticTarget,
}

#[derive(Debug, Clone, PartialEq)]
struct ContractProjection {
    label: String,
    input: FixturePort,
    output: FixturePort,
    parameter: FixtureParameter,
    pure: bool,
    capability: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixturePort {
    name: String,
    value_type: String,
    space: FixtureSpace,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureParameter {
    name: String,
    value_type: String,
    unit: String,
    finite: bool,
    default: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct FixtureResult {
    path: Path,
    projection: ContractProjection,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureFile {
    contract: FixtureContract,
    equivalence_tolerance: f64,
    vertex_budget: usize,
    paths: HashMap<String, FixturePath>,
    cases: Vec<FixtureCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureContract {
    label: String,
    input: FixturePort,
    output: FixturePort,
    parameter: FixtureParameter,
    operation_profile: FixtureOperationProfile,
    pure: bool,
    capability: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureOperationProfile {
    line_join: FixtureLineJoin,
    miter_limit: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FixtureLineJoin {
    Miter,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureCase {
    id: String,
    assertion: FixtureAssertion,
    #[serde(default)]
    comparison_path: Option<String>,
    runs: Vec<FixtureRun>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FixtureAssertion {
    NativeIdentity,
    NativeEquivalent,
    BezierNativeDistinct,
    Failures,
    ConsumerIndependent,
    TimeIndependent,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureRun {
    source: FixtureSource,
    distance: FixtureDistance,
    time: f64,
    expected: FixtureExpected,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum FixtureSource {
    Path { path: String, space: FixtureSpace },
    Missing,
    OtherType,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum FixtureDistance {
    Finite { value: f64 },
    NonFinite { value: NonFiniteValue },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum NonFiniteValue {
    Nan,
    PositiveInfinity,
    NegativeInfinity,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum FixtureExpected {
    Success,
    Failure {
        reason: FixtureDiagnostic,
        target: DiagnosticTarget,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixturePath {
    contours: Vec<FixtureContour>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureContour {
    closed: bool,
    vertices: Vec<FixtureVertex>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureVertex {
    point: [f64; 2],
    in_tangent: [f64; 2],
    out_tangent: [f64; 2],
}

fn fixture() -> &'static FixtureFile {
    static FIXTURE: OnceLock<FixtureFile> = OnceLock::new();
    FIXTURE.get_or_init(|| serde_json::from_str(FIXTURE_JSON).expect("valid SDK-S0 fixture"))
}

fn case(id: &str) -> &'static FixtureCase {
    fixture()
        .cases
        .iter()
        .find(|case| case.id == id)
        .unwrap_or_else(|| panic!("missing fixture case {id}"))
}

fn point(value: [f64; 2]) -> Point {
    Point {
        x: value[0],
        y: value[1],
    }
}

fn path(name: &str) -> Path {
    let fixture_path = fixture()
        .paths
        .get(name)
        .unwrap_or_else(|| panic!("missing fixture path {name}"));
    Path {
        contours: fixture_path
            .contours
            .iter()
            .map(|contour| Contour {
                closed: contour.closed,
                vertices: contour
                    .vertices
                    .iter()
                    .map(|vertex| Vertex {
                        point: point(vertex.point),
                        in_tangent: point(vertex.in_tangent),
                        out_tangent: point(vertex.out_tangent),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn input(source: &FixtureSource) -> FixtureInput {
    match source {
        FixtureSource::Path { path: name, space } => FixtureInput::Path2D {
            path: path(name),
            space: *space,
        },
        FixtureSource::Missing => FixtureInput::Missing,
        FixtureSource::OtherType => FixtureInput::OtherType,
    }
}

fn distance(distance: FixtureDistance) -> f64 {
    match distance {
        FixtureDistance::Finite { value } => value,
        FixtureDistance::NonFinite {
            value: NonFiniteValue::Nan,
        } => f64::NAN,
        FixtureDistance::NonFinite {
            value: NonFiniteValue::PositiveInfinity,
        } => f64::INFINITY,
        FixtureDistance::NonFinite {
            value: NonFiniteValue::NegativeInfinity,
        } => f64::NEG_INFINITY,
    }
}

fn projection() -> ContractProjection {
    let contract = &fixture().contract;
    ContractProjection {
        label: contract.label.clone(),
        input: contract.input.clone(),
        output: contract.output.clone(),
        parameter: contract.parameter.clone(),
        pure: contract.pure,
        capability: contract.capability.clone(),
    }
}

fn offset_op(distance: f64) -> ResolvedPathOp {
    let profile = &fixture().contract.operation_profile;
    ResolvedPathOp::Offset {
        distance,
        line_join: match profile.line_join {
            FixtureLineJoin::Miter => LineJoin::Miter,
        },
        miter_limit: profile.miter_limit,
    }
}

fn native_oracle(path: &Path, distance: f64, time: f64) -> Result<Path, PathOpError> {
    apply(path, &offset_op(distance), time)
}

fn fixture_vertex_count(path: &Path) -> usize {
    path.contours
        .iter()
        .map(|contour| contour.vertices.len())
        .sum()
}

fn evaluate(
    input: FixtureInput,
    distance: f64,
    time: f64,
) -> Result<FixtureResult, FixtureFailure> {
    let path = match input {
        FixtureInput::Missing => {
            return Err(FixtureFailure {
                reason: FixtureDiagnostic::MissingInput,
                target: DiagnosticTarget::Source,
            });
        }
        FixtureInput::OtherType => {
            return Err(FixtureFailure {
                reason: FixtureDiagnostic::TypeMismatch,
                target: DiagnosticTarget::Source,
            });
        }
        FixtureInput::Path2D { path, space } => {
            if space != FixtureSpace::CanonicalLocal {
                return Err(FixtureFailure {
                    reason: FixtureDiagnostic::SpaceMismatch,
                    target: DiagnosticTarget::Source,
                });
            }
            path
        }
    };

    if !distance.is_finite() {
        return Err(FixtureFailure {
            reason: FixtureDiagnostic::NonFiniteDistance,
            target: DiagnosticTarget::Distance,
        });
    }
    if fixture_vertex_count(&path) > fixture().vertex_budget {
        return Err(FixtureFailure {
            reason: FixtureDiagnostic::BudgetExceeded,
            target: DiagnosticTarget::SourceBudget,
        });
    }

    let path = native_oracle(&path, distance, time).map_err(|error| match error {
        PathOpError::OpenPathOffsetUnsupported => FixtureFailure {
            reason: FixtureDiagnostic::OpenContourUnsupported,
            target: DiagnosticTarget::Source,
        },
    })?;
    Ok(FixtureResult {
        path,
        projection: projection(),
    })
}

fn evaluate_run(run: &FixtureRun) -> Result<FixtureResult, FixtureFailure> {
    evaluate(input(&run.source), distance(run.distance), run.time)
}

fn fixture_failure(expected: &FixtureExpected) -> FixtureFailure {
    match expected {
        FixtureExpected::Failure { reason, target } => FixtureFailure {
            reason: *reason,
            target: *target,
        },
        FixtureExpected::Success => panic!("fixture run expected success"),
    }
}

fn assert_expected_success(expected: &FixtureExpected) {
    assert!(matches!(expected, FixtureExpected::Success));
}

fn source_path(run: &FixtureRun) -> Path {
    match &run.source {
        FixtureSource::Path { path: name, .. } => path(name),
        FixtureSource::Missing | FixtureSource::OtherType => {
            panic!("fixture run has no Path2D source")
        }
    }
}

fn assert_path_equivalent(got: &Path, expected: &Path) {
    assert_eq!(got.contours.len(), expected.contours.len());
    for (got_contour, expected_contour) in got.contours.iter().zip(&expected.contours) {
        assert_eq!(got_contour.closed, expected_contour.closed);
        assert_eq!(got_contour.vertices.len(), expected_contour.vertices.len());
        for (got, expected) in got_contour
            .vertices
            .iter()
            .zip(expected_contour.vertices.iter())
        {
            assert_point(got.point, expected.point);
            assert_point(got.in_tangent, expected.in_tangent);
            assert_point(got.out_tangent, expected.out_tangent);
        }
    }
}

fn assert_point(got: Point, expected: Point) {
    let epsilon = fixture().equivalence_tolerance;
    assert!(
        (got.x - expected.x).abs() < epsilon,
        "x: got {got:?}, expected {expected:?}"
    );
    assert!(
        (got.y - expected.y).abs() < epsilon,
        "y: got {got:?}, expected {expected:?}"
    );
}

#[test]
fn s0_projection_is_typed_and_pure() {
    let projection = projection();
    assert_eq!(projection.label, "Offset Path");
    assert_eq!(projection.input.name, "source");
    assert_eq!(projection.input.value_type, "Path2D");
    assert_eq!(projection.input.space, FixtureSpace::CanonicalLocal);
    assert_eq!(projection.output.name, "result");
    assert_eq!(projection.output.value_type, "Path2D");
    assert_eq!(projection.output.space, FixtureSpace::CanonicalLocal);
    assert_eq!(projection.parameter.name, "distance");
    assert_eq!(projection.parameter.value_type, "scalar");
    assert_eq!(projection.parameter.unit, "canonical-length");
    assert!(projection.parameter.finite);
    assert_eq!(projection.parameter.default, 0.0);
    assert!(projection.pure);
    assert_eq!(projection.capability, "path2d-offset");
}

#[test]
fn s0_p1_zero_distance_is_native_identity() {
    let case = case("S0-P1");
    assert_eq!(case.assertion, FixtureAssertion::NativeIdentity);
    let run = &case.runs[0];
    assert_expected_success(&run.expected);
    let source = source_path(run);
    let result = evaluate_run(run).unwrap();
    let expected = native_oracle(&source, distance(run.distance), run.time).unwrap();
    assert_path_equivalent(&result.path, &expected);
    assert_path_equivalent(&result.path, &source);
}

#[test]
fn s0_p2_and_p3_finite_offsets_match_native_oracle() {
    for id in ["S0-P2", "S0-P3"] {
        let case = case(id);
        assert_eq!(case.assertion, FixtureAssertion::NativeEquivalent);
        let run = &case.runs[0];
        assert_expected_success(&run.expected);
        let source = source_path(run);
        let expected = native_oracle(&source, distance(run.distance), run.time).unwrap();
        let result = evaluate_run(run).unwrap();
        assert_path_equivalent(&result.path, &expected);
    }
}

#[test]
fn s0_p4_bezier_offset_uses_native_oracle_without_new_numeric_golden() {
    let case = case("S0-P4");
    assert_eq!(case.assertion, FixtureAssertion::BezierNativeDistinct);
    let run = &case.runs[0];
    assert_expected_success(&run.expected);
    let source = source_path(run);
    let expected = native_oracle(&source, distance(run.distance), run.time).unwrap();
    let chord_result = native_oracle(
        &path(case.comparison_path.as_deref().unwrap()),
        distance(run.distance),
        run.time,
    )
    .unwrap();
    let result = evaluate_run(run).unwrap();
    assert_path_equivalent(&result.path, &expected);
    assert_ne!(result.path, chord_result);
}

#[test]
fn s0_n1_open_contour_is_typed_failure() {
    let case = case("S0-N1");
    assert_eq!(case.assertion, FixtureAssertion::Failures);
    let run = &case.runs[0];
    assert_eq!(evaluate_run(run), Err(fixture_failure(&run.expected)));
}

#[test]
fn s0_n2_non_finite_distance_is_rejected_before_native_oracle() {
    let case = case("S0-N2");
    assert_eq!(case.assertion, FixtureAssertion::Failures);
    for run in &case.runs {
        assert_eq!(evaluate_run(run), Err(fixture_failure(&run.expected)));
    }
}

#[test]
fn s0_n3_missing_and_wrong_type_do_not_become_empty_paths() {
    let case = case("S0-N3");
    assert_eq!(case.assertion, FixtureAssertion::Failures);
    for run in &case.runs {
        assert_eq!(evaluate_run(run), Err(fixture_failure(&run.expected)));
    }
}

#[test]
fn s0_n4_wrong_space_is_not_implicitly_converted() {
    let case = case("S0-N4");
    assert_eq!(case.assertion, FixtureAssertion::Failures);
    let run = &case.runs[0];
    assert_eq!(evaluate_run(run), Err(fixture_failure(&run.expected)));
}

#[test]
fn s0_n5_consumers_receive_independent_values() {
    let case = case("S0-N5");
    assert_eq!(case.assertion, FixtureAssertion::ConsumerIndependent);
    let source = source_path(&case.runs[0]);
    assert_expected_success(&case.runs[0].expected);
    assert_expected_success(&case.runs[1].expected);
    let first = evaluate(
        FixtureInput::Path2D {
            path: source.clone(),
            space: FixtureSpace::CanonicalLocal,
        },
        distance(case.runs[0].distance),
        case.runs[0].time,
    )
    .unwrap();
    let second = evaluate(
        FixtureInput::Path2D {
            path: source.clone(),
            space: FixtureSpace::CanonicalLocal,
        },
        distance(case.runs[1].distance),
        case.runs[1].time,
    )
    .unwrap();
    assert_ne!(first.path, second.path);
    assert_eq!(source, source_path(&case.runs[0]));
}

#[test]
fn s0_n6_fixture_budget_rejects_partial_success() {
    let case = case("S0-N6");
    assert_eq!(case.assertion, FixtureAssertion::Failures);
    let run = &case.runs[0];
    assert_eq!(evaluate_run(run), Err(fixture_failure(&run.expected)));
}

#[test]
fn s0_n7_offset_is_independent_of_ambient_time() {
    let case = case("S0-N7");
    assert_eq!(case.assertion, FixtureAssertion::TimeIndependent);
    assert_expected_success(&case.runs[0].expected);
    assert_expected_success(&case.runs[1].expected);
    let at_zero = evaluate_run(&case.runs[0]).unwrap();
    let at_later_time = evaluate_run(&case.runs[1]).unwrap();
    assert_eq!(at_zero, at_later_time);
}
