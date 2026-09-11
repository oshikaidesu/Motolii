//! Trim Paths — 輪郭を長さで切り出す。`chop` もこの窓の切り出しを使う。
use crate::doc::vector::geom::{
    is_straight, lerp_point, segment_sample_lengths, t_at_length, Contour, Path, Point, Vertex,
};
use crate::doc::vector::TrimMultiple;

pub(crate) fn split_bezier(v0: &Vertex, v1: &Vertex, t: f64) -> (Vertex, Vertex, Vertex) {
    if is_straight(v0, v1) {
        let m = lerp_point(v0.point, v1.point, t);
        return (
            Vertex {
                point: v0.point,
                in_tangent: v0.in_tangent,
                out_tangent: Point::ZERO,
            },
            Vertex {
                point: m,
                in_tangent: Point::ZERO,
                out_tangent: Point::ZERO,
            },
            Vertex {
                point: v1.point,
                in_tangent: Point::ZERO,
                out_tangent: v1.out_tangent,
            },
        );
    }
    let p0 = v0.point;
    let p1 = v0.point.add(v0.out_tangent);
    let p2 = v1.point.add(v1.in_tangent);
    let p3 = v1.point;
    let a = lerp_point(p0, p1, t);
    let b = lerp_point(p1, p2, t);
    let cc = lerp_point(p2, p3, t);
    let d = lerp_point(a, b, t);
    let e = lerp_point(b, cc, t);
    let m = lerp_point(d, e, t);
    (
        Vertex {
            point: p0,
            in_tangent: v0.in_tangent,
            out_tangent: a.sub(p0),
        },
        Vertex {
            point: m,
            in_tangent: d.sub(m),
            out_tangent: e.sub(m),
        },
        Vertex {
            point: p3,
            in_tangent: cc.sub(p3),
            out_tangent: v1.out_tangent,
        },
    )
}

fn sub_bezier(v0: &Vertex, v1: &Vertex, t0: f64, t1: f64) -> (Vertex, Vertex) {
    if t0 <= 0.0 && t1 >= 1.0 {
        return (*v0, *v1);
    }
    let (_, tail_start, tail_end) = split_bezier(v0, v1, t0.max(0.0));
    if t1 >= 1.0 {
        return (tail_start, tail_end);
    }
    let denom = (1.0 - t0).max(f64::EPSILON);
    let local_t1 = ((t1 - t0) / denom).clamp(0.0, 1.0);
    let (head_start, head_end, _) = split_bezier(&tail_start, &tail_end, local_t1);
    (head_start, head_end)
}

pub(crate) struct FlatSegment {
    pub(crate) contour_idx: usize,
    pub(crate) v0: Vertex,
    pub(crate) v1: Vertex,
    pub(crate) len: f64,
}

fn contour_segments(c: &Contour) -> Vec<(Vertex, Vertex)> {
    let n = c.vertices.len();
    let m = if c.closed { n } else { n.saturating_sub(1) };
    (0..m)
        .map(|i| (c.vertices[i], c.vertices[(i + 1) % n]))
        .collect()
}

pub(crate) fn flatten_segments(contours: &[Contour]) -> Vec<FlatSegment> {
    let mut out = Vec::new();
    for (ci, c) in contours.iter().enumerate() {
        if c.vertices.len() <= 1 {
            continue;
        }
        for (v0, v1) in contour_segments(c) {
            let (_, len) = segment_sample_lengths(&v0, &v1);
            out.push(FlatSegment {
                contour_idx: ci,
                v0,
                v1,
                len,
            });
        }
    }
    out
}

fn wrap01(x: f64) -> f64 {
    let mut r = x % 1.0;
    if r < 0.0 {
        r += 1.0;
    }
    r
}

fn resolve_windows(start: f64, end: f64, offset: f64) -> Vec<(f64, f64)> {
    let s = start + offset;
    let mut e = end + offset;
    if e < s {
        e += 1.0;
    }
    let coverage = (e - s).clamp(0.0, 1.0);
    if coverage <= f64::EPSILON {
        return Vec::new();
    }
    let s_wrapped = wrap01(s);
    let e_pos = s_wrapped + coverage;
    if e_pos <= 1.0 + 1e-9 {
        vec![(s_wrapped, e_pos.min(1.0))]
    } else {
        vec![(s_wrapped, 1.0), (0.0, e_pos - 1.0)]
    }
}

pub(crate) fn extract_window(segments: &[FlatSegment], from: f64, to: f64) -> Vec<Contour> {
    if to - from <= f64::EPSILON {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current: Vec<Vertex> = Vec::new();
    let mut current_contour: Option<usize> = None;
    let mut acc = 0.0;
    for seg in segments {
        let seg_start = acc;
        let seg_end = acc + seg.len;
        acc = seg_end;

        let overlaps = seg_end > from + 1e-12 && seg_start < to - 1e-12;
        let boundary_break = current_contour.is_some_and(|idx| idx != seg.contour_idx);
        if boundary_break && !current.is_empty() {
            result.push(Contour {
                vertices: std::mem::take(&mut current),
                closed: false,
            });
        }
        if !overlaps {
            if !current.is_empty() {
                result.push(Contour {
                    vertices: std::mem::take(&mut current),
                    closed: false,
                });
            }
            current_contour = None;
            continue;
        }
        current_contour = Some(seg.contour_idx);

        let (cum, seg_total) = segment_sample_lengths(&seg.v0, &seg.v1);
        let local_from = (from - seg_start).max(0.0);
        let local_to = (to - seg_start).min(seg.len);
        let t0 = if local_from <= f64::EPSILON {
            0.0
        } else {
            t_at_length(&cum, seg_total, local_from)
        };
        let t1 = if local_to >= seg.len - f64::EPSILON {
            1.0
        } else {
            t_at_length(&cum, seg_total, local_to)
        };
        let (sv, ev) = sub_bezier(&seg.v0, &seg.v1, t0, t1);
        if current.is_empty() {
            current.push(sv);
        }
        current.push(ev);
    }
    if !current.is_empty() {
        result.push(Contour {
            vertices: current,
            closed: false,
        });
    }
    result
}

pub(crate) fn trim(path: &Path, start: f64, end: f64, offset: f64, multiple: TrimMultiple) -> Path {
    let windows = resolve_windows(start, end, offset);
    if windows.is_empty() {
        return Path::new();
    }
    match multiple {
        TrimMultiple::Simultaneously => {
            let mut out = Path::new();
            for c in path {
                if c.vertices.len() <= 1 {
                    out.push(c.clone());
                    continue;
                }
                let segs = flatten_segments(std::slice::from_ref(c));
                let total: f64 = segs.iter().map(|s| s.len).sum();
                if total <= f64::EPSILON {
                    out.push(c.clone());
                    continue;
                }
                for (fs, ft) in &windows {
                    out.extend(extract_window(&segs, fs * total, ft * total));
                }
            }
            out
        }
        TrimMultiple::Individually => {
            let segs = flatten_segments(path);
            let total: f64 = segs.iter().map(|s| s.len).sum();
            if total <= f64::EPSILON {
                return path.clone();
            }
            let mut out = Path::new();
            for (fs, ft) in &windows {
                out.extend(extract_window(&segs, fs * total, ft * total));
            }
            out
        }
    }
}
