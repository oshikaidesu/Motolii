//! 選んだ層の箱と輪郭、そこで効いているカメラ。

use super::*;

pub(super) fn group_local_bounds(
    view: &StoreView<'_>,
    resolved: &[ResolvedLayer],
    group: LayerId,
    mut leaf_bounds: impl FnMut(&ResolvedLayer) -> Option<crate::render::media::SpatialBounds>,
) -> Option<crate::render::media::SpatialBounds> {
    let group_world = resolved.iter().find(|layer| layer.id == group && !layer.ghost)?.placement.world_transform?;
    if !group_world.is_finite() || group_world.matrix3.determinant() == 0.0 { return None; }
    let local_from_world = group_world.inverse();
    if !local_from_world.is_finite() { return None; }
    let parents: HashMap<_, _> = view.layers().into_iter().filter_map(|layer| {
        Some((layer, view.attrs(layer).ok().flatten()?.parent?))
    }).collect();
    let matte_sources: std::collections::HashSet<_> = resolved.iter().filter(|layer| !layer.clip_to_below).filter_map(|layer| layer.matte.map(|matte| matte.layer)).collect();
    let mut points = Vec::new();
    for leaf in resolved {
        if matches!(leaf.source, LayerSource::Camera | LayerSource::Stage | LayerSource::Group | LayerSource::Null) || leaf.placement.opacity <= 0.0 || matte_sources.contains(&leaf.id) { continue; }
        let mut parent = parents.get(&leaf.id).copied();
        let mut visited = std::collections::HashSet::new();
        let mut descendant = false;
        while let Some(id) = parent {
            if id == group { descendant = true; break; }
            if !visited.insert(id) { break; }
            parent = parents.get(&id).copied();
        }
        if !descendant { continue; }
        let Some(bounds) = leaf_bounds(leaf) else { continue; };
        let world = leaf.placement.world_transform?;
        let local = local_from_world * world;
        for x in [bounds.min[0], bounds.max[0]] {
            for y in [bounds.min[1], bounds.max[1]] {
                for z in [bounds.min[2], bounds.max[2]] {
                    points.push(local.transform_point3(glam::vec3(x, y, z)).to_array());
                }
            }
        }
    }
    crate::render::media::SpatialBounds::from_points(points).ok()
}

impl Engine {
    pub fn selected_layer_size(
        &self,
        view: &StoreView<'_>,
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<[f32; 2]> {
        let resolved = self.resolved_for(view, t)?;
        self.selected_layer_size_in(view, &resolved, layer_id, t)
    }

    /// 解いた層の一覧を持っている側(Stage の paint)は、層ごとに解き直さない。
    pub fn selected_layer_size_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[crate::picture::resolved::ResolvedLayer],
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<[f32; 2]> {
        self.selected_layer_bounds_in(view, resolved, layer_id, t).map(|bounds| bounds.size_xy())
    }

    /// Camera 層 `id` の姿勢。層ターゲットは Stage の枠・Depth の点と同じ bounds の中心を見る(Document だけなら anchor)。
    pub fn camera_of_layer_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[ResolvedLayer],
        id: LayerId,
        t: RationalTime,
    ) -> Result<crate::doc::core::ResolvedCamera, crate::render::engine::EngineError> {
        let store = |e: crate::doc::store::StoreError| crate::render::engine::EngineError::Store(e.to_string());
        let mut camera = crate::picture::resolve::camera::camera_of_layer(view, id, t).map_err(store)?;
        let Some(target) = crate::picture::resolve::camera::camera_target_layer(view, id, t).map_err(store)? else { return Ok(camera) };
        let (Some(bounds), Some(comp)) = (self.selected_layer_bounds_in(view, resolved, target, t), view.composition().map_err(store)?) else { return Ok(camera) };
        let comp = comp.spec();
        let Some(world) = resolved.iter().find(|layer| layer.id == target && !layer.ghost).and_then(|layer| layer.placement.world_transform) else { return Ok(camera) };
        let point = world.transform_point3(glam::Vec3::from(bounds.center()));
        camera.center = [point.x - comp.width as f32 * 0.5, point.y - comp.height as f32 * 0.5];
        camera.target_z = point.z;
        Ok(camera)
    }

    /// 作中カメラ。描画・Stage・Depth はすべてこれを通す。
    pub fn resolve_camera_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[ResolvedLayer],
        t: RationalTime,
    ) -> Result<crate::doc::core::ResolvedCamera, crate::render::engine::EngineError> {
        let store = |e: crate::doc::store::StoreError| crate::render::engine::EngineError::Store(e.to_string());
        match crate::picture::resolve::camera::active_camera_layer(view, t).map_err(store)? {
            Some(id) => self.camera_of_layer_in(view, resolved, id, t),
            None => crate::picture::resolve::camera::resolve_camera(view, t).map_err(store),
        }
    }

    /// 作中カメラの意味は FrameGraph Camera node が所有する。
    ///
    /// This accessor never compiles or evaluates a second graph. Callers that
    /// can arrive before production evaluation must prepare the revision/time
    /// through `frame_graph_document_camera` first.
    pub fn resolve_camera(
        &self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<crate::doc::core::ResolvedCamera, crate::render::engine::EngineError> {
        self.frame_graph_cached_camera(view, t)
            .ok_or_else(|| crate::render::engine::EngineError::Store("FrameGraph camera is not prepared for this revision/time".into()))
    }

    pub fn selected_layer_bounds_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[ResolvedLayer],
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<crate::render::media::SpatialBounds> {
        let layer = resolved.iter().find(|layer| layer.id == layer_id && !layer.ghost)?;
        if layer.source == LayerSource::Group {
            return group_local_bounds(view, resolved, layer_id, |leaf| self.leaf_local_bounds(view, leaf, t));
        }
        self.leaf_local_bounds(view, layer, t)
    }

    /// The points the layer actually draws, in the same local space as `selected_layer_bounds_in`:
    /// every vertex of a mesh or point cloud, the four corners of a flat picture. `None` for groups
    /// and for anything else the box already describes.
    pub fn selected_layer_outline_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[ResolvedLayer],
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<Vec<glam::Vec3>> {
        let layer = resolved.iter().find(|layer| layer.id == layer_id && !layer.ghost)?;
        let bounds = self.leaf_local_bounds(view, layer, t)?;
        let spatial = match &layer.source {
            LayerSource::File { path, .. } if crate::render::media::is_mesh_path(&path) => {
                let model = self.models.get(path)?;
                Some((model.bounds(), model.vertices.iter().copied().collect::<Vec<_>>()))
            }
            LayerSource::File { path, .. } if is_point_cloud_path(path) => {
                let cloud = self.point_clouds.get(path)?;
                Some((cloud.bounds(), cloud.silhouette.iter().copied().collect()))
            }
            _ => None,
        };
        Some(match spatial {
            // Drawn as `depth_scaled(world) * (v - origin)`: the same origin the box uses.
            Some((model, vertices)) => {
                let origin = glam::vec3(model.min[0], model.min[1], (model.min[2] + model.max[2]) * 0.5);
                vertices.into_iter().map(|v| v - origin).collect()
            }
            None => [[0, 0], [1, 0], [1, 1], [0, 1]].into_iter().map(|[x, y]| glam::vec3(
                if x == 0 { bounds.min[0] } else { bounds.max[0] },
                if y == 0 { bounds.min[1] } else { bounds.max[1] },
                0.0,
            )).collect(),
        })
    }

    fn leaf_local_bounds(
        &self,
        view: &StoreView<'_>,
        layer: &ResolvedLayer,
        t: RationalTime,
    ) -> Option<crate::render::media::SpatialBounds> {
        use crate::render::media::SpatialBounds;
        let layer_id = layer.id;
        let comp = view.composition().ok().flatten()?.spec();
        let planar = |bounds: SpatialBounds, natural: [f32; 2]| {
            let displayed = layer_size(layer, natural);
            let scale = [displayed[0] / natural[0], displayed[1] / natural[1]];
            SpatialBounds::from_points([
                [bounds.min[0] * scale[0], bounds.min[1] * scale[1], 0.0],
                [bounds.max[0] * scale[0], bounds.max[1] * scale[1], 0.0],
            ]).ok()
        };
        match &layer.source {
            LayerSource::Text => {
                let document = crate::picture::resolve::text::resolved_text_document(view, layer_id, t).ok().flatten()?;
                let partner = crate::extensions::text::morph(&layer.effects)
                    .and_then(|(target, amount)| crate::picture::resolve::text::resolved_text_document(view, target, t).ok().flatten().map(|d| (d, amount)));
                let key = TextCacheKey::new(layer_id, &document, partner.as_ref().map(|(d, a)| (d, *a)), t, comp.width, comp.height).moving(text::Flow::of(layer));
                let cached = self.text_textures.get(&key)?;
                planar(cached.bounds?, [comp.width as f32, comp.height as f32])
            }
            LayerSource::Shape => {
                let shapes = crate::render::engine::render::shown_shapes(&crate::picture::shapes::shapes_at(view, layer_id, t).ok()?, layer);
                let canvas = content_canvas(&shapes).ok().flatten()?;
                let key = ShapeCacheKey::new(layer_id, &shapes, canvas.width, canvas.height);
                let _cached = self.shape_textures.get(&key)?;
                let natural = [canvas.width as f32, canvas.height as f32];
                let bounds = crate::picture::shapes_ops::content_bounds(&shapes).ok().flatten()?;
                planar(SpatialBounds {
                    min: [bounds[0] as f32 + canvas.origin_x as f32, bounds[1] as f32 + canvas.origin_y as f32, 0.0],
                    max: [bounds[2] as f32 + canvas.origin_x as f32, bounds[3] as f32 + canvas.origin_y as f32, 0.0],
                }, natural)
            }
            LayerSource::Camera | LayerSource::Stage | LayerSource::Null | LayerSource::Group => None,
            LayerSource::Particles => self.particle_frames.get(&layer_id).map(|frame| frame.bounds),
            LayerSource::File { path, .. } => {
                let spatial = if crate::render::media::is_mesh_path(&path) {
                    Some(self.models.get(path)?.bounds())
                } else if is_point_cloud_path(path) {
                    Some(self.point_clouds.get(path)?.bounds())
                } else { None };
                if let Some(bounds) = spatial {
                    let [x, y, z] = bounds.size();
                    Some(SpatialBounds { min: [0.0, 0.0, -z * 0.5], max: [x, y, z * 0.5] })
                } else {
                    let info = self.probes.get(path)?;
                    let natural = [info.width as f32, info.height as f32];
                    planar(SpatialBounds { min: [0.0; 3], max: [natural[0], natural[1], 0.0] }, natural)
                }
            }
        }
    }
}
