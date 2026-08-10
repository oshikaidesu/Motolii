use motolii_doc::pathgeom::{Contour, Path, Point, Vertex};

const CIRCLE_KAPPA: f64 = 0.552_284_749_830_793_6;
const CUBIC_STEPS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeRecipe {
    Rect { center: Point, size: Point },
    Circle { center: Point, radius: f64 },
}

impl ShapeRecipe {
    pub fn lower(self) -> Path {
        match self {
            Self::Rect { center, size } => {
                let half = Point {
                    x: size.x * 0.5,
                    y: size.y * 0.5,
                };
                Path {
                    contours: vec![Contour::closed([
                        Point {
                            x: center.x - half.x,
                            y: center.y - half.y,
                        },
                        Point {
                            x: center.x + half.x,
                            y: center.y - half.y,
                        },
                        Point {
                            x: center.x + half.x,
                            y: center.y + half.y,
                        },
                        Point {
                            x: center.x - half.x,
                            y: center.y + half.y,
                        },
                    ])],
                }
            }
            Self::Circle { center, radius } => {
                let k = radius * CIRCLE_KAPPA;
                Path {
                    contours: vec![Contour {
                        closed: true,
                        vertices: vec![
                            Vertex {
                                point: Point {
                                    x: center.x + radius,
                                    y: center.y,
                                },
                                in_tangent: Point { x: 0.0, y: -k },
                                out_tangent: Point { x: 0.0, y: k },
                            },
                            Vertex {
                                point: Point {
                                    x: center.x,
                                    y: center.y + radius,
                                },
                                in_tangent: Point { x: k, y: 0.0 },
                                out_tangent: Point { x: -k, y: 0.0 },
                            },
                            Vertex {
                                point: Point {
                                    x: center.x - radius,
                                    y: center.y,
                                },
                                in_tangent: Point { x: 0.0, y: k },
                                out_tangent: Point { x: 0.0, y: -k },
                            },
                            Vertex {
                                point: Point {
                                    x: center.x,
                                    y: center.y - radius,
                                },
                                in_tangent: Point { x: -k, y: 0.0 },
                                out_tangent: Point { x: k, y: 0.0 },
                            },
                        ],
                    }],
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FillContribution {
    pub path: Path,
    pub paint: PlanarPaint,
    pub draw_order: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanarPaint {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub start_color: [f32; 4],
    pub end_color: [f32; 4],
}

impl PlanarPaint {
    pub fn solid(color: [f32; 4]) -> Self {
        Self {
            start: [0.0, 0.0],
            end: [1.0, 0.0],
            start_color: color,
            end_color: color,
        }
    }

    #[cfg(test)]
    fn sample_premultiplied(self, position: [f32; 2], coverage: f32) -> [f32; 4] {
        let direction = [self.end[0] - self.start[0], self.end[1] - self.start[1]];
        let length_squared = direction[0] * direction[0] + direction[1] * direction[1];
        let t = if length_squared > f32::EPSILON {
            (((position[0] - self.start[0]) * direction[0]
                + (position[1] - self.start[1]) * direction[1])
                / length_squared)
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
        let premultiply = |color: [f32; 4]| {
            [
                color[0] * color[3],
                color[1] * color[3],
                color[2] * color[3],
                color[3],
            ]
        };
        let start = premultiply(self.start_color);
        let end = premultiply(self.end_color);
        let coverage = coverage.clamp(0.0, 1.0);
        std::array::from_fn(|channel| {
            (start[channel] + (end[channel] - start[channel]) * t) * coverage
        })
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ContributionError {
    #[error("Path2D probe accepts exactly one closed contour")]
    UnsupportedContour,
    #[error("Path2D probe contribution is truncated")]
    Truncated,
    #[error("Path2D probe contribution has an unsupported version")]
    UnsupportedVersion,
}

impl FillContribution {
    pub fn encode(&self) -> Result<Vec<u8>, ContributionError> {
        let [contour] = self.path.contours.as_slice() else {
            return Err(ContributionError::UnsupportedContour);
        };
        if !contour.closed || contour.vertices.len() < 3 {
            return Err(ContributionError::UnsupportedContour);
        }

        let mut bytes = Vec::with_capacity(64 + contour.vertices.len() * 48);
        bytes.extend_from_slice(b"M2DP");
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        bytes.extend_from_slice(&self.draw_order.to_le_bytes());
        for value in [
            self.paint.start[0],
            self.paint.start[1],
            self.paint.end[0],
            self.paint.end[1],
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for color in [self.paint.start_color, self.paint.end_color] {
            for channel in color {
                bytes.extend_from_slice(&channel.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&(contour.vertices.len() as u32).to_le_bytes());
        for vertex in &contour.vertices {
            for value in [
                vertex.point.x,
                vertex.point.y,
                vertex.in_tangent.x,
                vertex.in_tangent.y,
                vertex.out_tangent.x,
                vertex.out_tangent.y,
            ] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ContributionError> {
        let mut reader = Reader::new(bytes);
        if reader.take(4)? != b"M2DP" || reader.u32()? != 2 {
            return Err(ContributionError::UnsupportedVersion);
        }
        let draw_order = reader.f32()?;
        let paint = PlanarPaint {
            start: [reader.f32()?, reader.f32()?],
            end: [reader.f32()?, reader.f32()?],
            start_color: [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?],
            end_color: [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?],
        };
        let count = reader.u32()? as usize;
        if count < 3 {
            return Err(ContributionError::UnsupportedContour);
        }
        let mut vertices = Vec::with_capacity(count);
        for _ in 0..count {
            vertices.push(Vertex {
                point: Point {
                    x: reader.f64()?,
                    y: reader.f64()?,
                },
                in_tangent: Point {
                    x: reader.f64()?,
                    y: reader.f64()?,
                },
                out_tangent: Point {
                    x: reader.f64()?,
                    y: reader.f64()?,
                },
            });
        }
        if !reader.remaining().is_empty() {
            return Err(ContributionError::UnsupportedVersion);
        }
        Ok(Self {
            path: Path {
                contours: vec![Contour {
                    vertices,
                    closed: true,
                }],
            },
            paint,
            draw_order,
        })
    }

    pub fn tessellate_convex(&self) -> Result<TriangleMesh, ContributionError> {
        let [contour] = self.path.contours.as_slice() else {
            return Err(ContributionError::UnsupportedContour);
        };
        if !contour.closed || contour.vertices.len() < 3 {
            return Err(ContributionError::UnsupportedContour);
        }

        let mut vertices = sample_outline(&self.path)?;
        vertices.pop(); // 閉路終点を残すと最後のfanが退化三角形になる。

        // ponytail: the first proof only needs convex Rect/Circle fills; replace this fan with
        // the adopted Vello tessellator when concave paths or holes enter the completion contract.
        let indices = (1..vertices.len() - 1)
            .flat_map(|index| [0_u32, index as u32, index as u32 + 1])
            .collect();
        Ok(TriangleMesh { vertices, indices })
    }
}

/// Rerunの既存LineStrips2DでPathOp結果を観察するため、1閉輪郭を折れ線化する。
pub fn sample_outline(path: &Path) -> Result<Vec<[f32; 2]>, ContributionError> {
    let [contour] = path.contours.as_slice() else {
        return Err(ContributionError::UnsupportedContour);
    };
    if !contour.closed || contour.vertices.len() < 3 {
        return Err(ContributionError::UnsupportedContour);
    }

    let mut points = Vec::new();
    for index in 0..contour.vertices.len() {
        let current = contour.vertices[index];
        let next = contour.vertices[(index + 1) % contour.vertices.len()];
        if index == 0 {
            points.push([current.point.x as f32, current.point.y as f32]);
        }
        let p0 = current.point;
        let p1 = add(current.point, current.out_tangent);
        let p2 = add(next.point, next.in_tangent);
        let p3 = next.point;
        let curved = current.out_tangent != Point::ZERO || next.in_tangent != Point::ZERO;
        let steps = if curved { CUBIC_STEPS } else { 1 };
        for step in 1..=steps {
            let point = cubic(p0, p1, p2, p3, step as f64 / steps as f64);
            points.push([point.x as f32, point.y as f32]);
        }
    }
    Ok(points)
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriangleMesh {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

fn add(a: Point, b: Point) -> Point {
    Point {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}

fn cubic(p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
    let u = 1.0 - t;
    Point {
        x: u.powi(3) * p0.x
            + 3.0 * u.powi(2) * t * p1.x
            + 3.0 * u * t.powi(2) * p2.x
            + t.powi(3) * p3.x,
        y: u.powi(3) * p0.y
            + 3.0 * u.powi(2) * t * p1.y
            + 3.0 * u * t.powi(2) * p2.y
            + t.powi(3) * p3.y,
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ContributionError> {
        if self.bytes.len() < count {
            return Err(ContributionError::Truncated);
        }
        let (head, tail) = self.bytes.split_at(count);
        self.bytes = tail;
        Ok(head)
    }

    fn u32(&mut self) -> Result<u32, ContributionError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> Result<f32, ContributionError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn f64(&mut self) -> Result<f64, ContributionError> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn remaining(&self) -> &'a [u8] {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_and_circle_lower_to_closed_paths_and_roundtrip() {
        for recipe in [
            ShapeRecipe::Rect {
                center: Point::ZERO,
                size: Point { x: 1.0, y: 0.5 },
            },
            ShapeRecipe::Circle {
                center: Point { x: 0.2, y: -0.1 },
                radius: 0.4,
            },
        ] {
            let contribution = FillContribution {
                path: recipe.lower(),
                paint: PlanarPaint::solid([0.2, 0.4, 0.8, 0.6]),
                draw_order: 3.0,
            };
            let decoded = FillContribution::decode(&contribution.encode().unwrap()).unwrap();
            assert_eq!(decoded, contribution);
            let mesh = decoded.tessellate_convex().unwrap();
            assert!(!mesh.indices.is_empty());
            assert!(mesh.vertices.len() >= 4);
        }
    }

    #[test]
    fn higher_draw_order_source_over_is_the_overlap_oracle() {
        fn over(fg: [f32; 4], bg: [f32; 4]) -> [f32; 4] {
            let fg = [fg[0] * fg[3], fg[1] * fg[3], fg[2] * fg[3], fg[3]];
            let bg = [bg[0] * bg[3], bg[1] * bg[3], bg[2] * bg[3], bg[3]];
            [
                fg[0] + bg[0] * (1.0 - fg[3]),
                fg[1] + bg[1] * (1.0 - fg[3]),
                fg[2] + bg[2] * (1.0 - fg[3]),
                fg[3] + bg[3] * (1.0 - fg[3]),
            ]
        }

        let actual = over([0.1, 0.6, 1.0, 0.65], [1.0, 0.2, 0.4, 0.8]);
        let expected = [0.345, 0.446, 0.762, 0.93];
        assert!(
            actual
                .into_iter()
                .zip(expected)
                .all(|(actual, expected)| (actual - expected).abs() < 0.000_01)
        );
    }

    #[test]
    fn sampled_outline_is_closed() {
        let path = ShapeRecipe::Circle {
            center: Point::ZERO,
            radius: 0.4,
        }
        .lower();
        let points = sample_outline(&path).unwrap();
        assert_eq!(points.first(), points.last());
        assert!(points.len() > path.contours[0].vertices.len());
    }

    #[test]
    fn planar_gradient_is_clipped_by_path_coverage() {
        let paint = PlanarPaint {
            start: [-1.0, 0.0],
            end: [1.0, 0.0],
            start_color: [1.0, 0.0, 0.0, 1.0],
            end_color: [0.0, 0.0, 1.0, 1.0],
        };

        assert_eq!(paint.sample_premultiplied([0.0, 0.0], 0.0), [0.0; 4]);
        assert_eq!(
            paint.sample_premultiplied([0.0, 0.0], 1.0),
            [0.5, 0.0, 0.5, 1.0]
        );
    }
}
