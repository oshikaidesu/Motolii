use peniko::kurbo::{Affine, Point};
use std::ops::Range;

pub(crate) fn bounded(value: f64, range: Option<(f64, f64)>) -> Result<f64, &'static str> {
    if !value.is_finite() {
        return Err("Value must be finite");
    }
    match range {
        Some((min, max)) if !min.is_finite() || !max.is_finite() || min > max => {
            Err("Invalid value range")
        }
        Some((min, max)) => Ok(value.clamp(min, max)),
        None => Ok(value),
    }
}

fn map_point(transform: Affine, value: [f64; 2]) -> [f64; 2] {
    let point = transform * Point::new(value[0], value[1]);
    [point.x, point.y]
}

fn about(anchor: [f64; 2], transform: Affine) -> Affine {
    let anchor = Point::new(anchor[0], anchor[1]).to_vec2();
    Affine::translate(anchor) * transform * Affine::translate(-anchor)
}

pub(crate) fn translate2(value: [f64; 2], delta: [f64; 2]) -> [f64; 2] {
    map_point(Affine::translate((delta[0], delta[1])), value)
}

pub(crate) fn scale_about(value: [f64; 2], anchor: [f64; 2], ratio: [f64; 2]) -> [f64; 2] {
    map_point(
        about(anchor, Affine::scale_non_uniform(ratio[0], ratio[1])),
        value,
    )
}

pub(crate) fn rotate_about(value: [f64; 2], anchor: [f64; 2], angle_radians: f64) -> [f64; 2] {
    map_point(about(anchor, Affine::rotate(angle_radians)), value)
}

pub(crate) fn translate_frames(start: i64, delta: i64) -> Result<i64, &'static str> {
    start.checked_add(delta).ok_or("Frame position overflow")
}

/// Frame intervals use [start, end); empty intervals are valid mathematical values.
pub(crate) fn frame_span(start: i64, duration: i64) -> Result<Range<i64>, &'static str> {
    if duration < 0 {
        return Err("Frame interval has negative length");
    }
    Ok(start..translate_frames(start, duration)?)
}

pub(crate) fn split_span(
    span: Range<i64>,
    cut: i64,
) -> Result<(Range<i64>, Range<i64>), &'static str> {
    if span.start > span.end {
        return Err("Frame interval is reversed");
    }
    if cut < span.start || cut > span.end {
        return Err("Cut is outside the frame interval");
    }
    Ok((span.start..cut, cut..span.end))
}

pub(crate) fn shift_span(span: Range<i64>, delta: i64) -> Result<Range<i64>, &'static str> {
    if span.start > span.end {
        return Err("Frame interval is reversed");
    }
    Ok(translate_frames(span.start, delta)?..translate_frames(span.end, delta)?)
}

pub(crate) fn clamp_to_span(value: i64, span: Range<i64>) -> Result<i64, &'static str> {
    if span.start >= span.end {
        return Err("Cannot clamp to an empty frame interval");
    }
    Ok(value.clamp(span.start, span.end - 1))
}
