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

/// 縁の丸み(fillet)。前の蓋の縁を半径 `radius` の四分円(`chamfer` なら 1 段の面取り)にする。
/// 法線が蓋から側面へ連続して回るので、ガラスはここで下の絵を歪める。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Bevel {
    pub radius: f32,
    pub segments: u32,
    pub chamfer: bool,
}

/// 立体を作る族の読み取り結果(Extrude の奥行き + Bevel)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Solid {
    pub depth: f32,
    pub bevel: Option<Bevel>,
}

impl Solid {
    /// 立体の全長。奥行きが丸みより浅ければ丸みが奥行きになる(丸みだけの板 = レンズの縁を持つ板)。
    pub fn extent(&self) -> f32 {
        self.depth.max(self.bevel.map_or(0.0, |b| b.radius))
    }
    pub fn hash_key(&self, hasher: &mut impl std::hash::Hasher) {
        use std::hash::Hash;
        self.depth.to_bits().hash(hasher);
        if let Some(b) = self.bevel { b.radius.to_bits().hash(hasher); b.segments.hash(hasher); b.chamfer.hash(hasher); }
    }
}

/// 閉じた折れ線の重複点(閉じるための末尾の複製、連続する同じ点)を落とす。
fn dedup_closed(line: &[glam::Vec2]) -> Vec<glam::Vec2> {
    let mut out: Vec<glam::Vec2> = Vec::with_capacity(line.len());
    for p in line {
        if out.last().is_some_and(|q| q.distance_squared(*p) <= 1e-8) { continue; }
        out.push(*p);
    }
    while out.len() > 1 && out.first().unwrap().distance_squared(*out.last().unwrap()) <= 1e-8 { out.pop(); }
    out
}

/// 閉じた折れ線の各頂点の外向きの寄せ(隣り合う辺の法線の平均 × miter 長)。距離 r を掛けると、
/// 隣り合う辺を r ずつ内側へ平行移動した時の角に一致する(鋭角は 3 倍で止める)。`outward` は輪郭の巻きの向き。
fn vertex_normals(line: &[glam::Vec2], outward: f32) -> Vec<glam::Vec2> {
    let n = line.len();
    let edge_normal = |i: usize| {
        let e = line[(i + 1) % n] - line[i];
        if e.length_squared() <= f32::EPSILON { glam::Vec2::ZERO } else { glam::vec2(e.y, -e.x).normalize() * outward }
    };
    (0..n).map(|i| {
        let prev = edge_normal((i + n - 1) % n);
        let next = edge_normal(i);
        let sum = prev + next;
        if sum.length_squared() <= 1e-8 { return next; }
        let direction = sum.normalize();
        let miter = 1.0 / direction.dot(next).max(1.0 / 3.0);
        direction * miter
    }).collect()
}

/// 輪郭を z = 0 の前の蓋、z = extent の後ろの蓋、側面へ(AE と同じく前面は層の面に留まり、
/// 奥行きはカメラから遠ざかる側へ伸びる)。座標は層の local px、uv は絵(size)への割合。
/// 蓋は閉じた輪郭の fill、側面は閉じた輪郭は外向き、開いた輪郭は帯のまま。
/// Bevel があれば閉じた輪郭の前の縁を丸め、前の蓋は半径ぶん内側へ寄せる。
pub(crate) fn geometry(outlines: &[(Vec<PathContour>, PathFillRule)], size: [f32; 2], solid: Solid) -> Geometry {
    let uv = |p: glam::Vec2| glam::vec2(p.x / size[0].max(1.0), p.y / size[1].max(1.0)).clamp(glam::Vec2::ZERO, glam::Vec2::ONE);
    let extent = solid.extent();
    let bevel = solid.bevel.filter(|b| b.radius > 0.0);
    let vertex = |p: glam::Vec2| re_renderer::renderer::PathVertex { point: p, in_tangent: glam::Vec2::ZERO, out_tangent: glam::Vec2::ZERO };
    let mut g = Geometry { positions: Vec::new(), normals: Vec::new(), texcoords: Vec::new(), indices: Vec::new() };
    for (contours, rule) in outlines {
        let flattened: Vec<(Vec<glam::Vec2>, bool)> = re_renderer::renderer::flattened_contours(contours)
            .into_iter().map(|(line, closes)| (if closes { dedup_closed(&line) } else { line }, closes)).filter(|(line, _)| line.len() >= 2).collect();
        // 前の蓋: Bevel があれば折れ線を法線方向へ半径ぶん寄せた輪郭、無ければ元の輪郭。
        let front: Vec<PathContour> = match bevel {
            None => contours.iter().filter(|c| c.closed).cloned().collect(),
            Some(b) => flattened.iter().filter(|(_, closes)| *closes).map(|(line, _)| {
                let area: f32 = line.iter().zip(line.iter().cycle().skip(1)).map(|(a, c)| a.x * c.y - c.x * a.y).sum::<f32>() * 0.5;
                let outward = if area < 0.0 { -1.0 } else { 1.0 };
                let normals = vertex_normals(line, outward);
                PathContour { closed: true, vertices: line.iter().zip(&normals).map(|(p, n)| vertex(*p - *n * b.radius)).collect() }
            }).collect(),
        };
        let closed: Vec<PathContour> = contours.iter().filter(|c| c.closed).cloned().collect();
        for (z, normal, flip, source) in [(0.0, -glam::Vec3::Z, false, &front), (extent, glam::Vec3::Z, true, &closed)] {
            let (cap, cap_indices) = re_renderer::renderer::fill_triangles(source, *rule);
            let base = g.positions.len() as u32;
            g.positions.extend(cap.iter().map(|p| p.extend(z)));
            g.normals.extend(std::iter::repeat(normal).take(cap.len()));
            g.texcoords.extend(cap.iter().map(|p| uv(*p)));
            g.indices.extend(cap_indices.chunks_exact(3).map(|t| {
                if flip { glam::uvec3(base + t[0], base + t[2], base + t[1]) } else { glam::uvec3(base + t[0], base + t[1], base + t[2]) }
            }));
        }
        for (line, closes) in &flattened {
            let area: f32 = line.iter().zip(line.iter().cycle().skip(1)).map(|(a, b)| a.x * b.y - b.x * a.y).sum::<f32>() * 0.5;
            let outward = if *closes && area < 0.0 { -1.0 } else { 1.0 };
            let edges = if *closes { line.len() } else { line.len() - 1 };
            // 側面の始まる深さ。丸みがあれば四分円の終わり(z = radius)から。
            let wall_from = match bevel { Some(b) if *closes => b.radius.min(extent), _ => 0.0 };
            if let (Some(b), true) = (bevel, *closes) {
                let normals = vertex_normals(line, outward);
                let steps = if b.chamfer { 1 } else { b.segments.max(1) };
                // 四分円: 中心 c = (p − n·r, z = r)。位置 = c + r(sin θ·n − cos θ·ẑ)、法線 = (sin θ·n, −cos θ)。
                let ring = |p: glam::Vec2, n: glam::Vec2, k: u32| -> (glam::Vec3, glam::Vec3) {
                    let theta = std::f32::consts::FRAC_PI_2 * k as f32 / steps as f32;
                    let (sin, cos) = theta.sin_cos();
                    let position = (p - n * b.radius * (1.0 - sin)).extend(b.radius * (1.0 - cos));
                    let normal = (n.normalize_or_zero() * sin).extend(-cos).normalize();
                    (position, normal)
                };
                for i in 0..edges {
                    let (ia, ib) = (i, (i + 1) % line.len());
                    let (a, b2) = (line[ia], line[ib]);
                    if (b2 - a).length_squared() <= f32::EPSILON { continue; }
                    let (ua, ub) = (uv(a - normals[ia].normalize_or_zero() * WALL_INSET), uv(b2 - normals[ib].normalize_or_zero() * WALL_INSET));
                    for k in 0..steps {
                        let (a0, na0) = ring(a, normals[ia], k);
                        let (a1, na1) = ring(a, normals[ia], k + 1);
                        let (b0, nb0) = ring(b2, normals[ib], k);
                        let (b1, nb1) = ring(b2, normals[ib], k + 1);
                        let base = g.positions.len() as u32;
                        g.positions.extend([a0, b0, b1, a1]);
                        g.normals.extend([na0, nb0, nb1, na1]);
                        g.texcoords.extend([ua, ub, ub, ua]);
                        g.indices.extend([glam::uvec3(base, base + 1, base + 2), glam::uvec3(base, base + 2, base + 3)]);
                    }
                }
            }
            if wall_from >= extent { continue; }
            for i in 0..edges {
                let (a, b) = (line[i], line[(i + 1) % line.len()]);
                let e = b - a;
                if e.length_squared() <= f32::EPSILON { continue; }
                let n = glam::vec2(e.y, -e.x).normalize() * outward;
                let base = g.positions.len() as u32;
                g.positions.extend([a.extend(wall_from), b.extend(wall_from), b.extend(extent), a.extend(extent)]);
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
        solid: Solid,
    ) -> Result<Option<GpuModelData>, CompositorError> {
        let depth = solid.extent();
        let g = geometry(outlines, size, solid);
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
                field_anchor: false,
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
        let g = geometry(&[square(10.0)], [10.0, 10.0], Solid { depth: 4.0, bevel: None });
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
        assert!(geometry(&[], [10.0, 10.0], Solid { depth: 4.0, bevel: None }).indices.is_empty());
    }

    /// 丸みを付けると、前の蓋は半径ぶん内側へ寄り、縁は段数ぶんの帯になり、法線が蓋(−z)から側面へ連続して回る。
    #[test]
    fn a_bevel_rounds_the_front_rim_with_continuous_normals() {
        let g = geometry(&[square(20.0)], [20.0, 20.0], Solid { depth: 10.0, bevel: Some(Bevel { radius: 4.0, segments: 4, chamfer: false }) });
        // 前の蓋の 4 隅は (4,4)…(16,16) に寄る
        let front: Vec<_> = g.positions.iter().zip(&g.normals).filter(|(p, n)| p.z == 0.0 && **n == -glam::Vec3::Z).map(|(p, _)| p.truncate()).collect();
        assert!(front.iter().all(|p| (4.0 - 1e-3..=16.0 + 1e-3).contains(&p.x) && (4.0 - 1e-3..=16.0 + 1e-3).contains(&p.y)), "{front:?}");
        // 帯: 4 辺 × 4 段 × 4 頂点 = 64 頂点。最初の段の法線はほぼ −z、最後は側面の法線に近い
        let rim: Vec<_> = g.positions.iter().zip(&g.normals).filter(|(p, _)| p.z > 0.0 && p.z < 4.0 + 1e-3).collect();
        assert!(!rim.is_empty());
        assert!(g.normals.iter().all(|n| (n.length() - 1.0).abs() < 1e-3));
        let first_ring = g.positions.iter().zip(&g.normals).filter(|(p, _)| p.z == 0.0 && p.x >= 4.0 - 1e-3 && p.x <= 16.0 + 1e-3).count();
        assert!(first_ring > 0);
        assert!(g.positions.iter().all(|p| p.z >= 0.0 && p.z <= 10.0 + 1e-4));
        // 側面は z = 4 から 10
        assert!(g.positions.iter().any(|p| (p.z - 4.0).abs() < 1e-4) && g.positions.iter().any(|p| (p.z - 10.0).abs() < 1e-4));
        // 奥行き 0 でも丸みだけなら全長 = 半径
        assert_eq!(Solid { depth: 0.0, bevel: Some(Bevel { radius: 6.0, segments: 2, chamfer: true }) }.extent(), 6.0);
    }

    /// 効果の Extrude + Bevel でも(Depth 属性無しで)立体になり、傾けると側面が見える。
    #[test]
    fn extrude_and_bevel_effects_make_a_solid_without_the_depth_attribute() {
        use crate::doc::store::{EffectId, EffectInstance};
        let mut engine = crate::render::engine::Engine::new().unwrap();
        let t = RationalTime::ZERO;
        let mut doc = rectangle_document(0.0, 35.0);
        let flat = engine.render_frame(&doc.view(), t).unwrap();
        doc.apply_all([
            Intent::SetEffects { layer: LayerId(1), effects: vec![
                EffectInstance { id: EffectId(0), plugin_id: crate::doc::store::solid::EXTRUDE.into() },
                EffectInstance { id: EffectId(1), plugin_id: crate::doc::store::solid::BEVEL.into() },
            ] },
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(0), "depth").unwrap(), value: Value::F64(24.0) },
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(1), "radius").unwrap(), value: Value::F64(6.0) },
        ]).unwrap();
        let solid = engine.render_frame(&doc.view(), t).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert!(lit(&solid) > lit(&flat) + 40, "側面と縁が見える: {} vs {}", lit(&solid), lit(&flat));
        assert!(matches!(&engine.extrusions[&LayerId(1)].1.bounds.max, [_, _, z] if (*z - 24.0).abs() < 1e-3), "全長は奥行き");
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
