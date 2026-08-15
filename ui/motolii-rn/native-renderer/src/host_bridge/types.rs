use motolii_ui::AppStageGeometry;

/// Host投影。revision変化時だけTimelineへ適用する。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineProjection {
    pub host_handle: Option<String>,
    pub revision: String,
    pub projection_generation: String,
    pub primary_layer_id: Option<String>,
    /// wire `current_time` の {num,den}。欠落時は0/1。
    pub current_time: (i64, i64),
    /// wire `timeline.duration` の {num,den}。
    pub timeline_duration: Option<(i64, i64)>,
    /// wire `timeline.fps`。timeline欠落時はNone。
    pub fps: Option<(i64, i64)>,
    pub bounds: Vec<(String, String)>,
    /// wire `timeline` がある時だけ。欠落時は旧host互換fallback。
    pub timeline_layers: Option<Vec<HostTimelineLayer>>,
    /// wire `stage_geometry`。欠落・壊れている時はNone（timeline投影は落とさない）。
    pub stage_geometry: Option<HostStageGeometry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostTerminalDiagnostic {
    pub reason: String,
    pub host_handle: Option<String>,
    pub stage_handle: Option<String>,
    pub timeline_handle: Option<String>,
    pub expected_projection_generation: Option<String>,
    pub actual_projection_generation: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTerminalResult {
    pub accepted: bool,
    pub diagnostics: Vec<HostTerminalDiagnostic>,
    pub message: Option<String>,
    pub projection: Option<HostTimelineProjection>,
}

impl HostTerminalResult {
    pub(crate) fn stamp(&self) -> Option<(u64, u64)> {
        let projection = self.projection.as_ref()?;
        Some((
            projection.revision.parse().ok()?,
            projection.projection_generation.parse().ok()?,
        ))
    }

    pub(crate) fn feedback(&self) -> Option<&str> {
        self.message.as_deref().or_else(|| {
            self.diagnostics
                .first()
                .map(|diagnostic| diagnostic.reason.as_str())
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostCatalogProjection {
    pub effects: Vec<HostCatalogEffect>,
    pub sources: Vec<HostCatalogSource>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostCatalogEffect {
    pub plugin_id: String,
    pub name: String,
    pub effect_version: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostCatalogSource {
    pub plugin_id: String,
    pub name: String,
    pub effect_version: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostStageGeometry {
    pub layers: Vec<HostStageGeometryLayer>,
    pub layers_truncated: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostStageGeometryLayer {
    pub layer_id: String,
    pub corners: [[f64; 2]; 4],
    pub position: [f64; 2],
    pub rotation: f64,
    pub scale: [f64; 2],
}

impl HostStageGeometryLayer {
    pub(crate) fn from_corners(layer_id: impl Into<String>, corners: [[f64; 2]; 4]) -> Self {
        Self {
            layer_id: layer_id.into(),
            corners,
            position: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }
    }
}

impl From<AppStageGeometry> for HostStageGeometry {
    fn from(geometry: AppStageGeometry) -> Self {
        Self {
            layers: geometry
                .layers
                .into_iter()
                .map(|layer| HostStageGeometryLayer {
                    layer_id: layer.layer_id,
                    corners: layer.corners,
                    position: layer.position,
                    rotation: layer.rotation,
                    scale: layer.scale,
                })
                .collect(),
            layers_truncated: geometry.layers_truncated,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineLayer {
    pub layer_id: String,
    pub display_name: String,
    pub start_secs: f64,
    pub duration_secs: f64,
    pub position_keys: Vec<HostTimelineKey>,
    /// wire `param_keys`。欠落は空。scene keys へ position_keys と union。
    pub param_keys: Vec<HostTimelineKey>,
    pub effects: Vec<HostTimelineEffect>,
    pub effects_truncated: bool,
    pub source_params: Vec<HostTimelineSourceParam>,
    pub source_params_truncated: bool,
    pub visible: bool,
    pub solo: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineEffect {
    pub effect_use_id: String,
    pub plugin_id: String,
    pub params: Vec<HostTimelineEffectParam>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineEffectParam {
    pub param_id: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineSourceParam {
    pub param_id: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineKey {
    pub key_id: u64,
    pub time_secs: f64,
    /// wire `value`。[f64;2] がある時だけ。sceneには載せない。
    pub value: Option<[f64; 2]>,
}
