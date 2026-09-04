pub(crate) fn displacement(
    origin: f64,
    current: f64,
    slop: f64,
    already_moving: bool,
) -> Option<f64> {
    let delta = current - origin;
    (delta.is_finite() && (already_moving || delta.abs() >= slop)).then_some(delta)
}
