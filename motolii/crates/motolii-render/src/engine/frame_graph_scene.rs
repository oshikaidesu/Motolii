use crate::doc::core::{CompSpec, LayerPlacement, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneValue, SolverPlanValue};
use crate::render::compositor::LayerContent;
use crate::render::compositor::effects::surface_program::SurfaceRecipe;
use crate::render_graph::{Composed, Extrusion, ImageInput, LayerWork, RasterSource, RenderGraph};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses, WorldLight};
use crate::render::engine::{Engine, EngineError};

/// The prepared world of one document frame: every contribution as a view-independent picture or
/// resource (no view's camera, window, selection or history is in it), and the frame's one light.
#[derive(Clone)]
pub(super) struct GpuSceneValue {
    pub layers: Vec<LayerWithPasses>,
    pub layer_ids: Vec<LayerId>,
    pub light: WorldLight,
}

/// How precisely a frame is prepared, as the views that will show it asked before it was prepared
/// (their densest; a tick decides it). It changes raster resolution and how finely outlines are
/// cut — never geometry, placement, membership or visibility.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Precision {
    /// Device pixels per composition pixel for pictures (a power of two, at most 1).
    pub picture: f32,
    /// Device pixels per composition pixel for outlines (a power of two, at least 1).
    pub magnification: f32,
}

impl Default for Precision {
    fn default() -> Self { Self { picture: 1.0, magnification: 1.0 } }
}

impl Precision {
    /// The precision a set of views showing `density` device pixels per composition pixel asks for.
    pub(crate) fn for_density(density: f32) -> Self {
        if density <= 0.0 { return Self::default(); }
        let stepped = (2.0_f32).powf(density.max(1.0 / 16.0).log2().ceil());
        Self { picture: stepped.min(1.0), magnification: stepped.max(1.0) }
    }
}

/// The authored camera, as far as a preparation may see it: only through these two narrowly named
/// doors, never as a general input. Culling, level of detail and bounds do not open them.
#[derive(Clone, Copy)]
pub(in crate::engine) struct LegacyCameraSeam(ResolvedCamera);

impl LegacyCameraSeam {
    pub(in crate::engine) fn new(authored: ResolvedCamera) -> Self { Self(authored) }
    /// UNDECIDED (boundary audit 2026-09-23): the camera a plate, matte, clip group, flatten or
    /// image input is taken through. Whether such a picture belongs to the output camera or to each
    /// view is an open creative decision; until it is ruled, it is the authored camera, as before.
    pub(in crate::engine) fn composition_picture(self) -> ResolvedCamera { self.0 }
    /// Where 2D and 2.5D layers stand in the world a capture or the block solver sees: 2.5D is
    /// placed relative to a camera by definition, and world captures place it by the output's
    /// (ruling 2026-09-23, render-orchestration). 3D placement does not read it (the identity).
    pub(in crate::engine) fn camera_relative_world(self) -> ResolvedCamera { self.0 }
}

/// One preparation: its composition, precision, the authored-camera seam, and the plates waiting
/// for its one world light. It exists only while a document frame (or a picture that is a frame of
/// its own) is prepared; a view never sees it.
pub(in crate::engine) struct Preparation {
    pub(in crate::engine) comp: CompSpec,
    pub(in crate::engine) precision: Precision,
    pub(in crate::engine) seam: LegacyCameraSeam,
    pub(in crate::engine) plates: PlateWork,
}

/// Plates whose picture waits for the preparation's world light, the members they hold (the
/// light scene), and whether something read a plate before the light was taken.
#[derive(Default)]
pub(in crate::engine) struct PlateWork {
    pub(in crate::engine) pending: Vec<super::render::build::PendingPlate>,
    /// Plates the views materialize, whose members' pictures are prepared with the pending plates.
    pub(in crate::engine) view: Vec<std::sync::Arc<crate::render::compositor::ViewPlate>>,
    pub(in crate::engine) light_scene: Vec<LayerWithPasses>,
    pub(in crate::engine) member_ids: Vec<LayerId>,
    pub(in crate::engine) deferring: bool,
    /// The light the frame was already lit by (a frame prepared again because a plate was read early).
    pub(in crate::engine) known_light: Option<WorldLight>,
    pub(in crate::engine) needed_early: bool,
}

impl Preparation {
    pub(in crate::engine) fn new(comp: CompSpec, precision: Precision, seam: LegacyCameraSeam) -> Self {
        Self { comp, precision, seam, plates: PlateWork::default() }
    }
    /// A picture prepared as a frame of its own inside this one (another time's composite, an
    /// analysis picture): same composition, precision and seam; its own plates.
    fn nested(&self) -> Self { Self::new(self.comp, self.precision, self.seam) }
}

impl Engine {
    pub(super) fn prepare_gpu_scene_with_solver(
        &mut self,
        scene: &SceneValue,
        solver: &SolverPlanValue,
        prep: &mut Preparation,
        time: crate::doc::core::RationalTime,
        fps: crate::doc::store::Fps,
    ) -> Result<GpuSceneValue, EngineError> {
        // Members first, then the frame's one light capture (it sees the members of every
        // plate), then the plates' pictures lit by it.
        self.preparation_events.clear();
        prep.plates.deferring = true;
        let prepared = self.prepare_gpu_scene_with_solver_members(scene, solver, prep, time, fps);
        prep.plates.deferring = false;
        let mut prepared = prepared?;
        if !prep.plates.needed_early {
            prepared.light = self.light_and_bake(prep, &prepared.layers)?;
            return Ok(prepared);
        }
        // Something in the frame (a matte, a clip group) reads a plate's picture while the frame is
        // prepared: the light scene just collected lights the frame once, and the frame is
        // prepared again with its plates baked in place under that same light.
        let light = self.light_the_scene(prep, &prepared.layers)?;
        prep.plates.pending.clear();
        prep.plates.view.clear();
        // The draft pass's record is dropped with its work; only the capture it made stays.
        self.preparation_events.retain(|event| matches!(event, PreparationEvent::Capture(_)));
        prep.plates.known_light = Some(light.clone());
        let mut prepared = self.prepare_gpu_scene_with_solver_members(scene, solver, prep, time, fps)?;
        prepared.light = light;
        Ok(prepared)
    }

    /// A picture prepared as a frame of its own (another time's composite, a frozen frame): its
    /// plates wait for its one capture like a frame's. Returns what `f` made and its light.
    pub(super) fn as_own_frame<T>(&mut self, parent: &Preparation, f: impl FnOnce(&mut Self, &mut Preparation) -> Result<(T, Vec<LayerWithPasses>), EngineError>) -> Result<(T, WorldLight), EngineError> {
        self.as_frame_part(parent, true, f)
    }

    /// Like `as_own_frame`, but when `own_frame` is false the pictures belong to the frame being
    /// evaluated (a read for analysis): its plates are baked unlit by any scene reflection, and no
    /// capture is made for them — a frame has one.
    pub(super) fn as_frame_part<T>(&mut self, parent: &Preparation, own_frame: bool, f: impl FnOnce(&mut Self, &mut Preparation) -> Result<(T, Vec<LayerWithPasses>), EngineError>) -> Result<(T, WorldLight), EngineError> {
        let mut prep = parent.nested();
        prep.plates.deferring = true;
        let made = f(self, &mut prep);
        prep.plates.deferring = false;
        let (value, top) = made?;
        if own_frame { return Ok((value, self.light_and_bake(&mut prep, &top)?)); }
        let unlit = WorldLight::default();
        self.bake_pending_plates(&mut prep, &unlit)?;
        Ok((value, unlit))
    }

    /// The frame's light, captured once, and its waiting plates baked with it.
    fn light_and_bake(&mut self, prep: &mut Preparation, top: &[LayerWithPasses]) -> Result<WorldLight, EngineError> {
        let light = self.light_the_scene(prep, top)?;
        self.bake_pending_plates(prep, &light)?;
        Ok(light)
    }

    /// The frame's world light, once, from its light scene: the top-level layers (not the
    /// plates' pictures, which are not baked yet) and every plate's members, each where it is.
    fn light_the_scene(&mut self, prep: &mut Preparation, top: &[LayerWithPasses]) -> Result<WorldLight, EngineError> {
        // A plate's picture is not baked yet: its members stand for it.
        let is_plate = |layer: &LayerWithPasses| matches!(layer.layer.content, crate::render::compositor::LayerContent::Plate(_))
            || prep.plates.pending.iter().any(|plate| plate.is_picture_of(&layer.layer.content));
        let mut scene: Vec<LayerWithPasses> = top.iter().filter(|layer| !is_plate(layer)).cloned().collect();
        let same_as_frame = scene.len() == top.len() && prep.plates.light_scene.is_empty();
        // The plates' members this capture sees (the top level is the frame's own layer list).
        self.light_scene_ids = std::mem::take(&mut prep.plates.member_ids);
        scene.extend(prep.plates.light_scene.drain(..));
        let (pictures, paddings, spills) = self.compositor.effective_layer_textures(&scene)?;
        let world = prep.seam.camera_relative_world();
        let inputs = crate::render::compositor::sequential_inputs(&scene, &pictures, &paddings, &spills, world, world);
        let environment = self.compositor.world_environment.clone();
        let (light, meshes, views) = self.compositor.capture_world_light(prep.comp, &inputs, environment.as_deref())?;
        drop(inputs);
        self.world_light_captures += 1;
        self.preparation_events.push(PreparationEvent::Capture(self.world_light_captures));
        Ok(WorldLight { views, light, meshes: meshes.filter(|_| same_as_frame), serial: self.world_light_captures })
    }

    fn prepare_gpu_scene_with_solver_members(
        &mut self,
        scene: &SceneValue,
        solver: &SolverPlanValue,
        prep: &mut Preparation,
        time: crate::doc::core::RationalTime,
        fps: crate::doc::store::Fps,
    ) -> Result<GpuSceneValue, EngineError> {
        let physics_overlays: std::collections::HashSet<LayerId> = self.overlay_frames.iter()
            .filter_map(|(layer, frame)| frame.physics.then_some(*layer))
            .collect();
        if physics_overlays.is_empty() {
            // The whole world is prepared: whether a view sees a contribution is the view's question.
            let mut prepared = self.prepare_gpu_scene_incremental(scene, prep)?;
            let started = std::time::Instant::now();
            self.prepare_frame_graph_blocks(scene, solver, prep, time, fps, &mut prepared)?;
            if self.blocks.object_count() > 0 { self.ledger.claim("blocks", "block solver", format!("{} objects", self.blocks.object_count()), started.elapsed()); }
            self.drawn_layers = prepared.layers.len();
            return Ok(prepared);
        }

        // Physics Trace reads the solver's current-frame state. Build the
        // ordinary scene first, solve motion/physics, then lower the complete
        // scene once the overlay can read those results.
        let base_scene = SceneValue {
            layers: scene.layers.iter()
                .filter(|layer| !physics_overlays.contains(&layer.layer))
                .cloned()
                .collect(),
        };
        let mut base = self.prepare_gpu_scene(&base_scene, prep)?;
        self.prepare_frame_graph_blocks(scene, solver, prep, time, fps, &mut base)?;

        let mut prepared = self.prepare_gpu_scene(scene, prep)?;
        let world = prep.seam.camera_relative_world();
        for (id, layer) in prepared.layer_ids.iter().copied().zip(prepared.layers.iter_mut()) {
            self.attach_block_id(id, &mut layer.layer, prep.comp, world);
        }
        self.drawn_layers = prepared.layers.len();
        Ok(prepared)
    }

    pub(super) fn prepare_gpu_scene(&mut self, scene: &SceneValue, prep: &mut Preparation) -> Result<GpuSceneValue, EngineError> {
        self.compositor.refresh_catalog_programs();
        let catalog = self.compositor.catalog.clone();
        let graph = crate::render_lowering::lower_scene(scene, &catalog)
            .map_err(|error| EngineError::Store(error.to_string()))?;
        self.adopt_world_environment(&graph)?;
        self.execute_render_graph(&graph, prep)
    }

    /// The top-level graph's environment becomes the composition's light for every draw in the
    /// frame, including the nested draws of plates.
    pub(super) fn adopt_world_environment(&mut self, graph: &crate::render_graph::RenderGraph) -> Result<(), EngineError> {
        let path = graph.layers.iter().rev().find_map(|work| match &work.content {
            RasterSource::EnvironmentMap { path } => Some(path.clone()),
            _ => None,
        });
        self.compositor.world_environment = match path {
            Some(path) => match self.environment_content_for(&path)?.0 {
                Some(LayerContent::Environment(environment)) => Some(environment),
                _ => None,
            },
            None => None,
        };
        Ok(())
    }

    /// The scene's contributions as material-space pictures the host reads back. `own_frame`: the
    /// scene is an evaluation of its own (a frozen frame at its time) and is lit like a frame; else
    /// it belongs to the frame being evaluated (an analysis read) and makes no capture of its own.
    pub(super) fn prepare_gpu_pictures(&mut self, scene: &SceneValue, parent: &Preparation, own_frame: bool) -> Result<GpuSceneValue, EngineError> {
        self.compositor.refresh_catalog_programs();
        let catalog = self.compositor.catalog.clone();
        let graph = crate::render_lowering::lower_scene_as_pictures(scene, &catalog)
            .map_err(|error| EngineError::Store(error.to_string()))?;
        self.adopt_world_environment(&graph)?;
        let (mut prepared, light) = self.as_frame_part(parent, own_frame, |engine, prep| {
            let prepared = engine.execute_render_graph(&graph, prep)?;
            let top = prepared.layers.clone();
            Ok((prepared, top))
        })?;
        prepared.light = light;
        Ok(prepared)
    }

    /// Cassette executor: reads only the backend-neutral graph.
    pub(super) fn execute_render_graph(&mut self, graph: &RenderGraph, prep: &mut Preparation) -> Result<GpuSceneValue, EngineError> {
        let mut prepared = Vec::with_capacity(graph.layers.len());
        for work in &graph.layers {
            prepared.push(self.execute_layer(work, prep)?);
        }
        self.compose_prepared(graph, &prepared, prep)
    }

    /// Clip groups and mattes over prepared contributions, in scene order.
    fn compose_prepared(&mut self, graph: &RenderGraph, prepared: &[Option<LayerWithPasses>], prep: &mut Preparation) -> Result<GpuSceneValue, EngineError> {
        // A matte or clip group reads its layers' pictures now: plates waiting for the frame's light
        // are baked first, each with its own capture.
        if !prep.plates.pending.is_empty() && graph.output.iter().any(|composed| composed.mask.is_some() || !composed.atop.is_empty()) {
            prep.plates.needed_early = true;
        }
        let mut groups = std::collections::HashMap::new();
        let mut layers = Vec::with_capacity(graph.output.len());
        let mut layer_ids = Vec::with_capacity(graph.output.len());
        for composed in &graph.output {
            let started = std::time::Instant::now();
            let realized = self.realize_composed(graph, prepared, composed, &mut groups, prep)?;
            if composed.mask.is_some() || !composed.atop.is_empty() {
                let why = if composed.mask.is_some() { "matte composed" } else { "clip group composed" };
                self.ledger.claim("compose", format!("layer {}", graph.layers[composed.base].id.0), why, started.elapsed());
            }
            if let Some(layer) = realized {
                layers.push(layer);
                layer_ids.push(graph.layers[composed.base].id);
            }
        }
        Ok(GpuSceneValue { layers, layer_ids, light: WorldLight::default() })
    }

    /// How finely an outline is cut: one device pixel at the precision the views asked for, on the
    /// layer's own authored scale. No camera reads it: a view that magnifies further is a new
    /// precision request, not a property of the prepared world.
    fn outline_tolerance(&self, work: Option<&LayerWork>, natural: [f32; 2], vector: bool, precision: Precision) -> f32 {
        let Some(work) = work else { return 0.05 };
        let m = work.placement.transform.matrix2;
        let scale = m.x_axis.length().max(m.y_axis.length()).max(1e-6);
        let density = (scale * precision.magnification).max(1.0);
        let mut exact = ((density * 1024.0).round() / 1024.0).max(1.0);
        if !work.passes.is_empty() {
            let reach = work.passes.iter().map(|p| p.padding() as f32).fold(0.0f32, f32::max);
            let limit = self.compositor.ctx.device.limits().max_texture_dimension_2d as f32;
            let extent = natural[0].max(natural[1]).max(1.0) + 2.0 * reach;
            exact = exact.min((limit / extent).max(1.0));
        }
        let stepped = (density * (1.0 - 1e-4)).log2().ceil().exp2().max(1.0);
        (0.05 / if vector { stepped } else { exact }).max(1e-6)
    }

    fn execute_raster(&mut self, id: LayerId, key: LayerId, source: &RasterSource, work: Option<&LayerWork>, prep: &mut Preparation) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        let comp = prep.comp;
        let order = work.map_or(0, |work| work.placement.order);
        Ok(match source {
            RasterSource::None => (None, [0.0, 0.0]),
            RasterSource::Vector { shapes, vector, remember, field_step } => {
                let step = field_step.then_some(crate::render::engine::texture::FIELD_STEP);
                // An unbent vector outline is painted exactly from its curves: no cut to choose.
                let tolerance = if *vector && step.is_none() {
                    0.05
                } else {
                    let natural = match self.cached_shape(key, false, shapes, *vector, step, None) {
                        Some(hit) => hit.natural,
                        None => crate::picture::shapes_ops::content_canvas(shapes)?
                            .map_or([1.0; 2], |canvas| [canvas.width as f32, canvas.height as f32]),
                    };
                    self.outline_tolerance(work, natural, *vector, prep.precision)
                };
                self.shape_texture_from_shapes(shapes, key, *vector, tolerance, comp, step, *remember)?
            }
            RasterSource::CanvasVector { shapes } => self.text_texture_from_shapes(shapes, key, comp)?,
            RasterSource::Mesh { path } => self.mesh_content_for(path, comp)?,
            RasterSource::EnvironmentMap { path } => self.environment_content_for(path)?,
            RasterSource::Image { path, time } => self.file_content_for(path, *time, id, comp)?,
            RasterSource::Points { positions, colors, sizes, bounds, links } => (
                Some(LayerContent::Cloud {
                    positions: positions.clone(),
                    colors: colors.clone(),
                    bounds: *bounds,
                    point_size: 1.0,
                    sizes: Some(sizes.clone()),
                    sprites: true,
                    links: links.clone(),
                }),
                [bounds.max[0].max(1.0), bounds.max[1].max(1.0)],
            ),
            RasterSource::Isolate { graph, average } => {
                let prepared = self.execute_render_graph(graph, prep)?;
                if prepared.layers.is_empty() {
                    (None, [comp.width as f32, comp.height as f32])
                } else {
                    let placement = LayerPlacement {
                        transform: glam::Affine2::IDENTITY,
                        world_transform: None,
                        opacity: 1.0,
                        order,
                        z: 0.0,
                        rotation_x: 0.0,
                        rotation_y: 0.0,
                        plane: None,
                    };
                    let baked = if Self::plate_reads_view(&prepared.layers) {
                        if prep.plates.deferring || prep.plates.known_light.is_none() {
                            prep.plates.member_ids.extend(prepared.layer_ids.iter().copied());
                        }
                        self.view_plate(prep, prepared.layers, placement, *average)?
                    } else if prep.plates.deferring || prep.plates.known_light.is_none() {
                        prep.plates.member_ids.extend(prepared.layer_ids.iter().copied());
                        self.defer_isolated_layers(prep, prepared.layers, placement, *average)?
                    } else {
                        let density = prep.precision.picture;
                        self.bake_isolated_layers(prep, prepared.layers, CompositeBlendMode::Normal, placement, *average, density)?
                    };
                    (Some(baked.content), baked.size)
                }
            }
        })
    }

    fn execute_layer(&mut self, work: &LayerWork, prep: &mut Preparation) -> Result<Option<LayerWithPasses>, EngineError> {
        let comp = prep.comp;
        let host = if work.host_picture { self.overlay_content(work.id, comp)? } else { None };
        let frozen = self.frame_graph_frozen_content(work);
        let (content, natural, frozen_padding, frozen_frame, frozen_hit) = if let Some((content, natural, padding, frame)) = frozen {
            (Some(content), natural, padding, frame, true)
        } else if let Some((content, natural)) = host {
            (Some(content), natural, 0, None, false)
        } else {
            let (content, natural) = self.execute_raster(work.id, work.content_key, &work.content, Some(work), prep)?;
            // A picture drawn above one pixel per unit carries its logical frame.
            // A plate baked at the views' density carries it too, so its effects keep their size.
            let frame = matches!(work.content, RasterSource::Vector { .. } | RasterSource::Isolate { .. })
                .then(|| content.as_ref().and_then(|content| content.texture()))
                .flatten()
                .map(|texture| crate::render::compositor::effects::vism::ImageFrame { size: natural, origin: [0.0; 2], pixels: texture.width_height() });
            (content, natural, 0, frame, false)
        };
        let Some(mut content) = content else { return Ok(None) };
        if let (LayerContent::Texture(texture), Some(extrusion)) = (&content, &work.extrude) {
            content = self.frame_graph_extruded_content(work, extrusion, texture.clone(), natural)?;
        }
        // A plate waiting for the frame's light is a picture of its own, drawn once and not written
        // again: it needs no snapshot (and a snapshot now would copy it before it is baked).
        let waiting_plate = prep.plates.pending.iter().any(|plate| plate.is_picture_of(&content));
        if !frozen_hit && !waiting_plate && !work.image_inputs.is_empty() {
            content = match &content {
                LayerContent::Texture(texture) => self.compositor.snapshot_texture(texture)
                    .map(LayerContent::Texture)
                    .unwrap_or_else(|| content.clone()),
                LayerContent::LinearTexture(texture) => self.compositor.snapshot_texture(texture)
                    .map(LayerContent::LinearTexture)
                    .unwrap_or_else(|| content.clone()),
                _ => content,
            };
        }
        let (passes, pass_sources) = if frozen_hit {
            (Vec::new(), Vec::new())
        } else {
            // A history is keyed by its layer and chain here; the drawing that runs a pass on its
            // own picture (a view) adds itself to the key when it runs it.
            let mut direct_passes = work.passes.clone();
            self.stamp_feedback(&mut direct_passes, work.id, work.instance, 0, None);
            let mut plate_passes = work.after_passes.clone();
            self.stamp_feedback(&mut plate_passes, work.id, work.instance, 1, None);

            (
                direct_passes.into_iter().chain(plate_passes).collect(),
                self.frame_graph_image_sources(&work.image_inputs, prep)?,
            )
        };
        let layer = Layer {
            content,
            size: natural,
            placement: work.placement,
            projection: work.projection,
            blend_mode: work.blend,
            shading: Default::default(),
            displace: work.displace,
            clip: work.clip,
            shadow: work.shadow,
            frame: frozen_frame,
        };
        let layer = self.apply_masks_to_layer(layer, &work.masks, natural, frozen_frame)?;
        let mut layer = self.apply_material_recipe(layer, work.id, &work.material, work.source_is_file, work.source_tick, natural, frozen_frame)?;
        if matches!(layer.content, LayerContent::Model(_) | LayerContent::Texture(_) | LayerContent::LinearTexture(_)) {
            let recipe = SurfaceRecipe { unlit: matches!(&layer.content, LayerContent::Model(model) if model.planar_size.is_some()), owner: work.id.0, ..work.surface.clone() };
            layer.shading = self.compositor.surface_shading_from(&recipe).map_err(EngineError::Store)?;
        }
        let layer = self.flatten_if_asked(prep, layer, work.isolate)?;
        Ok(Some(LayerWithPasses { layer, passes, padding: frozen_padding, pass_sources, cut: Vec::new() }))
    }

    /// An absent picture contributes no coverage, so a picture masked by an
    /// absent picture is absent too.
    fn realize_composed(
        &mut self,
        graph: &RenderGraph,
        prepared: &[Option<LayerWithPasses>],
        composed: &Composed,
        groups: &mut std::collections::HashMap<usize, Option<LayerWithPasses>>,
        prep: &mut Preparation,
    ) -> Result<Option<LayerWithPasses>, EngineError> {
        if !groups.contains_key(&composed.base) {
            let mut base = prepared[composed.base].clone();
            for &upper in &composed.atop {
                let Some(upper) = prepared[upper].as_ref() else { continue };
                let Some(current) = base.take() else { break };
                base = match self.clip_onto_base(current.clone(), &upper.layer, &upper.passes)? {
                    Some(clipped) => Some(clipped),
                    None => {
                        self.layer_failures.push(format!(
                            "layer {} clips to a base without a texture (point cloud / model bases are not clippable)",
                            graph.layers[composed.base].id.0
                        ));
                        Some(current)
                    }
                };
            }
            groups.insert(composed.base, base);
        }
        let Some(base) = groups[&composed.base].clone() else { return Ok(None) };
        let Some((masks, mode)) = &composed.mask else { return Ok(Some(base)) };
        let mut sources = Vec::with_capacity(masks.len());
        for mask in masks {
            if let Some(source) = self.realize_composed(graph, prepared, mask, groups, prep)? { sources.push(source); }
        }
        let source = match sources.len() {
            0 => return Ok(None),
            1 => { let source = sources.remove(0); self.apply_effects_before_matte(prep, source.layer, &source.passes)? }
            _ => {
                let placement = sources[0].layer.placement;
                self.bake_isolated_layers(prep, sources, CompositeBlendMode::Normal, placement, false, 1.0)?
            }
        };
        let target = self.apply_effects_before_matte(prep, base.layer, &base.passes)?;
        let layer = self.compositor.matte_layer(prep.comp, prep.seam.composition_picture(), &target, &source, *mode)?;
        Ok(Some(LayerWithPasses { layer, passes: Vec::new(), padding: 0, pass_sources: Vec::new(), cut: Vec::new() }))
    }

    fn frame_graph_extruded_content(
        &mut self,
        work: &LayerWork,
        extrusion: &Extrusion,
        texture: crate::render::compositor::GpuTexture2D,
        natural: [f32; 2],
    ) -> Result<crate::render::compositor::LayerContent, EngineError> {
        let solid = extrusion.solid;
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        texture.handle.hash(&mut hasher);
        solid.hash_key(&mut hasher);
        extrusion.stretch[0].to_bits().hash(&mut hasher);
        extrusion.stretch[1].to_bits().hash(&mut hasher);
        let key = hasher.finish();
        if let Some((cached, model)) = self.extrusions.get(&work.id) {
            if *cached == key {
                return Ok(crate::render::compositor::LayerContent::Model(model.clone()));
            }
        }

        let rectangle = |size: [f32; 2]| {
            let vertex = |x: f32, y: f32| re_renderer::renderer::PathVertex {
                point: glam::vec2(x, y),
                in_tangent: glam::Vec2::ZERO,
                out_tangent: glam::Vec2::ZERO,
            };
            vec![(
                vec![re_renderer::renderer::PathContour {
                    closed: true,
                    vertices: vec![
                        vertex(0.0, 0.0),
                        vertex(size[0], 0.0),
                        vertex(size[0], size[1]),
                        vertex(0.0, size[1]),
                    ],
                }],
                re_renderer::renderer::PathFillRule::NonZero,
            )]
        };

        let outlines = match &extrusion.outline {
            Some(shapes) => match crate::picture::shapes_ops::content_canvas(shapes)? {
                Some(canvas) => crate::render::compositor::paths::outlines(shapes, &canvas)?,
                None => Vec::new(),
            },
            None => rectangle(natural),
        };

        let Some(model) = self.compositor.extrude_model(&outlines, texture.clone(), natural, solid)? else {
            return Ok(crate::render::compositor::LayerContent::Texture(texture));
        };
        let model = std::sync::Arc::new(model);
        self.extrusions.insert(work.id, (key, model.clone()));
        Ok(crate::render::compositor::LayerContent::Model(model))
    }

    fn frame_graph_frozen_content(
        &mut self,
        work: &LayerWork,
    ) -> Option<(
        crate::render::compositor::LayerContent,
        [f32; 2],
        u32,
        Option<crate::render::compositor::effects::vism::ImageFrame>,
    )> {
        let timing_start = work.freeze?;
        if self.freezing == Some(work.id) {
            return None;
        }
        let comp_frame = self.compositor.clock?.get(2).copied()?.round() as i64;
        let layer_frame = comp_frame - timing_start;
        let Self { frozen, compositor, .. } = self;
        let picture = frozen.load(
            work.id,
            layer_frame,
            &mut |bytes, width, height| compositor.upload_rgba16f("motolii-frozen", bytes, width, height).ok(),
        )?;
        Some((
            crate::render::compositor::LayerContent::LinearTexture(picture.texture),
            picture.natural,
            picture.padding,
            picture.frame,
        ))
    }

    fn frame_graph_image_sources(
        &mut self,
        rows: &[Vec<ImageInput>],
        prep: &Preparation,
    ) -> Result<Vec<Vec<crate::render::compositor::GpuTexture2D>>, EngineError> {
        rows.iter().map(|row| {
            let mut textures = Vec::with_capacity(row.len());
            for input in row {
                match self.frame_graph_image_source(input, prep)? {
                    Some(texture) => textures.push(texture),
                    None => {
                        textures.clear();
                        break;
                    }
                }
            }
            Ok(textures)
        }).collect()
    }

    fn frame_graph_image_source(
        &mut self,
        input: &ImageInput,
        prep: &Preparation,
    ) -> Result<Option<crate::render::compositor::GpuTexture2D>, EngineError> {
        let (time, namespace) = match input {
            ImageInput::Absent => return Ok(None),
            ImageInput::Refused { layer } => {
                self.layer_failures.push(format!("指した層 {} の絵が無い(自分自身か、無い層)", layer.0));
                return Ok(None);
            }
            ImageInput::Raster { time, namespace, .. } | ImageInput::Graph { time, namespace, .. } => (*time, *namespace),
        };
        let previous_clock = self.compositor.clock;
        let previous_namespace = self.feedback_namespace;
        self.feedback_namespace = namespace;
        self.set_frame_graph_source_clock(time);
        let result = (|| match input {
            ImageInput::Absent | ImageInput::Refused { .. } => Ok(None),
            ImageInput::Raster { id, source, .. } => {
                // Another time is a frame of its own: its plates wait for its one capture.
                let ((content, _), _light) = self.as_own_frame(prep, |engine, own| {
                    let made = engine.execute_raster(*id, *id, source, None, own)?;
                    Ok((made, Vec::new()))
                })?;
                let Some(texture) = content.and_then(|content| content.texture().cloned()) else {
                    return Ok(None);
                };
                Ok(self.compositor.snapshot_texture(&texture))
            }
            ImageInput::Graph { graph, background, absent_when_empty, .. } => {
                let (prepared, light) = self.as_own_frame(prep, |engine, own| {
                    let prepared = engine.execute_render_graph(graph, own)?;
                    let top = prepared.layers.clone();
                    Ok((prepared, top))
                })?;
                if prepared.layers.is_empty() && *absent_when_empty { return Ok(None); }
                let texture = self.compositor.bake_picture(prep.comp, prep.seam.composition_picture(), &prepared.layers, *background, 1.0, Some(&light), None)?;
                Ok(self.compositor.import_premultiplied(&texture).ok())
            }
        })();
        self.compositor.clock = previous_clock;
        self.feedback_namespace = previous_namespace;
        result
    }

    fn set_frame_graph_source_clock(&mut self, time: crate::doc::core::RationalTime) {
        let delta = self.compositor.clock.map_or(1.0 / 30.0, |clock| clock[1].max(1.0e-9));
        let frame = (time.as_seconds_f64() as f32 / delta).round();
        self.compositor.clock = Some([time.as_seconds_f64() as f32, delta, frame]);
    }

}


mod reuse;
pub(super) use reuse::ContributionCache;

#[cfg(test)]
mod tests;


/// What the frame's preparation did, in order: the invariants are read from this, not from pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::engine) enum PreparationEvent {
    /// A frame-level world light capture, by serial.
    Capture(u64),
    /// A plate baked, lit by the capture of this serial (0: unlit, a read within the frame).
    Bake(u64),
}
