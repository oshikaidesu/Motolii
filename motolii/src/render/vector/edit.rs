
use crate::render::vector::geom::{Contour, Path, Point, Vertex};
use crate::render::vector::ops;
#[cfg(test)]
use crate::render::vector::geom::bezier_point;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum PathEditError {
    #[error("contour index {index} is out of range (path has {len} contours)")]
    ContourOutOfRange { index: usize, len: usize },
    #[error("vertex index {index} is out of range (contour has {len} vertices)")]
    VertexOutOfRange { index: usize, len: usize },
    #[error("edge index {index} is out of range (contour has {len} edges)")]
    EdgeOutOfRange { index: usize, len: usize },
    #[error("split parameter t={0} must be in the open interval (0, 1)")]
    SplitParameterOutOfRange(f64),
}

pub fn with_contour<F>(path: &Path, contour: usize, f: F) -> Result<Path, PathEditError>
where
    F: FnOnce(&Contour) -> Result<Contour, PathEditError>,
{
    let c = path.get(contour).ok_or(PathEditError::ContourOutOfRange {
        index: contour,
        len: path.len(),
    })?;
    let updated = f(c)?;
    let mut out = path.clone();
    out[contour] = updated;
    Ok(out)
}

pub fn insert_vertex(contour: &Contour, at: usize, vertex: Vertex) -> Result<Contour, PathEditError> {
    let len = contour.vertices.len();
    if at > len {
        return Err(PathEditError::VertexOutOfRange { index: at, len });
    }
    let mut vertices = contour.vertices.clone();
    vertices.insert(at, vertex);
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

pub fn remove_vertex(contour: &Contour, at: usize) -> Result<Contour, PathEditError> {
    let len = contour.vertices.len();
    if at >= len {
        return Err(PathEditError::VertexOutOfRange { index: at, len });
    }
    let mut vertices = contour.vertices.clone();
    vertices.remove(at);
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

pub fn move_vertex(contour: &Contour, at: usize, point: Point) -> Result<Contour, PathEditError> {
    let len = contour.vertices.len();
    if at >= len {
        return Err(PathEditError::VertexOutOfRange { index: at, len });
    }
    let mut vertices = contour.vertices.clone();
    vertices[at].point = point;
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

pub fn set_handles(
    contour: &Contour,
    at: usize,
    in_tangent: Point,
    out_tangent: Point,
) -> Result<Contour, PathEditError> {
    let len = contour.vertices.len();
    if at >= len {
        return Err(PathEditError::VertexOutOfRange { index: at, len });
    }
    let mut vertices = contour.vertices.clone();
    vertices[at].in_tangent = in_tangent;
    vertices[at].out_tangent = out_tangent;
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

pub fn close_path(contour: &Contour) -> Contour {
    Contour {
        vertices: contour.vertices.clone(),
        closed: true,
    }
}

pub fn open_path(contour: &Contour) -> Contour {
    Contour {
        vertices: contour.vertices.clone(),
        closed: false,
    }
}

pub fn split_segment(contour: &Contour, edge: usize, t: f64) -> Result<Contour, PathEditError> {
    if !(t > 0.0 && t < 1.0) {
        return Err(PathEditError::SplitParameterOutOfRange(t));
    }
    let n = contour.vertices.len();
    let edge_count = if n < 2 {
        0
    } else if contour.closed {
        n
    } else {
        n - 1
    };
    if edge >= edge_count {
        return Err(PathEditError::EdgeOutOfRange { index: edge, len: edge_count });
    }
    let next = (edge + 1) % n;
    let v0 = contour.vertices[edge];
    let v1 = contour.vertices[next];
    let (head, mid, tail) = ops::split_bezier(&v0, &v1, t);

    let mut vertices = contour.vertices.clone();
    vertices[edge] = head;
    vertices[next] = tail;
    vertices.insert(edge + 1, mid);
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

pub type FlatVertex = ([f64; 2], [f64; 2], [f64; 2]);

pub fn contour_to_flat(contour: &Contour) -> (Vec<FlatVertex>, bool) {
    let vertices = contour
        .vertices
        .iter()
        .map(|v| {
            (
                [v.point.x, v.point.y],
                [v.in_tangent.x, v.in_tangent.y],
                [v.out_tangent.x, v.out_tangent.y],
            )
        })
        .collect();
    (vertices, contour.closed)
}

pub fn contour_from_flat(vertices: &[FlatVertex], closed: bool) -> Contour {
    let vertices = vertices
        .iter()
        .map(|&(point, in_tangent, out_tangent)| Vertex {
            point: Point {
                x: point[0],
                y: point[1],
            },
            in_tangent: Point {
                x: in_tangent[0],
                y: in_tangent[1],
            },
            out_tangent: Point {
                x: out_tangent[0],
                y: out_tangent[1],
            },
        })
        .collect();
    Contour { vertices, closed }
}
