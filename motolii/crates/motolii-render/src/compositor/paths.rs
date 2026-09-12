//! 形・文字の輪郭と塗りをrerunへ渡す。通常は三角形、画像が必要な境界だけtextureを受け取る。
use crate::doc::vector::{Brush, Canvas, Contour, Gradient, Point, ShapeNode};
use crate::render::compositor::{Compositor, CompositorError, GpuTexture2D};
use re_renderer::renderer::{PathContour, PathDrawDataBuilder, PathFillRule, PathLineCap, PathLineJoin, PathStroke, PathVertex};
use re_renderer::view_builder::{BlendWithBackground, OrthographicCameraMode, Projection, RenderMode, TargetConfiguration, ViewBuilder, ViewBuilderId};
use re_renderer::{Rgba, Rgba32Unmul};

fn contours(path: &[Contour], origin: Point) -> Vec<PathContour> {
    let at = |p: Point| glam::vec2((p.x + origin.x) as f32, (p.y + origin.y) as f32);
    let rel = |p: Point| glam::vec2(p.x as f32, p.y as f32);
    path.iter()
        .map(|c| PathContour {
            closed: c.closed,
            vertices: c.vertices.iter().map(|v| PathVertex { point: at(v.point), in_tangent: rel(v.in_tangent), out_tangent: rel(v.out_tangent) }).collect(),
        })
        .collect()
}

fn byte(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// 筆の色。単色は定数、gradient は点ごとに評価する(直線 2 色なら三角形の補間で厳密)。
fn paint(brush: &Brush, alpha: f64, origin: Point) -> Box<dyn Fn(glam::Vec2) -> Rgba32Unmul> {
    match brush {
        Brush::Solid(c) => {
            let color = Rgba32Unmul([byte(c.r), byte(c.g), byte(c.b), byte(alpha)]);
            Box::new(move |_| color)
        }
        Brush::Gradient(g) => {
            let g: Gradient = g.clone();
            Box::new(move |p| {
                let c = g.color_at(g.parameter(Point { x: p.x as f64 - origin.x, y: p.y as f64 - origin.y }));
                Rgba32Unmul([byte(c.r), byte(c.g), byte(c.b), byte(alpha)])
            })
        }
    }
}

/// 形の木を描き手へ積む。群は平らにし、演算(trim・角丸…)は解決済みの輪郭で渡す。
fn build_at_tolerance(shapes: &[ShapeNode], canvas: &Canvas, tolerance: f32) -> Result<PathDrawDataBuilder, CompositorError> {
    let origin = Point { x: canvas.origin_x as f64, y: canvas.origin_y as f64 };
    let mut b = PathDrawDataBuilder::default().with_tolerance(tolerance);
    for shape in crate::doc::vector::flatten(shapes).map_err(|e| CompositorError::Draw(e.to_string()))? {
        for instance in crate::doc::vector::resolve(&shape).map_err(|e| CompositorError::Draw(e.to_string()))? {
            let outline = contours(&instance.path, origin);
            if let Some(fill) = shape.fill.as_ref().filter(|f| !f.hidden) {
                let rule = match fill.rule { crate::doc::vector::FillRule::NonZero => PathFillRule::NonZero, crate::doc::vector::FillRule::EvenOdd => PathFillRule::EvenOdd };
                b.fill(&outline, rule, &*paint(&fill.brush, fill.opacity * instance.opacity, origin));
            }
            if let Some(stroke) = shape.stroke.as_ref().filter(|s| !s.hidden) {
                let s = PathStroke {
                    width: stroke.width as f32,
                    cap: match stroke.cap { crate::doc::vector::LineCap::Butt => PathLineCap::Butt, crate::doc::vector::LineCap::Round => PathLineCap::Round, crate::doc::vector::LineCap::Square => PathLineCap::Square },
                    join: match stroke.join { crate::doc::vector::LineJoin::Miter => PathLineJoin::Miter, crate::doc::vector::LineJoin::Round => PathLineJoin::Round, crate::doc::vector::LineJoin::Bevel => PathLineJoin::Bevel },
                    miter_limit: stroke.miter_limit as f32,
                    dash: stroke.dash.as_ref().map(|d| (d.pattern.iter().map(|v| *v as f32).collect(), d.offset as f32)),
                };
                b.stroke(&outline, &s, &*paint(&stroke.brush, stroke.opacity * instance.opacity, origin));
            }
        }
    }
    Ok(b)
}

/// 押し出しに渡す輪郭: 形ごとの fill の輪郭と規則。塗りの無い形も輪郭は持つ(側面だけ立つ)。
pub(crate) fn outlines(shapes: &[ShapeNode], canvas: &Canvas) -> Result<Vec<(Vec<PathContour>, PathFillRule)>, CompositorError> {
    let origin = Point { x: canvas.origin_x as f64, y: canvas.origin_y as f64 };
    let mut out = Vec::new();
    for shape in crate::doc::vector::flatten(shapes).map_err(|e| CompositorError::Draw(e.to_string()))? {
        let rule = match shape.fill.as_ref().map(|f| f.rule) {
            Some(crate::doc::vector::FillRule::EvenOdd) => PathFillRule::EvenOdd,
            _ => PathFillRule::NonZero,
        };
        for instance in crate::doc::vector::resolve(&shape).map_err(|e| CompositorError::Draw(e.to_string()))? {
            out.push((contours(&instance.path, origin), rule));
        }
    }
    Ok(out)
}

impl Compositor {
    pub(crate) fn path_model(&mut self, shapes: &[ShapeNode], canvas: &Canvas, tolerance: f32) -> Result<Option<super::GpuModelData>, CompositorError> {
        let builder = build_at_tolerance(shapes, canvas, tolerance)?;
        if builder.is_empty() { return Ok(None); }
        let mesh = builder.into_mesh(&self.ctx, "vector layer");
        let vertices = mesh.vertex_positions.clone();
        let mut instances = re_renderer::CpuModel::from_single_mesh(mesh).into_gpu_meshes(&self.ctx)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        for instance in &mut instances {
            // A painted plane uses the same ordered, non-depth-writing phase as an alpha rectangle.
            // Its fills and strokes must not occlude each other as separate volumes after a field.
            for material in &mut std::sync::Arc::make_mut(&mut instance.gpu_mesh).materials {
                material.has_transparency = true;
            }
        }
        Ok(Some(super::GpuModelData {
            revision: super::mesh::next_model_revision(),
            planar_size: Some([canvas.width as f32, canvas.height as f32]),
            bounds: crate::render::media::SpatialBounds { min: [0.0; 3], max: [canvas.width as f32, canvas.height as f32, 0.0] },
            instances: std::sync::Arc::new(instances), vertices: std::sync::Arc::new(vertices),
        }))
    }

    /// 形の木を canvas の大きさの texture に描く。何も描かなければ None。
    /// 出口は他の層と同じ premultiplied texture なので、raster 効果はそのまま掛かる。
    /// `pixel_budget`: 絵に使ってよい画素数の上限。投影の密度がそれを越える(層が comp の何倍も大きい・
    /// 極端な拡大)時は密度を落とす — 効果は素材全体に掛かる(広がりの法)が、費用は comp の定数倍で止める。
    pub(crate) fn render_paths(&mut self, label: &'static str, shapes: &[ShapeNode], canvas: &Canvas, density: f32, pixel_budget: u64) -> Result<Option<GpuTexture2D>, CompositorError> {
        let limit = self.ctx.device.limits().max_texture_dimension_2d;
        let area = (canvas.width.max(1) as f64) * (canvas.height.max(1) as f64);
        let by_budget = ((pixel_budget.max(1) as f64) / area).sqrt() as f32;
        let density = density.max(1.0).min(by_budget.max(1.0)).min(limit as f32 / canvas.width.max(canvas.height).max(1) as f32);
        let width = (canvas.width as f32 * density).ceil() as u32;
        let height = (canvas.height as f32 * density).ceil() as u32;
        let builder = build_at_tolerance(shapes, canvas, 0.05 / density)?;
        if builder.is_empty() {
            return Ok(None);
        }
        let draw_data = builder.build(&self.ctx, label);
        let texture = self.create_blend_scratch_texture(width, height);
        let mut view_builder = ViewBuilder::new_with_external_resolved(
            &self.ctx,
            TargetConfiguration {
                name: label.into(),
                render_mode: RenderMode::Deterministic,
                resolution_in_pixel: [width, height],
                view_from_world: macaw::IsoTransform::IDENTITY,
                projection_from_view: Projection::Orthographic {
                    camera_mode: OrthographicCameraMode::TopLeftCornerAndExtendZ,
                    vertical_world_size: canvas.height as f32,
                    far_plane_distance: 1000.0,
                },
                pixels_per_point: 1.0,
                blend_with_background: BlendWithBackground::Premultiplied,
                ..Default::default()
            },
            ViewBuilderId::new(self.next_readback),
            &texture,
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;
        view_builder.queue_draw(&self.ctx, draw_data);
        let command_buffer = view_builder.draw(&self.ctx, Rgba::TRANSPARENT).map_err(|e| CompositorError::Draw(e.to_string()))?;
        self.pending.push(command_buffer);
        self.next_effect_key += 1;
        let imported = self
            .ctx
            .texture_manager_2d
            .import_gpu_premultiplied(self.next_effect_key, &self.ctx, &texture)
            .map_err(|e| CompositorError::Effect(e.to_string()))?;
        Ok(Some(imported))
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::store::{Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, RationalTime, ShapeNode};
    use crate::doc::vector::{Brush, Fill, FillRule, PathSource, Point, Rgb, Shape, Stroke, LineCap, LineJoin};

    fn pixel(rgba: &[u8], w: usize, x: usize, y: usize) -> [u8; 4] {
        let i = (y * w + x) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
    }

    /// 形の層は fork の描き手から生まれる: 矩形の塗りは中で白、外は背景のまま、線は縁に乗る。
    #[test]
    fn a_shape_layer_is_drawn_by_the_path_renderer() {
        let (w, h) = (64u32, 64u32);
        let fps = Fps::try_new(30, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: w, height: h, fps, duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        let red = Rgb { r: 1.0, g: 0.0, b: 0.0 };
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetShapes { layer, shapes: vec![ShapeNode::Leaf(Shape {
                source: PathSource::Rectangle { size: Point { x: 32.0, y: 32.0 } },
                ops: Vec::new(),
                fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), rule: FillRule::NonZero, opacity: 1.0, hidden: false }),
                stroke: Some(Stroke { brush: Brush::Solid(red), width: 4.0, cap: LineCap::Butt, join: LineJoin::Miter, miter_limit: 4.0, opacity: 1.0, hidden: false, dash: None }),
            })] },
        ]).unwrap();
        let mut engine = crate::render::engine::Engine::new().unwrap();
        let t = RationalTime::try_from_frame(0, fps).unwrap();
        let rgba = engine.render_frame(&doc.view(), t).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        // 層は枠の左上に置かれ、canvas は輪郭 ±(線の半分 + 1px)。矩形は comp の 3..35、線はその両側 2px。
        let center = pixel(&rgba, w as usize, 19, 19);
        assert!(center[0] > 240 && center[1] > 240 && center[2] > 240, "中は白: {center:?}");
        let outside = pixel(&rgba, w as usize, 60, 60);
        assert!(outside[0] < 8 && outside[1] < 8 && outside[2] < 8, "外は背景: {outside:?}");
        let edge = pixel(&rgba, w as usize, 3, 19);
        assert!(edge[0] > 200 && edge[1] < 60, "縁は赤い線: {edge:?}");
        let beyond = pixel(&rgba, w as usize, 40, 19);
        assert!(beyond[0] < 8, "線の外は背景: {beyond:?}");
    }
}
