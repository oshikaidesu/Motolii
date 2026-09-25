use std::sync::Arc;

use crate::doc::core::{LayerPlacement, RationalTime};
use crate::doc::store::{LayerId, LayerProjection, ShapeNode};
use crate::picture::resolved::ResolvedMask;

use crate::render::compositor::effects::surface_program::SurfaceRecipe;
use crate::render::compositor::extrude::Solid;

use crate::render::compositor::{BlendMode, ClipSpec, CloudLinks, EffectPass, MatteMode, PointDisplace};
use crate::render::media::SpatialBounds;

/// What a contribution rasterizes before placement.
#[derive(Clone)]
pub enum RasterSource {
    None,
    /// Outlines on their own canvas. `vector` keeps them resolution-independent;
    /// `field_step` subdivides them so a field can bend the outline.
    Vector { shapes: Arc<Vec<ShapeNode>>, vector: bool, remember: bool, field_step: bool },
    /// Outlines on the output canvas.
    CanvasVector { shapes: Arc<Vec<ShapeNode>> },
    Mesh { path: String },
    Image { path: String, time: RationalTime },
    EnvironmentMap { path: String },
    Points {
        positions: Arc<Vec<[f32; 3]>>,
        colors: Arc<Vec<[u8; 4]>>,
        sizes: Arc<Vec<f32>>,
        bounds: SpatialBounds,
        links: Option<Arc<CloudLinks>>,
    },
    /// A nested graph flattened into one picture; `average` divides by its members.
    Isolate { graph: Arc<RenderGraph>, average: bool },
}

/// A second picture an effect reads.
#[derive(Clone)]
pub enum ImageInput {
    Absent,
    /// A named layer that cannot be read; reported, never substituted.
    Refused { layer: LayerId },
    Raster { id: LayerId, source: RasterSource, time: RationalTime, namespace: u64 },
    Graph { graph: Arc<RenderGraph>, background: [f32; 4], absent_when_empty: bool, time: RationalTime, namespace: u64 },
}

/// Depth given to a flat picture. `outline: None` extrudes the picture's rectangle.
#[derive(Clone)]
pub struct Extrusion {
    pub solid: Solid,
    pub outline: Option<Arc<Vec<ShapeNode>>>,
    pub stretch: [f32; 2],
}

/// Planar warps and whether a spatial field deforms the material.
#[derive(Clone, Debug, Default)]
pub struct MaterialRecipe {
    pub warps: Vec<EffectPass>,
    pub spatial: bool,
}

/// One contribution: its picture and how it is drawn.
#[derive(Clone)]
pub struct LayerWork {
    /// Resource namespace for caches and feedback history.
    pub id: LayerId,
    pub instance: u32,
    pub content_key: LayerId,
    pub content: RasterSource,
    /// A host-supplied picture replaces `content`.
    pub host_picture: bool,
    /// Reuse a stored picture for this frame when one exists; value is the in point.
    pub freeze: Option<i64>,
    pub extrude: Option<Extrusion>,
    pub placement: LayerPlacement,
    pub projection: LayerProjection,
    pub blend: BlendMode,
    pub surface: SurfaceRecipe,
    pub displace: PointDisplace,
    pub clip: Option<ClipSpec>,
    pub shadow: f32,
    /// Light the layer gives (its Glow's intensity), read by the scratch Lighting Pack.
    pub emission: f32,
    pub passes: Vec<EffectPass>,
    pub after_passes: Vec<EffectPass>,
    pub image_inputs: Vec<Vec<ImageInput>>,
    pub masks: Vec<ResolvedMask>,
    pub material: MaterialRecipe,
    pub source_is_file: bool,
    pub source_tick: i64,
    pub isolate: bool,
}

impl LayerWork {
    /// Whether an effect reads another picture (see `SceneLayerValue::reads_other_pictures`).
    pub fn reads_other_pictures(&self) -> bool {
        self.image_inputs.iter().any(|row| !row.is_empty())
    }
}

/// A picture built from contributions: a base, pictures drawn source-atop onto
/// it, and optionally masked by the union of other composed pictures.
#[derive(Clone, Debug, PartialEq)]
pub struct Composed {
    pub base: usize,
    pub atop: Vec<usize>,
    pub mask: Option<(Vec<Composed>, MatteMode)>,
}

#[derive(Clone, Default)]
pub struct RenderGraph {
    pub layers: Vec<LayerWork>,
    pub output: Vec<Composed>,
}

impl RenderGraph {
    pub fn is_empty(&self) -> bool { self.output.is_empty() }

}
