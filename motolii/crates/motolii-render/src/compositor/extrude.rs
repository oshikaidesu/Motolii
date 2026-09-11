//! 板の押し出し。輪郭(形・文字は fill の輪郭、画は矩形)を前後の蓋と側面の網にし、
//! 層の絵をそのまま貼る。網になるので scale.z・表面効果・遮蔽は 3D 素材と同じ道を通る。
use re_renderer::mesh::{CpuMesh, Material};
use re_renderer::renderer::{PathContour, PathFillRule};
use re_renderer::{CpuModel, Rgba32Unmul};

use crate::render::compositor::{Compositor, CompositorError, GpuModelData, GpuTexture2D};
use crate::render::media::SpatialBounds;

/// 側面の色は輪郭のこの内側(px)を貼る。輪郭の上は AA で半透明なので、1 px 内へ寄せる。
const WALL_INSET: f32 = 1.0;

pub(crate) struct Geometry {
    pub positions: Vec<glam::Vec3>,
    pub normals: Vec<glam::Vec3>,
    pub texcoords: Vec<glam::Vec2>,
    pub indices: Vec<glam::UVec3>,
}

/// 輪郭を z = 0 の前の蓋、z = depth の後ろの蓋、側面へ(AE と同じく前面は層の面に留まり、
/// 奥行きはカメラから遠ざかる側へ伸びる)。座標は層の local px、uv は絵(size)への割合。
/// 蓋は閉じた輪郭の fill、側面は閉じた輪郭は外向き、開いた輪郭は帯のまま。
pub(crate) fn geometry(outlines: &[(Vec<PathContour>, PathFillRule)], size: [f32; 2], depth: f32) -> Geometry {
    let uv = |p: glam::Vec2| glam::vec2(p.x / size[0].max(1.0), p.y / size[1].max(1.0)).clamp(glam::Vec2::ZERO, glam::Vec2::ONE);
    let mut g = Geometry { positions: Vec::new(), normals: Vec::new(), texcoords: Vec::new(), indices: Vec::new() };
    for (contours, rule) in outlines {
        let closed: Vec<PathContour> = contours.iter().filter(|c| c.closed).cloned().collect();
        let (cap, cap_indices) = re_renderer::renderer::fill_triangles(&closed, *rule);
        // 前の蓋はカメラ側(-z)、後ろは +z。後ろは巻きを返す。
        for (z, normal, flip) in [(0.0, -glam::Vec3::Z, false), (depth, glam::Vec3::Z, true)] {
            let base = g.positions.len() as u32;
            g.positions.extend(cap.iter().map(|p| p.extend(z)));
            g.normals.extend(std::iter::repeat(normal).take(cap.len()));
            g.texcoords.extend(cap.iter().map(|p| uv(*p)));
            g.indices.extend(cap_indices.chunks_exact(3).map(|t| {
                if flip { glam::uvec3(base + t[0], base + t[2], base + t[1]) } else { glam::uvec3(base + t[0], base + t[1], base + t[2]) }
            }));
        }
        for (line, closes) in re_renderer::renderer::flattened_contours(contours) {
            let area: f32 = line.iter().zip(line.iter().cycle().skip(1)).map(|(a, b)| a.x * b.y - b.x * a.y).sum::<f32>() * 0.5;
            let outward = if closes && area < 0.0 { -1.0 } else { 1.0 };
            let edges = if closes { line.len() } else { line.len() - 1 };
            for i in 0..edges {
                let (a, b) = (line[i], line[(i + 1) % line.len()]);
                let e = b - a;
                if e.length_squared() <= f32::EPSILON { continue; }
                let n = glam::vec2(e.y, -e.x).normalize() * outward;
                let base = g.positions.len() as u32;
                g.positions.extend([a.extend(0.0), b.extend(0.0), b.extend(depth), a.extend(depth)]);
                g.normals.extend(std::iter::repeat(n.extend(0.0)).take(4));
                let (ua, ub) = (uv(a - n * WALL_INSET), uv(b - n * WALL_INSET));
                g.texcoords.extend([ua, ub, ub, ua]);
                g.indices.extend([glam::uvec3(base, base + 1, base + 2), glam::uvec3(base, base + 2, base + 3)]);
            }
        }
    }
    g
}

impl Compositor {
    /// 押し出した網を GPU へ。bounds は xy の始点を 0、z を ±depth に固定し、置く時に local 座標を
    /// そのまま使う(`spatial_placement_from_bounds` は bounds.min の xy と z の中心を原点へ引く)。
    pub(crate) fn extrude_model(
        &mut self,
        outlines: &[(Vec<PathContour>, PathFillRule)],
        texture: GpuTexture2D,
        size: [f32; 2],
        depth: f32,
    ) -> Result<Option<GpuModelData>, CompositorError> {
        let g = geometry(outlines, size, depth);
        if g.indices.is_empty() {
            return Ok(None);
        }
        let max = g.positions.iter().fold(glam::Vec3::splat(f32::MIN), |m, p| m.max(*p));
        let bbox = macaw::BoundingBox::from_points(g.positions.iter().copied());
        let count = g.positions.len();
        let index_count = g.indices.len() as u32 * 3;
        let mesh = CpuMesh {
            label: "extruded layer".into(),
            triangle_indices: g.indices,
            vertex_positions: g.positions.clone(),
            vertex_colors: vec![Rgba32Unmul::WHITE; count],
            vertex_normals: g.normals,
            vertex_texcoords: g.texcoords,
            materials: smallvec::smallvec![Material {
                albedo_is_premultiplied: true,
                label: "layer picture".into(),
                index_range: 0..index_count,
                albedo: texture,
                albedo_factor: re_renderer::Rgba::WHITE,
            }],
            bbox,
        };
        let instances = CpuModel::from_single_mesh(mesh)
            .into_gpu_meshes(&self.ctx)
            .map_err(|error| CompositorError::Draw(error.to_string()))?;
        Ok(Some(GpuModelData {
            planar_size: None,
            revision: super::mesh::next_model_revision(),
            instances: std::sync::Arc::new(instances),
            bounds: SpatialBounds { min: [0.0, 0.0, -depth], max: [max.x, max.y, depth] },
            vertices: std::sync::Arc::new(crate::render::media::silhouette_points(g.positions)),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{property, Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerProjection, LayerSource, LayerTiming, PropertyId, RationalTime, ShapeNode, Value};
    use crate::doc::vector::{Brush, Fill, FillRule, PathSource, Point, Rgb, Shape};

    fn square(size: f32) -> (Vec<PathContour>, PathFillRule) {
        let v = |x: f32, y: f32| re_renderer::renderer::PathVertex { point: glam::vec2(x, y), in_tangent: glam::Vec2::ZERO, out_tangent: glam::Vec2::ZERO };
        (vec![PathContour { closed: true, vertices: vec![v(0.0, 0.0), v(size, 0.0), v(size, size), v(0.0, size)] }], PathFillRule::NonZero)
    }

    /// 正方形は蓋 2 枚(2 三角形ずつ)と側面 4 枚(2 三角形ずつ)。蓋は ±z、側面の法線は外向きで z=0。
    #[test]
    fn a_square_extrudes_to_two_caps_and_four_walls() {
        let g = geometry(&[square(10.0)], [10.0, 10.0], 4.0);
        assert_eq!(g.indices.len(), 4 + 8);
        assert_eq!(g.positions.len(), 8 + 16);
        assert!(g.positions.iter().all(|p| p.z == 0.0 || p.z == 4.0));
        let caps = &g.normals[..8];
        assert!(caps[..4].iter().all(|n| *n == -glam::Vec3::Z) && caps[4..].iter().all(|n| *n == glam::Vec3::Z));
        for wall in g.normals[8..].chunks_exact(4) {
            assert_eq!(wall[0].z, 0.0);
            let centre = g.positions[8..].iter().map(|p| p.truncate()).sum::<glam::Vec2>() / 16.0;
            let at = g.positions[8 + g.normals[8..].iter().position(|n| n == &wall[0]).unwrap()].truncate();
            assert!((at - centre).dot(wall[0].truncate()) > 0.0, "外向き: {wall:?}");
        }
        assert!(g.texcoords.iter().all(|uv| (0.0..=1.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y)));
        assert!(geometry(&[], [10.0, 10.0], 4.0).indices.is_empty());
    }

    fn rectangle_document(depth: f64, tilt_y: f64) -> Document {
        let fps = Fps::try_new(30, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 96, height: 96, fps, duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        let put = |name: &str, value: Value| Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value };
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoPointFiveD), ..Default::default() } },
            Intent::SetShapes { layer, shapes: vec![ShapeNode::Leaf(Shape {
                source: PathSource::Rectangle { size: Point { x: 32.0, y: 32.0 } },
                ops: Vec::new(),
                fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), rule: FillRule::NonZero, opacity: 1.0, hidden: false }),
                stroke: None,
            })] },
            put(property::POSITION, Value::Vec2([48.0, 48.0])),
            put(property::ANCHOR, Value::Vec2([17.0, 17.0])),
            put(property::DEPTH, Value::F64(depth)),
            put(property::ROTATION_Y, Value::F64(tilt_y)),
        ]).unwrap();
        doc
    }

    fn lit(rgba: &[u8]) -> usize {
        rgba.chunks_exact(4).filter(|p| p[0] > 24).count()
    }

    /// 奥行きを持つ矩形は正面からは板と同じ範囲を占め(網なので照明で少し暗い)、
    /// 横へ傾けると側面が見えて板より広く光る。
    /// 絵と奥行きが変わらない間は網を作り直さない。
    #[test]
    fn a_deep_rectangle_shows_its_side_when_tilted() {
        let t = RationalTime::ZERO;
        let mut engine = crate::render::engine::Engine::new().unwrap();
        let flat = engine.render_frame(&rectangle_document(0.0, 0.0).view(), t).unwrap();
        let deep_doc = rectangle_document(24.0, 0.0);
        assert_eq!(deep_doc.view().resolved_layers(t).unwrap()[0].depth, 24.0);
        let deep = engine.render_frame(&deep_doc.view(), t).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        if let Ok(dir) = std::env::var("MOTOLII_EXTRUDE_DUMP") {
            for (name, px) in [("flat", &flat), ("deep", &deep)] {
                image::RgbaImage::from_raw(96, 96, px.clone()).unwrap().save(format!("{dir}/{name}.png")).unwrap();
            }
        }
        let same = flat.chunks_exact(4).zip(deep.chunks_exact(4)).filter(|(a, b)| (a[0] > 24) == (b[0] > 24)).count();
        assert!(same as f32 / (96.0 * 96.0) > 0.99, "正面は板と同じ範囲: {same}");
        let first = engine.extrusions[&LayerId(1)].1.clone();
        engine.render_frame(&deep_doc.view(), t).unwrap();
        assert!(std::sync::Arc::ptr_eq(&first, &engine.extrusions[&LayerId(1)].1), "同じ絵と奥行きなら作り直さない");

        let flat_tilted = engine.render_frame(&rectangle_document(0.0, 70.0).view(), t).unwrap();
        let deep_tilted = engine.render_frame(&rectangle_document(24.0, 70.0).view(), t).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        if let Ok(dir) = std::env::var("MOTOLII_EXTRUDE_DUMP") {
            for (name, px) in [("flat_tilted", &flat_tilted), ("deep_tilted", &deep_tilted)] {
                image::RgbaImage::from_raw(96, 96, px.clone()).unwrap().save(format!("{dir}/{name}.png")).unwrap();
            }
        }
        assert!(lit(&deep_tilted) > lit(&flat_tilted) + 40, "側面が見える: {} vs {}", lit(&deep_tilted), lit(&flat_tilted));
    }
}
